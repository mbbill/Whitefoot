# Containers: the library, written in wf

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> **sixth draft, after falsifier round 5 and the owner's rulings of 2026-09-03
> evening**; this file has been brought to that draft and carries no rule text of
> its own. Where a sentence here disagrees with `DESIGN.md`, `DESIGN.md` wins.
>
> **Every spelling below that the kernel does not have today is a proposal**, listed
> in `DESIGN.md` 3.S with its alternatives and marked PROPOSED. Nothing here is
> decided.

Tree read: `batch/0116-containers-and-resources` at `main 30602914`,
`spec/kernel-spec.md` **v0.41 ACTIVE**. Bare three- and four-digit line numbers are
that file. Nothing here is implemented, and the forms `DESIGN.md` adds compile
nowhere. Every clause, call and comparison is written in the v0.41 surface: infix
`== != < <= > >=`, and `::` before a call's type and region arguments.

## What round 5 and the owner's rulings changed, so this file is not read as current

Five things a reader of the fifth draft will look for and not find.

- **No helper takes a container by `&uniq`.** R1 makes every helper value-in,
  value-out: it takes the run and hands it back, and its contract relates a result to
  an input. The fifth draft's `&uniq` destination and the exit datum that published
  its post-state are both withdrawn, because round 5 showed the exit datum had no
  callee-side placement and restored D1 through it.
- **A ring is not a library type.** [BLK-1]'s typestate is a *window*, so a queue, a
  ring and a deque are all one `FixedVector<T, n>` used from both ends: no `Option`
  per slot, no tag, ordinary element access, exact `len`. The fifth draft's
  `Ring<T, n>` over `Option<T>` cost about seven times the memory of a hand-written
  byte ring and deleted in-place slot mutation.
- **`seq_exchange` is not a kernel row.** §3.1 writes the swap in three statements
  over rows the kernel already has, which is why L18 removed it — and states what
  writing it that way costs.
- **A lease is `linear` by declaration.** R2 gives a library store the property the
  criterion cannot see, so a lease that is dropped, or a refusal that is ignored, is
  a compile error rather than a silent block leak. §3.4 is the pool.
- **Every signature carries the row its body exhibits.** A measure read through a
  borrow is `reads` at the caller (probe `t10`), and an allocating row is `reads`,
  `allocates` and `writes` of the same provider path. Round 5 found every signature
  in the fifth draft's library wrong in one direction or the other.

## 1. What the library is for, and what it is not

**For.** To discharge L18's obligation in both directions. A capability the fourth
draft put in the kernel is either written here in wf, or the kernel lacked a
primitive and `DESIGN.md` 3.L.6 says which — and a row the fifth draft put in the
kernel that turns out to be writable here leaves it. That test is what decides the
kernel's size, and it is the reason the kernel has twelve declaration-domain
operations rather than forty.

**Not for.** It is not a standard library proposal, it is not complete, and it is not
optimized. Four discipline sentences from `DESIGN.md` 3.L.0 govern every function
below and are not repeated per function: every body is three-address; `0_u64` and
never `Z`; a signature declares exactly the row its body exhibits; and a generic that
*reuses* a value is written per element class, because a writer's generic cannot
serve a copy and an affine instantiation from one body (probes `m12`, `m14`). What is
**not** a limit any more is capacity genericity: [MSR-6] makes a const generic a
value, an endpoint and a clause operand, so `filled<T, const n>` is one body where
the fifth draft needed one per capacity.

## 2. The one fact discipline this file leans on

Every function below is a *value-in, value-out* transformation, and they all rest on
the same four sentences of `DESIGN.md` 3.K.

- **A measure is a term with descriptor-storage support** [MSR-2], so an element
  write does not kill a length, a sibling-field write does not kill a length, and an
  element-position `replace` of a descriptor does.
- **Every row publishes every measure it writes, exactly** [BLK-0]. This is the
  sentence round 5 found missing, and it is why each invariant below is preserved by
  **one** published premise rather than by three: `seq_place` publishes
  `room(result) = room(vector) - 1` itself instead of leaving `room` to be
  reconstructed from `len + room = cap`, which costs two premises before the goal is
  reached and which [ENT-6] 3019 does not admit. Probes `g3` and `g4` are the same
  loop without and with the published relation.
- **An in-place exchange is not a declaration event** [MSR-3], so a header invariant
  over a run an appending loop rewrites survives its own backedge.
