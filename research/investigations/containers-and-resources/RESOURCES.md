# Resources: providers, the envelope, and resource-closed programs

The resource half of the containers-and-resources design for batch 0116. Tree
read: `batch/0116-containers-and-resources` at `main` a40c7e70, spec **v0.40
ACTIVE**. Bare four-digit line numbers are `spec/kernel-spec.md` at that tip;
every other citation names its file. The container half — `FixedVector`,
`HeapVector`, `ArenaVector`, `PoolVector`, `Span`, `MutSpan`, `AppendView` — is
drafted beside this file in `CONTAINERS.md`, and this file uses those names
without redefining them.

**Nothing here is implemented.** No compiler code was written for it, the
specification text below is draft text for a work branch and not an amendment,
and every syntax form this design adds is marked where it first appears.
Section 9 separates what I executed against the current compiler from what I
only reasoned about.

Provenance: the owner's four rulings of 2026-09-01 (heap as an explicit
capability value handed to `main`; `resource-closed` as a compiler-derived,
writer-requirable property with one language and no dialects; no recursion that
accumulates frames and no depth certificates; typed failures with a runtime that
acquires everything before `main` and never allocates after) are decisions, not
proposals, and this file builds on them rather than re-litigating them. The
supporting analysis those rulings were made against is extracted beside this file
in `EVIDENCE-owner-discussion-2026-08-31.md`.

## Contents

