# Resources: providers, the envelope, and resource-closed programs

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> **eighth draft, after falsifier round 7 and the owner's decisions of 2026-09-03 and
> 2026-09-04**; this file has been brought to that draft and carries no rule text of its
> own. Where a sentence here disagrees with `DESIGN.md`, `DESIGN.md` wins.
>
> **Every language-surface addition below is now the owner's decision**, recorded in
> `DESIGN.md` 3.S, and **nothing is PROPOSED any more**. Of the seventh draft's three
> open items, `on_propagate` [S28] is **REJECTED**, `seq_rebase` [S29] is **WITHDRAWN to
> the library**, and the seven [SYS-8] signatures over views [S30] are **ADOPTED**; of
> the eighth draft's three, decided 2026-09-04, `seq_reslice` [S31] is **NOT ADOPTED as
> an operation** because a shared view over a writable one is [OWN-6]'s child reborrow,
> a linearity bound on a generic parameter [S32] is **ADOPTED**, and `ReserveOutcome`
> [S33] is **ADOPTED**.

The resource half of the batch 0116 drafting round, reduced to the material
`DESIGN.md` does not carry: the goals and non-goals it was written against, and the
three writer's-eye migrations that show what a resource-closed program actually
costs to write. The laws, the rules, the envelope algebra, the amendment register,
the surface decisions, the open questions and the verified-versus-reasoned register
are all in `DESIGN.md`.

Tree read: `batch/0116-containers-and-resources` at `main 30602914`,
`spec/kernel-spec.md` **v0.41 ACTIVE**, with **v0.42 merging**: v0.42 adds `[FORM-8]`
canonical region spelling, over **regions only**. Bare line numbers are v0.41. Nothing
here is implemented; every clause and call is written in the v0.41 surface, and **every
type and const argument of a user generic is written** ([FN-2] 1124, probe `q4`).

## What round 7 and the owner's decisions changed, so this file is not read as current

Ten things a reader of an earlier draft will look for and not find. The first two are
the owner's decisions of 2026-09-03.

- **D3: linearity is read against the SCOPE, and the release is derived where the
  capability is held.** A store-backed value is linear only in a scope that does not hold
  the capability its release needs; in a scope that holds it — a signature carrying
  `heap: &uniq Heap`, or `main` with the entry heap — the compiler derives the release on
  every leaving edge and charges it to that scope's `writes(heap)`. [RES-10]'s derived
  release transfer is where a hosted program's frees now appear, and **no `dispose`
  ceremony survives**: `bs_reserve` keeps one `dispose old;` as the *early* release and
  neither worked program has any. L2 is untouched — the capability is a held value named
  at a parameter.
- **D4: every loop body is implicitly a region block**, so a loop-body borrow of an outer
  binding is written bare. A **named** `region 'a { }` inside a loop body is unaffected
  and is still how a per-iteration store is reserved, because the implicit block has no
  binder. The amendment has **not** landed; `DESIGN.md` 3.K.0 and §7's B0b say so.
- **[RES-10] has TWO routes and the invariant route is deleted.** It asked [MSR-4] to
  discharge a goal about `delta`, which is a component of the composition's own map and
  not an [ENT-2] term of the language, so it could never fire; and the only thing an
  [INV-1] header invariant can state is a **level**, which is the vacuous shape round 6
  killed. What remains is (i) a trip-count bound that is a compile-time integer or a
  closed expression [MSR-4] establishes from the loop's endpoints and the function's
  verified `requires`, and (ii) the reusable-capacity route over `cap(store)` when every
  acquisition on the loop's paths is `saturating`. **The backedge delta is computed by
  the composition from the rows' declared deltas and is never proved.**
- **`saturating(d)` takes a store DESIGNATOR, not a store region** [S26, AMENDED].
  Every reusable-capacity domain in [RES-5] is a **runtime** store whose identity is not a
  region, so keyed to a region the clause could not be written for any domain the route
  that reads it applies to. The designator is a region name in scope or one of [RES-9]'s
  **six spec-fixed runtime-store names** — `handles submissions completions tasks lanes
  queue` — and [RES-7]'s source half quantifies over that closed set.