- **A contract relates a result to an input, single-state in both** [CALL-4]. There
  is no entry/exit convention to remember, because a parameter is an input and has
  one state. That is what makes `ensures len(rest) >= len(out)` — the non-shrink
  guarantee L14 was retired for — an ordinary clause.

### 2.1 D1, re-derived under R1

D1 is a caller keeping `len(line) = 10` across a callee that replaced the referent of
a `&uniq buffer<u8>`. Under R1 the program has no shape: a helper that transforms a
run takes it **by value** and hands it back, so what the caller holds afterwards is
the *result*, whose length the callee's own `ensures` describes and about which no
callee can be wrong; and a helper that only writes elements takes a length-fixed view
[CALL-3], whose length cannot change at all. Probe `t9` is D1 at this tip, still
accepted, and it is the accept that has no successor rather than the accept that
becomes a rejection.

The fifth draft closed D1 by classifying the kill at a parameter type it still
admitted, and round 5 defeated that twice: once with a fact published *after* the
kill (the exit datum) and once with an action that is not a write (`dispose` through
a shared borrow). A door per channel is not a closure; withdrawing the parameter is.

## 3. The library, written in wf

Each item states its **proof route** — which kernel rule discharges each obligation,
and which of those v0.41 already proves today. Where a probe from `DESIGN.md` 6.1 is
the same arithmetic at v0.41 scale it is named.

### 3.1 The swap, removal from the middle, clearing, and truncation

The swap is the fifth draft's `seq_exchange`, written in wf:

```wf-design
fn swap_with_last<T, const n: u64>(vector: own FixedVector<T, n>, at: own u64)
    -> result: own FixedVector<T, n>
    reads(vector), writes(vector) contract {
  requires at + 1_u64 <= len(vector);
  ensures len(result) >= len(vector);
} {
  doc "Exchanges the element at at with the last element.";
  let (rest, endv) = seq_take(vector: move vector);
  let old = replace rest[at] = move endv;
  let back = seq_place(vector: move rest, value: move old);
  return move back;
}
```

**Proof route.** `seq_take` needs `len(vector) > 0`, which the `requires` gives with
the standing `0 <= at`. The `replace` at `rest[at]` carries [OP-4]'s
`at < len(rest)`, and `seq_take` published `len(rest) = len(vector) - 1`, so the
`requires` is exactly what discharges it — which is why the `requires` is written
`at + 1_u64 <= len(vector)` and not `at < len(vector)`. `seq_place` needs
`room(rest) > 0`, and `seq_take` published `room(rest) = room(vector) + 1`. Three
statements, three published relations, and no premise reconstructed from the standing
identity.

**And the cost is stated.** One kernel row would have published
`len(result) = len(vector)` once; this form kills and re-establishes `len` twice, so a
caller carries the measure through three steps and the `requires` is one unit
tighter than it reads. `DESIGN.md` 3.K.3 records that as the trade L18 asks for, and
3.S has no entry for the row because this draft does not propose it.

`take_at` is then `swap_with_last` and `seq_take`, in two statements. Clearing is a
drain, and it is where the element class shows. For a non-linear element:

```wf-design
fn clear_bytes<const n: u64>(vector: own FixedVector<u8, n>) -> result: own FixedVector<u8, n>
    reads(vector), writes(vector) contract {
  ensures len(result) <= 0_u64;
} {
  doc "Removes every element from the end.";
  let count = len(vector);
  for @drain (
    at in 0_u64..count,
    invariant left: len(vector) + at >= count,
    invariant gone: len(vector) + at <= count
  ) {
    set (vector, dropped) = seq_take(vector: move vector);
  }
  invariant done: len(vector) <= 0_u64;
  return move vector;
}
```

**Proof route.** Both invariants have base `count + 0` against `count`.
`seq_take`'s `len(vector) > 0` discharges from `left` and `at < count`. On the
backedge `len` falls by one as `at` rises by one, each from `seq_take`'s own
published `len(rest) = len(vector) - 1`. `done` is the [INV-1] exact-exhaustion
conclusion at the continuation — probes `x1c` and `x1d` are that shape accepted
today — and it is what carries the exit fact past the loop, where the header batch is
removed. `dropped` is a later target of a multi-target `set` and is therefore an
ordinary `let` binding of the enclosing block [LIV-3], so it is a `u8` that goes out
of scope each iteration.

