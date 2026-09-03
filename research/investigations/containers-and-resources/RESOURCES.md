# Resources: providers, the envelope, and resource-closed programs

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> **fifth draft, after falsifier round 4 and the owner's minimality ruling of
> 2026-09-03**; this file has been brought to that draft and carries no rule text
> of its own. Where a sentence here disagrees with `DESIGN.md`, `DESIGN.md` wins.

The resource half of the batch 0116 drafting round, reduced to the material
`DESIGN.md` does not carry: the goals and non-goals it was written against, and the
three writer's-eye migrations that show what a resource-closed program actually
costs to write. The laws, the rules, the envelope algebra, the amendment register,
the open questions and the verified-versus-reasoned register are all in `DESIGN.md`.

Tree read: `batch/0116-containers-and-resources` at `main a40c7e70`,
`spec/kernel-spec.md` **v0.40 ACTIVE**. Bare four-digit line numbers are that file.
Nothing here is implemented.

## What round 4 and the ruling changed, so this file is not read as current

Five things a reader of the fourth draft will look for and not find.

- **There is no `Pool` store.** A pool of fixed-size runs is a library free list
  over runs an arena already granted — `CONTAINERS.md` §3.4 writes it — so `E` names
  one extent item and no `slots` row, which is L6's shape rather than a count.
  `PoolSlot`, `PoolVector`, `seq_lease` and `pool_frame` are gone with it.
- **A store's identity is per live activation of its region block**, not per entry.
  [PROV-5] refuses an `arena_extent` occurrence whose reserving function lies on a
  call-graph cycle, and says why the frame form needs no refusal. Probe `x8` is the
  program that gets the diagnostic and is accepted today.
- **A store's storage reclamation never stands in for its content's release**
  [PROV-6]. An arena's reset reclaims bytes; a run's elements get their own derived
  release on the edge leaving the run's binding's scope. Without the split every
  host handle placed in arena content leaked, which the compiler already refuses to
  emit (probe `a1`).
- **The disposal walk is bounded by the disposed type's containment height**, so it
  uses no auxiliary storage; a type whose containment graph has a cycle has no such
  bound and denies [RES-3] premise 3 on the new cleanup-scratch domain. Probe `x6`
  shows such a type is accepted today and probe `a8` shows its derived drop calling
  `realloc` and `wf_resource_abort`.
- **The handle table has all five parts** — capacity, acquire event, release event,
  refusal relation, multiplicity — and [RES-9] amends [SYS-10] and [SYS-2] 2295 to
  give it them. Its refusal stays on the existing `IoError` channel; there is no
  `NoRecord`.

## 1. Goals and non-goals

**Goal.** With the heap off, an accepted program is deterministic, never crashes,
never runs out of memory, and only logic errors remain over its whole lifetime
(owner rulings R12 and R13). The shape is a promise, not a guarantee about the
world: the compiler computes one finite, shaped envelope `E`, the program promises
never to demand more, and the environment decides whether it can deliver `E`.

**Non-goals**, each with the rule that records it. Disk space, host object
acquisition, network, CPU time, deadlines, fairness, power, device health and quota
revocation are outside [RES-1] and stay typed system outcomes or environment
conditions [RES-7]. A bounded general heap is still a general heap and is never part
of `E` [RES-4]. `par` permission is not taken in a marked build and the published row
is `lanes(1)` [RUN-1], [RUN-2]. Execution contexts are the follow-on's
(`DESIGN.md` 1.5). And an unmarked program keeps every [SCOPE-3] deferral it has
today: this design neither improves nor worsens it.

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
  let line = seq_fixed<u8, 4096>();
  ...
}
```

What the writer gives up is the unbounded case, and gets back the reason: `cap` is a
constant the formation publishes, so `room` is a standing fact, every `seq_place` is
discharged from a header invariant with no runtime branch, and the run is one
contribution to `stack(entry)` rather than an item of its own. What the writer must
now decide, and could previously avoid deciding, is the ceiling — which is the whole
of what "resource-closed" asks of a program.

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
  let work = seq_fixed<Entry, 64>();
  set (work, seeded) = try_place<Entry, 64>(vector: move work, value: move root);
  loop @walk {
    set (work, next) = try_take<Entry, 64>(vector: move work);
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

### 2.3 From an unbounded store to a bounded one

A driver's transmit path leases a block per packet. Under the fourth draft that was
a `Pool` store with a lease type; under the ruling it is a run of runs
(`CONTAINERS.md` §3.4), and the resource question is unchanged:

```wf-design
  region 'a {
    let scratch = arena_frame<65536, 16, 'a>();
    region {
      let made = pool_new<'a>(arena: &uniq scratch);
      ...
    }
  }
```

3.K.7.1 composes it: the eight takes happen once before the loop, so the bump
domain's backedge delta is zero and no iteration bound is needed; the lease and its
release are on the same path, so the free list's delta is zero too. A **retaining**
variant — one that keeps a lease across iterations — is bounded instead by route
(ii): the free list's `cap` is a type-level constant and `pool_take`'s refusal
cannot succeed on an empty list, so the composed peak is that constant. That route
composes across a call only because [RES-8] publishes a per-domain saturation flag
derived from declared rows; without it the shape is refused the moment `pool_take`
is one function down, which is where [PROV-6]'s virality clause puts it.

The diagnostic a writer gets when none of the three discharges applies names the
loop and the value:

```text
Semantics/Source [RES-3]: UnboundedStoreDemand
  domain: bump extent, store region 'a reserved at "scratch"
  the loop at @serve has backedge delta +1 and no bound
  the trip count names the runtime value "count"
  mechanical_fix: bound the loop with a compile-time constant trip count, state an
    [INV-1] invariant over len(scratch), or use the checked spelling, whose refusal
    cannot succeed on a full store
```

## 3. Evidence

Every probe cited here is in `DESIGN.md` 6.1 and 6.2 with its verdict. The four this
file rests on most: `x8`, the recursive region-plus-arena shape, **accepted today**,
which is the activation break; `x6` and `a8`, the self-referential type and its
`realloc`-and-abort drop glue, which is the cleanup-scratch domain; `a1`, the
compiler's own refusal to emit arena content with a release action, which is the
reset/content split; and `a4`'s `--stack-ledger` read, which shows the entry chain is
presently three disjoint roots and one available-stack number rather than a
per-context demand.
