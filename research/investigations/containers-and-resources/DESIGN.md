# Containers and resources: the integrated design

The single design for batch 0116. It merges the two drafts beside it,
`RESOURCES.md` (providers, the envelope `E`, the `resource-closed` judgment) and
`CONTAINERS.md` (owners, views, and the facts that cross a call), into one set of
laws, one set of rules, one vocabulary, and one amendment register. A reader who
has not read either draft can read this file alone. The drafts remain for their
detailed rationale, their rejected alternatives, and their probe registers; every
rule they stated normatively now lives here.

Tree read: `batch/0116-containers-and-resources` at `main` a40c7e70,
`spec/kernel-spec.md` **v0.40 ACTIVE**. Bare four-digit line numbers are that
file; every other citation names its file.

**Nothing here is implemented.** No compiler code was written for it. Section 3
is draft rule text for a work branch, not an amendment. Every program in
section 4 is design text and compiles nowhere. Section 6 separates what a
compiler executed in this session from what is argued on paper; every claim in
this file about *today's* compiler is a verdict from a probe re-run here, not a
verdict inherited from a draft.

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

---

## 2. The laws

Fifteen laws, merged from the two drafts and renumbered. Every rule in section 3
is an instance of one of them, and **a rule that cannot name its law is not
admitted.** L1 through L9 are the resource laws; L10 through L15 are the
container laws. Each states its rationale in one sentence and names the owner
ruling or the evidence it rests on. Owner rulings cite
`EVIDENCE-owner-discussion-2026-08-31.md` by its ruling id (R2 through R14) and
its accepted-conclusion id (B1 through B12).

**L1 (was R-L1). The envelope is the program's promise.** *A resource-closed
program declares one finite, shaped envelope `E` and promises that on every legal
execution, and on every finite prefix of an infinite one, its demand for covered
resources never exceeds `E`; whether an environment supplies `E` is a separate
fact about the environment.*
Rationale: split this way, `resource-closed(C, E)` is a static judgment about an
artifact and `Admitted(H, C, E)` is a run-time fact about a deployment, so the
property is checkable at compile time and actionable by a writer.
Rests on: owner ruling R13 (`L7036`), "do not get the direction backwards"; B8.

**L2 (was R-L2). No resource is ambient.** *Every covered resource enters the
program as a capability value the runtime hands to `main`, or as a store the
program reserves statically, and travels only by ordinary ownership; there is no
ambient allocator, ambient thread source, or ambient stack pool.*
Rationale: an effect row describes what a body did, while a held value is an
authority the body had, and only the second makes "this call graph cannot reach
the allocator" a signature fact rather than a whole-program re-derivation.
Rests on: probe `p5_ambient` (section 6), a nullary leaf function that allocates
while holding nothing, **accepted today**; and [FN-7] 1242, "there is no ambient
system state", whose last exception this law removes.

**L3 (was R-L3). Nothing fails silently, and nothing grows behind the writer.**
*Every operation that can fail to obtain a covered resource returns a typed value
naming the failure and handing back every affine input it did not consume; no
operation traps, aborts, retries, or promotes a store to a larger one.*
Rationale: v0.40 has zero writer-reachable runtime-trap families (spec line 6)
and yet heap exhaustion still ends a process with no source value, so the
trap-freedom claim is not yet honest for this one family.
Rests on: owner ruling R12 (`L5657-5666`), a pool with a silent fallback is worth
nothing; B3; audit answer Q8.

**L4 (was C-L3). No hidden growth.** *No operation both uses existing capacity
and acquires new capacity; every operation that may acquire capacity takes an
owner and a provider, names its allocation effect, and returns a typed failure,
while every operation that only uses existing capacity is total under a proved
capacity requirement and can allocate on no path.*
Rationale: one `push` cannot carry one return type and one effect row across
backings, and a growing push inside a loop leaves partial commitments that no
clean semantics describes.
Rests on: owner ruling R5 (`L2332`) killed the no-growth-at-all form; B2, B3, X1.

**L5 (was R-L4). The runtime is inside the envelope.** *The artifact `E`
describes is the writer's code, the compiler-derived cleanup and drop glue, the
`par` runtime, and the target adapter together; a resource any of them needs is
in `E`, or the program is not resource-closed.*
Rationale: a guarantee that stops at the edge of generated code is not a
guarantee, and the current runtime creates a worker thread on first `par`, maps a
diagnostic stack when a lane starts, initializes a completion ring lazily, and
reallocates a cleanup worklist.
Rests on: owner ruling R12, "the runtime must meet every requirement of
`res-closed`, and if it cannot, you must tell me why"; B12.

**L6 (was R-L5). Shape, not bytes.** *`E` is a list of tangible resources
(contiguous aligned extents, per-class slot counts, per-context stacks, lane
counts) and never one byte total, because a byte total cannot express the request
a fragmented store cannot serve.*
Rationale: sixteen bytes holding four four-byte objects, with the first and third
released, have eight free bytes and cannot serve an eight-byte request; alignment
is an independent counterexample.
Rests on: owner ruling R12, "even giving the heap a cap does not guarantee space
is available: the heap also has internal fragmentation"; B9, B11.

**L7 (was R-L6). Lowering before judgment.** *Tail recursion, including mutual
tail recursion, is rewritten into loops by the compiler before any resource
judgment runs; what the judgment sees is a call graph, and a graph that still has
a cycle has no finite stack envelope.*
Rationale: an optimization that may or may not fire cannot be a premise of a
guarantee, and the language already forbids an optimizer fact from changing
acceptance.
Rests on: owner rulings R3 (`L989`) and R12 (no depth certificates); B10.

**L8 (was R-L7). Demand is computed as if every acquisition succeeds.** *The
resource judgment replays each execution assuming every covered acquire succeeds;
it may never conclude that demand is small because a failed acquisition would
have ended the program.*
Rationale: without it the judgment is circular and always answers yes, since a
program whose first allocation fails has trivially bounded demand.
Rests on: B8's "every legal execution and every finite prefix" formulation.

**L9 (was R-L8). Stock, not flow.** *Resource-closedness bounds what is held at
once and what is consumed irreversibly; it never bounds how many times a program
acts.*
Rationale: a service loop that takes a slot, uses it, and releases it runs forever
with one live slot, and is exactly the program this property is for; the resource
that runs out is the slot, not the event.
Rests on: B8 (finite prefixes, not finite lifetimes); the distinction has teeth
in both directions, since a fixed append-only log is a consumable budget.

**L10 (was C-L1). A view is a value.** *A view is an affine value with a static
type, not a reference the callee writes through and not a hidden pointer to the
owner's header; a function that changes a view's state consumes it and returns
the new one, so every state change a view can cause is visible in the callee's
signature as a parameter consumed and a result produced.*
Rationale: the write-back problem ("the `len` the callee advanced never reaches
the owner") is answered without a hidden protocol, because the advanced `len` is
the result value; pass-by-pointer becomes an ABI choice.
Rests on: owner's settled decision of 2026-09-03 (views transformed by value,
`set buf = collect(...)`); B6.

**L11 (was C-L2). Length is a type fact or a contract fact, never a guess.** *At
every program point the checker's knowledge of a sequence's length comes from
exactly one of: the type, an established fact with live support [ENT-3, ENT-5],
or a verified contract relation [FN-9]; no rule infers a length from the shape of
an argument, the name of a callee, the absence of a write, or what a body was
seen to do.*
Rationale: this is D1 stated as a law, and it is why the repair is not "fix the
flag" but "have no flag derived from an actual to be wrong".
Rests on: `EVIDENCE-sweep-D1.md`; probe `d1` accepted today (section 6).

**L12 (was C-L4). The initialized prefix is the only initialization state.** *A
sequence's storage is exactly `[0, len)` initialized and `[len, cap)` raw; the
boundary is checker-maintained typestate carried by the owner's static type, and
no per-slot tag, `Option` wrapper, occupancy bitmap, or runtime discriminant
exists.*
Rationale: with no per-slot state, the checker never needs a quantified
proposition over slots; it needs one scalar relation, `len <= cap`, which is a
difference bound [ENT-4] already derives.
Rests on: owner's settled decision (`FixedVector<T, N>` holds affine `T` through
an initialized-prefix typestate); audit answers Q2, Q4, Q10.

**L13 (was C-L5). Release belongs to the owner's backing type.** *The release
action of a sequence is fixed by the owner's type under [STOR-3] and by nothing
else: drop `[0, len)` in ascending index order, then the backing's own release; a
view never releases the backing, and no source construct selects, replaces, or
observes the action.*
Rationale: the release row is a property of the backing, which is why owning
generics stay concrete and why there is no effect polymorphism to bridge them.
Rests on: B2's drop order; audit answer Q10; [STOR-3] 683.

**L14 (was C-L6). An `AppendView` reaches only what it appended.** *An
`AppendView` presents the spare window `[base, cap)` of its owner, where `base`
is the owner's length at formation; its own `len` counts what was appended
through it and starts at zero, no operation on it reaches an index below `base`,
and no operation on it decreases the owner's length.*
Rationale: this is what lets a caller's length fact stay alive across a callee
that appends, so the design does not buy soundness by discarding every length
fact at every call.
Rests on: B6; the owner's third call rule of 2026-09-03.

**L15 (was C-L7). Capacity is a proof term, not a value.** *`cap(v)` and
`room(v)` exist as [ENT-2] terms and in contract relations; no operation returns
either as a runtime `u64`, so programs prove with them and cannot branch on them.*
Rationale: an allocator that rounds a request up may not change any accepted
program's behavior, and growth policy stays out of the language because no
program can observe whether growth was exact, 1.5x, or 2x.
Rests on: audit answer Q9; B3's "logical capacity is a language value in the
descriptor and is not the allocator's usable size".

---

## 3. The rules

Nine families. `[PROV]` is the capability values and their operations, `[RES]`
the covered set, the envelope and the judgment, `[STK]` the stack, `[RUN]` the
runtime's own closure and the environment's half of the bargain, `[CNT]` the
sequence owners and their typestate, `[VIEW]` the views and the commit event,
`[CALL]` what survives a call, `[SEQ]` the operation table, and `[BLD]` the `par`
builder. Each rule states the judgment it creates, the fact it publishes, and
what it amends; section 3.12 collects every amendment in one register so nothing
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

### 3.1 `[PROV]`: capability values

**[PROV-1] Providers.** A *provider* is a value of one of the compiler-known
opaque nominal types `Heap`, `Arena<'p>`, and `Pool<'p, T, N>`. A provider is
affine [OWN-1], has no writer-visible component, and is the sole authority for
allocating from the store it names: `Heap` names one general-purpose growable
store, `Arena<'p>` names one contiguous extent served by a bump cursor, and
`Pool<'p, T, N>` names `N` interchangeable slots each holding exactly one `T`.
`Arena` and `Pool` are region-bearing under [STOR-5] and confined by [STOR-4]'s
relation exactly as `arena<'r, T>` is; `Heap` is not region-bearing, because it
is delivered as an `own` entry parameter and lives for the program.
*Judgment:* provider types are nominal and closed; no source declaration
introduces another. *Publishes:* for each provider place, the store identity
[RES-6]'s domain algebra tracks. *Law:* L2.

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
pool_static or arena_static`. *Publishes:* uniqueness of the `Heap`, which is the
fact [CNT-5] needs to keep a `HeapVector` release attributable without storing an
allocator identity in the value. *Law:* L2.

**[PROV-3] Every covered-store operation takes its provider.** An operation that
allocates from a store takes that store's provider as a written parameter, `own`
or `&uniq 'p`, and exhibits it. The provider-level rows are:

```text
| op                  | signature                                                                                    | effects          |
|---------------------|----------------------------------------------------------------------------------------------|------------------|
| box_new             | (heap: &uniq 'h Heap, value: own T) -> own Result<box<T>, OutOfMemory<T>>                      | allocates(heap)  |
| arena_new           | (arena: &uniq 'p Arena<'p>, value: own T) -> own arena<'p, T>                                   | allocates(arena) |
| arena_new_checked   | (arena: &uniq 'p Arena<'p>, value: own T) -> own Result<arena<'p, T>, NeedCapacity<T>>          | allocates(arena) |
| pool_take           | (pool: &uniq 'p Pool<'p, T, N>, value: own T) -> own slot<'p, T>                                | allocates(pool)  |
| pool_take_checked   | (pool: &uniq 'p Pool<'p, T, N>, value: own T) -> own Result<slot<'p, T>, PoolExhausted<T>>      | allocates(pool)  |
| pool_release        | (pool: &uniq 'p Pool<'p, T, N>, item: own slot<'p, T>) -> own T                                 | writes(pool)     |
| live, capacity      | (&'p Pool<'p, T, N>) -> own u64                                                                 | pure             |
| remaining           | (&'p Arena<'p>) -> own u64                                                                      | pure             |
```

The sequence rows that consume a provider are `[SEQ]`'s, not this table's:
`seq_reserve` and `seq_shrink` take the `Heap` or arena provider, and `seq_lease`
takes the pool provider. `buffer_new` and `buffer_vacant` do not appear, because
[CNT-1] retires `buffer<T>` from the writer surface entirely.
*Judgment:* an allocation call whose provider argument is missing, is not a
provider place, or is not writable is a hard error citing PROV-3 at the `call`.
*Publishes:* the provider place each allocation reaches, which is the footprint
[RUN-4] and the demand item [RES-6] both consume. *Amends:* the `box_new`,
`arena_new`, `buffer_new` and `buffer_vacant` rows of [OP-1] (793-798).
*Law:* L2, L3, L4.

**[PROV-4] `allocates` names a provider path.** The effect grammar's `allocates`
entry takes formal-rooted [EFF-1] paths naming provider state, in canonical
order, replacing the fixed atoms:

```text
effect := "reads" "(" path ("," path)* ")"
        | "writes" "(" path ("," path)* ")"
        | "allocates" "(" path ("," path)* ")"
```

An `allocates(p)` entry is exhibited exactly when the body reaches an allocation
whose provider argument projects to `p` under [EFF-2]'s call-boundary projection.
A body that allocates only from a fresh local provider frames out of its own
signature exactly as any other fresh-local state does, and the reserving
operation that created that provider is what appears in `E`.
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

**[PROV-6] Heap-reachability is a closed signature fact.** A function *reaches
the heap* when its own row carries an `allocates` entry whose path is rooted in a
`Heap`-typed formal, or when it calls a function that does. Because the
compilation unit is closed [PROG-1], there are no function values, and there is
no ambient store [PROV-2], the transitive closure over the call graph is exact
and is computed from signatures alone.
*Judgment:* none by itself; it is the premise of [RES-5]. *Publishes:* the
heap-reaching path, the ordered call chain from `main` to the allocation, which
is the diagnostic [RES-5] prints. *Law:* L2.

**[PROV-7] Provider-owned values and their release.** A value allocated from a
provider is released to that provider and to no other. For `Heap`, the provider
is unique [PROV-2], so `box<T>` and a `HeapVector`'s backing keep their present
storage class [STOR-1], their present compiler-derived free on the owner's
scope-exit edge, and their present empty release row [STOR-3, EFF-2]. For
`Arena<'p>`, an allocation's storage returns when `'p`'s block ends, exactly
[STOR-4]; an individual drop returns nothing to the cursor. For
`Pool<'p, T, N>`, a `slot<'p, T>` abandoned at a scope exit inside `'p` has a
compiler-derived release that returns the slot to the pool, and that release
contributes `writes(pool)` under [EFF-2]'s release-contribution rule, with the
provider path identified by the slot type's region argument.
*Judgment:* on every edge carrying a slot release, the provider place must be
reachable and writable; a release edge on which the provider is uniquely borrowed
elsewhere is a hard error citing PROV-7 at the owning scope exit. *Publishes:*
the release event [RES-6]'s pool algebra consumes. *Amends:* [STOR-3]'s
release-action list gains one row, and [EFF-2]'s empty-release-row sentence gains
its one exception. **This is the rule the design could not close**: the
reachability side-condition is stated, not derived (open question Q2).
*Law:* L3, L5, L13.

**[PROV-8] Reserving operations.** `pool_static<'p, T, N>()` and
`arena_static<'p, BYTES, ALIGN>()` each reserve one statically laid-out extent
*per source occurrence* and return the provider confined to `'p`. The reserved
extent is an ordinary place for [OWN-5] and for every footprint rule. Because
`'p` is lexical, at most one provider per occurrence is live at any program
point; because the call graph of a resource-closed program is acyclic [STK-2], no
occurrence is re-entered while its provider is live.
*Judgment:* the ordinary region and confinement judgments [OWN-3, OWN-4, STOR-4].
*Publishes:* one static-extent item of `E`, with size and alignment from `T`,
`N`, `BYTES` and `ALIGN`. *Amends:* nothing; adds two operation rows. *Note:* two
overlapping [PAR-1] statements that both reach one occurrence's extent are denied
by the ordinary footprint rule, with no new clause: the extent is a place, and
both statements write it. *Law:* L2, L6.

### 3.2 `[RES]`: the covered set, the envelope, and the judgment

**[RES-1] `CoreResources`.** Bare *resource-closed* means resource-closed for
exactly this set, and for no more:

