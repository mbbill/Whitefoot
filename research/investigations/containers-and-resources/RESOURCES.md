# Resources: providers, the envelope, and resource-closed programs

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> **sixth draft, after falsifier round 5 and the owner's rulings of 2026-09-03
> evening**; this file has been brought to that draft and carries no rule text of
> its own. Where a sentence here disagrees with `DESIGN.md`, `DESIGN.md` wins.
>
> **Every spelling below that the kernel does not have today is a proposal**, listed
> in `DESIGN.md` 3.S with its alternatives and marked PROPOSED. Nothing here is
> decided.

The resource half of the batch 0116 drafting round, reduced to the material
`DESIGN.md` does not carry: the goals and non-goals it was written against, and the
three writer's-eye migrations that show what a resource-closed program actually
costs to write. The laws, the rules, the envelope algebra, the amendment register,
the surface proposals, the open questions and the verified-versus-reasoned register
are all in `DESIGN.md`.

Tree read: `batch/0116-containers-and-resources` at `main 30602914`,
`spec/kernel-spec.md` **v0.41 ACTIVE**. Bare line numbers are that file. Nothing here
is implemented, and every clause and call is written in the v0.41 surface.

## What round 5 and the owner's rulings changed, so this file is not read as current

Six things a reader of the fifth draft will look for and not find.

- **A domain of `E` is a store, not a kind** [RES-5]. Five *algebras* are defined and
  a domain is a pair of an algebra and a store identity, so two arenas do not share
  one domain, a store minted inside a loop body has a domain whose life is one
  iteration, and [RES-10]'s route (ii) has a referent for `cap(store)`.
- **The map has a `retained` label and a `reset` transfer** [RES-10]. Without the
  first, a service loop with no `break` — one of the two programs goal A exists for —
  publishes an envelope with zero contribution from everything it does. Without the
  second, a region block re-entered by a loop leaves a positive backedge delta and
  the design's own recommended idiom for per-iteration scratch is refused.
- **A lease that is dropped is a compile error**, because `Lease` is `linear` by
  declaration [S18]. The fifth draft recorded the leak as bounded and visible in `E`;
  it was neither, and `DESIGN.md` Q0b says so.
- **`advance<T>` is a closed expression** [RES-5], because every take rounds the
  cursor to the store's own `align` and the padding is charged once per run rather
  than once per element. The fifth draft's exact form named `len(arena)`, a runtime
  cursor, which [RES-3] forbids in a bound.
- **A cyclic containment graph is refused at the type, in every program** [PROV-6],
  and not only under the marker. The fifth draft's disposition left the aborting
  release walk in every hosted program, which is the shape L3's last clause was
  written for.
- **The `acquires from` column is derived and the exclusion test reads a count**
  [RES-7]. Every may-suspend operation acquires a submission record and a completion
  record — which the runtime sources show for seven operations the fifth draft's
  column read `none` for — and an operation is excluded when a store it acquires from
  has count zero in the selected row, not when its item is absent.

## 1. Goals and non-goals

**Goal.** With the heap off, an accepted program is deterministic, never crashes,
never runs out of memory, and only logic errors remain over its whole lifetime
(owner rulings R12 and R13). The shape is a promise, not a guarantee about the
world: the compiler computes one finite, shaped envelope `E`, the program promises
never to demand more, and the environment decides whether it can deliver `E`.

**And "never runs out" now includes a store the program owns.** Round 5's sharpest
finding against the fifth draft was not about bytes: 4.1 lost a pool block on an
ignored refusal, with no diagnostic, no effect row and no envelope movement, so a
program that satisfied every premise of [RES-3] silently stopped making progress
after eight iterations. `DESIGN.md` §1.1 states the consequence — a program that
stops at three in the morning has not removed the class of failure either — and R2 is
what closes it.