1. [Goals and non-goals](#1-goals-and-non-goals)
2. [The laws](#2-the-laws)
3. [The rules](#3-the-rules)
4. [How E is computed](#4-how-e-is-computed)
5. [The startup protocol](#5-the-startup-protocol)
6. [The writer's view](#6-the-writers-view)
7. [Two worked programs](#7-two-worked-programs)
8. [Open questions](#8-open-questions)
9. [What I verified and what I reasoned](#9-what-i-verified-and-what-i-reasoned)

## 1. Goals and non-goals

**Goals.** Turn every resource a Whitefoot program can exhaust into a named value it
must hold in order to consume, so that "this subtree never touches the heap" and
"this program's peak demand is this list of regions and slot counts" are facts a
signature and a compiler judgment carry rather than facts a reviewer hopes are
true; give the writer one declaration that turns the second fact into a
compilation requirement, so a program intended for a bounded machine fails at
compile time rather than at three in the morning; make every way of failing to
get a resource a typed value that returns the affine inputs it did not consume,
so no reachable path in an accepted program is a trap, an abort, or a silent
promotion to a bigger store; and put the compiler-derived cleanup, the `par`
runtime, and the target adapter *inside* the same envelope as the writer's code,
because a guarantee that stops at the edge of generated code is not a guarantee.

**Non-goals.** This design does not promise that a program terminates, that it
meets a deadline, that it gets CPU time, that a file exists, that a disk has
space, that a network answers, or that a host does not kill it; it does not
bound how many times a program acts, only what it holds at once and what it
consumes irreversibly; it does not make a general-purpose heap safe, and it
deliberately refuses to give a bounded general heap the resource-closed label,
because total free bytes do not answer a request for a contiguous aligned
extent; it does not add a depth certificate, a resource solver, a search for
allocator placements, or any acceptance path with a budget or a timeout; it does
not define the container operations that consume providers, which are
`CONTAINERS.md`'s; and it does not attempt the `par` continuation-frame redesign
that section 8's Q9 shows is the real obstacle to a resource-closed program that
uses parallelism.

## 2. The laws

Eight laws. Every rule in section 3 is an instance of one of them, and a rule
that cannot name its law is not admitted. They are stated first for the reason
`research/investigations/claim-model/DESIGN.md` states its four: the earlier
rounds of this design repaired sentences one at a time and kept reproducing the
same class of hole — a resource that was counted in one place and not another.

**L1 — The envelope is the program's promise.** *A resource-closed program
declares one finite, shaped envelope `E` and promises that on every legal
execution, and on every finite prefix of an infinite one, its demand for covered
resources never exceeds `E`; whether an environment supplies `E` is a separate
fact about the environment.*

The owner's correction of 2026-09-01 is the whole of this law: the causal order
is that the compiler proves a bound, the program promises never to exceed it,
and only then does a deployment decide whether it can meet it. Stating it the
other way — a program is resource-closed once some host has committed memory to
it — makes the property un-checkable at compile time, makes it depend on a
machine the compiler never sees, and gives a writer nothing to act on. Split
this way, `resource-closed(C, E)` is a static judgment about an artifact,
`Admitted(H, C, E)` is a run-time fact about a deployment, and the useful
theorem is their conjunction ([RUN-6]). A program that is not resource-closed is
not thereby broken; it has simply made no promise, and its environment owes it
nothing in return.

**L2 — No resource is ambient.** *Every covered resource enters the program as a
capability value the runtime hands to `main`, or as a store the program reserves
statically, and travels only by ordinary ownership; there is no ambient
allocator, ambient thread source, or ambient stack pool.*

Today the heap is ambient: I compiled a nullary leaf function that allocates a
buffer while holding nothing (`p5_ambient.wf`, section 9), and the only record of
it is an effect row. An effect row is a good record, but it is a description of
what a body did, not an authority the body had to hold, so "this call graph
cannot reach the allocator" is a whole-program re-derivation rather than a
signature fact. Making the store a value fixes that in one move: a function that
does not take a `Heap` cannot allocate from the heap, a `main` that does not
select `command.heap` closes the program, and the closure argument is the
ordinary parameter-reachability argument the language already uses for system
state ([FN-7] 1242, "there is no ambient system state"). It also removes
the last exception to that sentence, which is exactly the shape of hole L2 is
meant to prevent.

**L3 — Nothing fails silently, and nothing grows behind the writer.** *Every
operation that can fail to obtain a covered resource returns a typed value
naming the failure and handing back every affine input it did not consume; no
operation traps, aborts, retries, or promotes a store to a larger one.*

This is [SCOPE-2] and [EFF-4] applied to the one family they currently exempt.
The v0.40 language has no runtime-trap families left (line 6: "runtime-trap
families +0/-1 (0 remain)") and yet heap exhaustion still ends a process with a
fixed record and no source value ([SCOPE-3] 29, and the `wf_resource_abort` path
the stack ledger still reports in a corpus program — section 9). A design that
leaves that in place while claiming trap-freedom is not honest, and a design
that answers it with a trap in a nicer wrapper is worse. The typed value is the
only answer that composes: it can be matched, propagated, or refused by proof,
and the "refused by proof" case is [RES-8], which is how a partial operation has
always been admitted here.

**L4 — The runtime is inside the envelope.** *The artifact that `E` describes is
the writer's code, the compiler-derived cleanup and drop glue, the `par`
runtime, and the target adapter together; a resource any of them needs is in `E`
or the program is not resource-closed.*

The failure mode this law exists to kill is a program that proves its own
allocations bounded while the runtime beneath it creates a worker thread on the
first `par`, maps a diagnostic stack when a lane starts, initializes a completion
ring lazily, or reallocates a cleanup worklist — all four of which the current
runtime does, by the read of 2026-09-01. Under L4 those are not implementation
details outside the guarantee; they are line items, and the ones that cannot be
moved before the barrier disqualify the target from supporting resource-closed
programs at all ([RUN-2]).

**L5 — Shape, not bytes.** *`E` is a list of tangible resources — contiguous
aligned extents, per-class slot counts, per-context stacks, lane counts — and
never a single byte total, because a byte total cannot express the request a
fragmented store cannot serve.*

Sixteen bytes holding four four-byte objects, with the first and third released,
have eight free bytes and cannot serve an eight-byte request. Alignment is an
independent counterexample: a free eight-byte hole at an odd offset does not
hold an eight-byte object of eight-byte alignment. This is why a general heap —
including a *bounded* general heap, which was the owner's specific objection —
can never be part of a resource-closed guarantee: proving max-live-bytes proves
nothing about whether the next request has a home. What can be in `E` is a store
whose allocation rule makes the question decidable: uniform slots (a request is
serviceable iff the live count is below capacity), a bump cursor (serviceable iff
the aligned cursor plus the size stays in the extent), and static placement
(decided at compile time).

**L6 — Lowering before judgment.** *Tail recursion, including mutual tail
recursion, is rewritten into loops by the compiler before any resource judgment
runs; what the judgment sees is a call graph, and if that graph still has a
cycle the program has no finite stack envelope.*

An optimization that may or may not fire cannot be a premise of a guarantee. If
tail-call elimination is left to the backend, then whether a program is
resource-closed depends on an optimizer's mood, which violates the language's
standing rule that facts-off compilation is correct and that optimizer facts
never change acceptance. So the rewrite is a mandatory lowering with stated
admission conditions ([STK-1]), it happens before frames are measured, and what
survives it is judged by [STK-2] with no depth certificates — the owner's third
ruling, and the one that keeps the stack judgment to arithmetic over a DAG.

**L7 — Demand is computed as if every acquisition succeeds.** *The resource
judgment replays each execution assuming every covered acquire succeeds; it may
never conclude that demand is small because a failed acquisition would have
ended the program.*

Without this law the judgment is circular in a way that always answers yes: a
program whose first allocation fails and returns immediately has trivially
bounded demand, so "bounded demand" would be provable of everything. The
abstract semantics the judgment runs over therefore has no exhaustion edge at
all for covered domains; typed *external* failures (a file that does not exist)
stay in the traces, because they are outcomes of the world rather than of the
envelope.

**L8 — Stock, not flow.** *Resource-closedness bounds what is held at once and
what is consumed irreversibly; it never bounds how many times a program acts.*

A service loop that accepts a connection, serves it from a fixed buffer and
closes it can run forever with one live handle and one live buffer, and it is
exactly the kind of program this property is for. Requiring a bound on lifetime
event counts would exclude it while adding nothing: the resource that can run out
is the slot, not the event. The distinction has teeth in both directions — a
fixed append-only log *is* a consumable budget, so writes to it are counted,
while writes to a fixed ring log are not — and section 4.1 makes it the top-level
split of the domains.

## 3. The rules

Four new families. `[PROV-n]` is the capability values and their operations,
`[RES-n]` the covered set, the envelope and the judgment, `[STK-n]` the stack,
and `[RUN-n]` the runtime's own closure and the environment's half of the
bargain. Each rule states the judgment it creates, the fact it publishes, and
what it amends; section 3.5 collects the amendments in one table so nothing is
changed silently. The family is `[PROV]` and not `[CAP]` because `[CAP-1]`
already exists (1962) and rule ids are never reused — and the collision is worth
a sentence, because `[CAP-1]` says the kernel defines *no writer-visible
capability category and no system-specific permission*, and this design does not
add one. A provider is an ordinary affine value, held under `own` or `&uniq`,
judged by place overlap and by the ordinary effect row, and interfering with
other statements through exactly the vocabulary `[CAP-1]` names. The word
*capability* here means *a value you must hold in order to act*, which is what
`FilePermit` already is, and not a second permission system beside ownership.

### 3.1 `[PROV]` — capability values

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
introduces another. *Publishes:* for each provider place, the store identity that
[RES-6]'s domain algebra tracks. *Law:* L2.

**[PROV-2] Unforgeable and uncopyable.** No source construct produces a `Heap`. A
`Heap` value exists only because the runtime minted exactly one before `main` and
transferred it through the [FN-7] standard-input table; it is affine, so it is
moved rather than copied, and a program holds at most one for its whole
execution. An `Arena<'p>` or `Pool<'p, T, N>` exists only as the result of a
reserving operation [PROV-8]. There is no operation that duplicates, reconstructs,
compares, serializes, or derives a provider from a non-provider value.

*Judgment:* a `construct` [GRAM-8] naming a provider nominal, and any other
source route to one, is a hard error citing PROV-2 at the complete `construct`,
with the restructuring `receive the provider as a parameter, or reserve one with
pool_static or arena_static`. *Publishes:* uniqueness of the `Heap` — the fact
[STOR-3] needs in order to keep a `buffer<T>` drop attributable without storing
an allocator identity in the value. *Law:* L2.

**[PROV-3] Every covered-store operation takes its provider.** An operation that
allocates from a store takes that store's provider as a written parameter, `own`
or `&uniq 'p`, and exhibits it. The amended rows are:

```wf-ops-draft
| op | signature | effects |
|---|---|---|
| `buffer_new`     | `(heap: &uniq 'h Heap, count: u64, fill: T) -> own Result<buffer<T>, OutOfMemory>`      | allocates(heap) |
| `buffer_vacant`  | `(heap: &uniq 'h Heap, count: u64) -> own Result<buffer<Option<T>>, OutOfMemory>`       | allocates(heap) |
| `box_new`        | `(heap: &uniq 'h Heap, value: own T) -> own Result<box<T>, OutOfMemory<T>>`             | allocates(heap) |
| `arena_new`      | `(arena: &uniq 'p Arena<'p>, value: own T) -> own arena<'p, T>`                         | allocates(arena) |
| `arena_new_checked` | `(arena: &uniq 'p Arena<'p>, value: own T) -> own Result<arena<'p, T>, NeedCapacity<T>>` | allocates(arena) |
| `pool_take`      | `(pool: &uniq 'p Pool<'p, T, N>, value: own T) -> own slot<'p, T>`                      | allocates(pool) |
| `pool_take_checked` | `(pool: &uniq 'p Pool<'p, T, N>, value: own T) -> own Result<slot<'p, T>, PoolExhausted<T>>` | allocates(pool) |
| `pool_release`   | `(pool: &uniq 'p Pool<'p, T, N>, item: own slot<'p, T>) -> own T`                       | writes(pool) |
| `live` `capacity` | `(&'p Pool<'p, T, N>) -> own u64`                                                     | pure |
| `remaining`      | `(&'p Arena<'p>) -> own u64`                                                            | pure |
```

*Judgment:* an allocation call whose provider argument is missing, is not a
provider place, or is not writable is a hard error citing PROV-3 at the `call`.
*Publishes:* the provider place each allocation reaches, which is the footprint
[RUN-4] and the demand item [RES-6] both consume. *Amends:* the `box_new`,
`arena_new`, `buffer_new` and `buffer_vacant` rows of [OP-1] (793-798) and the
`buffer_fits` obligation siting of [OP-9] (968-978), which is unchanged as a
*representability* predicate and is not an availability predicate. *Law:* L2, L3.

**[PROV-4] `allocates` names a provider path.** The effect grammar's `allocates`
entry takes formal-rooted [EFF-1] paths naming provider state, in canonical
order, replacing the fixed atoms:

```wf-ebnf-draft
effect := "reads" "(" path ("," path)* ")"
        | "writes" "(" path ("," path)* ")"
        | "allocates" "(" path ("," path)* ")"
```

An `allocates(p)` entry is exhibited exactly when the body reaches an allocation
whose provider argument projects to `p` under [EFF-2]'s call-boundary
projection. A body that allocates only from a fresh local provider frames out of
its own signature exactly as any other fresh-local state does, and the reserving
operation that created that provider is what appears in `E`.

*Judgment:* [EFF-2]'s both-ways row check applies unchanged. *Publishes:* the
provider-reachability edge [PROV-6] closes over. *Amends:* [EFF-1]'s effect
production (1363-1372); *retires* the effect-row atoms `heap` and `arena`
(META-5: unique fixed lowercase grammar atoms −2). *Law:* L2.

**[PROV-5] The entry gains one row.** The `command` standard-input table [FN-7]
gains ordinal 5:

| ordinal | label | written mode and type | supplied value |
|---|---|---|---|
| 5 | `command.heap` | `own Heap` | the one general-purpose store the runtime minted before `main` |

The row is optional like every other. A `main` that omits it receives no `Heap`,
and by [PROV-2] cannot obtain one.

*Judgment:* the ordinary [FN-7] label, order, mode and type checks. *Publishes:*
the whole-program fact `heap-unreachable` when the row is absent. *Amends:*
[FN-7]'s table (1221-1227), its canonical five-input byte sequence (1239), and
its effect-row sentence (1214), whose `allocates(heap)` becomes `allocates` over
the entry's own labelled provider input — the sentence's own principle, that main's
row is rooted in its labelled inputs, now covers allocation too. *Law:* L2.

**[PROV-6] Heap-reachability is a closed signature fact.** A function *reaches the
heap* when its own row carries an `allocates` entry whose path is rooted in a
`Heap`-typed formal, or when it calls a function that does. Because the
compilation unit is closed [PROG-1], there are no function values, and there is
no ambient store [PROV-2], the transitive closure over the call graph is exact
and is computed from signatures alone.

*Judgment:* none by itself; it is the premise of [RES-5]. *Publishes:*
`heap-reaching path` — the ordered call chain from `main` to the allocation,
which is the diagnostic [RES-5] prints. *Law:* L2.

**[PROV-7] Provider-owned values and their release.** A value allocated from a
provider is released to that provider and to no other. For `Heap`, the provider
is unique [PROV-2], so `box<T>` and `buffer<T>` keep their present types, their
present storage class [STOR-1], their present compiler-derived free on the owner's
scope-exit edge, and their present empty release row [STOR-3, EFF-2]. For
`Arena<'p>`, an allocation's storage is returned when `'p`'s block ends, exactly
[STOR-4]; an individual drop returns nothing to the cursor. For
`Pool<'p, T, N>`, a `slot<'p, T>` abandoned at a scope exit inside `'p` has a
compiler-derived release that returns the slot to the pool, and that release
contributes `writes(pool)` under [EFF-2]'s release-contribution rule, with the
provider path identified by the slot type's region argument.

*Judgment:* on every edge carrying a slot release, the provider place must be
reachable and writable; a release edge on which the provider is uniquely
borrowed elsewhere is a hard error citing PROV-7 at the owning scope exit.
*Publishes:* the release event [RES-6]'s pool algebra consumes. *Amends:*
[STOR-3]'s release-action list gains one row and [EFF-2]'s "a `box<T>` drop, a
`buffer<T>` drop, an `arena<'r, T>` region release ... each carry the empty
release row" gains its exception. **This is the one rule in this half I could not
close** — the reachability side-condition is stated, not derived, and section 8's
Q2 gives the two candidate mechanisms. *Law:* L3, L4.

**[PROV-8] Reserving operations.** `pool_static<'p, T, N>()` and
`arena_static<'p, BYTES, ALIGN>()` each reserve one statically laid-out extent
*per source occurrence* and return the provider confined to `'p`. The reserved
extent is an ordinary place for [OWN-5] and for every footprint rule. Because
`'p` is lexical, at most one provider per occurrence is live at any program
point; because the call graph of a resource-closed program is acyclic [STK-2], no
occurrence is re-entered while its provider is live.

*Judgment:* the ordinary region and confinement judgments [OWN-3, OWN-4,
STOR-4]. *Publishes:* one static-extent item of `E`, with size and alignment from
`T`, `N`, `BYTES` and `ALIGN`. *Amends:* nothing; adds two operation rows.
*Note:* two overlapping [PAR-1] statements that both reach one occurrence's
extent are denied by the ordinary footprint rule, with no new clause — the extent
is a place, and both statements write it. *Law:* L2, L5.

### 3.2 `[RES]` — the covered set, the envelope, and the judgment

**[RES-1] `CoreResources`.** Bare *resource-closed* means resource-closed for
exactly this set, and for no more:

| class | members |
|---|---|
| execution memory | the static image; every frame of every execution context; every worker-lane stack; every provider backing reserved by [PROV-8]; allocator and runtime metadata; compiler-derived cleanup scratch; the target adapter's own persistent buffers |
| execution capacity | `par` lanes; task records; submission, completion and wait records; queue slots; the runtime's fixed internal handle capacity |

An extension is written and never implied: `resource_closed(core + file_handles)`
is a different, stronger declaration, admitted only when the environment can
deliver an exclusive reservation of that kind, and no such extension is defined
in this version.

*Judgment:* fixes the domains [RES-3] quantifies over. *Publishes:* the name.
*Law:* L1, L4.

**[RES-2] The envelope `E`.** `E = E(P, T, C)` is a finite list of shaped items
computed for one program `P`, one selected target and ABI `T` [STOR-6], and one
runtime configuration `C` (principally the lane count `W`). Each item is one of:

```text
region(name, bytes, alignment, contiguous)   one extent the environment must deliver whole
slots(kind, count)                           interchangeable fixed-size records
stack(context, bytes)                        one execution context's maximum chain
lanes(count)                                 scheduler lanes, including the entry lane
```

No item is a bare byte total, and no two items are summed into one. Items are
not fungible: two `region` items are two extents.

*Judgment:* `E` is well-formed only if every item's arithmetic was performed in
the unbounded mathematical domain and is representable on `T`, the same standard
[STOR-6] already applies. *Publishes:* `E` itself, as a compilation artifact.
*Law:* L5.

**[RES-3] The resource-closed judgment.** For a program `P`, target `T`,
configuration `C` and envelope `E`, `resource-closed(P, T, C, E)` holds exactly
when, for every legal execution trace of the artifact `C(P)` [RUN-1] and every
finite prefix of that trace, replaying the prefix's covered acquisitions and
releases from `E` under each domain's own algebra [RES-6] leaves every acquisition
defined and every domain invariant intact. The traces are drawn from the
abstract demand semantics of L7, in which no covered acquisition fails.

*Judgment:* per domain, the composition of section 4.2 over the checked program
after [STK-1]'s lowering; deterministic, terminating, and free of search, budget
or timeout, as every acceptance judgment in this language must be. *Publishes:*
the property, and `E`. *Law:* L1, L7, L8.

**[RES-4] The entry requirement.** The entry may carry the marker
`resource_closed` before its `command` program-kind marker:

```wf-draft
resource_closed command fn main() -> status: own ExitStatus writes(uart) {
```

The marker changes no other rule. Every program is judged by exactly the same
rules; the marker only makes failure to establish [RES-3] a hard error citing
RES-4 rather than a reported property. There is no second language, no subset
mode, no alternative prelude, and no per-function marker: a *program* is
resource-closed or is not.

*Judgment:* on a marked entry, the first unestablished premise of [RES-3] is a
hard error naming its own cause — the heap-reaching path [RES-5], the call-graph
cycle [STK-2], the unbounded store [RES-6], or the unclosed runtime [RUN-2].
*Publishes:* nothing new. *Amends:* [FN-7], which fixes main's marker set.
*Law:* L1.

**[RES-5] The heap excludes resource-closedness.** A program whose call graph
reaches a `Heap` [PROV-6] is not resource-closed, and a `main` selecting
`command.heap` is by itself the rejection. A bounded general store is still a
general store: an envelope item can promise bytes, and cannot promise that the
next contiguous aligned request has a home.

*Judgment:* under [RES-4], a hard error citing RES-5 at the offending
`input_label` or at the deepest `call` of the heap-reaching path, rendering the
whole chain. *Publishes:* nothing. *Law:* L5.

**[RES-6] Store domains and their algebras.** Exactly three covered-store
domains are defined, each with its own deterministic state and transfer rules.
Nothing else is admitted, and a store outside this list contributes no envelope
item and denies [RES-3].

| domain | state | acquire | release | serviceable when |
|---|---|---|---|---|
| uniform slots (`Pool<'p, T, N>`, lane/task/completion records) | live count | +1 | −1 at the release event [PROV-7] | `live < N` |
| bump extent (`Arena<'p>`) | cursor, alignment | cursor advances by `round_up(cursor, align(T)) - cursor + size(T)` | nothing; the whole extent returns when `'p` ends | `remaining >= that advance` |
| static and frame placement | fixed offsets | none at run time | none | decided at compile time [STOR-6] |
| general heap (`Heap`) | — | — | — | **undecidable from `E`; excluded by [RES-5]** |

*Judgment:* the (peak, delta) composition of section 4.2 is admitted for the
uniform-slot domain only; the bump extent uses the monotone cursor domain, in
which a drop returns nothing. *Publishes:* per program point, per domain, the
live count or cursor bound. *Law:* L5.

**[RES-7] Typed failure, and what it retires.** Every operation that can fail to
obtain a covered resource returns a typed value that names the failure and hands
back every affine input it did not consume: `OutOfMemory`, `OutOfMemory<T>`,
`PoolExhausted<T>`, `NeedCapacity<T>`, and the container half's `Full<T>` and
`TooSmall`. No covered-resource failure is a trap, an abort, a process exit, a
retry, or a promotion to a larger store, in the writer's code or in the runtime.

*Judgment:* the ordinary [ERR-3]/[OWN-13] handling of a `Result`. *Publishes:*
on the `Err` edge, the returned owner's identity. *Retires:* the heap arm of the
exhaustion floor — the `wf_resource_abort` site for allocation refusal (batch
0079 item F4, `docs/done/0079-exhaustion-floor.md`) has no reachable caller once
allocation returns a value, and the stack arm survives only for programs that are
not resource-closed, since [STK-2] makes the overflow it reports unreachable in
one that is. *Amends:* [SCOPE-3] 29, whose "heap exhaustion ... may stop
execution at the host boundary without a Whitefoot value" ceases to be true of
the heap. *Law:* L3.

**[RES-8] A covered acquisition is a partial operation.** Each covered-store
acquisition comes in exactly two spellings, on the model of `+` and `+checked`
[OP-1]:

```text
pool_take(pool: p, value: v)          domain obligation  ilt(live(p), capacity(p))     -> own slot<'p, T>
pool_take_checked(pool: p, value: v)  total                                            -> own Result<slot<'p, T>, PoolExhausted<T>>
arena_new(arena: a, value: v)         domain obligation  ige(remaining(a), reserve<T>()) -> own arena<'p, T>
arena_new_checked(arena: a, value: v) total                                            -> own Result<arena<'p, T>, NeedCapacity<T>>
```

The proved form is admitted only when [ENT-6] discharges its exact goal in the
current ProofContext; an unproved goal is a static rejection with no fallback,
exactly as an unproved subscript is [OP-4]. `live`, `capacity` and `remaining`
are pure total queries whose results enter the proof context as ordinary
typed terms, and the transfer rules of [RES-6] publish `live' = live + 1` at a
take and `live' = live - 1` at a release, the same way [SYS-9] publishes a
system operation's enumerated relations. **The `Heap` has no proved form**: no
honest domain predicate exists for a general store (L5), so `buffer_new`,
`buffer_vacant` and `box_new` are total and return `Result` unconditionally.

*Judgment:* [ENT-6] discharge at the proved spelling; nothing at the checked
one. *Publishes:* the post-state relation on the store's live count or cursor.
*Law:* L3, L5.

**[RES-9] What bare resource-closedness does not cover.** Disk space, the
successful acquisition of a file, socket or other host object not exclusively
reserved before start, network reachability and throughput, CPU time, deadlines,
scheduler fairness, power, device health, host termination, and OS quota
revocation are outside [RES-1] and are outside every judgment in this file. They
remain typed system outcomes where the operation defines one ([SYS-7]'s error
classes), and environment conditions where it does not ([RUN-6]).

*Judgment:* none; a boundary statement. *Publishes:* nothing. *Law:* L1.

### 3.3 `[STK]` — the stack

**[STK-1] Tail lowering runs before every resource judgment.** For each strongly
connected component of the call graph in which *every* intra-component call edge
is in true tail position, the compiler rewrites the component into one dispatcher
loop before frames are measured. The rewrite is admitted only when, on every
intra-component edge: the call is the complete `expr` of a `return_stmt`; no
compiler-derived drop or release remains to run after it [STOR-3]; no child
reborrow of a caller-local place is live across it [OWN-6]; no `par` join is
outstanding; every affine local has been moved or released before the jump; and
the dispatcher frame is the maximum over the component's members of their
parameter and local state. A component that fails any condition is not rewritten.

*Judgment:* structural and per-edge; no proof search. *Publishes:* an acyclic
call graph, or a component that is still cyclic. *Amends:* nothing in [FN-6],
which continues to permit recursion; this is a lowering, not an admission rule.
*Law:* L6.

**[STK-2] Acyclicity is required, and there are no depth certificates.** After
[STK-1], a program whose call graph still contains a cycle has no finite stack
envelope and is not resource-closed. A `requires` bound on a recursion parameter,
a proof that a recursion argument decreases, and any other depth certificate are
**not** admitted as a substitute; this design defines no such construct and no
compiler is asked to discover one.

*Judgment:* under [RES-4], a hard error citing STK-2 that renders the complete
cycle in call order and the restructuring `rewrite the recursion as a loop over
an explicit FixedVector work list, or make every recursive call a tail call`.
*Publishes:* the cycle. *Law:* L6.

**[STK-3] The frame envelope.** For each execution context, the stack item of `E`
is the maximum over the context's entry points of

```text
Stack(f) = frame(f) + max over every possibly-active callee g of ( call_overhead(f, g) + Stack(g) )
```

taken over the acyclic graph of [STK-2], where the possibly-active callees include
those reached on error and propagation edges, compiler-derived drop glue, the
target adapter's helpers, the `par` worker entry and resume paths, and the ABI
save area. `frame(f)` is measured **after final code generation**, because the
frame a function actually needs is made of things that do not exist earlier —
the ABI frame record a non-leaf keeps and the callee-saved registers the
allocator chose to spill. An optional optimization may not raise a computed
envelope: an implementation either recomputes `E` after the optimization and
publishes the larger figure, or declines the optimization; it never publishes
one figure and emits code needing another.

*Judgment:* target-stage arithmetic under [STOR-6]'s checked-arithmetic
discipline. *Publishes:* one `stack(context, bytes)` item per context.
*Amends:* [STOR-6] 757-761, whose "the language therefore defines no numeric
per-array, per-object, or per-function frame ceiling" stands for the *language*
and is now joined, for a resource-closed program, by a computed per-context
envelope; and whose optional target-side conservative frame limit stops being the
only mechanism. *Law:* L4, L5.

**[STK-4] One item per execution context, and reentrancy is not free.** `E`
carries a `stack` item for the entry context and for each of the `W - 1` worker
lanes, plus one for every context the target profile introduces — a completion
helper, a bounded blocking helper, an interrupt or signal stack, an FFI callback
stack. A context whose depth cannot be bounded because control can re-enter the
Whitefoot call graph from outside it (a signal handler, an FFI callback, a host
reentrancy path) denies resource-closedness unless the profile gives it a
separately reserved stack item of its own.

*Judgment:* a profile-level check, not a source check. *Publishes:* the item
list. *Law:* L4.

**[STK-5] Stack exhaustion moves inside the model, for these programs only.**
For a resource-closed program, stack exhaustion is not a deferred external
resource condition: [STK-2] and [STK-3] make the maximum chain a computed item of
`E`, and under an admitted run [RUN-6] it is unreachable. For every other
program, [SCOPE-3]'s deferral stands unchanged, and so does the guard-page floor
that reports it.

*Judgment:* none; a scope statement. *Amends:* [SCOPE-3] 29-31. *Law:* L1.

### 3.4 `[RUN]` — runtime closure and admission

**[RUN-1] The artifact.** For every judgment in this file the artifact `C(P)` is
the writer's code, the compiler-derived cleanup and drop glue, the monomorphized
instances, the `par` runtime, the allocator and its metadata, and the qualified
target adapter — everything the process runs between the barrier and
`ProgramFinished`. *Law:* L4.

**[RUN-2] Runtime closure.** A runtime qualified for resource-closed programs
performs, after the `SourceStart` barrier and until `ProgramFinished`, no
covered acquisition whatsoever: no allocator call for runtime-owned storage, no
thread or helper creation, no stack, queue, table or worklist growth, no lazy
TLS or adapter initialization, no first-use mapping, and no first-error
formatting buffer. Every runtime record is established before the barrier or is
carved from a fixed backing that is already an item of `E`. Internal saturation
is answered by waiting, reuse, inline execution, or a semantically equivalent
sequential path, and never by growth, abort, or a source-visible outcome.
Teardown after `main` returns is under the same obligation.

*Judgment:* a conformance obligation on an implementation, auditable from the
emitted code and the runtime's own translation units; it is not a source
judgment and no source construct can weaken or waive it. An implementation that
cannot meet it does not support the `resource_closed` marker on that target, and
must say so rather than accept the marker. *Publishes:* the runtime's own items
of `E`. *Law:* L3, L4.

**[RUN-3] `par` enters `E` as a profile, not as an iteration count.** For each
supported lane count `W`, the runtime publishes one finite profile row: `W`
lanes (of which `W - 1` are host worker threads), `W - 1` worker stacks, a fixed
task-record count `K(W, d)` where `d` is the program's maximum nested `par`
depth, fixed queue capacities, and a fixed completion-record count. The number of
iterations of a `par`-permitted loop never appears in `E`: the runtime chunks the
index range lazily, so a loop of a billion iterations holds no more task records
than one of a thousand. On saturation the runtime executes inline, helps, waits,
or applies backpressure.

*Judgment:* a fixed-arithmetic composition (section 4.2's `par` rule) against
the selected profile row; the compiler emits no per-`W` clone of the program.
*Publishes:* the `lanes` and `slots` items of `E`. *Amends:* the sentence common
to [PAR-1] 1985, [PAR-2] 2020 and [PAR-3] 2049, "exhaustion of the execution
resources an implementation spends on overlapping is a resource condition under
[SCOPE-3] and is not an observable of this rule": for a resource-closed program
that exhaustion is not merely unobservable, it is unreachable, because the
resources are items of `E` and the saturation protocol above is what the runtime
does instead of acquiring more. *Law:* L4, L8.

**[RUN-4] The parallel footprint of an allocation is its provider place.** In
[PAR-1]'s written-footprint clause, "the caller region each `allocates(arena 'r)`
entry names after region substitution" is replaced by "the places each
`allocates` path reaches under the [EFF-2] call-boundary projection" — the same
projection the rule already applies to `reads` and `writes`. Two statements that
allocate from one provider therefore conflict, and two that allocate from
distinct providers do not.

*Judgment:* the existing [PAR-1] overlap judgment, with one fewer special case.
*Publishes:* nothing new. *Amends:* [PAR-1] 1969, and [PAR-2]/[PAR-3] through
their "forms every footprint exactly as [PAR-1] forms one" clauses. *Note:* the
clause is now largely redundant, since an allocation needs `&uniq` on its
provider and [OWN-5] exclusivity already denies two live exclusive loans on one
place; it is retained because an `own`-mode provider argument is not a loan.
*Law:* L2, L4.

**[RUN-5] The startup protocol.** Program start has four points, and the
covered guarantee spans the last three: `PreStart`, in which the launcher and
runtime materialize `E` and may fail; the `SourceStart` barrier, crossed only
when every item of `E` is established; `Running`, from the first statement of
`main`; and `ProgramFinished`, after `main` returns, every compiler-derived
release on the return edge has run, every outstanding `par` and completion record
has drained, and the runtime's bounded teardown is complete. A `PreStart` failure
is reported as `StartFailed(item)` on an implementation-defined channel using
fixed, preallocated storage; no source statement executes, no owner comes into
existence, no language cleanup runs, and no `ExitStatus` is produced.

*Judgment:* a target obligation, not a source judgment. *Publishes:* nothing to
source. *Amends:* [PROG-3] 1499-1509, whose start-time obligation gains the
materialization of `E` and whose `ProgramFinished` boundary is now named; its
existing "a start failure is a target or environment failure ... not a
source-language rejection" is exactly right and is kept verbatim. *Law:* L1, L4.

**[RUN-6] Admission, and the theorem.** `Admitted(H, C, E)` holds when an
environment `H` has actually established a grant implementing every item of `E`
before the barrier — committed backing rather than a reserved address range, real
lanes at their ready barrier, real queues and records — and, for the duration of
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

### 3.5 What this design amends, and what it retires

| existing rule | line | change | by |
|---|---|---|---|
| [SCOPE-3] | 27-31 | heap exhaustion leaves the deferred set (it becomes a value); stack exhaustion leaves it for resource-closed programs only; startup-resource failure stays outside, by name | [RES-7], [STK-5], [RUN-5] |
| [TYPE-2] | 352 | three opaque provider nominals join the opaque system class; `slot<'p, T>` joins the region-bearing types | [PROV-1] |
| [SET-2] | 508 | unchanged; a `slot<'p, T>` is region-bearing, so a stored one is already refused | — |
| [OWN-1] | 558 | unchanged; providers and slots are affine like every other owned composite | [PROV-1] |
| [OWN-5] | 580 | unchanged; a provider is an ordinary place and exclusivity does the work | [RUN-4] |
| [CAP-1] | 1962 | unchanged, and deliberately: providers add no capability category, no permission kind, and no second interference vocabulary; `own`, `&`, `&uniq`, place overlap and the effect row remain the complete authority | [PROV-1] |
| [STOR-1] | 670 | "`buffer<T>` is heap-owned" survives, but construction now requires the `Heap` value; storage class is still a function of type, not of an annotation | [PROV-3], [PROV-7] |
| [STOR-3] | 683 | one new release row (a pool slot returns to its pool, contributing `writes(pool)`); box/buffer/arena rows unchanged | [PROV-7] |
| [STOR-5] | 718 | `Arena<'p>`, `Pool<'p, T, N>` and `slot<'p, T>` join the region-bearing set and are therefore unstorable; `Heap` is not region-bearing and is an entry-owned value | [PROV-1] |
| [STOR-6] | 733-761 | the "no numeric frame ceiling" sentence keeps its scope; a computed per-context stack envelope is added for resource-closed programs, measured post-codegen | [STK-3] |
| [OP-1] rows | 793-798 | `box_new`, `buffer_new`, `buffer_vacant`, `arena_new` take a provider and (on `Heap`) return `Result`; eight rows are added | [PROV-3], [RES-8] |
| [OP-9] | 968 | unchanged in meaning: `buffer_fits` stays a representability predicate and never becomes an availability predicate | [RES-8] |
| [FN-6] | 1205 | unchanged: recursion stays permitted; it merely excludes a program from [RES-4] | [STK-2] |
| [FN-7] | 1210-1253 | one new input row `command.heap`; one new entry marker `resource_closed`; main's effect row admits `allocates` over its own labelled provider | [PROV-5], [RES-4] |
| [EFF-1] | 1363-1372 | `allocates` takes paths; the atoms `heap` and `arena` retire | [PROV-4] |
| [EFF-2] | 1386 | one exception to the empty-release-row sentence, for pool slots | [PROV-7] |
| [PAR-1] | 1969 | the `allocates(arena 'r)` region clause becomes the ordinary provider-place projection | [RUN-4] |
| [PAR-1/2/3] | 1989, 2024, 2049 | "execution-resource exhaustion is a [SCOPE-3] condition" gains the resource-closed case, in which it is unreachable | [RUN-3] |
| [PROG-3] | 1499 | the start-time obligation includes materializing `E`; `ProgramFinished` is named | [RUN-5] |
| [SYS-2] | 2264 | "no system operation allocates" is kept and strengthened: an adapter's own records come from `E` | [RUN-2] |
| [SYS-8] | 2482 | unchanged; range-bearing operations already borrow caller storage and allocate nothing | — |
| batch 0079 exhaustion floor | `docs/done/0079-exhaustion-floor.md` | the heap-refusal abort site loses its last reachable caller; the stack guard-page record survives only for programs that are not resource-closed | [RES-7] |

### 3.6 The seam `CONTAINERS.md` flags, answered

`CONTAINERS.md` records one open seam against this half: `Pool<'p, T, N>` names
`N` interchangeable single-`T` slots, and a `PoolVector` needs one **contiguous
run** of them. This file answers it, because the answer is a resource question.

A pool that serves *runs* of `k` slots is not a uniform-slot domain: whether a
run of 3 is serviceable is not decided by the live count, and the sixteen-byte
counterexample of L5 reappears one level down, at slot granularity. Adding a
run-lease to `Pool` would therefore take the pool out of [RES-6]'s admitted
domains and out of every envelope — the opposite of what the container wants
from it.

The shape that keeps the algebra is to lease **one slot whose content is the run**:

```wf-draft
let blocks = pool_static<'p, FixedVector<Record, 256>, 8>();
let empty = fixed_vector_new<Record, 256>();
let leased = pool_take(pool: &uniq 'p blocks, value: move empty);
```

The pool still holds eight interchangeable slots of one type, `live < 8` still
decides serviceability, and `PoolVector<'p, Record>` is exactly a lease of such a
slot with `cap = 256` fixed at lease — which is what `CONTAINERS.md`'s own
inventory row already says. Nothing new is needed on this side: a `FixedVector`
is frame-resident storage [STOR-1] and is not region-bearing, so it is a legal
slot content type, and its initialized-prefix typestate lives inside the value
where the container half already keeps it.

What this costs is that a pool's *block size* is fixed at reservation rather than
at lease, so a program wanting two block sizes reserves two pools and `E` names
both — which is exactly the shape L5 says an envelope has to have. The residual
for the container half is only whether it is content to have `cap` fixed at
reservation; if it needs a lease-time capacity, that is a new question and it is
a fragmentation question, not a container question.

## 4. How `E` is computed

### 4.1 The three kinds of quantity

Every covered resource is one of three kinds, and conflating them is the single
most common way to get a wrong answer (L8).

| kind | question | examples | bound |
|---|---|---|---|
| reusable capacity | how many are held **at once**? | pool slots, task records, completion records, lanes, queue slots | peak live |
| consumable budget | how much is **spent and not returned** in this epoch? | arena cursor bytes, a fixed append-only log's records | net consumed |
| external effect flow | how many times did it happen? | opens, writes, submissions | **not bounded, and not part of `E`** |

An infinite service loop that takes a slot, uses it and releases it has peak
live 1 and unbounded effect flow, and is resource-closed. A loop that takes a
slot per iteration and retains it needs an iteration bound or a structural
capacity cutoff, or it has no finite `E`.

### 4.2 Composition

Per resource kind `r`, a straight-line segment has a summary `(peak_r, delta_r)`
relative to its entry state: `peak` is the maximum additional simultaneous
occupancy during it, `delta` the additional occupancy still held on leaving it.
The primitive transfers are fixed:

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
           delta      = kept per exit variant, not joined into one interval when
                        a Result or Option variant already distinguishes the arms

call       substitute the callee's per-exit summary at the call site, with its
           formal capacity and provider terms replaced by the actual ones

loop       delta at the backedge = 0            -> peak is one iteration's peak; no
                                                   iteration bound is needed
           delta at the backedge > 0            -> a counted range, a structural
                                                   capacity cutoff (len <= N), or a
                                                   writer-supplied resource invariant
                                                   is required; otherwise no finite E

par        peak = (M - K) * d + K * p   for M logical iterations, per-iteration
                                        peak p and retained d, and K the profile's
                                        maximum concurrently live iterations
```

The `par` formula is the one place where a wrong constant is easy: `K` is the
runtime profile's window, not the lane count `W`, because an iteration that
submits a `may-suspend` operation and suspends leaves its resources outstanding
while its lane starts another. Scratch that each iteration releases scales with
`K`; output each iteration retains until the join scales with `M`. They are
different kinds and are never merged into one figure.

The (peak, delta) algebra is admitted for the uniform-slot domain only. The bump
extent uses its own monotone cursor domain, in which release is a no-op and only
the region's end returns the extent; the general heap has no admitted algebra at
all.

**What needs no writer annotation:** straight-line acquire/move/borrow/release;
lexical scopes and compiler-derived cleanup edges; branch joins; per-variant
retention that a `Result` or `Option` already distinguishes; `FixedVector`'s
`len <= N` and its initialized prefix (`CONTAINERS.md`); moving an owner into or
out of a container; a loop whose backedge restores the state; a counted loop
whose per-iteration delta is a fixed affine expression; a non-recursive call with
a computed summary; and a `par` loop composed by the formula above.

**What needs one:** a loop that may retain with no structural cutoff; a relation
across two containers (`len(active) + len(waiting) <= capacity`); a resource
returned only at a later milestone; an acquisition whose size is a computed
value; a `par` window the profile does not fix; and any place where the writer
wants a tighter answer than the per-branch maximum. These are written as ordinary
[INV-1] invariants over `live`, `capacity` and `remaining`, and the checker
verifies base, preservation and exit exactly as it does for any other invariant.
The checker never searches for one: it does not enumerate paths, guess loop
invariants, choose allocator placements, or divide a store between claimants.

### 4.3 Compile-time judgment versus start-up step

| # | what | when | who |
|---|---|---|---|
| 1 | tail-SCC rewrite [STK-1] | compile time, before any resource judgment | compiler |
| 2 | call-graph acyclicity [STK-2] | compile time, on the rewritten graph | compiler |
| 3 | provider reachability, heap-freedom [PROV-6] | compile time, from signatures | compiler |
| 4 | per-function per-domain demand summaries | compile time, bottom-up on the DAG | compiler |
| 5 | loop and `par` composition (4.2) | compile time | compiler |
| 6 | static-extent items from [PROV-8] occurrences | compile time | compiler |
| 7 | concrete sizes, alignments, strides, static image | target stage [STOR-6] | compiler |
| 8 | per-context frame envelope [STK-3] | target stage, **after code generation** | compiler |
| 9 | runtime profile row for each supported `W` [RUN-3] | fixed data of the qualified runtime | runtime |
| 10 | assembling `E` and emitting it as an artifact | target stage | compiler |
| 11 | choosing `W` for this run | `PreStart` | launcher |
| 12 | committing every `region` and `stack` item | `PreStart` | launcher/runtime |
| 13 | creating lanes and reaching the ready barrier | `PreStart` | runtime |
| 14 | initializing every adapter record and queue | `PreStart` | runtime |
| 15 | crossing `SourceStart` and invoking `main` | `PreStart` -> `Running` | runtime |

Steps 1-10 decide whether the program is resource-closed; steps 11-15 decide
whether this run is admitted. Neither judgment consults the other.

## 5. The startup protocol

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

**The entry runs on a stack the runtime owns.** `stack.entry` is an item of `E`
like any other, which means the entry context cannot be "whatever the host's
process stack left over": the runtime creates and commits it, exactly as it
already creates a runtime-owned entry stack today. The gap between today and
this design is one word — the current one is a large *reservation*, and [RUN-6]'s
`Admitted` needs a *commitment* — and on a host with overcommit those are
different facts about the same address range.

**How a failure before `main` is reported.** `StartFailed(item)` names the
envelope item that could not be established, in fixed preallocated storage on an
implementation-defined channel, exactly as [PROG-3] already reports a start
failure. It is not a source `Result`, not `main`'s return value, not an
`ExitStatus`, not a language trap, and not a source-language rejection [DIAG-1].
Partially acquired host resources are rolled back by the launcher.

**Why it is not a program failure.** The program's promise is conditional by
construction (L1): it undertook never to ask for more than `E`, and it has not
asked for anything, because no statement of it has run. Nothing the program could
have done would have made `E` smaller — `E` is what the compiler proved, and the
compiler had no smaller figure to offer. Reporting the failure as a source
outcome would also require a source value to exist before the source has started,
and would put a resource-acquisition path inside the very window [RUN-2] closes.
The honest split is the one [PROG-3] already draws and this design keeps: the
environment declines at the door, or admits and then owes the grant for the
duration.

## 6. The writer's view

The declaration is one marker on the entry:

```wf-draft
resource_closed command fn main() -> status: own ExitStatus pure {
```

Everything else the writer sees is a diagnostic. Each of the three walkthroughs
below is the same shape: a writer who did not know about any of this writes the
obvious program, gets one error that names the offending structure, and makes one
structural change.

### 6.1 From "it needs a growable vector" to a fixed store

```wf-draft
resource_closed command fn main(command.heap as heap: own Heap) -> status: own ExitStatus allocates(heap) {
  doc "Collects every decoded record.";
  let records = heap_vector_new<Record>(heap: &uniq 'h heap);
  ...
}
```

```text
error [RES-5]: this program cannot be resource-closed: it reaches the general heap
  --> records.wf:1:33
   |
 1 | resource_closed command fn main(command.heap as heap: own Heap) ...
   |                                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the Heap enters here
   |
   = heap-reaching path:
       main                      records.wf:1:1
         heap_vector_new<Record> records.wf:3:17
           buffer_new            (operation table)
   = a general store cannot appear in an envelope: total free bytes do not decide
     whether the next contiguous aligned request has a home
   = restructure: give the store a capacity the compiler can check --
     FixedVector<Record, N> over static or frame storage, or PoolVector over a
     Pool<'p, Record, N> reserved by pool_static; or drop the resource_closed
     marker and handle OutOfMemory as a value
```

The fix is a capacity decision, which is the decision the writer was avoiding:

```wf-draft
resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Collects up to 512 decoded records.";
  let records = fixed_vector_new<Record, 512>();
  ...
}
```

### 6.2 From recursive descent to an explicit work list

```wf-draft
fn parse_node['a](input: &'a Span<u8>, at: own u64) -> result: own Node pure {
  doc "Parses one node, recursing into its children.";
  ...
  let child = parse_node(input: input, at: next);
  ...
}
```

```text
error [STK-2]: this program cannot be resource-closed: its call graph has a cycle
  --> parse.wf:1:1
   |
 1 | fn parse_node['a](input: &'a Span<u8>, at: own u64) -> result: own Node pure {
   | ^^^^^^^^^^^^^^^^^ this function is in the cycle
   |
   = cycle, in call order:
       parse_node   parse.wf:5:15  -> parse_node
   = the call at parse.wf:5:15 is not in tail position: its result is used at
     parse.wf:6:3, so [STK-1] cannot rewrite the component into a loop
   = a recursion depth bound is not accepted here: this version admits no depth
     certificate, so a cycle has no finite stack envelope
   = restructure: carry the pending nodes in an explicit FixedVector<Frame, N>
     work list and loop, or make every recursive call the complete return
     expression
```

The fix turns the implicit stack into a declared one, and the declared one is an
envelope item the writer chose:

```wf-draft
fn parse['a](input: &'a Span<u8>) -> result: own Result<Node, TooDeep> pure {
  doc "Parses the whole input with an explicit pending-node stack.";
  let pending = fixed_vector_new<Frame, 64>();
  loop @walk {
    ...
  }
}
```

### 6.3 From an unbounded store to a bounded one

```wf-draft
  for @fill (i in 0_u64..count) {
    let blank = Page(bytes: array_new<u8, 4096>(0_u8));
    let page = pool_take(pool: &uniq 'p pages, value: move blank);
    let stored = fixed_vector_push(vector: &uniq 'v kept, value: move page);
  }
```

```text
error [RES-6]: this loop's demand on 'p has no finite bound
  --> pager.wf:12:5
   |
12 |     let page = pool_take(pool: &uniq 'p pages, value: blank);
   |                ^^^^^^^^^ acquires one slot of Pool<'p, Page, 64>
13 |     let stored = fixed_vector_push(vector: &uniq 'v kept, value: move page);
   |                  ^^^^^^^^^^^^^^^^^ retains it past the backedge
   |
   = per iteration: peak 1, delta +1 on 'p
   = the loop runs 0..count, and count is a runtime value with no upper bound in
     this ProofContext, so the peak is unbounded
   = supply one of:
       a requires on count (`requires ile(count, 64_u64);`)
       a structural cutoff in the loop (`if ieq(len(kept), 64_u64) { break @fill; }`)
       an invariant relating the two stores
         (`invariant slots_match_vector: ieq(live(pages), len(kept))`)
       or the checked spelling, and handle PoolExhausted
```

The proved spelling and the checked spelling are both accepted, and both keep
the program resource-closed — this is [RES-8], and it is what the writer picks
between:

```wf-draft
  for @fill (
    i in 0_u64..count,
    invariant kept_fits: ile(len(kept), 64_u64)
  ) {
    let room = ilt(len(kept), 64_u64);
    if room {
      let blank = Page(bytes: array_new<u8, 4096>(0_u8));
      let page = pool_take(pool: &uniq 'p pages, value: move blank);
      let stored = fixed_vector_push(vector: &uniq 'v kept, value: move page);
    } else {
      break @fill;
    }
  }
```

or, when the writer would rather answer the refusal than prove it away:

```wf-draft
    let blank = Page(bytes: array_new<u8, 4096>(0_u8));
    let attempt = pool_take_checked(pool: &uniq 'p pages, value: move blank);
    match attempt {
      Err(value: refused) => {
        let recovered = move refused.rejected;
        break @fill;
      }
      Ok(value: page) => {
        let stored = fixed_vector_push(vector: &uniq 'v kept, value: move page);
      }
    }
```

## 7. Two worked programs

Both programs are written in the current concrete syntax wherever the current
syntax reaches. **Every form this design adds is new and compiles nowhere:** the
`resource_closed` entry marker, the provider types and their operations, the
`command.heap` input row, `allocates` over a path, `live`/`capacity`/`remaining`,
and the container operations `CONTAINERS.md` owns. Byte figures in the envelopes
are illustrative; no implementation computes them, and none was measured.

### 7.1 A kernel-flavoured program that comes out resource-closed

A fixed-capacity run queue, a 256-byte UART transmit ring, and a 64-page pool
with typed exhaustion. It has no heap, no recursion, an acyclic call graph, and a
scheduler loop whose resource state is restored on every backedge.

```wf-draft
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
  define fill_within = ilt(fill, 256_u64);
  requires head_within;
  requires fill_within;
  ensures ilt(result, 256_u64);
} {
  doc "Computes one wrapped write position of a 256-byte ring.";
  let at = head + fill;
  let over = ige(at, 256_u64);
  if over {
    let wrapped = at - 256_u64;
    return wrapped;
  }
  return at;
}

fn ring_push['r](ring: &uniq 'r UartRing, byte: own u8) -> result: own unit reads(ring), writes(ring) {
  doc "Appends one byte, overwriting the oldest when the ring is full; overwriting is this ring's semantics, not a resource failure.";
  let head = deref(ring).head;
  let fill = deref(ring).fill;
  let head_within = ilt(head, 256_u64);
  let fill_within = ilt(fill, 256_u64);
  let usable = band(head_within, fill_within);
  if usable {
    let at = ring_index(head: head, fill: fill);
    set deref(ring).bytes[at] = byte;
    set deref(ring).fill = fill + 1_u64;
  } else {
    let rotated = ring_index(head: head, fill: 1_u64);
    set deref(ring).bytes[head] = byte;
    set deref(ring).head = rotated;
  }
  return unit;
}

fn render['u](page: &uniq 'u slot<'u, Page>, task: own Task) -> result: own u64 reads(page), writes(page) {
  doc "Formats one task into the borrowed page and reports how many bytes it wrote.";
  ...
  return written;
}

fn drain['u, 'r](page: &'u slot<'u, Page>, ring: &uniq 'r UartRing, count: own u64) -> result: own u64 reads(page), writes(ring) {
  doc "Copies one prefix of the page into the transmit ring.";
  let limit = imin(count, 4096_u64);
  for @copy (at in 0_u64..limit) {
    let byte = deref(page).bytes[at];
    let pushed = ring_push(ring: ring, byte: byte);
  }
  return limit;
}

fn service['p, 'r](pages: &uniq 'p Pool<'p, Page, 64>, ring: &uniq 'r UartRing, task: own Task) -> result: own u64 reads(pages, ring), writes(pages, ring), allocates(pages) {
  doc "Serves one task on a page it takes from the pool and returns before it leaves.";
  let blank = Page(bytes: array_new<u8, 4096>(0_u8));
  let attempt = pool_take_checked<'p, Page>(pool: pages, value: move blank);
  match attempt {
    Err(value: refused) => {
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
  doc "Runs a fixed scheduler over a static page pool and a transmit ring until the queue empties.";
  let ring = ring_new();
  let queue = fixed_vector_new<Task, 32>();
  region 'p {
    let pages = pool_static<'p, Page, 64>();
    let seeded = fixed_vector_push(vector: &uniq 'p queue, value: Task(kind: 1_u32, arg: 0_u64));
    match seeded {
      Err(value: full) => {
        let unplaced = move full.rejected;
        return exit_status(code: 1_u8);
      }
      Ok(value: placed) => {
      }
    }
    loop @scheduler {
      let next = fixed_vector_pop(vector: &uniq 'p queue);
      match next {
        None() => {
          break @scheduler;
        }
        Some(value: task) => {
          let sent = service<'p, 'p>(pages: &uniq 'p pages, ring: &uniq 'p ring, task: move task);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
```

**The envelope the compiler publishes.** Illustrative figures; the provenance
column is the load-bearing part.

```text
E(scheduler.wf, <embedded target>, C = { W = 1 })

  region  static.image          bytes     1_024   align   8   contiguous
  region  static.pool.pages     bytes   262_144   align  16   contiguous
  stack   entry                 bytes     5_312   align  16   contiguous
  lanes                         count         1
  slots   task.records          count         0
  slots   completion.records    count         0
```

| item | where it comes from | rule |
|---|---|---|
| `static.image` | the `const` items and the static parts of the emitted module | [STOR-6] |
| `static.pool.pages` | the one `pool_static<'p, Page, 64>()` occurrence: `64 * stride(Page)` at `align(Page)` | [PROV-8], [RES-6] |
| `stack.entry` | `main` (the ring, the `FixedVector<Task, 32>` and the provider live in this frame) + `service` (its 4096-byte staging `Page`) + `render`, measured post-codegen | [STK-3] |
| `lanes = 1` | no `par` in the program; the entry lane only | [RUN-3] |
| `task.records = 0`, `completion.records = 0` | no `par` statement and no `may-suspend` operation | [RUN-3] |

**Why it is resource-closed, item by item.**

| premise | how it is discharged |
|---|---|
| no heap | no `Heap` in `main`'s signature, so [PROV-6]'s closure is empty and [RES-5] does not fire |
| acyclic call graph | `main -> service -> {render, drain -> ring_push -> ring_index}`; no cycle, so [STK-1] rewrites nothing and [STK-2] passes |
| pool demand bounded | `service` has `(peak 1, delta 0)` on `'p`: the take and the release are on the same path, and the `Err` arm takes nothing. The scheduler loop's backedge delta is 0, so by section 4.2's loop rule **no iteration bound is needed** — the loop runs for as long as tasks keep arriving, and `live('p) <= 1` throughout |
| ring bounded | fixed storage in `main`'s frame; `ring_push` is total, and overwrite is the ring's defined semantics rather than a refusal |
| queue bounded | `FixedVector<Task, 32>`, storage in `main`'s frame, `len <= 32` structurally; a push at capacity returns `Full<Task>` with the task handed back |
| stack bounded | one chain, measured after code generation, one context |
| runtime closed | `W = 1`, no task or completion records; the runtime holds nothing this program can saturate |

**Notes and approximations.** `render`'s body is elided; its own subscript
obligations are ordinary [OP-4] work and are not what this program is about.
Three details are the honest cost of writing this in the real language rather
than in pseudocode. (i) `ring_push` guards `head < 256` and `fill < 256` before
using them, because a struct field carries no range and the checker has no source
for one; the `else` arm is the branch-whose-false-edge-cannot-be-taken price the
language already charges elsewhere, and it is a real instruction, not a
formality. (ii) `drain` clamps with `imin` instead of carrying a `requires`,
which keeps the caller free of a contract it would otherwise have to discharge
from `render`'s postcondition. (iii) the `region 'u` block in `service` exists
because the borrows of the page must end before the page is moved into
`pool_release`, and its results are carried out through pre-declared bindings
exactly as `tests/programs/dir_walk.wf` does. The staging `Page` in `service` is
a real 4096-byte frame object and is in the envelope where it belongs; the
obvious next operation is a `pool_static_filled` whose slots are constructed once
at reservation, which removes both the staging copy and that frame line. Whether
`pool_release` should be derivable at scope exit rather than written is [PROV-7]
and Q2. One more thing the envelope makes visible: this program reserves 64
pages and never holds more than one, so `static.pool.pages` is 256 KiB of an
envelope that a writer reading `E` would immediately shrink. That is the
envelope doing its job — it is a number someone can act on, which is what the
`allocates(heap)` row never was.

### 7.2 A hosted program that takes a `Heap` and is honest about `OutOfMemory`

```wf-draft
fn load['h](heap: &uniq 'h Heap, count: own u64) -> result: own Result<u64, OutOfMemory> allocates(heap) {
  doc "Allocates one working buffer and reports its length, returning a typed refusal when the store cannot serve it.";
  let fits = buffer_fits<u8>(count);
  if fits {
    let attempt = buffer_new(heap: heap, count: count, fill: 0_u8);
    match attempt {
      Err(value: refused) => {
        return Err<u64, OutOfMemory>(value: move refused);
      }
      Ok(value: work) => {
        let length = len(work);
        return Ok<u64, OutOfMemory>(value: length);
      }
    }
  }
  return Err<u64, OutOfMemory>(value: OutOfMemory());
}

command fn main(command.stdout as out: own Output, command.heap as heap: own Heap) -> status: own ExitStatus writes(out), allocates(heap) {
  doc "Asks the heap for one megabyte and turns a refusal into an exit status instead of a dead process.";
  region 'h {
    let outcome = load<'h>(heap: &uniq 'h heap, count: 1048576_u64);
    match outcome {
      Err(value: refused) => {
        return exit_status(code: 70_u8);
      }
      Ok(value: length) => {
        return exit_status(code: 0_u8);
      }
    }
  }
}
```

**Two judgments that are easy to confuse, and are separate here.**
`buffer_fits<u8>(count)` is the *representability* predicate [OP-9]: it decides
whether `count * stride(u8)` has an exact target representation, it is pure and
total, and the selected branch is what discharges `buffer_new`'s allocation-domain
obligation. It says nothing about availability. Availability is the `Result`, and
it is decided at run time by the store. This program proves the first and handles
the second, which is exactly the split the language already draws everywhere
else.

`main`'s `writes(out)` is not a leftover: `out` is never written by a statement,
and the entry declares it because the compiler-derived release of the `Output`
owner on the return edge contributes it, which is [EFF-2]'s own canonical case
(1431-1433). Nothing about providers changes that accounting; a provider is one
more owner whose disposition the row already has to state.

**What the compiler reports.**

```text
note: scheduler.wf is resource-closed; envelope written to scheduler.E
note: loader.wf is not resource-closed
  [RES-5] main selects command.heap
    heap-reaching path:  main -> load -> buffer_new
  a general store cannot appear in an envelope [L5], so no envelope is computed
  still true of this program:
    no covered-resource failure is a trap [RES-7]; buffer_new returns a value
    the heap is reachable only through the parameter above [PROV-6]
  not true of this program:
    it makes no resource promise, and its environment owes it none
```

That last block is the point of the whole design. The hosted program loses the
guarantee and keeps the honesty: `OutOfMemory` is a value on an ordinary edge,
its heap use is visible in one signature, and a reader can see in the entry line
whether the program can allocate at all.

## 8. Open questions

Each states the question, the two candidate answers, and my recommendation. None
of them is settled by anything in this file. One question that *is* settled here
rather than listed is the pool-run seam `CONTAINERS.md` raises against this half:
section 3.6 answers it.

**Q1. Does a resource-closed program have to *prove* every covered acquisition
succeeds, or may it handle a typed refusal?**
*(a)* Strict: every covered acquisition uses the proved spelling; a reachable
`PoolExhausted` arm disqualifies the program. *(b)* Permissive: the proved
spelling and the checked spelling are both admitted, since neither can ask for
more than `E` ([RES-8] as drafted).
**Recommend (b).** A refusal answered inside the envelope is not a resource
failure; it is a decision the store is entitled to make, and it is precisely what
[RUN-3] *requires* the runtime to do when a queue saturates. Forbidding the
writer the protocol the runtime is obliged to use would be inconsistent, and it
would push writers toward proofs of facts they do not need. The stronger property
some code wants — "this take is never refused" — is available at the same site by
choosing the proved spelling, which is how every other partial operation in this
language works.

**Q2. How is a provider-owned value's release attributed to its provider?**
*(a)* Carry the provider identity in the value, so a drop anywhere can return the
storage. *(b)* Bind the value to the provider's region, so the release site is
lexically inside the provider's scope and the provider path is derivable from the
type ([PROV-7] as drafted).
**Recommend (b)**, with the reachability side-condition stated as a checked
premise rather than assumed. (a) costs a word in every value, makes release rows
depend on data flow rather than on types, and reintroduces exactly the hidden
ancestry [EFF-5] spent a section removing. (b)'s residual — a release edge on
which the provider is uniquely borrowed elsewhere — is real and is the one thing
in this half I could not close; a third candidate worth trying is making
`pool_release` the only legal disposal (a linear rather than affine slot type),
which removes the derived release entirely at the cost of a new ownership class.

**Q3. Where does a *hosted* resource-closed program's memory come from?**
*(a)* Static and frame storage only, as in 7.1. *(b)* One more entry row
delivering a committed region — `command.region as store: own Arena<'store>` —
so a hosted program can be handed a real extent at start.
**Recommend (a) for this version**, and (b) as the first extension once a hosted
program actually needs it. (a) needs no new entry row, no new start-time
protocol, and no new target contract; the moment a program needs a store larger
than its static image can hold, (b) is the honest way to get one, and it is a
one-row change to [FN-7] and one item in `E`.

**Q4. Does `E` fix the lane count, or is it a table over `W`?**
*(a)* One `W` compiled in, so `E` is a single list. *(b)* A finite profile table
`E(W)` for the runtime's supported lane counts, with `W` chosen at `PreStart`
([RUN-3] as drafted).
**Recommend (b).** Parallel permission is never an obligation ([PAR-1] 1988), so
`W = 1` must always be legal and a program that only runs at `W = 8` would make
the permission load-bearing. A table is finite, is fixed integer arithmetic per
row, and lets a deployment trade lanes for memory without recompiling. The cost
is that `E` has rows; the alternative costs a recompile per deployment.

**Q5. What happens when an optimization raises a computed frame envelope?**
*(a)* Recompute `E` after optimization and publish the larger figure. *(b)* Treat
the published `E` as a cap and decline any optimization that would exceed it.
**Recommend (b) as the default with (a) available**, because the envelope is a
contract with a deployment that may already have been sized against it, and
because "the optimizer decides how much stack the program promised" is the same
mistake as letting the optimizer decide whether tail calls happen (L6). (a) is
correct when nothing has been sized yet, and an implementation can offer both.

**Q6. Does an arena reclaim anything before its region ends?**
*(a)* Cursor only: a drop returns nothing, the extent returns when `'p` ends
([RES-6] as drafted). *(b)* LIFO: the top-most allocation may be popped.
**Recommend (a).** It is the rule with no ordering side-condition, it matches
what a bump allocator actually does, and (b)'s "only if it is on top" is a
premise the checker would have to carry through every branch and loop for a
saving no current program needs.

**Q7. Is `E` an emitted artifact, and is it part of program identity?**
*(a)* Diagnostic output only. *(b)* An emitted machine-readable file beside the
object, which a deployment reads to size the environment.
**Recommend (b), and explicitly not part of [PROG-2] compilation-unit identity.**
The envelope is useless if the deployment cannot read it, and the entire causal
story of L1 is that the environment consumes `E`. Keeping it out of unit identity
keeps `E` a derived fact about a build rather than an input the source depends on.

**Q8. Should there be a proved (non-`Result`) heap allocation for a program that
has proved its heap use bounded?**
*(a)* No: `Heap` allocation is always `Result` ([RES-8] as drafted). *(b)* Yes,
for a program that has proved a live-bytes bound.
**Recommend (a).** A live-bytes bound is exactly the proof L5 says is not
sufficient — the fragmentation counterexample survives it — so (b) would let a
false premise remove a real failure path. If a program wants a proved
acquisition, it wants a pool or an arena, and the type system already says so.

**Q9. `par` and the stack: how does a resource-closed program use parallelism?**
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

**Q10. What about control entering the call graph from outside it?**
*(a)* Prohibit reentrancy in the resource-closed profile: no signal handler, FFI
callback or host reentry may execute Whitefoot code. *(b)* Admit it, with a
separately reserved stack item per reentrant context in `E`.
**Recommend (a) for this version.** (b) is the right long-run answer and the rule
is already drafted for it ([STK-4]), but admitting it now means bounding the
depth of a call graph the compiler does not fully see, which is exactly the class
of promise this design refuses to make elsewhere.

## 9. What I verified and what I reasoned

**Compiled.** I built the gate-profile `whitefootc` from this tree and ran five
probes. They establish current behaviour, not this design; no program in this
file compiles, because none of its syntax exists.

| probe | program | result | what it establishes |
|---|---|---|---|
| `p1_noinput.wf` | a `command` entry selecting no standard input | **accepted** | 7.1's entry shape is legal today |
| `p2_forever.wf` | an entry whose only statement is a `loop` with no `break` | **rejected**, [FN-1] `FunctionFallthrough` | an unbounded service loop must still leave the function on some edge; 7.1's scheduler loop therefore breaks on an empty queue. This corrected the program I first wrote |
| `p3_rec.wf` | an ordinary self-recursive function called from `main` | **accepted** | recursion is permitted today [FN-6]; [STK-2] is a new restriction on resource-closed programs and retires nothing |
| `p4_undeclared.wf` | a body that allocates while declaring `pure` | **rejected**, [EFF-2] `EffectMismatch`, missing `allocates(heap)` | allocation is already exhibited-and-checked both ways; [PROV-4] changes what the entry names, not whether it is checked |
| `p5_ambient.wf` | a nullary leaf function that allocates a buffer while holding nothing | **accepted** | the heap is ambient today. This is L2's evidence and the single fact the capability half of this design exists to change |
| `p6_unproved.wf` | `buffer_new` on an unbounded runtime length | **rejected** at target layout, `Unrepresentable(RuntimeSizedAllocation)` | allocation *size* is already a static obligation while availability is not — the split 7.2 relies on |

I also ran the existing `--stack-ledger` on `tests/programs/recursive_tree.wf`.
It reports, from the post-codegen assembly of that same compilation, a per-frame
table, a `STACK cycle` row for the recursive function with its bytes-per-level and
the number of levels the runtime's stack holds, and a `STACK chain` row per root —
and one of those chains runs through the compiler's drop glue into
`wf_resource_abort`. Three things in this design are that output's direct
consequences: [STK-3]'s insistence that frames are measured after code generation
(the ledger's own header explains that a pre-codegen number reports zero bytes for
the function a program dies in), [STK-2]'s cycle diagnostic (the ledger already
finds and prints the cycle), and [RES-7]'s claim that the abort site has a
reachable caller today.

**Read, not run.** Every rule citation in section 3.5 was read at the line given
in `spec/kernel-spec.md` at a40c7e70: [SCOPE-3] 27-31, [TYPE-2] 352, [SET-2] 508,
[OWN-1] 558, [OWN-5] 580, [STOR-1] 670, [STOR-3] 683, [STOR-5] 718, [STOR-6]
733-761, [OP-1] 793-798, [OP-9] 968-994, [FN-6] 1205, [FN-7] 1210-1253, [EFF-1]
1363-1372, [EFF-2] 1386-1420, [EFF-5] 1444-1450, [PAR-1] 1965-1989, [PAR-2] 2024,
[PAR-3] 2049, [PROG-3] 1499-1509, [SYS-2] 2158-2280, [SYS-8] 2482.

**Reasoned, and not verified anywhere.**

- Every judgment in section 3. None is implemented, and none has been executed
  against a program.
- The composition algebra of 4.2. Its sequence and branch rules are standard and
  I believe them; its `par` rule depends on a runtime profile that does not exist
  yet, and its loop rule's claim that a zero-delta backedge needs no iteration
  bound is the one I would attack first if I were falsifying this file.
- [PROV-7]. The release-site reachability premise is stated, not derived. It is
  the known hole.
- Every byte figure in section 7. They are illustrative; nothing computed them.
- The claim that [STK-1]'s admission conditions are sufficient for a correct
  mutual-tail-recursion rewrite. They are the conditions I could name; I did not
  attempt a proof and I did not enumerate the shapes they reject.
- The claim in [PROV-2] that `Heap` uniqueness is enough to keep `buffer<T>`
  non-region-bearing and its release row empty. This is load-bearing for keeping
  [STOR-1], [STOR-3] and [STOR-5] almost unchanged, and it deserves a falsifier
  of its own: a program that holds two heap-backed buffers across a boundary where
  the `Heap` has been moved.
- Everything about the current runtime's closure. The gaps named in section 2's
  L4 paragraph come from the 2026-09-01 read of the backend sources, not from a
  fresh audit in this session, and [RUN-2] is written as an obligation precisely
  because I cannot certify any existing target meets it.

**Falsifiers this file is asking for, in the order I would run them.**

1. Rewrite one existing corpus program that uses `buffer<T>` against [PROV-3] and
   [RES-7] by hand, and count what the `Result` return costs at every call site.
   If the answer is "every function that touches a buffer grows an error route",
   the operation split of [RES-8] is wrong somewhere.
2. Hand-execute 4.2 on 7.1 and on a program whose loop retains conditionally, and
   check that the branch rule's per-variant retention survives a `propagate` edge.
3. Attack [PROV-2]'s uniqueness argument with the two-buffer program above.
4. Attack [STK-1] with a mutual tail recursion carrying a live child reborrow
   across the jump, and check that the stated conditions reject it.
5. Attack L8 with a fixed append-only log: confirm that the design counts its
   records as a consumable budget and not as an effect flow, and that a program
   which writes to it in an unbounded loop is correctly *not* resource-closed.