```text
| class              | members                                                                      |
|--------------------|------------------------------------------------------------------------------|
| execution memory   | the static image; every frame of every execution context; every worker-lane   |
|                    | stack; every provider backing reserved by [PROV-8]; allocator and runtime      |
|                    | metadata; compiler-derived cleanup scratch; the adapter's persistent buffers   |
| execution capacity | par lanes; task records; submission, completion and wait records; queue slots; |
|                    | the runtime's fixed internal handle capacity                                   |
```

An extension is written and never implied: `resource_closed(core + file_handles)`
is a different, stronger declaration, admitted only when the environment can
deliver an exclusive reservation of that kind, and no such extension is defined
in this version.
*Judgment:* fixes the domains [RES-3] quantifies over. *Law:* L1, L5.

**[RES-2] The envelope `E`.** `E = E(P, T, C)` is a finite list of shaped items
computed for one program `P`, one selected target and ABI `T` [STOR-6], and one
runtime configuration `C` (principally the lane count `W`). Each item is one of:

```text
region(name, bytes, alignment, contiguous)   one extent the environment must deliver whole
slots(kind, count)                           interchangeable fixed-size records
stack(context, bytes)                        one execution context's maximum chain
lanes(count)                                 scheduler lanes, including the entry lane
```

No item is a bare byte total, and no two items are summed into one. Items are not
fungible: two `region` items are two extents.
*Judgment:* `E` is well-formed only if every item's arithmetic was performed in
the unbounded mathematical domain and is representable on `T`, the same standard
[STOR-6] already applies. *Publishes:* `E` itself, as a compilation artifact.
*Law:* L6.

**[RES-3] The resource-closed judgment.** For a program `P`, target `T`,
configuration `C` and envelope `E`, `resource-closed(P, T, C, E)` holds exactly
when, for every legal execution trace of the artifact `C(P)` [RUN-1] and every
finite prefix of that trace, replaying the prefix's covered acquisitions and
releases from `E` under each domain's own algebra [RES-6] leaves every
acquisition defined and every domain invariant intact. The traces are drawn from
the abstract demand semantics of L8, in which no covered acquisition fails.
*Judgment:* per domain, the composition of section 3.2.1 over the checked program
after [STK-1]'s lowering; deterministic, terminating, and free of search, budget
or timeout, as every acceptance judgment in this language must be. *Publishes:*
the property, and `E`. *Law:* L1, L8, L9.

**[RES-4] The entry requirement.** The entry may carry the marker
`resource_closed` before its `command` program-kind marker:

```wf-design
resource_closed command fn main() -> status: own ExitStatus writes(uart) {
```

The marker changes no other rule. Every program is judged by exactly the same
rules; the marker only makes failure to establish [RES-3] a hard error citing
RES-4 rather than a reported property. There is no second language, no subset
mode, no alternative prelude, and no per-function marker: a *program* is
resource-closed or is not.
*Judgment:* on a marked entry, the first unestablished premise of [RES-3] is a
hard error naming its own cause: the heap-reaching path [RES-5], the call-graph
cycle [STK-2], the unbounded store [RES-6], or the unclosed runtime [RUN-2].
*Amends:* [FN-7], which fixes main's marker set. *Law:* L1.

**[RES-5] The heap excludes resource-closedness.** A program whose call graph
reaches a `Heap` [PROV-6] is not resource-closed, and a `main` selecting
`command.heap` is by itself the rejection. A bounded general store is still a
general store: an envelope item can promise bytes, and cannot promise that the
next contiguous aligned request has a home.
*Judgment:* under [RES-4], a hard error citing RES-5 at the offending
`input_label` or at the deepest `call` of the heap-reaching path, rendering the
whole chain. *Law:* L6.

**[RES-6] Store domains and their algebras.** Exactly three covered-store domains
are defined, each with its own deterministic state and transfer rules. Nothing
else is admitted, and a store outside this list contributes no envelope item and
denies [RES-3].

```text
| domain                         | state           | acquire                  | release            | serviceable when      |
|--------------------------------|-----------------|--------------------------|--------------------|-----------------------|
| uniform slots                  | live count      | +1                       | -1 at [PROV-7]     | live < N              |
|  (Pool, lane/task/completion)  |                 |                          |                    |                       |
| bump extent (Arena<'p>)        | cursor, align   | cursor advances by       | nothing; the whole | remaining >= that     |
|                                |                 | round_up(cursor,align(T))| extent returns     | advance               |
|                                |                 | - cursor + size(T)       | when 'p ends       |                       |
| static and frame placement     | fixed offsets   | none at run time         | none               | decided at compile    |
|                                |                 |                          |                    | time [STOR-6]         |
| general heap (Heap)            | -               | -                        | -                  | undecidable from E;   |
|                                |                 |                          |                    | excluded by [RES-5]   |
```

*Judgment:* the (peak, delta) composition of 3.2.1 is admitted for the
uniform-slot domain only; the bump extent uses the monotone cursor domain, in
which a drop returns nothing. *Publishes:* per program point, per domain, the
live count or cursor bound. *Law:* L6.

**[RES-7] Typed failure, and what it retires.** Every operation that can fail to
obtain a covered resource returns a typed value that names the failure and hands
back every affine input it did not consume: `OutOfMemory<T>`, `PoolExhausted<T>`,
`NeedCapacity<T>`, `Full<T>`, and `TooSmall`. No covered-resource failure is a
trap, an abort, a process exit, a retry, or a promotion to a larger store, in the
writer's code or in the runtime.
*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result`. *Publishes:* on
the `Err` edge, the returned owner's identity. *Retires:* the heap arm of the
exhaustion floor; the `wf_resource_abort` site for allocation refusal (batch 0079
item F4, `docs/done/0079-exhaustion-floor.md`) has no reachable caller once
allocation returns a value, and the stack arm survives only for programs that are
not resource-closed. *Amends:* [SCOPE-3] 29, whose "heap exhaustion ... may stop
execution at the host boundary without a Whitefoot value" ceases to be true of
the heap. *Law:* L3.

**[RES-8] A covered acquisition is a partial operation.** Each covered-store
acquisition comes in exactly two spellings, on the model of `+` and `+checked`
[OP-1]:

```text
pool_take(pool: p, value: v)          domain obligation ilt(live(p), capacity(p))       -> own slot<'p, T>
pool_take_checked(pool: p, value: v)  total                                             -> own Result<slot<'p,T>, PoolExhausted<T>>
arena_new(arena: a, value: v)         domain obligation ige(remaining(a), reserve<T>()) -> own arena<'p, T>
arena_new_checked(arena: a, value: v) total                                             -> own Result<arena<'p,T>, NeedCapacity<T>>
```

The proved form is admitted only when [ENT-6] discharges its exact goal in the
current ProofContext; an unproved goal is a static rejection with no fallback,
exactly as an unproved subscript is [OP-4]. `live`, `capacity` and `remaining`
are pure total queries whose results enter the proof context as ordinary typed
terms, and the transfer rules of [RES-6] publish `live' = live + 1` at a take and
`live' = live - 1` at a release, the way [SYS-9] publishes a system operation's
enumerated relations. **The `Heap` has no proved form**: no honest domain
predicate exists for a general store (L6), so every heap acquisition is total and
returns `Result` unconditionally.
*Judgment:* [ENT-6] discharge at the proved spelling; nothing at the checked one.
*Publishes:* the post-state relation on the store's live count or cursor.
*Law:* L3, L6.

**[RES-9] What bare resource-closedness does not cover.** Disk space, the
successful acquisition of a file, socket or other host object not exclusively
reserved before start, network reachability and throughput, CPU time, deadlines,
scheduler fairness, power, device health, host termination, and OS quota
revocation are outside [RES-1] and outside every judgment in this file. They
remain typed system outcomes where the operation defines one ([SYS-7]'s error
classes), and environment conditions where it does not ([RUN-6]).
*Judgment:* none; a boundary statement. *Law:* L1.

#### 3.2.1 How `E` is composed

Every covered resource is one of three kinds, and conflating them is the single
most common way to get a wrong answer (L9).

```text
| kind                 | question                          | examples                              | bound         |
|----------------------|-----------------------------------|---------------------------------------|---------------|
| reusable capacity    | how many are held at once?        | pool slots, task and completion       | peak live     |
|                      |                                   | records, lanes, queue slots           |               |
| consumable budget    | how much is spent and not         | arena cursor bytes, a fixed           | net consumed  |
|                      | returned in this epoch?           | append-only log's records             |               |
| external effect flow | how many times did it happen?     | opens, writes, submissions            | not bounded,  |
|                      |                                   |                                       | not in E      |
```

Per resource kind `r`, a straight-line segment has a summary `(peak_r, delta_r)`
relative to its entry state. The primitive transfers are fixed:

```text
acquire one       (peak 1, delta +1)
release one       (peak 0, delta -1)
move an owner     (peak 0, delta  0)      moving into a container acquires nothing
borrow an owner   (peak 0, delta  0)
```

and the compositions are:

```text
sequence   peak(A;B)  = max(peak(A), delta(A) + peak(B))
           delta(A;B) = delta(A) + delta(B)

branch     peak       = componentwise max over arms
           delta      = kept per exit variant, not joined into one interval when a
                        Result or Option variant already distinguishes the arms

call       substitute the callee's per-exit summary at the call site, with its
           formal capacity and provider terms replaced by the actual ones

loop       delta at the backedge = 0   -> peak is one iteration's peak; no iteration
                                          bound is needed
           delta at the backedge > 0   -> a counted range, a structural capacity cutoff
                                          (len <= N), or a writer-supplied resource
                                          invariant is required; otherwise no finite E

par        peak = (M - K) * d + K * p   for M logical iterations, per-iteration peak p
                                        and retained d, and K the profile's maximum
                                        concurrently live iterations
```

`K` is the runtime profile's window, not the lane count `W`, because an iteration
that submits a `may-suspend` operation and suspends leaves its resources
outstanding while its lane starts another. Scratch each iteration releases scales
with `K`; output each iteration retains until the join scales with `M`. They are
different kinds and are never merged into one figure.

**What needs no writer annotation:** straight-line acquire, move, borrow and
release; lexical scopes and compiler-derived cleanup edges; branch joins;
per-variant retention a `Result` or `Option` already distinguishes;
`FixedVector`'s `len <= N` and its initialized prefix; moving an owner into or
out of a container; a loop whose backedge restores the state; a counted loop
whose per-iteration delta is a fixed affine expression; a non-recursive call with
a computed summary; and a `par` loop composed by the formula above.

**What needs one:** a loop that may retain with no structural cutoff; a relation
across two containers (`len(active) + len(waiting) <= capacity`); a resource
returned only at a later milestone; an acquisition whose size is a computed
value; a `par` window the profile does not fix; and any place where the writer
wants a tighter answer than the per-branch maximum. These are written as ordinary
[INV-1] invariants over `live`, `capacity`, `remaining`, `len`, `cap` and `room`,
and the checker verifies base, preservation and exit exactly as for any other
invariant. **Three of those six terms are not [INV-1] atoms today** and one whole
class of consumer cannot read the result; that is open question Q6, and it is the
largest risk this design carries. The checker never searches for an invariant: it
does not enumerate paths, guess loop invariants, choose allocator placements, or
divide a store between claimants.

#### 3.2.2 Compile-time judgment versus start-up step

```text
 1  tail-SCC rewrite [STK-1]                        compile time, before any resource judgment  compiler
 2  call-graph acyclicity [STK-2]                   compile time, on the rewritten graph        compiler
 3  provider reachability, heap-freedom [PROV-6]    compile time, from signatures               compiler
 4  per-function per-domain demand summaries        compile time, bottom-up on the DAG          compiler
 5  loop and par composition (3.2.1)                compile time                                compiler
 6  static-extent items from [PROV-8] occurrences   compile time                                compiler
 7  concrete sizes, strides, static image           target stage [STOR-6]                       compiler
 8  per-context frame envelope [STK-3]              target stage, after code generation         compiler
 9  runtime profile row for each supported W        fixed data of the qualified runtime         runtime
10  assembling E and emitting it as an artifact     target stage                                compiler
11  choosing W for this run                        PreStart                                     launcher
12  committing every region and stack item          PreStart                                    launcher
13  creating lanes and reaching the ready barrier   PreStart                                    runtime
14  initializing every adapter record and queue     PreStart                                    runtime
15  crossing SourceStart and invoking main          PreStart -> Running                         runtime
```

Steps 1 to 10 decide whether the program is resource-closed; steps 11 to 15
decide whether this run is admitted. Neither judgment consults the other.

### 3.3 `[STK]`: the stack

**[STK-1] Tail lowering runs before every resource judgment.** For each strongly
connected component of the call graph in which *every* intra-component call edge
is in true tail position, the compiler rewrites the component into one dispatcher
loop before frames are measured. The rewrite is admitted only when, on every
intra-component edge: the call is the complete `expr` of a `return_stmt`; no
compiler-derived drop or release remains to run after it [STOR-3]; no child
reborrow of a caller-local place is live across it [OWN-6]; no `par` join is
outstanding; every affine local has been moved or released before the jump; and
the dispatcher frame is the maximum over the component's members of their
parameter and local state. A component failing any condition is not rewritten.
*Judgment:* structural and per-edge; no proof search. *Publishes:* an acyclic
call graph, or a component that is still cyclic. *Amends:* nothing in [FN-6],
which continues to permit recursion; this is a lowering, not an admission rule.
*Law:* L7.

**[STK-2] Acyclicity is required, and there are no depth certificates.** After
[STK-1], a program whose call graph still contains a cycle has no finite stack
envelope and is not resource-closed. A `requires` bound on a recursion parameter,
a proof that a recursion argument decreases, and every other depth certificate
are **not** admitted as a substitute; this design defines no such construct and
no compiler is asked to discover one.
*Judgment:* under [RES-4], a hard error citing STK-2 that renders the complete
cycle in call order and the restructuring `rewrite the recursion as a loop over
an explicit FixedVector work list, or make every recursive call a tail call`.
*Law:* L7.

**[STK-3] The frame envelope.** For each execution context, the stack item of `E`
is the maximum over the context's entry points of

```text
Stack(f) = frame(f) + max over every possibly-active callee g of ( call_overhead(f, g) + Stack(g) )
```

taken over the acyclic graph of [STK-2], where the possibly-active callees include
those reached on error and propagation edges, compiler-derived drop glue, the
target adapter's helpers, the `par` worker entry and resume paths, and the ABI
save area. `frame(f)` is measured **after final code generation**, because the
frame a function actually needs is made of things that do not exist earlier: the
ABI frame record a non-leaf keeps, and the callee-saved registers the allocator
chose to spill. An optional optimization may not raise a computed envelope: an
implementation either recomputes `E` after the optimization and publishes the
larger figure, or declines the optimization; it never publishes one figure and
emits code needing another.
*Judgment:* target-stage arithmetic under [STOR-6]'s checked-arithmetic
discipline. *Publishes:* one `stack(context, bytes)` item per context. *Amends:*
[STOR-6] 757-761, whose "the language therefore defines no numeric per-array,
per-object, or per-function frame ceiling" keeps its scope for the *language* and
is joined, for a resource-closed program, by a computed per-context envelope
measured post-codegen. *Law:* L5, L6.

**[STK-4] One item per execution context, and reentrancy is not free.** `E`
carries a `stack` item for the entry context and for each of the `W - 1` worker
lanes, plus one for every context the target profile introduces: a completion
helper, a bounded blocking helper, an interrupt or signal stack, an FFI callback
stack. A context whose depth cannot be bounded because control can re-enter the
Whitefoot call graph from outside it (a signal handler, an FFI callback, a host
reentrancy path) denies resource-closedness unless the profile gives it a
separately reserved stack item of its own.
*Judgment:* a profile-level check, not a source check. *Law:* L5.

**[STK-5] Stack exhaustion moves inside the model, for these programs only.** For
a resource-closed program, stack exhaustion is not a deferred external resource
condition: [STK-2] and [STK-3] make the maximum chain a computed item of `E`, and
under an admitted run [RUN-6] it is unreachable. For every other program,
[SCOPE-3]'s deferral stands unchanged, and so does the guard-page floor that
reports it.
*Judgment:* none; a scope statement. *Amends:* [SCOPE-3] 29-31. *Law:* L1.

### 3.4 `[RUN]`: runtime closure and admission

**[RUN-1] The artifact.** For every judgment in this file the artifact `C(P)` is
the writer's code, the compiler-derived cleanup and drop glue, the monomorphized
instances, the `par` runtime, the allocator and its metadata, and the qualified
target adapter: everything the process runs between the barrier and
`ProgramFinished`. *Law:* L5.

**[RUN-2] Runtime closure.** A runtime qualified for resource-closed programs
performs, after the `SourceStart` barrier and until `ProgramFinished`, no covered
acquisition whatsoever: no allocator call for runtime-owned storage, no thread or
helper creation, no stack, queue, table or worklist growth, no lazy TLS or
adapter initialization, no first-use mapping, and no first-error formatting
buffer. Every runtime record is established before the barrier or is carved from
a fixed backing that is already an item of `E`. Internal saturation is answered
by waiting, reuse, inline execution, or a semantically equivalent sequential
path, and never by growth, abort, or a source-visible outcome. Teardown after
`main` returns is under the same obligation.
*Judgment:* a conformance obligation on an implementation, auditable from the
emitted code and the runtime's own translation units; it is not a source judgment
and no source construct can weaken or waive it. An implementation that cannot
meet it does not support the `resource_closed` marker on that target, and must
say so rather than accept the marker. *Publishes:* the runtime's own items of `E`.
*Law:* L3, L5.

