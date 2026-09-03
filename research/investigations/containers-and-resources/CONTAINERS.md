# Containers: the library, written in wf

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> **fifth draft, after falsifier round 4 and the owner's minimality ruling of
> 2026-09-03**; this file has been brought to that draft and carries no rule text
> of its own. Where a sentence here disagrees with `DESIGN.md`, `DESIGN.md` wins.

The owner's ruling of 2026-09-03 fixes what the kernel is: it admits only what a
writer cannot implement in wf, container capabilities are abstracted to the lowest
common primitive, and only the truly unimplementable part enters the spec. That
turned the fourth draft's five owner types, thirty-odd operations and three views
into one primitive in two brandings, fourteen operations and two views, and moved
everything else here.

**This file is the library half of the partition test.** `DESIGN.md` 3.L states the
test, tabulates its result, writes out the two functions that earned kernel
additions, and lists the seven additions; §3 below is the rest of the library,
written in wf against `DESIGN.md` 3.K and against the unchanged v0.40 rules, with
each item's proof obligations walked. None of it is a rule, none of it is blessed,
and `DESIGN.md` 5.0 asks whether any of it should ship at all — the recommendation
there is that it ship as test programs under `tests/programs/` and not as a `std`.

Tree read: `batch/0116-containers-and-resources` at `main a40c7e70`,
`spec/kernel-spec.md` **v0.40 ACTIVE**. Bare four-digit line numbers are that
file. Nothing here is implemented, and the forms `DESIGN.md` adds compile nowhere.

## What round 4 and the ruling changed, so this file is not read as current

The fourth draft's container vocabulary is gone. `FixedVector<T, n>` survives
unchanged; `HeapVector`, `ArenaVector` and `PoolVector` are one kernel type
`Vector<'s, T>` at three regions; `FixedRing` is §3.2 below; `AppendView`, `absorb`
and L14 are deleted, and their one job — a caller keeping a length across an
appending callee — is [CALL-4]'s exit datum over a `&uniq` parameter; the `Pool`
store, `PoolSlot`, `seq_lease` and the pool seam are §3.4; `update p by op(...)` is
an admission on `set` [LIV-3]; the three failure structs are ordinary user nominals;
and [CNT-1] to [CNT-7] and [SEQ-0] are retired in favour of [BLK-0] to [BLK-4].

Two rules that were **capability** rather than text are worth naming here, because a
reader of the fourth draft will look for them. [CNT-7] refused a `&uniq` parameter
whose direct type is a container; it is **deleted**, the shape it protected is
refused by [CALL-5]'s conservative kill, and every helper in §3 below is written
through the `&uniq` run parameter it used to forbid. And the store brand's spelling
is settled by a separate amendment that lands first (`DESIGN.md` 3.K.0), so a
heap-derived type here carries no visible brand and an arena- or pool-derived one
writes it exactly where the same store appears at two positions of one declaration.

## 1. What the library is for, and what it is not

**For.** To discharge L18's obligation. A capability the fourth draft put in the
kernel is either written here in wf, or the kernel lacked a primitive and
`DESIGN.md` 3.L.6 says which. That test is what decides the kernel's size, and it is
the reason the kernel has fourteen operations rather than forty.

**Not for.** It is not a standard library proposal, it is not complete, and it is
not optimized. Where a function below is written at a concrete element type rather
than generically, the reason is stated once: a writer's generic cannot serve a copy
and an affine instantiation from one body (probes `m12`, `m14`), and a const generic
is not readable as a value (`m16`). That is an [OWN-1] question (`DESIGN.md` Q8) and
a compiler-capability question, not a partition finding.

## 2. The one fact discipline this file leans on

Every function below is a *value-in, value-out* transformation or a `&uniq`
transformation, and both rest on the same three sentences of `DESIGN.md` 3.K.

- **A measure is a term with descriptor-storage support** [MSR-2], so an element
  write does not kill a length, a sibling-field write does not kill a length, and an
  element-position `replace` of a descriptor does.
- **An in-place exchange is not a declaration event** [MSR-3], so a header invariant
  over a run an appending loop rewrites survives its own backedge: the term stays,
  the facts over it die by [MSR-2], and the row's declared relation over its call
  datum — which has empty support — re-establishes them.