- **The reset is a PAIRED transfer and the scope composition is new.** A block's map is
  its body's map with each derived release and each store reset applied **at every label
  at which that edge leaves the block**, and a reset cancels *by definition* exactly the
  composed delta of that block's own map at that label on that store's domain. Without
  the pairing the recommended per-iteration idiom composed to an interval and was refused
  in both spellings; without the scope rule a `break`, a `give`, a `propagate` or a
  `return` out of a region block carried the block's positive delta with the reset charged
  nowhere.
- **There is an OVERLAP composition and an extraction.** Statements an implementation may
  execute with overlapping execution compose by the componentwise **sum** of peaks, and a
  staged [PAR-3] permission by `k * p` for the runtime's published outstanding-work bound;
  charging an overlap like a sequence let a marked driver's `read_at` loop hold `k`
  submission records where `E` promised one. And one stated sentence turns the map into
  `E`'s figure: the **max** over labels, never a sum.
- **`E` carries a stack item per execution context, and the runtime's own materializations
  are in it.** The shipping floor maps a 64 KiB alternate stack **per attaching thread**,
  so a one-lane build has two, and the host thread's surviving stack was named nowhere;
  [RES-1] makes an alternate stack a stack item, [STK-3] measures every live context, and
  [RUN-4] makes `StartFailed` mandatory. Three [QUAL-2] failures of the shipping
  implementation are recorded in `DESIGN.md` 6.2 rather than hidden.
- **The handle table's refusal is a variant, and it is publishable.** Under [S25] alone
  `reserve_file`'s exhaustion was a **class** of an `IoError` payload, and no [CALL-4]
  route is conditioned on a class, so the `Err` edge published `len(factory)` alone and no
  marked program could derive `room(factory)` after a refusal. **[S33] is adopted**: the
  operation returns `own ReserveOutcome`, and [RES-6] publishes `room(factory) = 0` on the
  `Exhausted()` arm through [CALL-4]'s existing per-variant route, so [RES-10]'s
  reusable-capacity route may read the refusal beside `saturating` and `cap(store)`.
- **A reserving occurrence must be a statement of its own region block and of no loop
  inside it** [PROV-5], and an extent item is named by (concrete instance, `region_stmt`
  NodePath) so monomorphization gives two instances two items.
- **A domain of `E` is a store and it carries a KIND column** [RES-5]; a cycle through the
  **release graph** — the graph the release walk actually traverses — is refused at the
  type in every program [PROV-6]; the `acquires from` column is derived over **actions**,
  reaching [SYS-5]'s three release actions as well as [SYS-2]'s seven operations [RES-7];
  and `retained` and `return` are labels that compose by the same formula as every other
  [RES-10].
## 1. Goals and non-goals

**Goal.** With the heap off, an accepted program is deterministic, never crashes,
never runs out of memory, and only logic errors remain over its whole lifetime
(owner rulings R12 and R13). The shape is a promise, not a guarantee about the
world: the compiler computes one finite, shaped envelope `E`, the program promises
never to demand more, and the environment decides whether it can deliver `E`.

**And "never runs out" now includes a store the program owns.** Rounds 5 and 6 found
the same failure at two stores. At the **pool**, a block was lost on an ignored refusal
with no diagnostic, no effect row and no envelope movement, so a program satisfying
every premise of [RES-3] silently stopped making progress after eight iterations; the
proved release closes it. At the **arena**, a service loop took 256 bytes per turn from
a frame store the accounting certified bounded at that store's capacity, and stopped
making progress after 256 turns; [RES-10]'s consumable-budget rule closes that one.
`DESIGN.md` §1.1 states the consequence: a program that stops at three in the morning
has not removed the class of failure either.

**Non-goals**, each with the rule that records it. Disk space, host object
acquisition, network, CPU time, deadlines, fairness, power, device health and quota
revocation are outside [RES-1] and stay typed system outcomes or environment
conditions [RES-7]. A bounded general heap is still a general heap and is never part
of `E` [RES-4]. No `par` construct is emitted in a marked build and the published row
is `lanes(1)` [RUN-1], [RUN-2]. Execution contexts are the follow-on's
(`DESIGN.md` 1.5) — with one thing decided here that the fifth draft left open: **a
worker lane is an execution context**, which is what [PROV-5]'s activation refusal
reads. And an unmarked program keeps every [SCOPE-3] deferral it has today, with one
exception: a type whose **capability-released-leaf** graph has a cycle is refused in
every program, because a rule that is a hard error under a marker and a process abort
without one is not one rule. Stating it over that graph rather than over containment is
round 6's correction, and it is what keeps an arena- or frame-backed recursive
structure — whose release walk is empty — compiling.