For a **linear** element type the same loop must do something with `dropped`, and the
signature says so: it carries the store's provider and its `writes` row, which is
[PROV-6]'s virality made visible exactly where it should be. The library therefore
has two functions with two signatures rather than one function with a condition on
its element type.

`truncate` is the same loop with `keep` as the endpoint and
`requires keep <= len(vector);`.

### 3.2 Queues, rings and deques

**There is nothing to write.** Under [BLK-1]'s window a run's initialized set is the
`len` slots beginning at `head`, so a queue is a `FixedVector<T, n>` with
`seq_place` at the back and `seq_take_front` at the front, a stack is the same run
with `seq_take`, and a deque is all four rows. There is no `Option`, no tag, no head
field, no fill field and no wrapping arithmetic in the writer's code, `v[i]` reaches
the element at logical offset `i` directly, and `len` is exact.

The fifth draft's `Ring<T, n>` over `FixedVector<Option<T>, n>` is what this replaces,
and the comparison is the window's whole justification. Under [OP-9] 992 a payload
enum sequences a `(4,4)` tag before its payload, so `Option<u8>` is `(8,8)`:

```text
| a 256-slot byte ring          | language-ceiling size                |
|-------------------------------|--------------------------------------|
| fifth draft: Ring<u8, 256>    | 256 x 8 + 8 + 16  =  2072 bytes      |
| this draft: FixedVector<u8,256>| 256 x 1 + 24      =   280 bytes      |
```

That is the **language** ceiling, which is what Appendix A.1 is built from and what
`E` publishes and a deployment sizes against. And for an affine element type the
`Option` ring additionally deleted in-place mutation, because no place reaches inside
an enum payload: a DMA descriptor in a slot had to be displaced, matched, rebuilt and
replaced back, two moves of a 2072-byte record per completion. Under the window
`set ring[i].flags = 2_u32;` is an ordinary element write.

**One thing a ring gives up**, and it is [VIEW-2]'s `requires head(vector) <= 0_u64`:
a `Span` is contiguous and a wrapped window is not, so a run that has had a front
removal cannot be viewed until it is drained into one that has not. A transmit path
that hands bytes to `write_once` therefore drains its ring into a `filled` staging
run, which is one copy the fifth draft's ring did not need — and which the fifth
draft's ring paid for eight times over in bytes.

### 3.3 The growable vector and its growth policy

This is the item that decides whether the kernel's four per-slot rows are enough,
because a growth policy must relocate a run's contents **in order**.

```wf-design
struct Bytes {
  v: Vector<u8>;
}

enum Grown {
  Grew(value: Bytes, room: u64);
  Refused(value: Bytes);
}

fn bs_reserve(s: own Bytes, heap: &uniq Heap, total: own u64) -> grown: own Grown
    reads(s.v, heap), allocates(heap), writes(s.v, heap) contract {
  requires total >= len(s.v);
  ensures when Grew(value: ready, room: spare): spare + len(ready.v) >= total;
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
        invariant made: len(built) >= at,
        invariant spare: room(built) + at >= total
      ) {
        set (s.v, byte) = seq_take_front(vector: move s.v);
        set built = seq_place(vector: move built, value: byte);
      }
      let old = replace s.v = move built;
      dispose old using (deref(heap));
      let free = room(s.v);
      return Grew(value: move s, room: free);
    }
  }
}
```

**Proof route, and what the window bought.** `seq_heap` publishes `len(built) = 0`,
`cap(built) = total` and `room(built) = total` on its `Some` arm — all three, which is
[BLK-0]'s completeness sentence. `left` and `gone` bound the source, `made` and
`spare` bound the destination, and each is preserved by one published relation:
`seq_take_front`'s `len(rest) = len(vector) - 1` and `seq_place`'s
`len(result) = len(vector) + 1` and `room(result) = room(vector) - 1`. `seq_place`'s
`room(built) > 0` discharges from `spare` and `at < count <= total`, the last from the
`requires`. At the exit `len(s.v) <= 0` and `len(built) >= count`, and
`spare + len(ready.v) >= total` is [MSR-2]'s identity at the constructed field.