- **A `&uniq` parameter's measure denotes its exit datum in an `ensures`** [CALL-4],
  so a helper that changes a borrowed run can say what it did. Without that one
  sentence every capacity proof collapses into the function that owns the run, and
  half of §3 is unwritable.

### 2.1 D1, re-derived under these rules

D1 is a caller keeping `len(line) = 10` across a callee that replaced the referent
of a `&uniq buffer<u8>`. Under [CALL-5] no transport reads the actual's spelling, so
a `&uniq` run parameter is classified by its **type**: the callee can call
`seq_place` and `seq_take` through it, so the caller's measures die at the call and
the stale subscript is refused at [OP-4]. Probe `x11` is that program without the
`replace` — a callee writing one element through `&uniq buffer<u8>` and a caller
carrying `len(out) = 4` into a subscript — **ACCEPTED today**, and it is the accept
that becomes a rejection.

The precision the old flag was buying is bought instead by the type: a `MutSpan`
argument is element-only because its type admits nothing else [CALL-3], and that is
the only parameter shape whose projected write spares a measure.

## 3. The library, written in wf

Each item states its **proof route** — which kernel rule discharges each obligation,
and which of those v0.40 already proves today. Where a probe from `DESIGN.md` 6.1 is
the same arithmetic at v0.40 scale it is named.

### 3.1 Removal from the middle, clearing, and truncation

```wf-design
fn take_at<T, const n: u64>(vector: own FixedVector<T, n>, index: own u64)
    -> (rest: own FixedVector<T, n>, value: own T)
    reads(vector), writes(vector) contract {
  requires ilt(index, len(vector));
  ensures ile(len(rest) + 1_u64, len(vector));
} {
  doc "Removes the element at index by exchanging it with the last and taking the end.";
  let count = len(vector);
  let last = count - 1_u64;
  let swapped = seq_exchange(vector: move vector, first: index, second: last);
  let (shorter, taken) = seq_take(vector: move swapped);
  return move shorter, move taken;
}
```

**Proof route.** `let count = len(vector);` establishes `count = len(vector)` by
[ENT-3.S6]. The exact `-` needs `count >= 1`, which follows from
`requires ilt(index, len(vector))` and the standing `Z <= index` [MSR-2].
`seq_exchange` needs `ilt(index, len)` — the `requires` — and `ilt(last, len)`,
which is `count - 1 < count`. `seq_take` needs `igt(len(swapped), Z)`, and
`seq_exchange` declares `len(result) = len(vector)`. The `ensures` names the
consumed parameter's **entry datum** [CALL-4], which the consume in the same
statement cannot kill. The permutation a caller needs — the former last element is
now at `index` — is this function's own `ensures` to state, not the kernel's; the
fourth draft made it an inventory row's prose.

Clearing is a drain, and it is where the element type's class shows. For a
non-linear element the writer drops each element by letting it go:

```wf-design
fn clear_bytes<const n: u64>(vector: own FixedVector<u8, n>) -> result: own FixedVector<u8, n>
    reads(vector), writes(vector) contract {
  ensures ile(len(result), Z);
} {
  doc "Removes every element from the end.";
  let held = move vector;
  let count = len(held);
  for @drain (
    at in 0_u64..count,
    invariant left: ige(len(held) + at, count),
    invariant gone: ile(len(held) + at, count)
  ) {
    set (held, dropped) = seq_take(vector: move held);
  }
  return move held;
}
```

**Proof route.** Both invariants have base `count + Z` against `count`.
`seq_take`'s `igt(len(held), Z)` discharges from `left` and `at < count`. On the
backedge `len` falls by one as `at` rises by one, so both are preserved. At the exit
`at = count`, so `gone` gives `ile(len(held), Z)`. For a **linear** element type the
same loop disposes each `dropped` instead of dropping it, and the function then
carries the store's provider and its `writes` row — which is [PROV-6]'s virality,
visible in the signature exactly as it should be. The fourth draft's `seq_clear` row
carried a `T non-linear` condition to avoid saying that; the library says it by
having two functions with two signatures.