**Non-goals**, each with the rule that records it. Disk space, host object
acquisition, network, CPU time, deadlines, fairness, power, device health and quota
revocation are outside [RES-1] and stay typed system outcomes or environment
conditions [RES-7]. A bounded general heap is still a general heap and is never part
of `E` [RES-4]. No `par` construct is emitted in a marked build and the published row
is `lanes(1)` [RUN-1], [RUN-2]. Execution contexts are the follow-on's
(`DESIGN.md` 1.5) — with one thing decided here that the fifth draft left open: **a
worker lane is an execution context**, which is what [PROV-5]'s activation refusal
reads. And an unmarked program keeps every [SCOPE-3] deferral it has today, with one
exception: a type whose containment graph has a cycle is refused in every program,
because a rule that is a hard error under a marker and a process abort without one is
not one rule.

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
derived release and the marked program carries no `dispose` anywhere. That is the
asymmetry `DESIGN.md` Q0c records: goal A's programs pay nothing for R2, and goal B's
pay one statement per store-backed value.

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
  set (work, seeded) = try_place::<Entry, 64>(vector: move work, value: move root);
  loop @walk {
    set (work, next) = try_take::<Entry, 64>(vector: move work);
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

**A third fact is new.** Both statements are [LIV-3]'s multi-target `set` at a
binding declared outside the loop, which [OWN-11] 646 forbids today; [LIV-1]'s join
agreement replaces that prohibition and admits exactly this shape, because the
exchange keeps the root live at every point. The fifth draft asserted that
replacement in its register and not in [LIV-1]'s body, and both of its worked
programs depended on it.

### 2.3 From an unbounded store to a bounded one

A driver's transmit path leases a block per packet. Under the fourth draft that was
a `Pool` store with a lease type; under the ruling it is a run of runs
(`CONTAINERS.md` §3.4), and the resource question is unchanged:

```wf-design
  region 'a {
    let scratch = arena_frame::<65536, 16, 'a>();
    let made = pool_new::<'a>(arena: &uniq scratch);
    ...
  }
```

[RES-10] composes it: the eight takes happen once before the loop, so the bump
domain's backedge delta is zero and no iteration bound is needed; the lease and its
release are on the same path, so the free list's delta is zero too. **What makes that
second sentence a fact rather than an assumption is R2 and not the envelope**: the
free list is frame placement, whose [RES-5] algebra has no acquire and no release, so
[RES-10] computes nothing about it and premise 3 says nothing about it. The pool
stays full because `Lease` is linear, `Option<Lease>` is linear by containment, and
the refusal arm therefore cannot be dropped. That is the honest division of labour
between the two halves of this design, and the fifth draft had neither half doing it.

A **retaining** variant — one that keeps a lease across iterations — is bounded
instead by route (ii): the free list's `cap` is a type-level constant and
`pool_take`'s refusal cannot succeed on an empty list, so the composed peak is that
constant. That route composes across a call only because [RES-8] publishes a
**declared** `saturating(p)` fact; the fifth draft derived it from the body,
transitively, which is what [CALL-5] forbids and what [ENT-1] 2661 forbids a second
time by reading which premise discharged a goal.

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
the store's reset. Under the fifth draft's five primitive transfers the only one that
fitted a reset was `release one`, whose delta is `-1`, so the block left `+255` on the
backedge, `max(d) > 0`, and none of the three discharges applied: the loop has no
constant trip count, a *proved* acquisition succeeds on a full store by construction
so the saturation route is false, and no header invariant can name a store minted
inside the body. The design's own recommended idiom was refused. [RES-10]'s `reset`
transfer has delta `-len(store)`, which is the exact inverse of everything the block
accumulated, so `delta(region_block) = 0` falls out of the arithmetic instead of
being asserted in prose.

The diagnostic a writer gets when none of the three discharges applies names the
loop and the value:

```text
Semantics/Source [RES-3]: UnboundedStoreDemand
  domain: (bump extent, store region 'a reserved at "scratch")
  the loop at @serve has backedge delta +1 and no bound
  the trip count names the runtime value "count"
  mechanical_fix: bound the loop with a compile-time constant trip count, state an
    [INV-1] invariant over len(scratch), use the checked spelling, whose refusal
    cannot succeed on a full store, or move the reservation inside the loop, where
    the block's own reset composes to a zero backedge delta
```

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