**[RUN-3] `par` enters `E` as a profile, not as an iteration count.** For each
supported lane count `W`, the runtime publishes one finite profile row: `W` lanes
(of which `W - 1` are host worker threads), `W - 1` worker stacks, a fixed
task-record count `K(W, d)` where `d` is the program's maximum nested `par`
depth, fixed queue capacities, and a fixed completion-record count. The number of
iterations of a `par`-permitted loop never appears in `E`: the runtime chunks the
index range lazily, so a loop of a billion iterations holds no more task records
than one of a thousand. On saturation the runtime executes inline, helps, waits,
or applies backpressure.
*Judgment:* a fixed-arithmetic composition (3.2.1's `par` rule) against the
selected profile row; the compiler emits no per-`W` clone of the program.
*Publishes:* the `lanes` and `slots` items of `E`. *Amends:* the sentence common
to [PAR-1] 1989, [PAR-2] 2024 and [PAR-3] 2049, "exhaustion of the execution
resources an implementation spends on overlapping is a resource condition under
[SCOPE-3] and is not an observable of this rule": for a resource-closed program
that exhaustion is not merely unobservable, it is unreachable. *Law:* L5, L9.

**[RUN-4] The parallel footprint of an allocation is its provider place.** In
[PAR-1]'s written-footprint clause, "the caller region each `allocates(arena 'r)`
entry names after region substitution" is replaced by "the places each
`allocates` path reaches under the [EFF-2] call-boundary projection", the same
projection the rule already applies to `reads` and `writes`. Two statements that
allocate from one provider therefore conflict, and two that allocate from
distinct providers do not.
*Judgment:* the existing [PAR-1] overlap judgment, with one fewer special case.
*Amends:* [PAR-1] 1969, and [PAR-2]/[PAR-3] through their "forms every footprint
exactly as [PAR-1] forms one" clauses. *Note:* the clause is now largely
redundant, since an allocation needs `&uniq` on its provider and [OWN-5]
exclusivity already denies two live exclusive loans on one place; it is retained
because an `own`-mode provider argument is not a loan. *Law:* L2, L5.

**[RUN-5] The startup protocol.** Program start has four points, and the covered
guarantee spans the last three:

```text
PreStart
    select W from the runtime's profile table
    materialize every item of E:
        commit each region (committed backing, not a reserved address range)
        commit each stack, including every worker lane's
        create W-1 lanes and park them at the ready barrier
        establish every queue, task, completion and wait record
        initialize every adapter record, TLS block and runtime table
    any step may fail -> StartFailed(item); nothing below happens

SourceStart  (the barrier)
    every item of E is established; no covered acquisition remains outstanding
    the runtime enters its closed mode [RUN-2]

Running
    main executes; source and runtime draw only on E

ProgramFinished
    main returns an ExitStatus
    every compiler-derived release on the return edge has run
    every outstanding par task and completion record has drained
    the runtime's bounded teardown is complete
```

A `PreStart` failure is reported as `StartFailed(item)` on an
implementation-defined channel using fixed, preallocated storage; no source
statement executes, no owner comes into existence, no language cleanup runs, and
no `ExitStatus` is produced. It is not a source `Result`, not `main`'s return
value, not a language trap, and not a source-language rejection [DIAG-1]. The
program's promise is conditional by construction (L1): it undertook never to ask
for more than `E`, and it has not asked for anything, because no statement of it
has run.
*Judgment:* a target obligation, not a source judgment. *Amends:* [PROG-3]
1499-1509, whose start-time obligation gains the materialization of `E` and whose
`ProgramFinished` boundary is now named; its existing "a start failure is a
target or environment failure ... not a source-language rejection" is kept
verbatim. *Law:* L1, L5.

**[RUN-6] Admission, and the theorem.** `Admitted(H, C, E)` holds when an
environment `H` has actually established a grant implementing every item of `E`
before the barrier (committed backing rather than a reserved address range, real
lanes at their ready barrier, real queues and records) and, for the duration of
the run, does not revoke it and permits no unmodelled competitor to consume from
it. Then:

```text
resource-closed(C, E)  and  Admitted(H, C, E)
---------------------------------------------
no covered-resource exhaustion in run(H, C)
```

An environment that later revokes the grant, kills the process, or violates the
target profile has falsified `Admitted`; it has not falsified the program's
property, and no rule of this file is stated in terms of it.
*Judgment:* none by the compiler. *Publishes:* the deployment contract, which is
`E` itself. *Law:* L1.

### 3.5 `[CNT]`: owners and typestate

**[CNT-1] The owner inventory.** Exactly four sequence owners, each with a static
backing fixed by its type.

```text
| type                 | backing              | placement            | provider   | cap        | growth      |
|----------------------|----------------------|----------------------|------------|------------|-------------|
| FixedVector<T, N>    | inline, N slots      | frame, static, or a  | none       | N, a       | never       |
|                      |                      | field of such an     |            | constant   |             |
|                      |                      | owner                |            |            |             |
| HeapVector<T>        | one heap allocation, | frame-resident       | Heap, to   | runtime    | seq_reserve |
|                      | none while empty     | descriptor           | grow       |            |             |
| ArenaVector<'r, T>   | one arena block      | frame-resident       | the arena  | runtime    | seq_reserve |
|                      | in 'r                | descriptor           | of 'r      |            | in 'r       |
| PoolVector<'p, T, N> | one pool lease of a  | frame-resident       | the pool   | N, from    | never       |
|                      | FixedVector<T, N>    | descriptor           | of 'p, at  | the pool's |             |
|                      | slot                 |                      | lease      | slot type  |             |
```

A container type is a compiler-owned nominal: it has no writer-visible field, is
constructed only by the `[SEQ]` operations, and has no source construction form
[GRAM-8]. An ordinary struct whose invariants are reproved at every use is
refused, because `len <= cap` would then be a fact with support the writer can
kill, and [ENT-5] would delete it at the first unrelated `set`.
*Amends:* [TYPE-2], four added composite types. *Law:* L4, L12.

**[CNT-2] Container state is typestate, not stored data the writer can reach.**
Each owner carries `len` and, where it is not a constant, `cap`. The checker
holds `len(v)`, `cap(v)` and `room(v)` as [ENT-2] length-class terms of fragment
type `u64`. The implicit facts `Z <= len(v)`, `len(v) - cap(v) <= 0`,
`Z <= room(v)`, and, for `FixedVector<T, N>` and `PoolVector<'p, T, N>`,
`cap(v) = N`, hold at every program point at which `v` is live, exactly as
[ENT-2]'s `len(P) = N` implicit fact does for `array`.
`room(v)` denotes `cap(v) - len(v)` and exists so that a "there is spare for this
whole source" requirement is a **two-term difference bound** rather than a
three-term affine relation: `ile(len(source), room(out))` normalizes to L0 while
`ile(len(source), cap(out) - len(out))` does not. `room` is never computed by
subtraction in the fact domain; every operation that changes `len` publishes the
matching `room` relation directly.
*Amends:* [ENT-2] clause (b), which today admits `len(P)` only for `array`,
`slice`, and `buffer`, and which gains `cap(P)` and `room(P)` as terms of the
same class. *Law:* L11, L15.

**[CNT-3] Raw slots are unreachable.** No `[SEQ]` operation, no subscript, and no
borrow yields a place in `[len, cap)`. A subscript on an owner or view carries
the ordinary [OP-4] obligation `ilt(index, len(base))`, against `len` and never
against `cap`. There is no uninitialized read to reject, because there is no
spelling that reaches one (L12). *Law:* L12.

**[CNT-4] Affine elements.** `T` may be affine in every owner. The initialized
prefix is what makes this sound: an element enters at `len` and leaves at
`len - 1`, so no slot is read before it is written or after it is taken.
`FixedVector<Handle, 64>` is the kernel object table the owner named.
*Amends:* [TYPE-2]'s `array` restriction only by not inheriting it; `array<T, N>`
keeps its copy-only element domain, because `array` carries no length separate
from `N`, so every slot is live at once and there is no prefix boundary to make
an affine element's entry and exit unambiguous. *Verified today:*
`array_new<box<u64>, 4>` is [OP-1] `InvalidOperation` (probe `p9`, section 6), so
this is new capability, not a restatement. *Law:* L12.

**[CNT-5] Release.** The release action of every owner, under [STOR-3]:

```text
drop element [0]                  ascending index order, each element's own
drop element [1]                    compiler-derived drop
...
drop element [len-1]
release backing:
  FixedVector    nothing (inline in its owner)
  HeapVector     one compiler-derived heap free
  ArenaVector    nothing at the value; the block goes with 'r  [STOR-4]
  PoolVector     one lease return, which writes the pool provider's state [PROV-7]
```

Only `PoolVector`'s backing release carries a nonempty effect row, contributed
under [EFF-2]'s release contribution. *Amends:* [STOR-3]'s `buffer<T>` drop
sentence by superseding it. *Law:* L13.

**[CNT-6] Containers are storable; views are not.** A container type is
region-free except `ArenaVector<'r, T>` and `PoolVector<'p, T, N>`, which are
region-bearing exactly as `arena<'r, T>` is. A `FixedVector` or `HeapVector` may
be a struct field, `box` content, or arena content. *Extends:* [STOR-5]'s
region-bearing relation to the two region-bearing owners and to all four view
types. *Law:* L13.

**[CNT-7] Acquiring capacity is owner-level and provider-bearing.** Every
operation that may change `cap(v)` takes the owner **by value**, takes the
provider, and names its allocation effect. It returns `Result` when acquisition
can fail and the owner directly when it cannot, because the old backing is kept
on failure. There is no capacity-changing operation on a borrow and none on a
view. *Law:* L4.

**[CNT-8] A container type never appears behind `&uniq`.** A `param`, `rtype`, or
`let`-bound holder whose mode is `&uniq 'r` and whose direct type is a container
type is a hard error citing CNT-8 at the complete `param` (or `rtype`) node, with
the restructuring `pass a MutSpan or AppendView for element and append work, or
take the owner by value and return it`. A shared `&'r` container parameter
remains legal: it can observe `len` and read elements and can change nothing.

This is the rule that retires D1's shape. *Retires:* the writer-facing
`&uniq buffer<T>` and `&uniq Container` state-borrow forms. `&uniq` survives
everywhere its referent's length is a type fact rather than state: a `&uniq` to a
struct holding `array<T, N>` fields, to a `slot<'p, T>`, or to a `MutSpan` is
legal, because no operation on any of them can change a length ([CNT-9], L11).
*Law:* L11.

**[CNT-9] `array<T, N>` is retained unchanged**, as the `len = cap = N` case. A
program that needs no length carries no length, and `tests/programs/fir_filter.wf`
is untouched by this design. *Law:* L11, L12.

### 3.6 `[VIEW]`: views, formation, and write-back

**[VIEW-1] The four views.**

```text
| type              | reads             | writes elements   | changes length      | may allocate | affine |
|-------------------|-------------------|-------------------|---------------------|--------------|--------|
| Span<'r, T>       | yes               | no                | no                  | no           | yes    |
| MutSpan<'r, T>    | yes               | yes               | no, fixed by type   | no           | yes    |
| AppendView<'r, T> | the window it     | the window it     | grows the window    | no           | yes    |
|                   | appended          | appended          | only                |              |        |
| Builder<'r, T>    | no                | its claimed slots | no, fixed at claim  | no           | yes    |
```

Each is an `own` affine value carrying a region `'r`, exactly as `slice<'r, T>`
does today. `Span<'r, T>` **is** today's `slice<'r, T>` renamed; the rename is
the whole of the change to it. *Amends:* [TYPE-2] (three added view types),
[STOR-5] (all four are region-bearing and unstorable), [OWN-1] (all four are
affine). *Law:* L10.

**[VIEW-2] Formation freezes the owner.** A view is formed from a borrow of the
owner and holds that loan for `'r`:

```text
seq_span(&'r v)             -> own Span<'r, T>          shared loan on v
seq_mut_span(&uniq 'r v)    -> own MutSpan<'r, T>       exclusive loan on v
seq_append_view(&uniq 'r v) -> own AppendView<'r, T>    exclusive loan on v
```

While the loan is live, [OWN-5] already forbids moving, dropping, growing, or
otherwise writing `v`; no new exclusivity rule is needed, and the freeze is the
existing loan. Formation publishes:

```text
seq_span         len(s)  = len(v)
seq_mut_span     len(m)  = len(v)
seq_append_view  len(a)  = 0,  room(a) = room(v),  cap(a) + len(v) = cap(v)
```

The last is a difference relation over live terms, so `cap(a)` needs no
subtraction in the fact domain. *Amends:* [ENT-3.S6], which today has one
`slice_of` row; these are three rows of the same kind. *Law:* L10, L14, L15.

**[VIEW-3] View provenance is slice provenance.** Every view value carries the
finite origin set [OWN-5] defines for slices, formed and preserved by the same
sentences: formation makes a singleton, and binding, moving, passing, and
returning preserve the set. An access through a view is judged as one access
through every origin. *Amends:* [OWN-5] by generalizing "`slice<'r, T>` value" to
"view value"; no clause of it changes shape. *Law:* L10.

**[VIEW-4] `MutSpan`'s length is fixed by its type.** No `[SEQ]` operation takes a
`MutSpan` and produces a different length, and none takes one and changes its
owner's length. This is a closed property of the operation table, readable from
the type alone, and it is what [CALL-3] consumes. *Law:* L11.

**[VIEW-5] `AppendView` is a spare window.** Its `base` is the owner's length at
formation and is not a source-visible value. `len(a)` counts what this view
appended. Every `[SEQ]` operation on an `AppendView` acts on `[base + i]` for
`0 <= i < len(a)`; `seq_truncate` on an `AppendView` may reduce `len(a)` to zero
and no further. A callee that receives an `AppendView` therefore cannot reduce
its caller's `len(v)`, which is why [CALL-3] can leave the caller's length facts
alive. *Law:* L14.

**[VIEW-6] `absorb` is the commit event.**

```wf-design
let written = absorb(view: move a);
```

`absorb` consumes the `AppendView`, ends its append window, and returns `own u64`.
Its checker judgment, in this order, mirrors [ENT-3.S5]'s commit-value discipline
for a `set`:

1. the operand's origin set is resolved to one owner place `P` ([VIEW-7]);
2. the result value is bound to the compiler-owned commit value `w`, with
   `w = len(a)` established at it;
3. every fact supported by `len(P)` dies, under [ENT-5] clause (a), as a
   whole-place length event on `P`;
4. only then are `written = w` and `len(P) = old + w` established, where `old` is
   the term the state held for `len(P)` immediately before step 3 when one was
   derivable, and no relation is established when none was.

Step 4's `old + w` is a three-term relation and therefore not an L0 difference
bound. Two derivable cases carry it: `old` derivable as a constant `k` gives
`len(P) = k + w`, a difference bound over `w`; otherwise the checker retains
`len(P) - old >= 0` and `room(P) = old_room - w` is likewise carried only when
`old_room` is a constant. This is honest, narrow, and is open question Q9.
*Law:* L10, L14.

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
unchanged, so the abandoned elements are neither leaked nor double-dropped, and
no fact about `len(P)` was ever published. Not absorbing is therefore a
well-defined, safe program that discards work, which is what makes `absorb` an
ordinary operation rather than a must-use obligation. *Law:* L13, L14.

**[VIEW-9] Views are never stored** [STOR-5], and never returned except under
[VIEW-10]. *Law:* L10.

**[VIEW-10] View return provenance.** [FN-1]'s slice-result ceiling applies
unchanged to each view type: a function whose written result is `own Span<'r, T>`
(respectively `MutSpan`, `AppendView`, `Builder`) has the ceiling containing
`immutable-const` and the formal-view origin of every parameter whose written
mode and type are exactly that same view type with the same formal region and
element type. A borrow-mode result of direct view type stays rejected for
[FN-1]'s stated reason: two provenance relations, one summary. *Amends:* [FN-1]
by generalizing "slice" to "view". *Law:* L10.

### 3.7 `[CALL]`: what survives a call

This is D1's section. Exactly three transports exist, and **each reads only the
callee's declared parameter modes and types and its declared contract.** These
are the owner's three call rules of 2026-09-03.

**[CALL-1] Through a shared borrow, every fact survives.** For an argument whose
parameter mode is `&'r`, of any type, container and view included, the call is
not a kill event for any fact supported by the actual's resolved place. Ground:
[OWN-5] admits no write through a shared holder, so [EFF-2] can project no
`writes` occurrence onto that place, so [ENT-5] clause (b) does not fire.
*Verified today* for `&'a buffer<u8>`: probe `p6` (section 6) keeps
`len(line) = 10` across the call and the subsequent `line[9_u64]` is accepted.
*Law:* L11.