`truncate` is the same loop with `keep` as the endpoint and
`requires ile(keep, len(vector))`.

### 3.2 `FixedRing`: removal from the front

A ring is the one shape a prefix cannot express, and it is written as a full run of
`Option<T>` plus a head and a fill. Its initialized storage is the run's whole
`[0, n)`; the *queue* is `[head, head + fill)` modulo `n`, which is the writer's own
arithmetic over the run's stable slots and is not language typestate, so L12 is
untouched.

```wf-design
struct Ring<T, const n: u64> {
  slots: FixedVector<Option<T>, n>;
  head: u64;
  fill: u64;
}

fn ring_new<T, const n: u64>() -> result: own Ring<T, n> pure contract {
  ensures ige(len(result.slots), n);
} {
  doc "Builds an empty ring over a full run of vacant slots.";
  let slots = vacant<T, n>();
  return Ring<T, n>(slots: move slots, head: 0_u64, fill: 0_u64);
}

fn ring_place<T, const n: u64>(ring: own Ring<T, n>, value: own T)
    -> (rest: own Ring<T, n>, unplaced: own Option<T>)
    reads(ring.slots, ring.head, ring.fill), writes(ring.slots, ring.fill) contract {
  requires ige(len(ring.slots), n);
  ensures ige(len(rest.slots), n);
} {
  doc "Appends one value at the tail, handing it back when the ring is full.";
  let held = move ring;
  let full = ige(held.fill, n);
  if full {
    let back = Some<T>(value: move value);
    return move held, move back;
  }
  let sum = held.head +wrap held.fill;
  let at = sum;
  let over = ige(sum, n);
  if over {
    set at = sum -wrap n;
  }
  let fresh = Some<T>(value: move value);
  let vacated = replace held.slots[at] = move fresh;
  set held.fill = held.fill +wrap 1_u64;
  let none = None<T>();
  return move held, move none;
}
```

**Proof route, and one honest cost.** The `replace` at `held.slots[at]` carries
[OP-4]'s `ilt(at, len(held.slots))`. `at` is below `n` on both arms of the `over`
branch, from `sum < 2n` (each addend below `n`) and the branch's own fact. And
`len(held.slots) >= n` is the `requires`, which the caller discharges from
`ring_new`'s `ensures` — **and that is what the construct placement of [MSR-3] is
for**. Without it, `construct Ring(slots: move slots, ...)` consumes `slots`, the
fact `len(slots) >= n` dies with its support, `len(ring.slots)` is a fresh [ENT-2]
term about which nothing is known, and every ring operation has to re-derive its
own backing's length with a runtime branch that is statically true. The construct
placement establishes the constructed value's field measures equal to the operand's
own datums, which is exactly [ENT-3.S5]'s copy equality at a measured field, and
with it the `requires`/`ensures` pair carries the fact from one operation to the
next. This is the seventh kernel addition the partition test found (`DESIGN.md` 3.L.6).

`ring_take` is the mirror: `replace held.slots[head] = None<T>();`, match the
displaced `Option`, advance `head` modulo `n`, decrement `fill`. Its `Some` arm
yields the element; its `None` arm cannot be reached when `fill > 0` and the writer
either proves that or reports it, which is ordinary program logic.

### 3.3 The growable vector and its growth policy

This is the item that decides whether the kernel's three per-slot rows are enough,
because a growth policy must relocate a run's contents in order, and the kernel has
no bulk move.

The growth policy itself:

```wf-design
fn bs_reserve(s: &uniq Bytes, heap: &uniq Heap, additional: own u64)
    -> grew: own Bool
    reads(s.v, heap), writes(s.v, heap), allocates(heap) {
  doc "Grows the backing run, preserving element order, or reports that the store refused.";
  let count = len(deref(s).v);
  match count +checked additional {
    Ok(value: want) => {
      let taken = seq_heap<u8>(heap: &uniq deref(heap), count: want);
      match taken {
        Some(value: fresh) => {
          let built = move fresh;
          for @drain (
            at in 0_u64..count,
            invariant left: ige(len(deref(s).v) + at, count),
            invariant gone: ile(len(deref(s).v) + at, count),
            invariant made: ige(len(built), at),
            invariant spare: ige(room(built) + at, want)
          ) {
            set (deref(s).v, byte) = seq_take(vector: move deref(s).v);
            set built = seq_place(vector: move built, value: byte);
          }
          let half = count / 2_u64;
          for @flip (
            at in 0_u64..half
          ) {
            let mirror = count -wrap 1_u64 -wrap at;
            set built = seq_exchange(vector: move built, first: at, second: mirror);
          }
          let old = replace deref(s).v = move built;
          dispose old using (deref(heap));
          return True();
        }
        None() => {
          return False();
        }
      }
    }
    Err(error: overflow) => {
      return False();
    }
  }
}
```