**The window is what makes this work at all**, and it is the second reason it is in
the kernel. Draining from the **front** and appending at the **back** copies the run
in order, so there is no reversal to undo. The fifth draft's prefix drained from the
end, so `built` came out backwards and a second `@flip` loop had to put it back —
a loop round 5 showed carrying two undischargeable [OP-4] obligations, because
`let mirror = count -wrap 1_u64 -wrap at;` is two operations in one expression (probe
`t13`), the wrapping form hands the checker a fresh atom with no ordering
(`docs/patterns.md` P8), and `half <= count` from `count / 2_u64` is a fact no premise
of [MSR-4] produces. All of that is gone rather than repaired.

**And the `+checked` route is gone with it.** The fifth draft computed
`count +checked additional` and claimed `want >= count` from the `Ok` arm; [ENT-3.S7]
2791 admits that only for a **constant** addend and probes `g1` and `g2` are the two
halves. This version takes the **total** capacity as its parameter and states
`requires total >= len(s.v)`, so the caller does the addition where it has the facts
to prove it and no widening of S7 is proposed.

`old` is linear, so [LIV-1] would refuse the return edge without the `dispose`, and
`dispose old using (deref(heap));` is the borrowed-provider spelling [PROV-6] admits.
`Grown` has a linear field, so it is linear by containment, so neither arm can be
dropped by the caller — which is what makes 4.2's two `dispose` statements
mandatory rather than conscientious.

`bs_shrink` is the same function with `total < count`, and its `requires` becomes
`total <= len(s.v)` with the drain bounded by `total`.

### 3.4 The block pool, and where the obligation goes

The fourth draft made a slot pool the third kernel store, with a provider, a lease, a
`PoolSlot`, a `PoolVector`, a `PoolExhausted`, six operation rows and its own seam
section. Under the minimality ruling it is a **run of runs**: the storage comes from
an arena once, and the recycling is ordinary value movement over the outer run.

**Where the linear obligation goes is the modelling decision**, and getting it wrong
is instructive. The obligation belongs on the value that is *handed out*, not on the
container of spares:

```wf-design
linear struct Lease['s] {
  run: Vector<'s, u8>;
}

struct BlockPool['s] {
  free: FixedVector<Vector<'s, u8>, 8>;
}
```

`Lease` is linear by declaration [S18], so a writer holding one must return it or take
it apart. `BlockPool`'s free list holds bare runs, which are arena-backed and
therefore affine, so the pool itself is an ordinary value with an ordinary derived
release. Had the free list held leases, the pool would have been linear by
containment and an **empty** one would still have had no route out — a run of a
declaration-linear element type is linear whatever its length, and neither `dispose`
nor a destructuring consume reaches a run. `DESIGN.md` Q13 records that shape and its
two candidate fixes; this file avoids it.

```wf-design
fn pool_new['s](arena: &uniq Arena<'s, 65536, 16>) -> result: own Option<BlockPool<'s>>
    reads(arena), allocates(arena), writes(arena) contract {
  ensures when Some(value: made): len(made.free) >= 8_u64;
} {
  doc "Carves eight 256-byte runs out of the arena and holds them as a free list.";
  let free = seq_fixed::<Vector<'s, u8>, 8>();
  for @carve (
    at in 0_u64..8_u64,
    invariant grown: len(free) >= at,
    invariant spare: room(free) + at >= 8_u64
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
    reads(pool.free), writes(pool.free) {
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
    -> (rest: own BlockPool<'s>, unreturned: own Option<Lease<'s>>)
    reads(pool.free), writes(pool.free) {
  doc "Returns one lease to the free list, handing it back when the list is full.";
  let spare = room(pool.free);
  let fits = spare > 0_u64;
  if fits {
    let Lease(run: back) = move lease;
    set pool.free = seq_place(vector: move pool.free, value: move back);
    return move pool, None<Lease<'s>>();
  }
  return move pool, Some<Lease<'s>>(value: move lease);
}
```

**The refusal path of `pool_new` is worth one sentence**, because R2 makes it
checkable. `free` at that point holds runs, which are affine, so returning `None`
drops them and the arena reclaims their bytes with the region — which is correct, and
which is only correct because the free list does not hold leases. Under the other
modelling the same `return` would be a [PROV-6] error, and the writer would have to
drain a partially built list on a refusal path.

**Which regions survive, and why.** `'s` is written at every one of its occurrences
because it **relates** positions (3.K.0): the arena's type to the pool's, the pool's
to its field's, the pool's to the lease it hands out, and the lease's to its run.
Every loan region here relates nothing and is elided, and every call site elides `'s`
too because an operand determines it. That is the rule doing exactly what it is for.