## 2. The writer's view

Three migrations, each from a shape the corpus has to one the rules admit. They are
design text and compile nowhere.

### 2.1 From "it needs a growable vector" to a fixed store

A hosted program's `ByteVec` is a `buffer<u8>` plus a `fill`, grown by
`buffer_new`-and-copy. A resource-closed program cannot have it: growth needs a
store, and the only store a marked program may reach is one it reserved.

The rewrite is a frame-placed run at the size the program can prove it needs:

```wf-design
resource_closed command fn main() -> status: own ExitStatus pure {
  doc "Collects a bounded line and reports its length.";
  let line = seq_fixed::<u8, 4096>();
  ...
}
```

What the writer gives up is the unbounded case, and gets back the reason: `cap` is a
constant the formation publishes, so `room` is a standing fact, every `seq_place` is
discharged from a header invariant with no runtime branch, and the run is one
contribution to `stack(entry)` rather than an item of its own. What the writer must
now decide, and could previously avoid deciding, is the ceiling — which is the whole
of what "resource-closed" asks of a program.

**And the run is affine, not linear**, which is R2's other half read from the writer's
side: a frame-resident run needs no capability to reclaim, so it has an ordinary
derived release and the marked program carries no `dispose` anywhere. **Under D3 the
asymmetry that used to follow is gone**: a hosted program's store-backed values are
affine too in every scope that holds the provider, so goal B pays a parameter and an
effect row per capability-holding function rather than one written statement per value
per leaving edge. `dispose` remains, as the *early* release a writer chooses.

Where the ceiling genuinely is not a source constant, the answer is an
`arena_extent` reservation, whose `region` item a deployment grants separately (L6),
and whose refusal is a value.

### 2.2 From recursive descent to an explicit work list

`wfgrep.wf`'s `walk` is a recursive directory descent. [STK-2] refuses it under the
marker with no depth certificate admitted: a `requires` bound on a recursion
parameter and a proof that an argument decreases are both **not** substitutes,
because neither bounds the frame chain.

The rewrite is a bounded work list, which is what a kernel writes anyway:

```wf-design
  let work = seq_fixed::<Entry, 64>();
  set (work, seeded) = try_place(vector: move work, value: move root);
  loop @walk {
    set (work, next) = try_take(vector: move work);
    match next {
      None() => {
        break @walk;
      }
      Some(value: here) => {
        ...
      }
    }
  }
```

Two facts make this the honest shape rather than a workaround. The depth ceiling is
now written down, at `64`, where the recursion's was implicit in the host's stack.
And the refusal is a value: `try_place` hands the entry back when the list is full,
so a deep tree reports rather than dies. `docs/patterns.md` P15 is the pattern and
[STK-2]'s own diagnostic names it.

**A third fact is new.** Both statements are [LIV-2]'s one commit rule at a binding
declared outside the loop, which [OWN-11] 646 forbids today; [LIV-1]'s join agreement
replaces that prohibition and admits exactly this shape, because the commit leaves the
root live at every point. `work` names a binding in scope and is therefore a **place**;
`seeded` and `next` name none and declare one each. Neither call writes a type or const
argument, because the `vector` operand determines both (3.K.0).

### 2.3 From an unbounded store to a bounded one

A driver's transmit path leases a block per packet. Under the fourth draft that was
a `Pool` store with a lease type; under the ruling it is a run of runs
(`CONTAINERS.md` §3.4), and the resource question is unchanged:

```wf-design
  region 'a {
    let scratch = arena_frame::<65536, 16, 'a>();
    let made = pool_new(arena: &uniq scratch);
    ...
  }
```