**Proof route.** `count` and `want` are ordinary values, `want >= count` from
`+checked`'s `Ok` arm. `seq_heap` publishes `len(built) = Z`, `cap(built) = want`.
The `@drain` loop carries two invariants per run, exactly as `DESIGN.md` 3.L.3 and
§3.1 do:
`left` discharges `seq_take`'s `igt(len, Z)`, `spare` discharges `seq_place`'s
`igt(room, Z)` against `at < count <= want`, and `gone` and `made` deliver
`len(deref(s).v) <= Z` and `len(built) >= count` at the exit. The `@flip` loop's two
[OP-4] obligations are `ilt(at, len(built))` from `at < half <= count <= len(built)`
and `ilt(mirror, len(built))` from `mirror <= count - 1`. The divisor is a nonzero
literal. `old` is linear, so [LIV-1] would refuse the return edge without the
`dispose`, and `dispose old using (deref(heap));` is the borrowed-provider spelling
[PROV-6] admits and no earlier draft wrote down.

**Why `seq_exchange` is in the kernel.** The drain empties the old run from its end,
so `built` comes out reversed; the `@flip` loop puts it back. Without an exchange
there is no order-preserving relocation at all: `seq_place` only appends,
`seq_take` only removes from the end, and an element-position `replace` needs a
placeholder value of the element type. `seq_exchange` is therefore not a
convenience row — it is what makes every growth policy writable, and it is the fifth
of `DESIGN.md` 3.L.6's seven.

### 3.4 The block pool

The fourth draft made a slot pool the third kernel store, with a provider, a lease,
a `PoolSlot`, a `PoolVector`, a `PoolExhausted`, six operation rows and its own seam
section. Under the ruling it is a **run of runs**: the storage comes from an arena
once, and the recycling is ordinary value movement over the outer run.

```wf-design
struct BlockPool['s] {
  free: FixedVector<Vector<'s, u8>, 8>;
}

fn pool_new['s](arena: &uniq Arena<'s, 65536, 16>) -> result: own Option<BlockPool<'s>>
    allocates(arena), writes(arena) contract {
  ensures ige(len(result.free), Z);
} {
  doc "Carves eight 256-byte runs out of the arena and holds them as a free list.";
  let free = seq_fixed<Vector<'s, u8>, 8>();
  for @carve (
    at in 0_u64..8_u64,
    invariant grown: ige(len(free), at),
    invariant spare: ige(room(free) + at, 8_u64)
  ) {
    let taken = seq_arena<u8>(arena: &uniq deref(arena), count: 256_u64);
    match taken {
      Some(value: run) => {
        set free = seq_place(vector: move free, value: move run);
      }
      None() => {
        return None<BlockPool<'s>>();
      }
    }
  }
  let pool = BlockPool<'s>(free: move free);
  return Some<BlockPool<'s>>(value: move pool);
}

fn pool_take['s](pool: &uniq BlockPool<'s>) -> leased: own Option<Vector<'s, u8>>
    reads(pool.free), writes(pool.free) {
  doc "Leases one run, or reports that the free list is empty.";
  let spare = len(deref(pool).free);
  let any = igt(spare, 0_u64);
  if any {
    set (deref(pool).free, one) = seq_take(vector: move deref(pool).free);
    return Some<Vector<'s, u8>>(value: move one);
  }
  return None<Vector<'s, u8>>();
}

fn pool_release['s](pool: &uniq BlockPool<'s>, run: own Vector<'s, u8>)
    -> unplaced: own Option<Vector<'s, u8>>
    reads(pool.free), writes(pool.free) {
  doc "Returns one run to the free list, handing it back when the list is full.";
  let spare = room(deref(pool).free);
  let fits = igt(spare, 0_u64);
  if fits {
    set deref(pool).free = seq_place(vector: move deref(pool).free, value: move run);
    return None<Vector<'s, u8>>();
  }
  return Some<Vector<'s, u8>>(value: move run);
}
```