**[CALL-2] Through a value passed and returned, only the contract's facts exist
on the result.** An `own` argument is a consuming use [OWN-1], so [ENT-5] clause
(c) kills every fact whose support contains that binding's root. The result is a
fresh binding carrying exactly the callee's [FN-9]-verified relations under
[ENT-3.S12], and nothing else. In particular `len`, `cap` and `room` of the
result are unknown unless the callee's `ensures` states them.
*Verified today:* probe `p1` (section 6), `passthrough(out: move a)` returning
the same buffer, then `b[9_u64]`, is **rejected** with residual `9_u64 < len(b)`.
The transport this design needs already behaves correctly; what is missing is the
contract vocabulary to publish across it, which is [CALL-4]. *Law:* L11.

**[CALL-3] An element write through a length-fixed view never touches length
facts.** For an argument whose parameter's declared type is `MutSpan<'r, T>`,
`&uniq 'r MutSpan<'r, T>`, or `Builder<'r, T>`, the types [VIEW-4] and [BLD-1]
fix a length for, a projected callee `writes` occurrence kills every fact whose
support overlaps the viewed **element storage** and kills no length term over
that origin. For an argument whose parameter's declared type is
`AppendView<'r, T>`, the same holds, and in addition: the callee cannot decrease
the owner's length (L14, [VIEW-5]), so the caller's `len(v)` facts survive; and
it cannot increase it either, because only `absorb` publishes an increase and
[VIEW-7] denies `absorb` to a callee. For every other parameter type the
projected write kills length facts as an ordinary whole-place event. *Law:* L11,
L14.

**[CALL-4] Contract vocabulary for containers and views.** [FN-9]'s clause
grammar is extended by exactly three admissions, and by nothing else:

1. a clause operand may be `len(P)`, `cap(P)` or `room(P)` where `P` is an
   admitted formal place of container or view type (today: `len(P)` only, and
   only for `array`, `slice`, `buffer`);
2. a clause operand may be `len(result)`, `cap(result)` or `room(result)` when
   the written result type is a container or view type; this is the admission
   that makes `own`-in / `own`-out contracts possible at all, since today's
   result-datum restriction to fragment integers forbids it;
3. one comparison operand may be `t + k` for an admitted term `t` and a
   **constant** `k` (literal or named integer const), because `ile(x, y + k)`
   normalizes to the difference bound `x - y <= k`. A non-constant offset is not
   admitted here and is open question Q8.

So the canonical append contract is writable:

```wf-design
fn append_span['i, 'o](
  input: own Span<'i, u8>,
  output: own AppendView<'o, u8>
) -> written: own AppendView<'o, u8> reads(input), writes(output) contract {
  requires ile(len(input), room(output));
  ensures ile(len(written), cap(output));
} { ... }
```

The clause is single-state: `output` denotes the entry image of the parameter and
`written` the result, both under [FN-9]'s existing entry-image machinery, with no
second state, no `old()`, and no frame rule. Two-state `ensures` is rejected by
the owner and is not proposed anywhere in this design.
*Verified today:* probe `p4` (section 6) compiles a single-state `ensures`
anchored on `len(deref(destination))` with a fragment result, so the entry-image
half works. Probe `p2` shows `len(result)` does not parse today ([GRAM-9]), which
is why admission 2 is an amendment. *Law:* L11.

**[CALL-5] Multi-return.** A function may declare an ordered result tuple:

```wf-design
fn split_two['r](source: own MutSpan<'r, u8>, at: own u64)
  -> (head: own MutSpan<'r, u8>, tail: own MutSpan<'r, u8>) ...

let (head, tail) = split_two<'r>(source: move whole, at: 16_u64);
```

Each element has its own mode, type, and, under [CALL-4], its own contract
relations; the destructuring `let` binds each as an ordinary fresh binding. The
result is not a value: there is no tuple type, no tuple place, and no way to
store or pass one. It is a return-and-bind form only, which keeps [STOR-5] and
[TYPE-2] untouched. Multi-return is load-bearing, not a convenience: `seq_take`
must return an owner and an element, and no single value can carry both, since an
enum payload or struct field holding a view is refused by [STOR-5].
*Verified today:* the syntax does not parse ([GRAM-2], probe `p8`), so this is new
syntax. *Law:* L10.

**[CALL-6] No transport reads the actual's spelling.** The three transports above
are selected by the callee's declared parameter mode and type and by its declared
contract. No rule of this design consults the argument expression's shape, the
callee's body, its name, or any per-parameter summary derived from its body. A
parameter type for which no transport is selected kills conservatively.

*This is D1 stated as a rule.* The located mechanism of D1, `argument_referent`
returning `element = true` for every `&uniq buffer<T>` actual
(`compiler/src/semantic/places.rs:349-355`), is a fact derived from the actual's
shape, and under CALL-6 no such fact exists to be derived. The precision it was
buying is bought instead by the type: a `MutSpan` argument is element-only
**because its type admits nothing else**. Applying CALL-6 to the residual
`&uniq buffer<T>` spelling yields `element = false`, which is exactly the sweep's
minimal sound repair, and is why the D1 conformance case turns XPASS at [OP-4] in
the batch that lands this family (section 7). *Law:* L11.

**[CALL-7] Reinitializing `set`: the statement L10 requires.** A consume-and-return
call cannot be written into a loop today. [SET-1] and [STOR-1] refuse an affine
`set` target, [SET-2]'s `replace` refuses a right-hand side that moved the target
root, and [OWN-1] says reinitialization requires a new `let`, which a loop body
cannot use because the next iteration needs the previous iteration's value. Both
halves are verified: probe `p10` is [STOR-1] `AffineSetTarget` and probe `p11` is
[OWN-1] `UseAfterMove` (section 6). So:

```wf-design
set buf = collect(source: move line, out: move buf);
```

`set p = e;` is additionally admitted when `p` is a bare local binding of affine
type **whose current value has already been consumed** (by `e` itself, or by an
earlier statement of the same lexical block) and `e` produces exactly `p`'s type.
Its judgment: evaluate `e` under ordinary rules, including the consume of `p`
inside it; every fact whose support contains `p`'s root dies at that consume
([ENT-5] clause (c)); then the binding is reinitialized with `e`'s value, live
and usable, with no observable program point between. It derives no drop and no
release, because the target holds no value, exactly as [SET-2]'s commit derives
none. Its [ENT-3] image is [SET-1]'s commit-value discipline with the kill
already performed by the consume.

The premise is one fact the checker already tracks: **the target is dead**.
[STOR-1]'s existing rejection of a `set` on a *live* affine place keeps its exact
wording and its `replace` mechanical fix; only the dead case is added. Because
the premise is deadness rather than "the right-hand side is a call", the same
statement also carries an owner out of a multi-return binding into a loop-carried
name:

```wf-design
let (rest, next) = seq_try_take(vector: move pending);
set pending = move rest;
```

*Amends:* [OWN-1] (one reinitialization route that is not a new `let`), [STOR-1]
and [SET-1] (whose affine-target rejections are narrowed to a live target), and
nothing in [GRAM-4]: the statement form already exists and only its premise
widens, so the language gains no second way to write an assignment. It is the sole
writer-facing cost of L10, and it buys the whole write-back story: the advanced
length reaches the owner because it *is* the value, and this `set` is where it
lands. Lowering is the ABI note the owner already made: a parameter moved in and
returned on every path is passed by pointer, so
`set view = seq_push(view: move view, value: byte);` lowers to a store and a
length increment on one in-place descriptor, with no copy. *Law:* L10.

### 3.8 `[SEQ]`: the operation table

One operation family per row, resolved by name and then by receiver type within
the family, exactly as `len` and `slice_of` resolve today [OP-1]. Constructors
carry distinct names rather than one overloaded `seq_empty`, because selecting a
row by the result type would be expected-type selection, which [TYPE-5] forbids
and [OP-1] refuses. `V` ranges over the four owners; `Prov` is the owner's
provider (`&uniq 'h Heap` for `HeapVector`, the arena or pool provider
otherwise), fixed by `[PROV-1]`.

```text
| id      | op              | receiver     | signature                                                                                   |
|---------|-----------------|--------------|---------------------------------------------------------------------------------------------|
| SEQ-1   | seq_fixed<T,N>  | -            | () -> own FixedVector<T, N>                                                                   |
| SEQ-2   | seq_heap<T>     | -            | () -> own HeapVector<T>                                                                       |
|         | seq_arena<'r,T> | -            | () -> own ArenaVector<'r, T>                                                                  |
| SEQ-3   | seq_lease       | -            | (pool: &uniq 'p Pool<'p, FixedVector<T,N>, K>) -> own Result<PoolVector<'p,T,N>, PoolExhausted>|
|         | seq_lease_proved| -            | (pool: &uniq 'p Pool<'p, FixedVector<T,N>, K>) -> own PoolVector<'p, T, N>                     |
| SEQ-4   | seq_len         | owner, view  | (v: &'r V) -> own u64                                                                         |
| SEQ-5   | seq_push        | AppendView   | (view: own AppendView<'r,T>, value: own T) -> own AppendView<'r, T>                            |
| SEQ-6   | seq_try_push    | AppendView   | (view: own AppendView<'r,T>, value: own T) -> (rest: own AppendView<'r,T>, unplaced: own Option<T>) |
| SEQ-7   | seq_pop         | AppendView   | (view: own AppendView<'r,T>) -> (rest: own AppendView<'r,T>, value: own T)                     |
| SEQ-8   | seq_truncate    | AppendView   | (view: own AppendView<'r,T>, keep: own u64) -> own AppendView<'r, T>                           |
| SEQ-9   | seq_place       | owner        | (vector: own V<T>, value: own T) -> own V<T>                                                   |
| SEQ-10  | seq_try_place   | owner        | (vector: own V<T>, value: own T) -> (rest: own V<T>, unplaced: own Option<T>)                  |
| SEQ-11  | seq_take        | owner        | (vector: own V<T>) -> (rest: own V<T>, value: own T)                                           |
| SEQ-12  | seq_try_take    | owner        | (vector: own V<T>) -> (rest: own V<T>, value: own Option<T>)                                   |
| SEQ-13  | p[i]            | owner, Span, | element place                                                                                 |
|         |                 | MutSpan      |                                                                                               |
| SEQ-14  | seq_get         | owner, Span, | (v: &'r V, index: own u64) -> own Option<T>,  T copy                                           |
|         |                 | MutSpan      |                                                                                               |
| SEQ-15  | seq_span        | owner        | (&'r v) -> own Span<'r, T>                                                                     |
| SEQ-16  | seq_mut_span    | owner        | (&uniq 'r v) -> own MutSpan<'r, T>                                                             |
| SEQ-17  | seq_append_view | owner        | (&uniq 'r v) -> own AppendView<'r, T>                                                          |
| SEQ-18  | absorb          | AppendView   | (view: own AppendView<'r,T>) -> own u64                                                        |
| SEQ-19  | seq_reserve     | HeapVector,  | (vector: own V<T>, provider: Prov, additional: own u64)                                        |
|         |                 | ArenaVector  |   -> own Result<V<T>, OutOfMemory<V<T>>>                                                       |
| SEQ-20  | seq_clear       | owner        | (vector: own V<T>) -> own V<T>                                                                 |
| SEQ-21  | seq_shrink      | HeapVector   | (vector: own HeapVector<T>, heap: &uniq 'h Heap) -> own HeapVector<T>                          |
```

Requirements, published facts, effects and failures:

```text
| id      | requires                       | publishes                                                    | effects                        | failure            |
|---------|--------------------------------|--------------------------------------------------------------|--------------------------------|--------------------|
| SEQ-1   | -                              | len = 0, cap = N, room = N                                    | pure                           | none               |
| SEQ-2   | -                              | len = 0, cap = 0, room = 0                                    | pure                           | none: an empty     |
|         |                                |                                                              |                                | growable owns no   |
|         |                                |                                                              |                                | backing            |
| SEQ-3   | proved form: ilt(live(pool),   | on Ok(value: r): len(r) = 0, cap(r) = N, room(r) = N          | allocates(pool),               | typed (checked     |
|         | capacity(pool))                |                                                              | writes(pool)                   | form only)         |
| SEQ-4   | -                              | n = len(v)                                                    | reads(v)                       | none               |
| SEQ-5   | igt(room(view), Z)             | len(result) = len(view) + 1, room(result) = room(view) - 1    | writes(view)                   | none, total        |
| SEQ-6   | -                              | ile(len(rest), cap(rest))                                     | writes(view)                   | Some returns the   |
|         |                                |                                                              |                                | value unconsumed   |
| SEQ-7   | igt(len(view), Z)              | len(rest) = len(view) - 1, room(rest) = room(view) + 1        | writes(view)                   | none               |
| SEQ-8   | ile(keep, len(view))           | len(result) = keep                                            | writes(view)                   | none; drops        |
|         |                                |                                                              |                                | [keep,len) desc.   |
| SEQ-9   | igt(room(vector), Z)           | len(result) = len(vector) + 1, room(result) = room(vector) -1 | -                              | none, total        |
| SEQ-10  | -                              | ile(len(rest), cap(rest))                                     | -                              | Some returns the   |
|         |                                |                                                              |                                | value unconsumed   |
| SEQ-11  | igt(len(vector), Z)            | len(rest) = len(vector) - 1                                   | -                              | none               |
| SEQ-12  | -                              | ile(len(rest), len(vector))                                   | -                              | None when empty    |
| SEQ-13  | ilt(i, len(p))  [OP-4]         | -                                                            | per access                     | none               |
| SEQ-14  | -                              | -                                                            | reads(v)                       | None out of range  |
| SEQ-15  | -                              | len(s) = len(v)                                               | pure                           | none               |
| SEQ-16  | -                              | len(m) = len(v)                                               | pure                           | none               |
| SEQ-17  | -                              | len(a) = 0, room(a) = room(v), cap(a) + len(v) = cap(v)        | pure                           | none               |
| SEQ-18  | -                              | [VIEW-6]                                                     | writes(view)                   | none               |
| SEQ-19  | -                              | on Ok(value: r): ige(room(r), additional), len(r)=len(vector) | allocates(provider),           | typed; on Err the  |
|         |                                |                                                              | writes(provider)               | vector returns     |
|         |                                |                                                              |                                | inside the error   |
| SEQ-20  | -                              | len(result) = 0, room(result) = cap(result)                   | -                              | none; drops        |
|         |                                |                                                              |                                | [0,len) descending |
| SEQ-21  | -                              | len(result) = len(vector)                                     | allocates(heap), writes(heap)  | none; on failure   |
|         |                                |                                                              |                                | keeps the larger   |
|         |                                |                                                              |                                | backing            |
```

`seq_place`, `seq_try_place`, `seq_take`, `seq_try_take`, `seq_clear` carry the
release row of any element they drop plus, for `PoolVector`, `writes(provider)`
from [CNT-5]; for a copy element type on a non-pool owner their row is `pure`.

Notes on the table:

- **[SEQ-5] is the operation the whole design exists for.** It is total,
  allocation-free on every backing, and lowers to `store` plus `len + 1` with no
  capacity branch, because its requirement is discharged before lowering. The
  writer calls a total `push` and the checker proves the requirement; the proof
  never rewrites a `Result`-returning operation into a `unit`-returning one.
- **There is no growing `push` anywhere.** A writer who wants push-with-growth
  writes the shell: reserve, form the view, push, absorb (L4).
- **The owner-level rows [SEQ-9] to [SEQ-12] are value-in, value-out.** They
  exist because an `AppendView` can only remove what it appended (L14), so a work
  queue or an object table needs a way to add and remove elements with no view in
  play. They are the owner's `set buf = f(... move buf)` shape at the table level,
  and they are what [CALL-2] transports.
- **[SEQ-19] returns the vector inside its error**, so a failed reserve loses
  nothing and changes nothing. The order is fixed: compute the new capacity and
  discharge its arithmetic and allocation-domain obligations [OP-9]; acquire;
  move elements; commit the descriptor; release the old backing. Nothing
  observable changes before the acquisition succeeds.
- **No row reads `cap` or `room` as a value** (L15).
- `ige(room(r), additional)` in [SEQ-19] is a two-term difference bound and needs
  no clause grammar at all, because a table operation's published facts are
  [ENT-3] rows rather than [FN-9] clauses. That is precisely why `room` was
  introduced: the same guarantee written as `ige(cap(r), len(r) + additional)`
  would be a three-term relation, which is open question Q8 for user functions.

### 3.9 `[BLD]`: the `par` builder

The problem [PAR-2] cannot solve as stated: a counted loop cannot share one
`AppendView`, because every iteration would write one `len`. The answer is to
reserve first and then give each iteration a slot it can prove is its own.

**[BLD-1] `Builder<'r, T>`** is the fourth view type: a claimed, write-once range
of a sequence's spare window. `len(b)` is its slot count and is fixed at
formation, so `Builder` is a length-fixed type under [CALL-3].

```text
seq_claim(view: own AppendView<'r,T>, count: own u64) -> own Builder<'r, T>
    requires ile(count, room(view))                       len(b) = count
builder_set(slots: &uniq 'b Builder<'r,T>, index: own u64, value: own T) -> unit
    requires ilt(index, len(slots))                       writes(slots)
seq_finish(builder: own Builder<'r,T>) -> own AppendView<'r, T>
    requires the coverage certificate [BLD-3]             len(result) = len(view) + len(builder)
```