**Proof route.** `pool_new`'s two invariants are `DESIGN.md` 3.L.3's. `pool_take`'s
and `pool_release`'s obligations are discharged by a **branch**, and the branch needs
`let spare = len(...)` and `let spare = room(...)` to be *facts* — which is
[ENT-3.S6] generalized over the four measures [BLK-0]. Today S6 2785 covers `len`
alone, so `let spare = room(v);` binds an unrelated fresh atom and the whole checked
half of this library is unwritable. That is the third of `DESIGN.md` 3.L.6's eight,
and it is also what makes the second mechanical fix in the design's flagship
diagnostic — "dominate the place with a branch on `room`" — a real route.

**What the pool does not publish, and what that costs.** `pool_take` cannot state
`room(got.run) >= 256_u64`, because a `Vector<'s, u8>` carries its capacity as a
measure and not in its type, so putting one into a `FixedVector` element and taking it
out loses the figure `pool_new` established. A caller that needs room therefore reads
it and branches, once per lease. That is the honest price of the pool being library
data rather than a kernel store, and `DESIGN.md` 4.1 pays it in the open.

**What the pool costs and what it buys.** It costs one `Option` per take and one
outer run of eight descriptors; a `Vector<'s, u8>` *is* a descriptor, so taking one
out of the free list moves a pointer and three words, not 256 bytes. It buys: no
third store, no lease store nominal, no exhaustion nominal, no per-store release row,
and no seam. `E` carries the arena's one item and nothing else, which is L6's shape
rather than a `slots` count. And round 3's rank-one break — a lease released into the
wrong store — is not merely a type error here, it is not a spelling: `pool_release`
puts the run back into the free list it came out of, and a second pool over a second
arena has a different `'s` in its element type.

### 3.5 Keyed families

Stable slot identity is `vacant<T, n>()` plus element-position `replace`: the run is
full at `n` and never moves, so no index is renumbered, and the occupancy is the
writer's own `Option` discriminant, which is data and not typestate.

```wf-design
fn occupy<T, const n: u64>(table: own FixedVector<Option<T>, n>, key: own u64, value: own T)
    -> (rest: own FixedVector<Option<T>, n>, displaced: own Option<T>)
    reads(table), writes(table) contract {
  requires key < len(table);
} {
  doc "Puts one value in the slot named by key and hands back whatever was there.";
  let fresh = Some<T>(value: move value);
  let out = replace table[key] = move fresh;
  return move table, move out;
}
```

**Proof route.** [OP-4] against `len(table)`, which is the `requires`; [SET-2]'s
element-position replace, which [PROV-3] use 3 does not reach because `Option<T>` is
not loan-bearing and which [BLK-4] admits for a branded `T` because the position's
owner names the same region. Probe `x7` compiles exactly this shape today —
`buffer_vacant`, two element-position `replace`s, and a surviving `len` — accepted.

`vacate` is the same statement with `None<T>()`. A keyed *map* over these is a hash
or a search the writer writes; the kernel's part is the stable run and the bounds
obligation, and a `FixedTable<T, n>` whose occupancy is a language typestate remains
`DESIGN.md` Q6's question.

**One price, stated rather than discovered** [PROV-6]: when `T` is linear the
displaced `Option<T>` is linear too, so every `occupy` owes the caller a `match` on
an arm the writer can see is dead. That is the correct consequence of type-based
linearity, and the pattern owed to `docs/patterns.md` should say so.

### 3.6 The convenience forms

The fourth draft's `update` statement and its `try` inventory rows are here, and none
of them needed a kernel rule.

```text
| fourth draft                            | this draft                                        |
|-----------------------------------------|---------------------------------------------------|
| update p by op(args);                   | set p = op(receiver: move p, args);               |
| update p by op(args) into x;            | set (p, x) = op(receiver: move p, args);          |
| seq_try_place(vector, value)            | a library fn: branch on room, place or hand back  |
| seq_try_take(vector)                    | a library fn: branch on len, take or None         |
| seq_try_push(view, value)               | the same, value in and value out                  |
| seq_clear, seq_truncate                 | §3.1                                              |
| seq_take_at                             | §3.1                                              |
| seq_exchange                            | §3.1, in three statements                         |
| seq_filled, seq_vacant                  | `DESIGN.md` 3.L.3                                 |
| seq_reserve_heap, seq_reserve_arena     | §3.3                                              |
| seq_shrink                              | §3.3, with total < count                          |
| seq_lease, seq_lease_proved             | §3.4                                              |
| FixedRing and its four rows             | nothing: a ring is a run [BLK-1]                  |
| seq_push, seq_pop, absorb               | `DESIGN.md` 3.L.4, value in and value out         |
```