**Which regions survive, and why.** `'s` is written at every one of its
occurrences because it **relates** positions (3.K.0): the arena's type to the pool's, the
pool's to its field's, and the pool's to the run it hands out. Every loan region
here relates nothing and is elided. That is the rule doing exactly what it is for —
the one region this code genuinely reasons about is written, and the ones that were
scaffolding are gone.

**Proof route.** `pool_new`'s two invariants are `DESIGN.md` 3.L.3's. `pool_take`'s and
`pool_release`'s obligations are discharged by a **branch**, and the branch needs
`let spare = len(...)` and `let spare = room(...)` to be *facts* — which is
[ENT-3.S6] generalized over the three measures [BLK-0]. Today S6 2779 covers `len`
alone, so `let spare = room(v);` binds an unrelated fresh atom and the whole checked
half of this library is unwritable. That is the fourth of `DESIGN.md` 3.L.6's seven, and it is
also what makes the second mechanical fix in the design's own flagship diagnostic —
"dominate the push with a branch on `room`" — a real route rather than a suggestion.

**What the pool costs and what it buys.** It costs one `Option` per lease and one
outer run of eight descriptors; a `Vector<'s, u8>` *is* a descriptor, so taking one
out of the free list moves a pointer and two words, not 256 bytes. It buys: no third
store, no lease nominal, no exhaustion nominal, no per-store release row, and no
seam. `E` carries the arena's one item and nothing else, which is L6's shape rather
than a `slots` count. And round 3's rank-one break — a lease released into the wrong
store — is not merely a type error here, it is not a spelling: `pool_release` puts
the run back into the *free list it came out of*, and a second pool over a second
arena has a different `'s` in its element type.

### 3.5 Keyed families

Stable slot identity is `vacant<T, n>()` plus element-position `replace`: the prefix
is full at `n` and never moves, so no index is renumbered, and the occupancy is the
writer's own `Option` discriminant, which is data and not typestate.

```wf-design
fn occupy<T, const n: u64>(table: own FixedVector<Option<T>, n>, key: own u64, value: own T)
    -> (rest: own FixedVector<Option<T>, n>, displaced: own Option<T>)
    reads(table), writes(table) contract {
  requires ilt(key, len(table));
} {
  doc "Puts one value in the slot named by key and hands back whatever was there.";
  let held = move table;
  let fresh = Some<T>(value: move value);
  let out = replace held[key] = move fresh;
  return move held, move out;
}
```

**Proof route.** [OP-4] against `len(held)`, which is the `requires`; [SET-2]'s
element-position replace, which [PROV-3] use 3 does not reach because `Option<T>` is
not loan-bearing and which [BLK-4] admits for a branded `T` because the position's
owner names the same region. Probe `x7` compiles exactly this shape today —
`buffer_vacant`, two element-position `replace`s, and a surviving `len` — **ACCEPTED**.

`vacate` is the same statement with `None<T>()`. A keyed *map* over these is a hash
or a search the writer writes; the kernel's part is the stable run and the bounds
obligation, and a `FixedTable<T, n>` whose occupancy is a language typestate remains
Q6's question rather than this design's.

**One price, stated rather than discovered** [PROV-6]: when `T` is linear the
displaced `Option<T>` is linear too, so every occupy owes a disposal on an arm the
writer can see is dead. That is the correct consequence of type-based linearity and
the pattern owed to `docs/patterns.md` should say so.

### 3.6 The convenience forms

The fourth draft's `update` statement and its `try` inventory rows are here, and
none of them needed a kernel rule.

