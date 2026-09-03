# Resources: providers, the envelope, and resource-closed programs

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> **seventh draft, after falsifier round 6 and the owner's decisions of 2026-09-03**;
> this file has been brought to that draft and carries no rule text of its own. Where
> a sentence here disagrees with `DESIGN.md`, `DESIGN.md` wins.
>
> **Every language-surface addition below is now the owner's decision**, recorded in
> `DESIGN.md` 3.S, which is a decision record rather than a proposal table. Three
> items remain PROPOSED: `on_propagate` [S28], `seq_rebase` [S29], and the seven
> [SYS-8] signatures over views [S30].

The resource half of the batch 0116 drafting round, reduced to the material
`DESIGN.md` does not carry: the goals and non-goals it was written against, and the
three writer's-eye migrations that show what a resource-closed program actually
costs to write. The laws, the rules, the envelope algebra, the amendment register,
the surface proposals, the open questions and the verified-versus-reasoned register
are all in `DESIGN.md`.

Tree read: `batch/0116-containers-and-resources` at `main 30602914`,
`spec/kernel-spec.md` **v0.41 ACTIVE**. Bare line numbers are that file. Nothing here
is implemented, and every clause and call is written in the v0.41 surface.

## What round 6 and the owner's decisions changed, so this file is not read as current

Seven things a reader of the sixth draft will look for and not find.

- **A domain of `E` is a store, not a kind, and it carries a KIND column** [RES-5].
  Four *algebras* are defined — the cleanup-scratch domain is deleted, because a
  compiler-derived walk is not a statement [RES-10] can attribute a transfer to and its
  frame cost is ordinary frame cost [STK-3] — and a domain is a pair of an algebra and a
  store identity. Each domain is **reusable capacity** or **consumable budget**, and
  [RES-10]'s store-capacity route applies to the first only, because a store's own
  refusal bounds what is **held** and says nothing about what is **spent**.
- **An arena or frame take inside a loop is charged trips × size** unless the loop
  encloses a region block that is entered and reset per iteration (owner-decided
  2026-09-03). A divergent loop, or a runtime trip count with no bound, therefore makes
  the program **not resource-closed**, and the diagnostic names the domain and the loop.
  Round 6 built the counterexample the sixth draft admitted — a `pure`, heap-free
  service loop taking 256 bytes per turn from a frame arena, certified bounded at the
  store's capacity while the program silently stopped making progress after 256 turns.
  Its author proposed re-keying **linearity** to fix it; the owner refused, because the
  criterion is what a release *needs* and this is accounting.
- **The map has a `retained` label and a `reset` transfer** [RES-10]. Without the
  first, a service loop with no `break` — one of the two programs goal A exists for —
  publishes an envelope with zero contribution from everything it does. Without the
  second, a region block re-entered by a loop leaves a positive backedge delta and
  the design's own recommended idiom for per-iteration scratch is refused.
- **A lease that is dropped is a *visible* discard, and a lease that is *returned* is
  a proof.** `Lease` is `linear` by declaration, which makes the discard a written
  statement rather than a silence — and not impossible, because a destructuring consume
  is a legal consume. A **directional** obligation is bought by proving the return:
  `pool_release` is the **proved** spelling, total under `requires room(pool.free) >
  0_u64`, so there is no refusal arm to discard. `DESIGN.md` Q0b records what changed.
- **`advance<T>` is a closed expression in its formula and its `count` is [RES-3]'s
  question** [RES-5]. Every take rounds the cursor to the store's own `align` and the
  padding is charged once per run; whether the operand is closed is decided at the
  acquisition, where premise 3 fails with the runtime value named. And **[RES-10]'s
  transfers are stated per algebra**, so a 256-byte take is charged 256 and not one,
  which the sixth draft's table got wrong for every domain but uniform slots.
- **A cycle through the CAPABILITY-RELEASED-LEAF graph is refused at the type, in
  every program** [PROV-6], and not only under the marker. The sixth draft stated the
  refusal over **containment**, which refuses every recursive structure in every
  program — including an arena-backed one whose release walk is empty — and
  `tests/programs/recursive_tree.wf` is in the corpus today. Stating it over the graph
  the walk follows keeps L3's no-abort clause true and costs no program.
- **The `acquires from` column is derived over ACTIONS, and the exclusion is split at
  the stage boundary** [RES-7]. Every may-suspend **action** acquires a submission and a
  completion record, which reaches [SYS-2]'s seven operations **and [SYS-5]'s three
  release actions** — a `ReadFile` close is a may-suspend action that reserves from the
  same fixed table every read uses, and the sixth draft counted none of them. And the
  test may not be a source rejection reading a figure the runtime publishes: the source
  half publishes a per-store **declared demand**, and the capacity match is [QUAL-2]'s
  qualification failure.
- **`retained` composes by the same formula as every other label, and there is a
  `return` label** [RES-10]. The sixth draft's `retained`-specific sequence clause lost
  everything a program acquired before entering a divergent loop, and no label carried a
  `return` edge at all, so a peak reached only on a returning path left the map.

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
neither route, and the sixth draft said otherwise. The free list is **not a domain**:
frame placement's [RES-5] algebra has no acquire and no release, so [RES-10] computes
nothing about it, and `saturating` is keyed to a **store region** [RES-8] while a
`BlockPool<'s>` free list is not a store. A retaining pool is bounded by its element
count, which is the `FixedVector`'s own type constant, and by the proved
`pool_release` that keeps every lease returnable. That is the honest statement, and
`saturating('s)` exists for the runtime record stores [RES-9] that genuinely are
domains.

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

The diagnostic a writer gets when none of the three discharges applies names the
loop and the value:

```text
Semantics/Source [RES-3]: UnboundedStoreDemand
  domain: (bump extent, store region 'a reserved at "scratch")
  the loop at @serve has backedge delta +256 and no discharge
  this domain's kind is consumable budget, so a store's own refusal bounds nothing
  mechanical_fix: bound the loop with a compile-time constant trip count, state an
    [INV-1] invariant over len(scratch) from which the backedge delta is <= 0, or
    move the reservation inside the loop, where the block's own reset composes to a
    zero backedge delta
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