The `try` rows are the interesting entry, because they are where "a convenience is
not a rule" is least obvious. Each is a branch on a measure and two returns:

```wf-design
fn try_place<T, const n: u64>(vector: own FixedVector<T, n>, value: own T)
    -> (rest: own FixedVector<T, n>, unplaced: own Option<T>)
    reads(vector), writes(vector) {
  doc "Appends one value, handing it back when the run is full.";
  let spare = room(vector);
  let fits = spare > 0_u64;
  if fits {
    set vector = seq_place(vector: move vector, value: move value);
    return move vector, None<T>();
  }
  return move vector, Some<T>(value: move value);
}
```

and `try_take` is its mirror on `len(vector) > 0`, returning
`(rest: own FixedVector<T, n>, taken: own Option<T>)`. Both rest on the same
[ENT-3.S6] generalization, and both are written per element class where the body
moves a `T`.

The kernel keeps only the **proved** spelling of each operation, because the proved
one is what cannot be written: a total operation at a capacity boundary either
refuses — which is a branch a writer writes — or displaces something, which L9 forbids
for an affine value.

## 4. From an unaware writer to an accepted program

Four walkthroughs, each ending at a program the rules accept.

**A place without a capacity proof.** `set v = seq_place(vector: move v, value: b);`
with nothing known about `room(v)` reports `[BLK-0] UndischargedOperationDomain` with
residual `0_u64 < room(v)` and four repairs: a header invariant over `room(v)`, a
dominating branch on `room(v)`, a larger run before the loop, or §3.6's `try_place`.
Three of the four are discharged in this file, and the branch route exists only
because [ENT-3.S6] generalizes over the four measures.

**A linear value that reaches a scope exit.** `[PROV-6] LinearValueNotDisposed` names
the binding, its store region, and the provider a `dispose` would need. In a
compile-time-sized program it never fires, because a frame-resident run needs no
capability to reclaim; in a hosted one it fires once per store-backed value per exit,
and §3.3's `bs_reserve` is what it looks like when it is satisfied. **The second
diagnostic is R2's**: `[PROV-6] LinearValueNotConsumed` names a value that is linear
by *declaration*, says that no leaf of it requires a capability so it cannot be
disposed, and points at the two routes that remain.

**A linear value taken apart.** `let page = move chunk.page;` on a linear `Chunk`
reports `[PROV-6] LinearValuePartiallyConsumed`, names the residual leaf, and points
at `let Chunk(page: p, spare: q) = move chunk;`. Probes `x4`, `g7` and `p6_partial`
are the program that is accepted today, and the third shows the residual being freed
by a derived drop. The refusal is stated over the **consume**, so it reaches
`dispose chunk.page using (heap);` as well.

**Two runs, one function.** Two stores in scope means both brands are written at
every position that names one, which is where the distinction is real; one store
means none is written anywhere. `DESIGN.md` 3.K.0 states the two criteria and 3.L.5
shows `byte_string.wf` with and without them — and counts the seven disposals R2
adds beside the thirty-seven brand items 3.K.0 removes.

## 5. Evidence

Every probe cited here is in `DESIGN.md` 6.1 with its verdict, and each was run twice
— once at v0.40 and once respelled at v0.41 — with the same result. The five this
file rests on most: `x1c` and `x1d`, the two-invariant construction loop whose exit
ordering discharges a subscript with no equality anywhere, **accepted**; `g4` against
`g3`, the same three-term header invariant with and without one published relation,
**accepted then rejected**, which is why [BLK-0] requires every measure on every
exit; `x7`, the vacant table with two element-position `replace`s and a surviving
`len`, **accepted**; `t8`, the in-place exchange, **rejected** by [STOR-1], which is
why [LIV-3] restates that rule's partition; and `t1` against `t4`, a const generic
and a named const in the same three positions, **rejected then accepted**, which is
why [MSR-6] is one of the eight.