**[BLD-2] Write-once slots.** A `Builder` slot is written at most once: [BLD-3]
gives each iteration a distinct index, and `builder_set` has no read path, so no
slot is observed before it is written. Elements written into a `Builder` and
never finished are dropped by the `Builder`'s release action, exactly as [VIEW-8]
does for an `AppendView`. *Law:* L12, L13.

**[BLD-3] The coverage certificate.** `seq_finish` is admitted only when its
operand's sole write history is one counted `for_stmt` whose body contains
exactly one `builder_set` on it, whose `index` actual is that loop's binder under
[PAR-2]'s retained affine map with `a = 1, b = 0`, and whose loop range is exactly
`0_u64..len(b)` with both endpoints admitted [ENT-2] terms. The certificate is
then: distinct binder values give distinct indices ([PAR-2]'s own argument), and
the half-open range covers `[0, len(b))` exactly. A violation is a hard error
citing BLD-3 at the `seq_finish` operand, with the restructuring `fill every
claimed slot in one counted loop indexed directly by its binder, or claim only
the slots you fill`. This is deliberately the narrowest rule that admits the
shape the owner named and refuses everything else rather than starting a search.
It is the weakest rule in this design and is open question Q10. *Law:* L12.

**[BLD-4] `par` permission comes from [PAR-2] unchanged.** `builder_set` is a
call whose `writes(slots)` projects to the builder's element storage; the
`&uniq 'b Builder` argument is rooted in a binding declared outside the loop, so
today [PAR-2] would deny it on the exclusive-loan condition. The single amendment
is: **a `&uniq` loan on a `Builder` whose only body use is one `builder_set` under
[BLD-3]'s map is refined to the single-element range `[i, i+1)`**, exactly as
[PAR-2] already refines a direct subscript write. No other condition of [PAR-2]
changes, and the accumulator, endpoint, and exit conditions are untouched.
*Law:* L12.

### 3.10 The pool seam, resolved

`CONTAINERS.md` flagged one seam and did not decide it: `Pool<'p, T, N>` names
`N` interchangeable single-`T` slots, and a `PoolVector` needs one **contiguous
run** of them. The resolution is `RESOURCES.md`'s, because the answer is a
resource question and L6 decides it.

A pool that serves *runs* of `k` slots is not a uniform-slot domain: whether a
run of 3 is serviceable is not decided by the live count, and L6's sixteen-byte
counterexample reappears one level down, at slot granularity. Adding a run-lease
to `Pool` would take the pool out of [RES-6]'s admitted domains and out of every
envelope, which is the opposite of what the container wants from it.

The shape that keeps the algebra is to lease **one slot whose content is the
run**:

```wf-design
region 'p {
  let blocks = pool_static<'p, FixedVector<Record, 256>, 8>();
  match seq_lease<'p, Record, 256, 8>(pool: &uniq 'p blocks) {
    Err(error: exhausted) => { ... }
    Ok(value: leased) => { ... }
  }
}
```

The pool still holds eight interchangeable slots of one type, `live < 8` still
decides serviceability, and `PoolVector<'p, Record, 256>` is exactly a lease of
such a slot. A `FixedVector` is frame-resident storage [STOR-1] and is not
region-bearing, so it is a legal slot content type, and its initialized-prefix
typestate lives inside the value where [CNT-2] already keeps it.

Two consequences, both recorded rather than hidden. **The capacity is fixed at
reservation, not at lease**, so `PoolVector` carries `N` in its type and
`seq_lease` takes no runtime capacity argument; `CONTAINERS.md`'s inventory row
("`cap` fixed at lease") is superseded here. And a program wanting two block
sizes reserves two pools, so `E` names both, which is exactly the shape L6 says
an envelope has to have.

### 3.11 One name per concept

Where the two drafts used different names, this is what the design uses and why.

```text
| concept                   | RESOURCES.md        | CONTAINERS.md      | chosen              | why                                            |
|---------------------------|---------------------|--------------------|---------------------|------------------------------------------------|
| construct an empty owner  | fixed_vector_new    | seq_fixed<T,N>     | seq_fixed<T,N>      | one prefix names one operation family, so a row |
|                           | heap_vector_new     | seq_heap<T>        | seq_heap<T>         | is selected by name and receiver type and never |
|                           |                     |                    |                     | by expected result type [OP-1, TYPE-5]          |
| append one element        | fixed_vector_push   | seq_push           | seq_push            | the same family; the backing is in the receiver |
|                           |                     |                    |                     | type, not in the operation name                 |
| remove one element        | fixed_vector_pop    | seq_pop            | seq_pop (view) and  | the view row cannot remove what another view    |
|                           |                     |                    | seq_take (owner)    | appended (L14), so an owner row is needed       |
| read-only view            | slice<'r, T>        | Span<'r, T>        | Span<'r, T>         | the rename is the whole change to it            |
| lease a pool block        | pool_take on a      | seq_lease with a   | seq_lease with no   | capacity comes from the pool's slot type        |
|                           | FixedVector slot    | runtime capacity   | capacity argument   | (section 3.10)                                  |
| pool-backed sequence      | (not named)         | PoolVector<'p, T>  | PoolVector<'p,T,N>  | N is a constant of the pool's slot type         |
| growth failure            | OutOfMemory,        | ResourceError      | OutOfMemory<V>,     | L3 requires the failure to hand back the affine |
|                           | OutOfMemory<T>      |                    | PoolExhausted<T>,   | inputs, so a payload-carrying family wins over  |
|                           |                     |                    | NeedCapacity<T>,    | one opaque union                                |
|                           |                     |                    | Full<T>, TooSmall   |                                                 |
| rebind a consumed owner   | (not named)         | rebind p = e;      | set p = e;          | the owner's own spelling of 2026-09-03; the     |
|                           |                     |                    |                     | premise is deadness, so the language gains no   |
|                           |                     |                    |                     | second assignment form                          |
| spare capacity of a view  | (not named)         | cap(a) with a      | room(a)             | makes "there is spare for this whole source" a  |
|                           |                     | subtraction        |                     | two-term difference bound                       |
| the property              | resource-closed     | (deferred)         | resource-closed     | the owner offered res-closed or a better name;  |
|                           | resource_closed     |                    | resource_closed     | the long spelling is the one in use             |
| the failure variant field | Err(value: e)       | Err(value: e)      | Err(error: e)       | [PRE-1] declares Err(error: E); both drafts     |
|                           |                     |                    |                     | wrote the wrong field name                      |
```

### 3.12 Amendment register

Every existing v0.40 rule this design changes or retires, and how. Line numbers
are `spec/kernel-spec.md` at a40c7e70.

```text
| rule           | line      | change                                                                    | by                |
|----------------|-----------|---------------------------------------------------------------------------|-------------------|
| [SCOPE-3]      | 27-31     | heap exhaustion leaves the deferred set (it becomes a value); stack        | [RES-7], [STK-5], |
|                |           | exhaustion leaves it for resource-closed programs only; startup-resource   | [RUN-5]           |
|                |           | failure stays outside, by name                                            |                   |
| [TYPE-2]       | 352       | three opaque provider nominals join the opaque system class; slot<'p,T>    | [PROV-1], [CNT-1],|
|                |           | joins the region-bearing types; +4 owners and +4 views; buffer<T> retires  | [VIEW-1]          |
|                |           | from the writer surface and survives as HeapVector's compiler-owned        |                   |
|                |           | backing; slice<'r,T> is renamed Span<'r,T>                                 |                   |
| [SET-1]        | 500       | the affine-target rejection is narrowed to a live affine target            | [CALL-7]          |
| [SET-2]        | 508       | unchanged; a slot<'p,T> is region-bearing, so a stored one is refused      | -                 |
| [OWN-1]        | 558       | providers, slots, owners and views are affine; one reinitialization route  | [PROV-1],[VIEW-1],|
|                |           | that is not a new let                                                     | [CNT-1], [CALL-7] |
| [OWN-5]        | 580       | "slice value" generalizes to "view value" throughout; no clause changes    | [VIEW-3]          |
|                |           | shape; a provider is an ordinary place and exclusivity does the work       |                   |
| [CAP-1]        | 1962      | unchanged, and deliberately: providers add no capability category, no      | [PROV-1]          |
|                |           | permission kind, and no second interference vocabulary                    |                   |
| [STOR-1]       | 670       | the four owners join the storage-class table; buffer<T>'s sentence and the | [CNT-1], [CNT-5], |
|                |           | growable-collection paragraph are superseded in place; the affine-set      | [CALL-7]          |
|                |           | rejection is narrowed to a live target                                    |                   |
| [STOR-3]       | 683       | +1 release row for a pool slot (contributing writes(pool)); the owner      | [PROV-7], [CNT-5] |
|                |           | release actions of [CNT-5] supersede the buffer<T> drop sentence          |                   |
| [STOR-4]       | 716       | unchanged; Arena, Pool, slot and the two region-bearing owners obey it     | [PROV-1], [CNT-6] |
| [STOR-5]       | 718       | Arena<'p>, Pool<'p,T,N>, slot<'p,T>, ArenaVector, PoolVector and all four  | [PROV-1], [CNT-6],|
|                |           | views join the region-bearing set; Heap is not region-bearing              | [VIEW-1]          |
| [STOR-6]       | 733-761   | the "no numeric frame ceiling" sentence keeps its scope; a computed        | [STK-3]           |
|                |           | per-context stack envelope is added for resource-closed programs,          |                   |
|                |           | measured post-codegen                                                     |                   |
| [OP-1]         | 793-798   | box_new and arena_new take a provider; buffer_new and buffer_vacant retire;| [PROV-3], [RES-8],|
|                |           | slice_of retires in favour of seq_span; eight provider rows and the        | [SEQ-*]           |
|                |           | complete [SEQ] table are added                                            |                   |
| [OP-4]         | 880       | indexable bases extend to the four owners, Span and MutSpan; the           | [CNT-3]           |
|                |           | obligation is against len, never cap                                      |                   |
| [OP-9]         | 968       | retained; buffer_fits stays a representability predicate and never becomes | [RES-8]           |
|                |           | an availability predicate; it is the allocation-domain predicate the       |                   |
|                |           | growing rows use                                                          |                   |
| [FN-1]         | 999       | the slice-return ceiling generalizes to a view-return ceiling; the result  | [VIEW-10],        |
|                |           | shape admits an ordered result list                                       | [CALL-5]          |
| [FN-6]         | 1205      | unchanged: recursion stays permitted; it merely excludes a program from    | [STK-2]           |
|                |           | [RES-4]                                                                   |                   |
| [FN-7]         | 1210-1253 | one new input row command.heap; one new entry marker resource_closed;      | [PROV-5], [RES-4] |
|                |           | main's effect row admits allocates over its own labelled provider          |                   |
| [FN-9]         | 1295      | clause operands admit len(P), cap(P), room(P) on container and view formal | [CALL-4],         |
|                |           | places and on a container or view result; one operand may be t + k for a   | [CALL-5]          |
|                |           | constant k; the result shape admits an ordered result list                 |                   |
| [EFF-1]        | 1363-1372 | allocates takes formal-rooted paths; the effect-row atoms heap and arena   | [PROV-4]          |
|                |           | retire (META-5: unique fixed lowercase grammar atoms -2)                   |                   |
| [EFF-2]        | 1386      | one exception to the empty-release-row sentence, for pool slots; "slice    | [PROV-7],         |
|                |           | parameter names the backing" generalizes to "view parameter"               | [VIEW-3]          |
| [PROG-3]       | 1499      | the start-time obligation includes materializing E; ProgramFinished is     | [RUN-5]           |
|                |           | named                                                                     |                   |
| [PAR-1]        | 1969      | the allocates(arena 'r) region clause becomes the ordinary provider-place  | [RUN-4]           |
|                |           | projection                                                                |                   |
| [PAR-1/2/3]    | 1989,2024,| "execution-resource exhaustion is a [SCOPE-3] condition" gains the         | [RUN-3]           |
|                | 2049      | resource-closed case, in which it is unreachable                           |                   |
| [PAR-2]        | 1994      | +1 refinement: a &uniq loan on a Builder under [BLD-3]'s map refines to    | [BLD-4]           |
|                |           | the single-element range [i, i+1)                                          |                   |
| [SYS-2]        | 2264      | "no system operation allocates" is kept and strengthened: an adapter's own | [RUN-2]           |
|                |           | records come from E                                                       |                   |
| [SYS-8]        | 2482      | retired in its buffer form: read_at, write_once, directory_next,           | [CNT-1], [VIEW-1] |
|                |           | host_copy_bytes, host_copy_utf8, open_directory and open_file take         |                   |
|                |           | MutSpan<'r,u8> or Span<'r,u8>; the start <= end and end <= len obligations |                   |
|                |           | are unchanged in form, with len(deref(buffer)) becoming len(view). This is |                   |
|                |           | the change that lets a heap-free program do I/O                            |                   |
| [ENT-2]        | 2671      | length terms extend to owners and views; cap(P) and room(P) are added as   | [CNT-2]           |
|                |           | terms of the same class                                                   |                   |
| [ENT-3.S5]     | 2724      | +1 source: absorb's commit value                                          | [VIEW-6]          |
| [ENT-3.S6]     | 2724      | +3 rows for the three view formations                                     | [VIEW-2]          |
| [ENT-5] (b)    | 2857      | superseded for containers and views by [CALL-1..3] and [CALL-6]; the       | [CALL-1], [CALL-3]|
|                |           | "element write never kills a length fact" sentence keeps its meaning for   | [CALL-6]          |
|                |           | array and gains a type-derived premise everywhere else                     |                   |
| [ENT-1],       | 2642,     | OPEN, and one amendment rather than several: length-class terms defined    | Q6, Q17           |
| [ENT-2],       | 2671,     | once in the term language with their support and kills [ENT-2, ENT-5], and |                   |
| [ENT-5],       | 2857,     | one numeric goal disposition shared by every consumer of a relation, with  |                   |
| [ENT-6],       | 2963,     | the per-family route lists retired from [ENT-6]. Not resolved by this      |                   |
| [INV-1]        | 3095      | design; section 5's Q6 states it                                          |                   |
| batch 0079     | docs/done/| the heap-refusal abort site loses its last reachable caller; the stack     | [RES-7]           |
| exhaustion     | 0079-...  | guard-page record survives only for programs that are not resource-closed  |                   |
| floor          |           |                                                                           |                   |
```

Retired outright, with no successor: the writer-facing `&uniq buffer<T>` and
`&uniq Container` state-borrow forms ([CNT-8]); `buffer_vacant`'s
`Option`-element construction, which [CNT-4] makes unnecessary (L12); and the
effect-row atoms `heap` and `arena` ([PROV-4]).

---

## 4. Two worked programs

Both are **design text**. Every form this design adds compiles nowhere: the
`resource_closed` entry marker, the provider types and their operations, the
`command.heap` input row, `allocates` over a path, `live`/`capacity`/`remaining`,
the container and view types, the `[SEQ]` rows, `absorb`, multi-return, the
reinitializing `set`, and `room(P)`. Where a body is elided the text says so.
Byte figures in the envelopes are illustrative; no implementation computed them,
and none was measured.

### 4.1 A kernel program with the heap absent

A fixed run queue of tasks, a 256-byte UART transmit ring fed by a polled
interrupt-style producer, and a 64-page pool with typed exhaustion. No heap, no
recursion, an acyclic call graph, and a scheduler loop whose resource state is
restored on every backedge.

```wf-design
struct Task {
  kind: u32;
  arg: u64;
}

struct Page {
  bytes: array<u8, 4096>;
}

struct UartRing {
  bytes: array<u8, 256>;
  head: u64;
  fill: u64;
}

fn ring_new() -> result: own UartRing pure {
  doc "Builds one empty transmit ring.";
  return UartRing(bytes: array_new<u8, 256>(0_u8), head: 0_u64, fill: 0_u64);
}

fn ring_index(head: own u64, fill: own u64) -> result: own u64 pure contract {
  define head_within = ilt(head, 256_u64);
  define fill_within = ile(fill, 256_u64);
  requires head_within;
  requires fill_within;
  ensures ilt(result, 256_u64);
} {
  doc "Computes one wrapped write position of a 256-byte ring.";
  let at = head + fill;
  invariant sum_bound: ilt(at, 512_u64);
  let over = ige(at, 256_u64);
  if over {
    let wrapped = at - 256_u64;
    invariant wrapped_bound: ilt(wrapped, 256_u64);
    return wrapped;
  }
  return at;
}

fn ring_push['r](ring: &uniq 'r UartRing, byte: own u8) -> result: own unit
    reads(ring), writes(ring) {
  doc "Appends one byte, overwriting the oldest when the ring is full; overwriting is this ring's defined semantics, not a resource failure.";
  let head = deref(ring).head;
  let fill = deref(ring).fill;
  let head_ok = ilt(head, 256_u64);
  if head_ok {
    let full = ige(fill, 256_u64);
    if full {
      let rotated = ring_index(head: head, fill: 1_u64);
      set deref(ring).bytes[head] = byte;
      set deref(ring).head = rotated;
    } else {
      let at = ring_index(head: head, fill: fill);
      set deref(ring).bytes[at] = byte;
      set deref(ring).fill = fill + 1_u64;
    }
  }
  return unit;
}

fn pump['r](ring: &uniq 'r UartRing, tick: own u64) -> produced: own u64
    reads(ring), writes(ring) {
  doc "Drains one edge of the transmit source into the ring; this stands in for an interrupt handler and is called by the scheduler, not by the device.";
  let low = cvt<u64, u8>(iand(tick, 255_u64));
  match low {
    Err(error: narrowed) => {
      return 0_u64;
    }
    Ok(value: byte) => {
      let pushed = ring_push(ring: ring, byte: byte);
      return 1_u64;
    }
  }
}

fn render['u](page: &uniq 'u slot<'u, Page>, task: own Task) -> written: own u64
    reads(page), writes(page) contract {
  ensures ile(written, 4096_u64);
} {
  doc "Formats one task into the borrowed page and reports how many bytes it wrote.";
  ...elided: ordinary subscript writes into deref(page).bytes, each with its own [OP-4] obligation...
}

fn drain['u, 'r](page: &'u slot<'u, Page>, ring: &uniq 'r UartRing, count: own u64)
    -> sent: own u64 reads(page), writes(ring) {
  doc "Copies one prefix of the page into the transmit ring.";
  let limit = imin(count, 4096_u64);
  for @copy (at in 0_u64..limit) {
    let byte = deref(page).bytes[at];
    let pushed = ring_push(ring: ring, byte: byte);
  }
  return limit;
}

fn service['p, 'r](pages: &uniq 'p Pool<'p, Page, 64>, ring: &uniq 'r UartRing, task: own Task)
    -> sent: own u64 reads(pages, ring), writes(pages, ring), allocates(pages) {
  doc "Serves one task on a page it takes from the pool and returns before it leaves.";
  let blank = Page(bytes: array_new<u8, 4096>(0_u8));
  let attempt = pool_take_checked<'p, Page>(pool: pages, value: move blank);
  match attempt {
    Err(error: refused) => {
      let recovered = move refused.rejected;
      let noted = ring_push(ring: ring, byte: 33_u8);
      return 0_u64;
    }
    Ok(value: page) => {
      let written = 0_u64;
      let sent = 0_u64;
      region 'u {
        set written = render<'u>(page: &uniq 'u page, task: move task);
        set sent = drain<'u, 'r>(page: &'u page, ring: ring, count: written);
      }
      let returned = pool_release<'p, Page>(pool: pages, item: move page);
      return sent;
    }
  }
}

resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Runs a fixed scheduler over a static page pool and a transmit ring until the run queue empties.";
  let ring = ring_new();
  let pending = seq_fixed<Task, 32>();
  let (queued, unplaced) = seq_try_place(vector: move pending, value: Task(kind: 1_u32, arg: 0_u64));
  set pending = move queued;
  match unplaced {
    Some(value: rejected) => {
      return exit_status(code: 1_u8);
    }
    None() => {
    }
  }
  let tick = 0_u64;
  region 'p {
    let pages = pool_static<'p, Page, 64>();
    region 'r {
      loop @scheduler {
        let produced = pump<'r>(ring: &uniq 'r ring, tick: tick);
        set tick = tick +wrap 1_u64;
        let (rest, next) = seq_try_take(vector: move pending);
        set pending = move rest;
        match next {
          None() => {
            break @scheduler;
          }
          Some(value: task) => {
            let sent = service<'p, 'r>(pages: &uniq 'p pages, ring: &uniq 'r ring, task: move task);
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
```

#### The envelope the compiler publishes

Illustrative figures; the provenance column is the load-bearing part.

```text
E(scheduler.wf, <embedded target>, C = { W = 1 })

  region  static.image          bytes     1_024   align   8   contiguous
  region  static.pool.pages     bytes   262_144   align  16   contiguous
  stack   entry                 bytes     5_312   align  16   contiguous
  lanes                         count         1
  slots   task.records          count         0
  slots   completion.records    count         0
```

```text
| item                    | where it comes from                                             | rule            |
|-------------------------|-----------------------------------------------------------------|-----------------|
| static.image            | the const items and the static parts of the emitted module      | [STOR-6]        |
| static.pool.pages       | the one pool_static<'p, Page, 64> occurrence:                    | [PROV-8],       |
|                         | 64 * stride(Page) at align(Page)                                 | [RES-6]         |
| stack.entry             | main (the ring, the FixedVector<Task, 32> and the provider live  | [STK-3]         |
|                         | in this frame) + service (its 4096-byte staging Page) + render,  |                 |
|                         | measured post-codegen                                            |                 |
| lanes = 1               | no par in the program; the entry lane only                      | [RUN-3]         |
| task.records = 0,       | no par statement and no may-suspend operation                   | [RUN-3]         |
| completion.records = 0  |                                                                 |                 |
```

#### Why it is resource-closed, item by item

```text
| premise               | how it is discharged                                                        |
|-----------------------|-----------------------------------------------------------------------------|
| no heap               | no Heap in main's signature, so [PROV-6]'s closure over the call graph is    |
|                       | empty and [RES-5] does not fire. main is pure, service exhibits              |
|                       | allocates(pages) and nothing exhibits allocates over a Heap-rooted path      |
| acyclic call graph    | main -> {pump, service}; service -> {render, drain, pool_take_checked,       |
|                       | pool_release}; drain -> ring_push -> ring_index. No cycle, so [STK-1]        |
|                       | rewrites nothing and [STK-2] passes                                          |
| pool demand bounded   | service has (peak 1, delta 0) on 'p: the take and the release are on the     |
|                       | same path, and the Err arm takes nothing. The scheduler loop's backedge      |
|                       | delta is 0, so by 3.2.1's loop rule no iteration bound is needed; the loop   |
|                       | runs as long as tasks keep arriving, and live('p) <= 1 throughout            |
| ring bounded          | fixed storage in main's frame; ring_push is total, and overwrite is the      |
|                       | ring's defined semantics rather than a refusal (L9: the byte is an effect    |
|                       | flow, the 256 slots are the stock)                                           |
| queue bounded         | FixedVector<Task, 32>, storage in main's frame, len <= 32 structurally by    |
|                       | [CNT-2]; a place at capacity returns the task through Option [SEQ-10]        |
| stack bounded         | one chain, measured after code generation, one context                      |
| runtime closed        | W = 1, no task or completion records; the runtime holds nothing this         |
|                       | program can saturate [RUN-2]                                                |
```

#### The writer's-eye walkthrough

`let ring = ring_new();` establishes a fresh owner in main's frame with no
length term of its own: `UartRing` is a struct, and its `array<u8, 256>` field
carries `len = 256` as a **type** fact ([CNT-9], L11), which is why a `&uniq` to
it is legal at all under [CNT-8] and why no call in this program can invalidate
it.

`let pending = seq_fixed<Task, 32>();` publishes `len(pending) = 0`,
`cap(pending) = 32`, `room(pending) = 32` ([SEQ-1]).

`let (queued, unplaced) = seq_try_place(vector: move pending, value: ...);` is
**[CALL-2]**, the second call rule, firing for the first time, and **[CALL-5]**
for the multi-return. `move pending` is a consuming use, so every fact supported
by `pending`'s root dies at the consume ([ENT-5] clause (c)): `len = 0`,
`cap = 32` and `room = 32` are all gone. What survives on the result is exactly
[SEQ-10]'s published relation `ile(len(queued), cap(queued))` and nothing else.
`set pending = move queued;` is **[CALL-7]**: `pending` is dead, so the statement
derives no drop, reinitializes the binding, and carries the owner into the loop.

`match unplaced { Some(...) => ... }` is where L3 lands in a program with no
heap: a fixed vector at capacity is a typed refusal that returns the task,
not a trap and not a silent drop.

`region 'p { let pages = pool_static<'p, Page, 64>(); }` publishes one
`region(static.pool.pages, 262144, 16, contiguous)` item of `E` ([PROV-8]) and
one provider place confined to `'p` ([STOR-4]). `live(pages) = 0` and
`capacity(pages) = 64` enter the proof context as ordinary terms ([RES-8]).

`let produced = pump<'r>(ring: &uniq 'r ring, tick: tick);` writes through a
`&uniq` to a **struct**, not to a container, so [CNT-8] does not fire and the
projected `writes(ring)` kills exactly the facts supported by the place `ring`
([ENT-5] clause (b)). No length fact of `pending` is supported by `ring`, so
nothing about the queue dies here. This is the design's answer to D1 stated
positively: the caller does not need to know what `pump` did, because `ring`'s
only length is a type fact.

`let (rest, next) = seq_try_take(vector: move pending);` is **[CALL-2]** again,
and **[CALL-5]** for the multi-return. `pending`'s facts die at the consume;
`rest` carries `ile(len(rest), len(pending))` and `next` is an ordinary
`Option<Task>` owner. The immediately following `set pending = move rest;` is
[CALL-7]'s second placement: the target is dead because the call consumed it,
and the right-hand side is a move rather than a call.

`let sent = service<'p, 'r>(pages: &uniq 'p pages, ring: &uniq 'r ring, task: move task);`
is the resource statement. Its row `allocates(pages)` is a **path**, rooted in
the `Pool` formal ([PROV-4]), so [PROV-6] sees one edge from `main` into the pool
and no edge into any `Heap`. The (peak, delta) summary substituted at this site
is `(1, 0)` on `'p` ([RES-6], 3.2.1's call rule), which is what makes the
scheduler loop's backedge delta zero and therefore what removes the need for an
iteration bound (L9).

Inside `service`, `pool_take_checked` is the checked spelling of [RES-8]. The
`Err` arm binds `refused`, moves the page back out of it, and pushes one byte to
the ring: no affine input is lost, which is L3. The `Ok` arm opens `region 'u` so
the two borrows of the page end before the page is moved into `pool_release`; its
two results are carried out through pre-declared bindings, and `set written = ...`
and `set sent = ...` here are ordinary [SET-1] copy commits on `u64`, not
[CALL-7] reinitializations.

`drain<'u, 'r>(page: &'u page, ...)` is **[CALL-1]**, the first call rule: the
page is passed as a shared borrow, so the call is a kill event for nothing at
all, and every fact `service` holds about `page` survives it. `render` takes the
same page as `&uniq`, so its `writes(page)` kills the facts supported by that
place; `service` holds none it needs afterwards.

The third call rule, **[CALL-3]**, does not fire in this program, because no view
crosses a call boundary here. It fires in 4.2.

Two honest costs, stated rather than hidden. `ring_push`'s outer `if head_ok`
has no `else`: it is the branch whose false edge cannot be taken, which the
language charges here as it does everywhere else, and it is a real instruction
rather than a formality, because a struct field carries no range and the checker
has no source for one. And `ring_index`'s two `invariant` statements are ordinary
[INV-1] work over live own-mode integer locals; they are provable today (probe
`v23`, section 6, is exactly this shape), and their `len`-anchored analogue is
not (probe `v24`), which is Q6.

One more thing the envelope makes visible: this program reserves 64 pages and
never holds more than one, so `static.pool.pages` is 256 KiB of an envelope a
writer reading `E` would immediately shrink. That is the envelope doing its job.
It is a number someone can act on, which is what the `allocates(heap)` row never
was.

**The interrupt is polled, and that is a design limit, not an accident.** `pump`
is called by the scheduler. A genuine interrupt handler re-enters the Whitefoot
call graph from outside it, which [STK-4] makes a second execution context
needing its own `stack` item in `E`, and which this version refuses (open
question Q13). A writer who needs a real handler today gets the refusal, not a
silent envelope that is wrong.

### 4.2 A hosted line collector over `Heap`

The same design with the heap on: growth is one named operation with a typed
failure, the append helper takes the owner by value and returns it, and
`OutOfMemory` is a value on an ordinary edge.

```wf-design
const CEILING: u64 = 4096;

fn collect['o, 's](out: own AppendView<'o, u8>, source: own Span<'s, u8>)
    -> filled: own AppendView<'o, u8> reads(source), writes(out) contract {
  requires ile(len(source), room(out));
  ensures ile(len(filled), cap(out));
} {
  doc "Appends every byte of source into the view's spare window.";
  let count = len(source);
  let acc = move out;
  for @copy (at in 0_u64..count, invariant behind: ile(len(acc), at)) {
    let byte = source[at];
    set acc = seq_push(view: move acc, value: byte);
  }
  return move acc;
}

fn grow['h](buf: own HeapVector<u8>, heap: &uniq 'h Heap, additional: own u64)
    -> outcome: own Result<HeapVector<u8>, OutOfMemory<HeapVector<u8>>>
    writes(heap), allocates(heap) {
  doc "Reserves spare capacity, handing the vector back unchanged when the store refuses.";
  return seq_reserve(vector: move buf, provider: heap, additional: additional);
}

fn next_line['i](input: &'i FixedVector<u8, 4096>, from: own u64)
    -> (start: own u64, stop: own u64) reads(input) contract {
  ensures ile(start, stop);
} {
  doc "Finds the half-open extent of one line beginning at from.";
  ...elided: one counted scan over [from, len(input)) with the ordinary [OP-4] obligations...
}

command fn main(command.stdout as out: own Output, command.heap as heap: own Heap)
    -> status: own ExitStatus writes(out, heap), allocates(heap) {
  doc "Collects the lines of one fixed input buffer into a heap vector, and reports a refusal instead of dying.";
  let input = seq_fixed<u8, 4096>();
  ...elided: the input is filled once by read_at over a MutSpan of input [SYS-8]...
  let buf = seq_heap<u8>();
  let total = 0_u64;
  region 'h {
    let reserved = grow<'h>(buf: move buf, heap: &uniq 'h heap, additional: CEILING);
    match reserved {
      Err(error: refused) => {
        let recovered = move refused.vector;
        return exit_status(code: 70_u8);
      }
      Ok(value: ready) => {
        set buf = move ready;
        let at = 0_u64;
        region 'fill {
          let view = seq_append_view(&uniq 'fill buf);
          loop @lines (invariant room: ile(len(view), CEILING)) {
            let done = ige(at, 4096_u64);
            if done {
              break @lines;
            }
            let (start, stop) = next_line<'fill>(input: &'fill input, from: at);
            region 's {
              let line = seq_span(&'s input);
              let fits = ile(stop - start, CEILING);
              if fits {
                set view = collect(source: move line, out: move view);
              } else {
                break @lines;
              }
            }
            set at = stop +wrap 1_u64;
          }
          set total = absorb(view: move view);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
```

#### The writer's-eye walkthrough

`let buf = seq_heap<u8>();` publishes `len = 0`, `cap = 0`, `room = 0` and, by
[SEQ-2], **allocates nothing**: an empty growable sequence owns no backing. That
is L4 at the constructor: the type says the owner can grow, and the constructor
still has an empty effect row.

`grow<'h>(buf: move buf, heap: &uniq 'h heap, ...)` is **[CALL-2]** on `buf` and
the single acquisition point of the whole program. `move buf` kills `len = 0`,
`cap = 0` and `room = 0`. The result is a `Result`, so nothing is established on
it until the match dispatches. On the `Err` arm, `refused.vector` is the original
owner handed back unchanged (L3, [SEQ-19]); the program moves it out, drops it on
the return edge, and exits with a status. **There is no path on which the process
disappears**, which is the whole of goal B. On the `Ok` arm, [SEQ-19]'s published
relations arrive: `ige(room(ready), CEILING)` and `len(ready) = len(buf)`. The
`set buf = move ready;` is [CALL-7] again.

`grow` also shows what `allocates` now means. Its row is
`writes(heap), allocates(heap)` where `heap` is its own formal, so [PROV-6]
computes one heap-reaching path `main -> grow -> seq_reserve` from signatures
alone. Delete `command.heap` from `main`'s parameter list and the program stops
compiling at the call, not at some allocation deep inside a library.

`let view = seq_append_view(&uniq 'fill buf);` is [SEQ-17] and [VIEW-2]: it takes
an exclusive loan on `buf` for `'fill`, and publishes `len(view) = 0`,
`room(view) = room(buf)`, `cap(view) + len(buf) = cap(buf)`. While that loan is
live, [OWN-5] already forbids moving, dropping or growing `buf`; the design adds
no exclusivity rule to get the freeze the writer wants.

`set view = collect(source: move line, out: move view);` is the owner's own
spelling, and it is where two call rules fire at once. On `line` it is
**[CALL-2]**: a `Span` passed by value, whose facts die at the consume and whose
callee publishes only what its contract says. On `view` it is **[CALL-2]** for
the transport and **[CALL-3]** for what it does *not* kill: the parameter's
declared type is `AppendView<'o, u8>`, so by L14 and [VIEW-5] the callee cannot
decrease `len(buf)`, cannot increase it (only `absorb` publishes an increase, and
[VIEW-7] denies `absorb` to a callee), and its projected `writes(out)` kills the
facts whose support overlaps the viewed element storage and no length term over
that origin. The caller's facts about `buf` are therefore alive **and true**
after the call, which is the property D1 violated by asserting it from the wrong
evidence.

`set total = absorb(view: move view);` is the commit ([VIEW-6]). In order: the
origin set resolves to `buf`; the commit value `w` is bound with `w = len(view)`;
every fact supported by `len(buf)` dies as a whole-place length event; and only
then are `total = w` and the sum relation established, in the derivable case
where the old length was a constant (here `len(buf) = 0` before the window
opened, so `len(buf) = w` is a difference bound). `absorb` is admitted here and
only here, because this is the function that formed the view ([VIEW-7]).

`total` survives the end of `region 'fill`: its support is the ordinary binding
and the commit value, and [VIEW-6] step 4 states the relation over the owner
place `buf` rather than through the holder.

#### What the compiler reports

```text
note: scheduler.wf is resource-closed; envelope written to scheduler.E
note: collector.wf is not resource-closed
  [RES-5] main selects command.heap
    heap-reaching path:  main -> grow -> seq_reserve
  a general store cannot appear in an envelope [L6], so no envelope is computed
  still true of this program:
    no covered-resource failure is a trap [RES-7]; seq_reserve returns a value
    the heap is reachable only through the parameter above [PROV-6]
  not true of this program:
    it makes no resource promise, and its environment owes it none
```

That last block is the point of the whole design. The hosted program loses the
guarantee and keeps the honesty: `OutOfMemory` is a value on an ordinary edge,
its heap use is visible in one signature, and a reader can see in the entry line
whether the program can allocate at all.

#### The one thing in 4.2 that does not verify

`collect`'s header invariant `ile(len(acc), at)` is **not writable today**, and
neither is the chain that discharges [SEQ-5]'s requirement `igt(room(acc), Z)`
from it. `len(acc)` is not an [INV-1] affine atom: probe `v2_len_atom`
(section 6) is rejected at **parse**, [GRAM-4], not merely at atom admission. And
even where the invariant can be stated over ordinary locals, its conclusion does
not reach a `len`-anchored [FN-9] query: probe `v24` is rejected while the
identical program with a parameter-anchored `ensures` (probe `v23`) is accepted.
That is Q6, it is a specification gap rather than a compiler defect, and it
decides whether this container surface is usable. Section 5 states it in full.

---

## 5. Open questions

The two drafts' lists, merged, with everything the owner's rulings settle removed.
**Dropped as settled, and not restated below:** whether the heap is a capability
value (it is); whether a bounded general heap may enter `E` (it may not, so no
live-bytes bound licenses a proved heap acquisition, which retires
`RESOURCES.md` Q8); whether recursion may accumulate frames (it may not);
whether `FixedVector` admits affine `T` (it does); whether container state may be
mutated through `&uniq` (it may not); whether an `ensures` may be two-state (it
may not); and the multi-return spelling (`-> (a: own T, b: own U)` with
`let (a, b) = f(...)`).

Each remaining question states two candidates and one recommendation.

**Q1. Does a resource-closed program have to *prove* every covered acquisition
succeeds, or may it handle a typed refusal?** *(a)* Strict: every covered
acquisition uses the proved spelling, and a reachable `PoolExhausted` arm
disqualifies the program. *(b)* Permissive: both spellings are admitted, since
neither can ask for more than `E` ([RES-8] as drafted).
**Recommend (b).** A refusal answered inside the envelope is not a resource
failure; it is a decision the store is entitled to make, and it is precisely what
[RUN-2] *requires* the runtime to do when a queue saturates. Forbidding the
writer the protocol the runtime is obliged to use would be inconsistent. The
stronger property some code wants is available at the same site by choosing the
proved spelling, which is how every other partial operation in this language
works.

**Q2. How is a provider-owned value's release attributed to its provider?**
*(a)* Carry the provider identity in the value, so a drop anywhere can return the
storage. *(b)* Bind the value to the provider's region, so the release site is
lexically inside the provider's scope and the provider path is derivable from the
type ([PROV-7] as drafted).
**Recommend (b)**, with the reachability side-condition stated as a checked
premise rather than assumed. (a) costs a word in every value, makes release rows
depend on data flow rather than on types, and reintroduces the hidden ancestry
[EFF-5] spent a section removing. (b)'s residual, a release edge on which the
provider is uniquely borrowed elsewhere, is real and is the one thing in the
resource half this design could not close. A third candidate worth trying is
making `pool_release` the only legal disposal, a linear rather than affine slot
type, which removes the derived release entirely at the cost of a new ownership
class.

**Q3. Where does a *hosted* resource-closed program's memory come from?**
*(a)* Static and frame storage only, as in 4.1. *(b)* One more entry row
delivering a committed region, `command.region as store: own Arena<'store>`, so a
hosted program can be handed a real extent at start.
**Recommend (a).** An entry row is a promise about what the runtime always hands
a program, and [FN-7]'s table earns its closedness by containing only inputs
every `command` program can meaningfully receive. A committed extent is not that:
its size is a property of one deployment, so `command.region` would either fix a
size in the language or deliver an extent whose size the program cannot know,
and both are worse than the program reserving what it needs with `arena_static`
and publishing it in `E`, where the deployment can read it. (b) becomes right the
day a program needs a store larger than its static image can hold, and it is
then a change to [FN-7] and one item in `E`, made for that reason rather than for
convenience.

**Q4. Does `E` fix the lane count, or is it a table over `W`?** *(a)* One `W`
compiled in, so `E` is a single list. *(b)* A finite profile table `E(W)` for the
runtime's supported lane counts, with `W` chosen at `PreStart` ([RUN-3] as
drafted).
**Recommend (b).** Parallel permission is never an obligation ([PAR-1] 1988), so
`W = 1` must always be legal, and a program that only runs at `W = 8` would make
the permission load-bearing. A table is finite, is fixed integer arithmetic per
row, and lets a deployment trade lanes for memory without recompiling.

**Q5. What happens when an optimization raises a computed frame envelope?**
*(a)* Recompute `E` after optimization and publish the larger figure. *(b)* Treat
the published `E` as a cap and decline any optimization that would exceed it.
**Recommend (b) as the default with (a) available**, because the envelope is a
contract with a deployment that may already have been sized against it, and
because "the optimizer decides how much stack the program promised" is the same
mistake as letting the optimizer decide whether tail calls happen (L7).

**Q6. Loop-carried length facts, in two halves.** This is the highest-value
question in the batch: it decides whether the container surface is usable at all.
Every migrated loop in section 4 needs the shape, and every one of them is
blocked.

*Half (a) is a specification gap, not a compiler defect.* The mechanism was
verified on `main` in `compiler/src/semantic/entailment/flow.rs`: a counted-loop
exhaustion export and an `invariant_stmt` publish only an **affine** fact
(`flow.rs:10507` pushes into `exhaustion.affine.facts`; nothing enters the L0
`FactState`), while the [FN-9] postcondition query (`prove_ordering`,
`flow.rs:5538`) reads the L0 closure plus one direct affine target and has **no
affine-left/L0-right bridge**. That bridge does exist, as
`affine_bound_via_l0_right` (`flow.rs:6205`), and the specification grants it by
name to exactly two consumers: [ENT-6] `SubscriptBounds` (spec line 3034) and
`SystemRange` (3078). [FN-9] is not one of them. Moreover the [FN-9] goal cannot
even be *formed* as an affine target when it mentions `len(P)`, because
`postcondition_affine_datum` maps `RelationDatum::Length` to `None`
(`flow.rs:1243`), and [ENT-6] (2993) forms the L0-to-affine index only from live
own integer bindings. [ENT-1] (2670) is explicit that adding a proof rule is a
specification amendment, never implementation strengthening, so this is an
amendment to write, not a bug to file.

The discriminating pair is decisive and was re-run here. Probe `v25`: a counted
loop with header invariant `ilt(at, capacity)` whose consumer is a subscript
`set deref(destination)[at] = 7_u8;` is **accepted** under [OP-4]. Probe `v26`:
the identical loop and invariant whose consumer is `ensures ilt(result, capacity)`
is **rejected** under [FN-9], residual `at - len(deref(destination)) <= -1`. The
facts are identical; only the consumer differs. Probe `v23` confirms the length
term is the discriminator: the same shape with a parameter-anchored
`ensures ile(result, count)` is **accepted**, and probe `v24`, identical but
length-anchored, is **rejected**. Consequence for this design: the specification's
own advertised repair, "a proved header or local invariant [INV-1]" (913, 3085),
does not reach any [FN-9] relation over `len(P)`, so every value-in / value-out
helper in section 3.8 and section 4 whose contract mentions a length cannot be
discharged from a loop today.
The defect is not that one consumer was forgotten. It is that **proof routes are
granted per consumer family, by name.** `SubscriptBounds` and `SystemRange` were
each handed the affine-left/L0-right bridge in their own paragraph; [FN-9] was
not, and separately acquired a direct-affine route (Q17) that no rule states. A
language in which "can this inequality be derived?" has a different answer
depending on which construct is asking has, in effect, several provers, and a
writer cannot reason about any of them: probes `v25` and `v26` are the same
proof asked twice.

Candidates: *(a1)* **one numeric goal disposition, shared by every consumer of a
relation.** [ENT-6] states, once, the complete ordered derivation for a numeric
goal: contradiction, the exact signed fact, the closed L0 state, direct `AUTO`
over the affine domain, and the affine-left/L0-right bridge. Every consumer asks
that one question: [OP-4] subscript bounds, [SYS-8] system range, [OP-2] integer
domain, [OP-9] allocation fit, [FN-8] requirements, [FN-9] normal-result
relations, and [INV-1] targets. The per-family route lists in [ENT-6] retire, and
the direct-affine branch Q17 records stops being undocumented because the route
set is stated once for everyone. *(a2)* keep the per-family lists and add [FN-9]
to the bridge's list, leaving `SubscriptBounds`, `SystemRange` and [FN-9] as
three separately enumerated grants.
**Recommend (a1).** It is the correct shape rather than the local repair: a
derivation system whose answer is a function of the question and the state, and
not of the syntactic construct that asked, is what [ENT-1] already promises when
it calls the fragment "a closed, deterministic, terminating derivation system
fixed completely by this specification". (a2) leaves the language with a proof
surface a writer has to memorize per construct, and it guarantees the same defect
recurs the next time a consumer is added. The bridge also stays where it belongs
under (a1): [INV-1]'s conclusions keep their value-image semantics ([ENT-5]'s "a
theorem over immutable value images", not a proposition that rereads mutable
bindings), and no new fact-flow direction is invented, because the two domains
meet in the disposition rather than by copying facts from one into the other.
That (a1) is also the amendment that resolves Q17 is a consequence of stating the
route set once, not a separate benefit.