[RES-10] composes it: the eight takes happen once before the loop, so the bump
domain's backedge delta is zero and no iteration bound is needed; the lease and its
release are on the same path, so the free list's delta is zero too. **What makes that
second sentence a fact rather than an assumption is R2 and not the envelope**: the
free list is frame placement, whose [RES-5] algebra has no acquire and no release, so
[RES-10] computes nothing about it and premise 3 says nothing about it. The pool
stays full because `pool_release` is the **proved** spelling: its
`requires room(pool.free) > 0_u64` is discharged at the call site from `pool_take`'s
own published relation, so there is no refusal arm and no path on which a lease is
discarded. `Lease` being `linear` is what makes a deliberate discard a written
statement; it is the proof, and not the modifier, that makes the return unavoidable.
That is the honest division of labour between the two halves of this design.

A **retaining** variant — one that keeps a lease across iterations — is bounded by
neither route, and an earlier draft said otherwise. The free list is **not a domain**:
frame placement's [RES-5] algebra has no acquire and no release, so [RES-10] computes
nothing about it, and `saturating` takes a **store designator** [RES-8, S26] while a
`BlockPool<'s>` free list is not a store at all. A retaining pool is bounded by its
element count, which is the `FixedVector`'s own type constant, and by the proved
`pool_release` that keeps every lease returnable. That is the honest statement, and the
designators `saturating` exists for are a region name in scope and [RES-9]'s six
spec-fixed runtime-store names — `handles submissions completions tasks lanes queue` —
which are the reusable-capacity domains route (ii) applies to.

**A per-iteration scratch extent is the shape the reset transfer exists for.** Write
the region block inside the loop —

```wf-design
  loop @serve {
    region 'a {
      let scratch = arena_frame::<4096, 16, 'a>();
      let staging = seq_arena_proved::<u8>(arena: &uniq scratch, count: 256_u64);
      ...
    }
  }
```

— and the block's body acquires 256 bytes on the bump domain while its exit edge runs
the store's reset. [RES-10]'s `acquire` transfer is `(peak a, delta +a)` for the
domain's own quantity, so the take is charged 256 and not one, and the `reset`
transfer's delta is `-len(store)`, the exact inverse of everything the block
accumulated, so `delta(region_block) = 0` falls out of the arithmetic instead of being
asserted in prose. **Without the block the same take is charged trips × 256 and the
loop is refused**, which is the owner's accounting ruling and the reason this idiom is
the recommended one rather than a convenience.

The diagnostic a writer gets when neither route discharges names the loop and the value:

```text
Semantics/Source [RES-3]: UnboundedStoreDemand
  domain: (bump extent, store region 'a reserved at "scratch")
  the loop at @serve has backedge delta +256 and no discharge
  this domain's kind is consumable budget, so a store's own refusal bounds nothing,
    which is why route (ii) does not apply here
  mechanical_fix: bound the loop with a trip count that is a compile-time integer or
    a closed expression this function's own requires establishes [RES-10] route (i),
    or move the reservation inside the loop, where the block's own paired reset
    composes to a zero backedge delta
```

**The route this diagnostic does not offer is the one round 7 deleted.** An earlier
draft's second discharge asked [MSR-4] to prove `delta <= 0` from an [INV-1] header
invariant over `len(scratch)`. `delta` is a component of [RES-10]'s own map and not a
term of the language, so no goal could be formed; and a header invariant states a
**level**, which bounds nothing about a backedge. Two routes remain and both test
compile-time data.

## 3. Evidence

Every probe cited here is in `DESIGN.md` 6.1 and 6.2 with its verdict. The five this
file rests on most: `x8`, the recursive region-plus-arena shape, **accepted today**,
which is the activation break; `x6` and `a8`, the self-referential type and its
`realloc`-and-abort drop glue, which is why [PROV-6] refuses the type at its
declaration in every program; `a1`, the compiler's own refusal to emit arena content
with a release action, which is the reset/content split; `p1_reclose` with the two
io_uring source reads, which is why [RES-7]'s column is derived from the may-suspend
contract and reads a store for eight operations; and `a4`'s `--stack-ledger` read,
which shows the entry chain is presently three disjoint roots and one
available-stack number rather than a per-context demand, and which is why [STK-3]
materializes the entry stack instead of reading it.