```text
| fourth draft                            | this draft                                        |
|-----------------------------------------|---------------------------------------------------|
| update p by op(args);                   | set p = op(receiver: move p, args);               |
| update p by op(args) into x;            | set (p, x) = op(receiver: move p, args);          |
| seq_try_place(vector, value)            | a library fn: branch on room, place or hand back  |
| seq_try_take(vector)                    | a library fn: branch on len, take or None         |
| seq_try_push(view, value)               | the same over a &uniq run                         |
| seq_clear, seq_truncate                 | §3.1                                              |
| seq_take_at                             | §3.1                                              |
| seq_filled, seq_vacant                  | `DESIGN.md` 3.L.3                                 |
| seq_reserve_heap, seq_reserve_arena     | §3.3                                              |
| seq_shrink                              | §3.3, with want < count                           |
| seq_lease, seq_lease_proved             | §3.4                                              |
| ring_place, ring_try_place, ring_take,  | §3.2                                              |
|   ring_try_take                         |                                                   |
| seq_push, seq_pop, absorb               | `DESIGN.md` 3.L.4, over a &uniq run               |
```

The `try` rows are the interesting entry, because they are where "a convenience is
not a rule" is least obvious. Each is a branch on a measure and two returns:

```wf-design
fn try_place<T, const n: u64>(vector: own FixedVector<T, n>, value: own T)
    -> (rest: own FixedVector<T, n>, unplaced: own Option<T>)
    reads(vector), writes(vector) {
  doc "Appends one value, handing it back when the run is full.";
  let held = move vector;
  let spare = room(held);
  let fits = igt(spare, 0_u64);
  if fits {
    set held = seq_place(vector: move held, value: move value);
    return move held, None<T>();
  }
  return move held, Some<T>(value: move value);
}
```

which is the same shape as `pool_release` and rests on the same [ENT-3.S6]
generalization. The kernel keeps only the **proved** spelling of each operation,
because the proved one is what cannot be written: a total operation at a capacity
boundary either refuses — which is a branch a writer writes — or displaces
something, which L9 forbids for an affine value.

`update`'s own two shapes are not lost, they are [LIV-3]'s two spellings of `set`,
and 3.K's [LIV-3] states why the exchange itself had to stay: it is the only form
the partition test could not write.

## 4. From an unaware writer to an accepted program

Four walkthroughs, each ending at a program the rules accept.

**A place without a capacity proof.** `set v = seq_place(vector: move v, value: b);`
with nothing known about `room(v)` reports `[BLK-0] UndischargedOperationDomain`
with residual `Z < room(v)` and four repairs: a header invariant over `room(v)`, a
dominating branch on `room(v)`, a larger run before the loop, or §3.6's `try_place`.
Three of the four are discharged in this file, and the branch route exists only
because [ENT-3.S6] generalizes over the three measures.

**A linear value that reaches a scope exit.** `[PROV-6] LinearValueNotDisposed`
names the binding, its store region, and the provider a `dispose` would need. In a
compile-time-sized program this diagnostic never fires, because a frame-resident run
is not linear; in a hosted one it fires once per store-backed value per exit, and
§3.3's `bs_reserve` is what it looks like when it is satisfied.

**A linear value taken apart.** `let page = move chunk.page;` on a linear `Chunk`
reports `[PROV-6] LinearValuePartiallyMoved`, names the residual leaf, and points at
`let Chunk(page: p, spare: q) = move chunk;`. Probe `x4` is the program that is
accepted today and leaks; the destructuring consume is the route out, and it is what
makes a slab free list writable at all.

**Two runs, one function.** Two stores in scope means both brands are written at
every position that names one, which is where the distinction is real; one store
means none is written anywhere. `DESIGN.md` 3.K.0 states the rule and 3.L.5 shows
`byte_string.wf`'s join with and without it.

## 5. Evidence

Every probe cited here is in `DESIGN.md` 6.1 and 6.2 with its verdict. The three
this file rests on most: `x1c` and `x1d`, the two-invariant construction loop whose
exit ordering discharges a subscript with no equality anywhere, **accepted**; `x7`,
the vacant table with two element-position `replace`s and a surviving `len`,
**accepted**; and `x2`/`x3`, the two spellings of the in-place exchange, both
**rejected** today, which is why [LIV-3] is a kernel addition and not a convenience.