*Half (b) is the design question.* `len(P)`, `cap(P)` and `room(P)` are not
[INV-1] affine atoms, whose admitted atoms are literals and "live own-mode
integer values" (3106). The rejection is at **parse**: probe `v2_len_atom` is
[GRAM-4], `expected [")", ",", "*", "+", "-"]` at the `(` of `len(`, so the
invariant every migrated loop needs cannot be *written*, and no amount of (a)
reaches it.
Candidates: *(b1)* **define the length-class terms once in the term language.**
[ENT-2] already says what a term is, and [ENT-5] already says what a length
term's support is (the viewed place's non-element root path) and when it dies.
`len(P)`, `cap(P)` and `room(P)` become terms there, with one statement of their
support and their kills, and they are then usable by every consumer of a term:
L0 difference bounds, [INV-1] affine atoms, [FN-9] clause operands, and [ENT-6]
goals alike, with [GRAM-4]'s `affine_expr` admitting the same term syntax the
rest of the fragment already admits. *(b2)* admit them as a special case inside
[INV-1]'s atom rule, leaving [ENT-2]'s term language and [FN-9]'s operand list
each with their own separate answer.
**Recommend (b1).** A length is one quantity; the language should have one
definition of it, in the one place that defines what a quantity is, and every
rule that consumes quantities should get it for free. (b2) produces a third
place where "what may name a length" is decided, which is the same per-consumer
fragmentation half (a) exists to remove, and it would leave the atom set and the
term set able to disagree. The alternative some writers reach for, mirroring
every loop-carried length into an ordinary `u64` local maintained by hand, is
what `wfgrep` does today and is worse than either candidate: it reintroduces the
two-values-that-must-agree shape the 2026-09-01 discussion rejected as X2, where
a local `filled` and a real `len` can disagree and nothing checks that they do
not.

The two halves are one amendment, not two: (b1) fixes what a length is and (a1)
fixes who may ask about it, and neither is useful alone.

**Q7. Does an arena reclaim anything before its region ends?** *(a)* Cursor only:
a drop returns nothing, and the extent returns when `'p` ends ([RES-6] as
drafted). *(b)* LIFO: the top-most allocation may be popped.
**Recommend (a).** It is the rule with no ordering side-condition, it matches
what a bump allocator actually does, and (b)'s "only if it is on top" is a
premise the checker would have to carry through every branch and loop for a
saving no current program needs.

**Q8. Non-constant offsets in contract relations.**
`ile(len(written), len(output) + n)` with `n` a parameter is not a difference
bound. `room(P)` ([CNT-2]) removes the most common instance, the "there is spare
for this whole source" requirement, but not the general case, and [VIEW-6]'s
`len(P) = old + w` sum is the same shape. Candidates: *(a)* admit three-term
affine relations into [FN-9]'s RelationTemplate and into L0. *(b)* route them
through [INV-1]'s affine domain, which already handles
`ile(sum, 255_u32 * (i + 1_u64))`.
**Recommend (b).** L0 is a difference-bound domain by design, and its
determinism and termination arguments ([ENT-4]'s "the least closure is unique and
finite") rest on that shape; widening it to carry three-term relations would
change what the closure is, for the sake of relations the affine domain already
represents exactly. The right statement is that a verified contract relation may
be *stated* in the affine domain and *queried* there, which is precisely what
Q6(a1)'s single disposition makes available to every consumer, so this question is
answered by the same amendment rather than beside it.

**Q9. `AppendView` write-back: scope-end commit or explicit `absorb`?**
*(a)* Commit automatically on the edge leaving the view's region. *(b)* Explicit
`absorb` ([VIEW-6]), with abandonment defined by a release action ([VIEW-8]).
**Recommend (b), as drafted.** (a) needs a must-be-live obligation on the view at
that edge, which would be the first obligation in the language attached to a
value rather than to an operation, and it puts a fact-publishing event on an edge
with no source spelling that [DIAG-1] must then locate. (b)'s cost is that a
writer can forget to absorb and silently discard work; that is *safe* rather than
*prevented*, because [VIEW-8] drops the pending elements, and the mitigation is a
compiler note where an `AppendView` with a nonzero proved `len` reaches its
release, not a rule. The arithmetic residual of [VIEW-6] step 4 is Q8's.

**Q10. Is [BLD-3]'s coverage certificate the right shape?** *(a)* Syntactic: one
counted loop, binder-indexed, range exactly `0..len(b)` (as drafted). *(b)* A
written `invariant` the writer proves at `seq_finish`, "every index below `len(b)`
was written".
**Recommend (a), and say plainly that it is a shape rule.** (b) needs a bounded
quantifier, which [INV-1]'s affine domain does not have and [ENT] declines. (a)
admits the one shape with a machine-checkable certificate and refuses everything
else rather than starting a search, which is the same choice [PAR-2] made for its
affine map.

**Q11. May a `MutSpan` be split for `par`?** *(a)*
`seq_split_at(span, at) -> (head, tail)` with `len(head) = at` and
`len(tail) = len(span) - at`. *(b)* No split; `par` writes go through `Builder`
only.
**Recommend (b), with (a) as the honest successor.** [OWN-5]'s origin relation
says that two accesses through one origin conflict, which is true and is what
makes views sound. (a) does not weaken that sentence; it needs a *second*
relation, disjointness of ranges over one origin, and that relation has to be
maintained by every rule that forms, moves, passes, returns and reborrows a view.
Adding half of it, enough to admit a split and not enough to be a general
statement about ranges, would leave the language with a soundness argument that
holds for the shapes someone thought of. (b) is complete as it stands: [BLD-3]'s
certificate covers the shape the owner named with rules that are checkable, and
[VIEW-3] is exactly where a range would live when the full relation is written.
That (a) also admits divide-and-conquer over a span is the reason to write that
relation properly rather than to approximate it.

**Q12. How do `len(view)` and `len(owner)` relate in the fact domain?**
*(a)* Distinct terms plus the [VIEW-2] and [VIEW-6] equalities (as drafted).
*(b)* One term: `len(a)` *is* `len(v)`.
**Recommend (a).** It is the only candidate compatible with [ENT-2] as written,
whose term identity is spelling identity and which explicitly declines to model
aliasing. (a)'s cost is near zero in practice, because the owner is frozen while
the view lives, so there is no useful reasoning about `len(v)` in that window.

**Q13. What about control entering the call graph from outside it?**
*(a)* Prohibit reentrancy in the resource-closed profile: no signal handler, FFI
callback, or host reentry may execute Whitefoot code. *(b)* Admit it, with a
separately reserved stack item per reentrant context in `E`.
**Recommend (a) for this version.** (b) is the right long-run answer and [STK-4]
is already drafted for it, but admitting it now means bounding the depth of a
call graph the compiler does not fully see, which is the class of promise this
design refuses to make elsewhere. The cost is visible in 4.1: the UART producer
is polled rather than interrupt-driven.

**Q14. `par` and the stack: how does a resource-closed program use parallelism?**
*(a)* Restrict resource-closed programs to `par` shapes whose lowering cannot
nest a stolen task on a waiting lane's stack, and execute every other shape
sequentially. *(b)* Require the compiler-managed work-first continuation redesign
(a resumable parent continuation, a return to the scheduler root on an unfinished
join, stable continuation frames) before any resource-closed program may contain
a `par`.
**Recommend (a) now and (b) as the real target.** The current runtime's wait path
executes a stolen task on the waiting lane's own stack, so the stack bound is not
the sequential bound plus a constant; a conservative envelope exists but grows
with the outstanding-slot pool, which is a bad promise to publish. (a) ships a
usable property immediately, because parallelism is a permission and sequential
execution is always a legal lowering. This is the largest engineering item this
design implies, and it is a lowering and runtime problem, not a checker problem.

**Q15. Is `E` an emitted artifact, and is it part of program identity?**
*(a)* Diagnostic output only. *(b)* An emitted machine-readable file beside the
object, which a deployment reads to size the environment.
**Recommend (b), and explicitly not part of [PROG-2] compilation-unit identity.**
The envelope is useless if the deployment cannot read it, and L1's whole causal
story is that the environment consumes `E`. Keeping it out of unit identity keeps
`E` a derived fact about a build rather than an input the source depends on.

**Q16. How is a view formed, syntactically?** *(a)* Operation calls,
`seq_span(&'r v)` (as drafted). *(b)* A suffix form, `v.span<'r>()`.
**Recommend (a).** A view formation is an operation of the table, resolved by
name and receiver type exactly as `len` and `slice_of` are, and it should be
spelled the way every other such operation is spelled. (b) would introduce a
second call syntax whose resolution rule ([GRAM-5] has no method-call production)
would have to be stated and kept consistent with [OP-1]'s, so the language would
carry two ways to name one kind of thing for the sake of one kind of thing.

**Q17. The same defect in the other direction, recorded rather than separately
answered.** `prove_ordering`'s second branch (`flow.rs:5578-5622`) accepts an
[FN-9] relation from a direct affine consequence alone. [ENT-4] (2853, "these
three dispositions are complete and exclusive [FN-8, FN-9]") and [DIAG-2] (1919,
which roots each discharged relation on "the exact local [ENT-4] derivation") do
not grant that route. So the same region of the checker that under-accepts for
one consumer over-accepts for another, and both facts have the same cause:
nothing states, once, what the admitted derivation routes for a numeric goal
are. Q6(a1) is the answer to both. It is listed separately only so that the
amendment is written knowing it has to reconcile two directions, not one.

---

## 6. Verified versus reasoned

**Verified** means a compiler executed it in the session that wrote this file.
The binary is the gate-profile `whitefootc` built from this tree
(`cargo build --locked --offline --profile gate --bin whitefootc`); every probe
below was re-run here rather than inherited from a draft, and every probe source
is reproduced in the two drafts or in the inline text of this section. No timing
figure from this machine appears anywhere in this file.

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
| p4                | single-state ensures ile(result, capacity) with             | ACCEPTED                                   |
|                   | define capacity = len(deref(destination)), one write        |                                            |
| p2                | ensures ige(len(result), capacity);                         | REJECTED [GRAM-9]                          |
| p8                | fn pair() -> (first: own u64, second: own u64)              | REJECTED [GRAM-2], expected IDENT          |
| p9                | array_new<box<u64>, 4>(move cell)                           | REJECTED [OP-1] InvalidOperation           |
| p10               | set a = take(b: move a); for a : own buffer<u8>             | REJECTED [STOR-1] AffineSetTarget          |
| p11               | let old = replace a = take(b: move a);                      | REJECTED [OWN-1] UseAfterMove              |
| p1_noinput        | a command entry selecting no standard input                 | ACCEPTED                                   |
| p2_forever        | an entry whose only statement is a loop with no break       | REJECTED [FN-1] FunctionFallthrough        |
| p3_rec            | an ordinary self-recursive function called from main        | ACCEPTED                                   |
| p4_undeclared     | a body that allocates while declaring pure                  | REJECTED [EFF-2] EffectMismatch, missing   |
|                   |                                                            | allocates(heap)                            |
| p5_ambient        | a nullary leaf function that allocates while holding nothing| ACCEPTED                                   |
| p6_unproved       | buffer_new on an unbounded runtime length                   | REJECTED at target layout,                 |
|                   |                                                            | Unrepresentable(RuntimeSizedAllocation)    |
| tests/programs/   | the migration baseline of section 4.2                       | COMPILES, exit 0; 11 buffer_new calls      |
| wfgrep.wf         |                                                            |                                            |
```

What each establishes, for the rules that rest on it: `d1` and the conformance
case make [CALL-6] and [CNT-8] a repair of a live defect rather than of a
hypothetical. `p1` shows [CALL-2] already behaves correctly, so what is missing
is the vocabulary to publish across it ([CALL-4]). `p6` shows [CALL-1] already
holds. `p7` shows `MutSpan` is new capability, not a rename. `p4` shows the
entry-image contract shape [CALL-4] extends is real today; `p2` shows
`len(result)` does not parse, so admission 2 is correctly labelled an amendment.
`p8` shows multi-return is new syntax. `p9` shows affine elements have no
construction route today, so [CNT-4] is new capability. `p10` and `p11` are the
two halves of [CALL-7]'s premise, taken from the compiler itself. `p1_noinput`
shows 4.1's entry shape is legal. `p2_forever` is why 4.1's scheduler loop breaks
on an empty queue. `p3_rec` shows recursion is permitted today, so [STK-2]
restricts resource-closed programs and retires nothing. `p4_undeclared` shows
allocation is already exhibited-and-checked both ways, so [PROV-4] changes what
the entry names, not whether it is checked. `p5_ambient` is L2's evidence and the
single fact the capability half of this design exists to change. `p6_unproved`
shows allocation *size* is already a static obligation while availability is not,
which is the split 4.2 relies on.

### 6.2 Q6, isolated

Seventeen probes over one shape: a function with
`define capacity = len(deref(destination))`, a counted loop, and a consumer of
the loop's exported fact.

```text
| probe                      | shape                                                   | verdict                          |
|----------------------------|---------------------------------------------------------|----------------------------------|
| v3_noloop                  | no loop; invariant_stmt then return                     | ACCEPTED                         |
| v8_let_local               | let at = capacity; return at                            | ACCEPTED                         |
| v9_set_local               | set at = capacity; return at                            | ACCEPTED                         |
| v4_loop_nowrite            | a loop that touches neither at nor capacity             | ACCEPTED                         |
| v12_loop_inv_then_reset    | loop with header invariant, then set at = capacity      | ACCEPTED                         |
| v21_export_chains_to_local | loop with header invariant, then                        | ACCEPTED: the export chains one  |
|                            | invariant via_local: ile(at, cap2) over a second local  | hop through a local equality     |
| v22_loop_then_inv_stmt     | loop with header invariant, then invariant tail_bound,  | REJECTED [FN-9]. The invariant   |
|                            | then return at                                          | itself proved; only the FN-9     |
|                            |                                                        | query failed                     |
| v23_param_anchored         | identical loop, ensures ile(result, count) over a       | ACCEPTED                         |
|                            | parameter                                              |                                  |
| v24_len_anchored           | identical loop, ensures ile(result, capacity) over      | REJECTED [FN-9],                 |
|                            | len(deref(destination))                                 | "at - len(deref(destination))    |
|                            |                                                        | <= 0" unproved                   |
| v25_subscript_consumer     | identical loop, consumer is                             | ACCEPTED under [OP-4]            |
|                            | set deref(destination)[at] = 7_u8;                      |                                  |
| v26_ensures_consumer       | identical loop, consumer is ensures ilt(result,         | REJECTED [FN-9],                 |
|                            | capacity)                                               | "at - len(deref(destination))    |
|                            |                                                        | <= -1" unproved                  |
| v2_len_atom, v5            | invariant fill_bound: ile(at, len(deref(destination)))  | REJECTED [GRAM-4] at parse       |
| v11, v10, v15, v17, v1, p3,| the several ways of writing the loop without one of the | REJECTED [FN-9], the same        |
| p5                         | accepted shapes above                                   | residual                         |
| v6, v7                     | header invariant not preserved on the backedge          | REJECTED [INV-1], Backedge       |
| v16                        | probe artefact: an undischarged count + 1_u64           | REJECTED [OP-2]                  |
```

The pair that matters is `v25` and `v26`: identical facts, different consumer,
opposite verdicts. The pair `v23` and `v24` identifies the discriminator as the
`len(P)` term rather than the loop. `v22` shows the [INV-1] conclusion is proved
and standing immediately before the return and still does not reach [FN-9].
`v2_len_atom` shows the atom half is a parse-level refusal. Section 5's Q6 states
the mechanism and the amendment.

### 6.3 Reasoned, and not verified anywhere

- **Every rule in section 3.** None is implemented, and none has been executed
  against a program. No compiler has seen `FixedVector`, `HeapVector`,
  `ArenaVector`, `PoolVector`, `Span` as a distinct type, `MutSpan`,
  `AppendView`, `Builder`, `Heap`, `Arena`, `Pool`, `slot`, `seq_*`, `absorb`,
  `pool_static`, `arena_static`, multi-return, the reinitializing `set`,
  `cap(P)`, `room(P)`, or the `resource_closed` marker.
- **Every program in section 4** and every diagnostic quoted there. The
  diagnostics follow [DIAG-1]'s single-rule, single-location,
  one-mechanical-fix discipline and the register of the real diagnostics quoted
  in 6.1, but no compiler emits them.
- **Every byte figure in 4.1's envelope.** They are illustrative; nothing
  computed them.
- **The composition algebra of 3.2.1.** Its sequence and branch rules are
  standard. Its `par` rule depends on a runtime profile that does not exist yet.
  Its loop rule's claim that a zero-delta backedge needs no iteration bound is
  the claim to attack first when falsifying this file.
- **[PROV-7].** The release-site reachability premise is stated, not derived. It
  is the known hole, and it is Q2.
- **The claim that [STK-1]'s admission conditions are sufficient** for a correct
  mutual-tail-recursion rewrite. They are the conditions the drafts could name;
  no proof was attempted and the rejected shapes were not enumerated.
- **The claim in [PROV-2] that `Heap` uniqueness is enough** to keep a
  `HeapVector`'s release row empty. This is load-bearing for keeping [STOR-1],
  [STOR-3] and [STOR-5] almost unchanged, and it deserves a falsifier of its own:
  a program holding two heap-backed vectors across a boundary where the `Heap`
  has been moved.
- **Everything about the current runtime's closure.** The gaps L5 names come
  from the read of 2026-09-01 recorded in `RESOURCES.md`, not from a fresh audit
  in this session, and [RUN-2] is written as an obligation precisely because no
  existing target can be certified to meet it.
- **[BLD-4]'s claim that [PAR-2] needs exactly one refinement.** It was checked
  by reading [PAR-2]'s conditions one at a time against the `Builder` shape; it
  was not checked by running the PAR ledger, which has no such shape to report on.
- **The claim that `wfgrep` becomes heap-free.** Its only `allocates(heap)`
  sources are eleven `buffer_new` calls (counted at the branch tip in this
  session) reaching three declared rows, all of which this design replaces with
  `seq_fixed<T, N>`. The substitution was not performed and compiled, because
  [SYS-8] cannot take a view today. The claim also moves roughly 95 KiB of
  buffers out of the heap and into stack frames, which is a stack-budget question
  ([STK-3]) rather than a free win.

### 6.4 Falsifiers this design asks for, in the order to run them

1. Land Q6(b1) and Q6(a1) on a branch and recompile probe `v24`. If the
   length-anchored postcondition still does not discharge, the design's
   value-in / value-out helpers do not work and section 3.8 needs a different
   contract shape.
2. Hand-execute 3.2.1 on 4.1 and on a program whose loop retains conditionally,
   and check that the branch rule's per-variant retention survives a `propagate`
   edge.
3. Attack [PROV-2]'s uniqueness argument with the two-vector program above.
4. Attack [STK-1] with a mutual tail recursion carrying a live child reborrow
   across the jump, and check that the stated conditions reject it.
5. Attack L9 with a fixed append-only log: confirm that the design counts its
   records as a consumable budget and not as an effect flow, and that a program
   writing to it in an unbounded loop is correctly *not* resource-closed.
6. Rewrite one existing corpus program against [SEQ-19] and [RES-7] by hand and
   count what the `Result` return costs at every call site. If the answer is
   "every function that touches a vector grows an error route", the operation
   split of [RES-8] and [CNT-7] is wrong somewhere.

---

## 7. Implementation order

Eleven batches. Each names the rules it implements and the test it adds. This is
an ordering, not a design choice: the design is section 3, and nothing here may
be read as trading a rule away for a cheaper batch. Nothing here is an approval
or a schedule; it is the order in which each batch's test can pass.

**B1. Type-derived call transports, and the retirement of container state
mutation through `&uniq`.** Rules: [CALL-1], [CALL-2], [CALL-3], [CALL-6],
[CNT-8]. This is first because it is the live defect and because it needs none of
the new types: [CNT-8] names the four owners of [CNT-1], which do not exist yet,
so today's `&uniq buffer<T>` keeps its spelling and gets [CALL-6]'s type-derived
classification, `element = false`, which is exactly the sweep's minimal sound
repair. Test: **`tests/conformance/cases/ent5-neg-callee-uniq-buffer-replace-kills-length.wf`
turns XPASS**, rejecting at [OP-4] with residual `9_u64 < len(line)` as its
manifest already expects; plus one positive case pinning [CALL-1] (a shared-borrow
call keeps the caller's length fact) so the repair is not a blanket kill.

**B2. The proof-surface amendment.** Rules: none new; amends [INV-1] and [FN-9]
per Q6(b1) and Q6(a1), and reconciles Q17 in the same pass. Second because every
later batch's contracts are unprovable without it. Tests: a conformance pair
mirroring probes `v23` and `v24` (parameter-anchored and length-anchored, both
accepted after the amendment), and one mirroring `v25`/`v26` so the two consumers
of one exported invariant agree.

**B3. Length-class terms and contract vocabulary.** Rules: [CNT-2], [CALL-4].
Test: a function whose `ensures` mentions `len(result)` and `room(output)`
compiles, where probe `p2` is [GRAM-9] today.

**B4. Owners, typestate, and release.** Rules: [CNT-1], [CNT-3], [CNT-4],
[CNT-5], [CNT-6], [CNT-7], [CNT-9], [SEQ-1], [SEQ-2], [SEQ-4], [SEQ-9] to
[SEQ-14], [SEQ-20]. Retires `buffer<T>` from the writer surface. Test: a
`FixedVector<Handle, 64>` object table with affine elements, accepted, where
probe `p9` is [OP-1] today. This batch supersedes B1's conformance case, whose
program no longer typechecks; the case's disposition is conformance evidence and
is recorded in `governance/APPROVALS.md` with the merge.

**B5. Views and the commit event.** Rules: [VIEW-1] to [VIEW-10], [SEQ-5] to
[SEQ-8], [SEQ-15] to [SEQ-18]. Test: an element write through a `MutSpan` is
accepted where probe `p7` is [SET-1] today; and an abandoned `AppendView` drops
what it appended and publishes nothing about the owner's length.

**B6. Reinitializing `set`, and multi-return.** Rules: [CALL-5], [CALL-7]. Test:
probe `p10`'s program is accepted and probe `p11`'s repair is no longer needed;
probe `p8`'s multi-return signature parses and binds.

**B7. Providers, and the heap as a value.** Rules: [PROV-1] to [PROV-8],
[SEQ-3], [SEQ-19], [SEQ-21], [RES-7], [RES-8]. Test: probe `p5_ambient`'s program
is **rejected** (an allocation with no provider held), and a `main` that omits
`command.heap` cannot reach any allocation; plus one case pinning that a failed
`seq_reserve` returns the vector unchanged.

**B8. System I/O over views.** Rules: the [SYS-8] amendment. Test:
`tests/programs/wfgrep.wf` migrated to `seq_fixed` and `MutSpan`, compiling with
no `allocates` entry anywhere on its call graph, which is the first program that
demonstrates goal A's container half end to end.

**B9. The stack judgment.** Rules: [STK-1] to [STK-5]. Test: a self-recursive
program is rejected under `resource_closed` citing STK-2 with the cycle rendered,
and its tail-recursive rewrite is accepted; probe `p3_rec` stays accepted without
the marker.

**B10. The envelope and the judgment.** Rules: [RES-1] to [RES-6], [RES-9],
[RUN-1] to [RUN-6]. Test: section 4.1's program is accepted, its `E` is emitted
and matches a pinned expectation, and section 4.2's program is reported not
resource-closed with the heap-reaching path rendered.

**B11. The builder and `par`.** Rules: [BLD-1] to [BLD-4], [RUN-3], [RUN-4].
Test: a counted `builder_set` fill receives [PAR-2] permission and its
`seq_finish` is admitted by [BLD-3]'s certificate, while a two-loop fill of the
same builder is rejected at the `seq_finish` operand.

Q14 (`par` and the stack) sits across B10 and B11 and is the largest engineering
item any of this implies; under its recommendation (a), B10 ships with `par`
shapes executed sequentially inside resource-closed programs, and B11 makes the
builder usable without changing that.
