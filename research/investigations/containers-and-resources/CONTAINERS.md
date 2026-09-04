# Containers: the library, written in wf

> **Superseded in place by `DESIGN.md`.** The integrated containers-and-resources
> design is `DESIGN.md` beside this file; read it first. `DESIGN.md` is at its
> **eighth draft, after falsifier round 7 and the owner's decisions of 2026-09-03**;
> this file has been brought to that draft and carries no rule text of its own. Where
> a sentence here disagrees with `DESIGN.md`, `DESIGN.md` wins.
>
> **Every language-surface addition below is now the owner's decision**, recorded in
> `DESIGN.md` 3.S, which is a decision record rather than a proposal table. Three
> items remain PROPOSED and are marked where they occur: `seq_reslice` [S31], a
> linearity bound on a generic parameter [S32], and `ReserveOutcome` [S33]. Of the
> seventh draft's three open items, `on_propagate` [S28] is **REJECTED**, `seq_rebase`
> [S29] is **WITHDRAWN to the library** (`DESIGN.md` 3.L.8), and the seven [SYS-8]
> signatures over views [S30] are **ADOPTED**.

Tree read: `batch/0116-containers-and-resources` at `main 30602914`,
`spec/kernel-spec.md` **v0.41 ACTIVE**, with **v0.42 merging**: v0.42 adds `[FORM-8]`
canonical region spelling, over **regions only**. Bare three- and four-digit line
numbers are v0.41. Nothing here is implemented, and the forms `DESIGN.md` adds compile
nowhere. Every clause, call and comparison is written in the v0.41 surface: infix
`== != < <= > >=`; **every type and const argument of a user generic is written**
([FN-2] 1124, probe `q4`), and a region argument is written exactly where [FORM-8]
writes it.

## What round 7 and the owner's decisions changed, so this file is not read as current

Nine things a reader of an earlier draft will look for and not find. The first two are
the owner's decisions of 2026-09-03 and govern every function below.

- **D3: linearity is read against the SCOPE.** A store-backed value is linear only in a
  scope that does not hold the capability its release needs. In a scope that holds it —
  a function whose signature carries `heap: &uniq Heap`, or `main` with the entry heap —
  the compiler derives the release on every leaving edge, charged to that scope's
  `writes(heap)`. **Every `dispose` ceremony an earlier draft's walkthroughs carried is
  gone**: `DESIGN.md` 3.L.5's `bs_reserve` keeps exactly one `dispose old;`, the *early*
  release, and neither worked program has any.
- **D4: every loop body is implicitly a region block.** A borrow of an outer binding
  inside a loop body is written **bare**, and an explicit `region { }` as the loop body's
  only enclosing block is a `[FORM]` rejection. That amendment has **not** landed;
  `DESIGN.md` 3.K.0 and §7's B0b say so, and probes `q2` and `q3` are the evidence.
- **The declaration domain has TWELVE operations, not thirteen.** `seq_rebase` is
  withdrawn to the library under L18: returning a wrapped window to its origin is a drain
  into a fresh run, written and priced in `DESIGN.md` 3.L.8.
- **No helper takes a container by `&uniq`, and a rule says so.** [BLK-4]'s fourth
  clause refuses a container nominal, a loan-bearing type **or an unbounded generic type
  parameter** as the direct or indirect referent of a `&uniq` parameter of a
  source-declared `fn`, over the reachability closure [PROV-4] computes. The generic
  clause is round 7's: `&uniq Holder<T>` at `T = buffer<u8>` compiles today.
- **Every contract that hands a measured value back is complete over every measure**
  [CALL-7], and **the obligation is decidable**: a syntactic per-measure, per-route clause
  condition with three type-decidable exclusions. A clause both of whose sides follow from
  [MSR-2]'s standing facts — `ensures head(result) <= cap(result);` — discharges nothing.
- **One `set` commit rule** (D2): the right-hand side is evaluated with every target dead
  from its own **read-out**, all targets are reinitialised at one commit, targets are
  pairwise non-overlapping, and a target that names a binding in scope is a commit and not
  a redeclaration. There is no swap or exchange operation anywhere.
- **A ring is not a library type.** [BLK-1]'s typestate is a *window*, so a queue, a ring
  and a deque are all one `FixedVector<T, n>` used from both ends: no `Option` per slot,
  no tag, ordinary element access, exact `len`.
- **A lease is `linear` by declaration, and its release is the PROVED spelling.** The
  modifier makes a discard visible and deliberate; it does not make it impossible, because
  a destructuring consume is a legal consume. A **directional** obligation is bought by
  proving the return, so §3.4's `pool_release` is total under
  `requires room(pool.free) > 0_u64` and has no refusal arm to discard. Its admission
  condition is round 7's: the modifier is admitted only on an **affine** nominal, never on
  a tag-only enum, which probe `q11` shows is copy today.
- **Every signature carries the row its body exhibits, in [EFF-1] 1369's canonical order
  `reads, writes, allocates`**, and **a `replace` is a kill and never a publication**
  [SET-2] 528 — so a value whose measures must survive is **constructed** into its owner
  through [MSR-3]'s construct placement, and a function returning a `replace`'s displaced
  value is refused by [CALL-7].
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
the same six sentences of `DESIGN.md` 3.K.

- **A measure is a term with descriptor-storage support** [MSR-2], so an element
  write does not kill a length, a sibling-field write does not kill a length, and an
  element-position `replace` of a descriptor does.
- **Every row publishes every measure it writes, exactly** [BLK-0]. This is the
  sentence round 5 found missing, and it is why each invariant below is preserved by
  **one** published premise rather than by three: `seq_place` publishes
  `room(result) = room(vector) - 1` itself instead of leaving `room` to be
  reconstructed from `len + room = cap`, which costs two premises before the goal is
  reached and which [ENT-6] 3015 does not admit. Probes `g3` and `g4` are the same
  loop without and with the published relation.
- **A `set` target that names a binding in scope keeps that binding's term** [LIV-2,
  MSR-3], so a header invariant over a run an appending loop rewrites survives its own
  backedge, and a target that resolves to no binding declares one.
- **A declared relation becomes a fact by [CALL-6]'s S13**, which substitutes each
  formal by that call's datum and each `writes` target by its post-state, establishes
  the relation on the normal continuation or at a selected arm, and gives it the
  ordinary L0 support of its substituted terms. Without that rule no proof route below
  has a first step.
- **A hand-back contract is complete** [CALL-7]: a function that constructs a measured
  value or receives one `own` and returns it publishes every measure of it, exactly
  where the body establishes an exact value and two-sidedly otherwise. That is why the
  contracts below are longer than the sixth draft's and why their callers compile.
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
[CALL-3], which reaches element storage only. Probe `r1` is D1 at this tip, still
accepted; probe `r5` is the same program with every region name elided, accepted by a
build that implements the region-spelling amendment.

**And a rule refuses the parameter**, which is round 6's finding: R1 was doctrine for
one draft and doctrine refuses no declaration. [BLK-4]'s fourth clause makes a
container nominal or a loan-bearing type inadmissible as the direct or indirect
referent of a `&uniq` parameter of a source-declared `fn`, over the reachability
closure [PROV-4] computes — so the one-field-wrapper defeat that killed [CNT-7] in
round 4 is closed by construction. The fifth draft closed D1 by classifying the kill
at a parameter type it still admitted, and round 5 defeated that twice: once with a
fact published *after* the kill and once with an action that is not a write. A door
per channel is not a closure; refusing the parameter is.

## 3. The library, written in wf

Each item states its **proof route** — which kernel rule discharges each obligation,
and which of those v0.41 already proves today. Where a probe from `DESIGN.md` 6.1 is
the same arithmetic at v0.41 scale it is named.

### 3.1 Removal from the middle, clearing, and truncation

**The transposition and `take_at` are written out in `DESIGN.md` 3.L.2**, with the
requirement stated correctly at `at + 2_u64 <= len(vector)` and the last position
handled by a dominating branch. That is the L18 removal of `seq_exchange` priced
against a program that compiles, which the sixth draft's `at + 1_u64` was not.
**There is no `swap` function here and no swap operation anywhere**: a swap of two
whole non-overlapping places is `set (p, q) = move q, move p;` under the one commit
rule, and a swap of two elements of one run is refused by that rule's non-overlap
condition and is the three statements 3.L.2 walks.

Clearing is a drain, and it is where the element class shows. For a non-linear
element:
```wf-design
fn clear_bytes<const n: u64>(vector: own FixedVector<u8, n>) -> result: own FixedVector<u8, n>
    reads(vector), writes(vector) contract {
  ensures len(result) <= 0_u64;
  ensures cap(result) == cap(vector);
  ensures room(result) == cap(vector);
  ensures head(result) == head(vector);
} {
  doc "Removes every element from the end.";
  let count = len(vector);
  let origin = head(vector);
  for @drain (
    at in 0_u64..count,
    invariant left: len(vector) + at >= count,
    invariant gone: len(vector) + at <= count,
    invariant still_lo: head(vector) >= origin,
    invariant still_hi: head(vector) <= origin
  ) {
    set (vector, dropped) = seq_take(vector: move vector);
  }
  invariant done: len(vector) <= 0_u64;
  return move vector;
}
```

**Proof route.** `left` and `gone` have base `count + 0` against `count`; `still_lo` and
`still_hi` have base `origin` against `origin`, `origin` being the [ENT-3.S6] equality
`let origin = head(vector);` establishes over the live term. `seq_take`'s
`len(vector) > 0` discharges from `left` and `at < count`. On the backedge `len` falls by
one as `at` rises by one, each from `seq_take`'s own published
`len(rest) = len(vector) - 1`, and the two `still_*` invariants are preserved by its
published `head(rest) = head(vector)`. `done` is the [INV-1] exact-exhaustion conclusion
at the continuation — probes `x1c` and `x1d` are that shape accepted today. **The two
`head` invariants are round 7's addition**: [ENT-5] 2942-2946 removes every fact whose
support the body writes at the backedge, so without them `head(result) == head(vector)`
has no premise and [CALL-7] is undischarged for `head`; they cost two invariants because
[INV-1] 3105 admits the four ordered relations and not `==` (`DESIGN.md` Q14). `room` and
`cap` follow from [MSR-2]'s identity and need no invariant. `dropped` is the second target
of a [LIV-2] `set`; it names no binding in scope, so it declares one, scoped to the
enclosing block, and it is a `u8` that goes out of scope each iteration.

For a **linear** element type the same loop must do something with `dropped`, and the
signature says so: it carries the store's provider and its `writes` row, which is
[PROV-6]'s virality made visible exactly where it should be — and under D3 it is also
what makes the release *derived* in that body rather than written. The library therefore
has two functions with two signatures rather than one function with a condition on its
element type; [S32] is the relief.

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

**One thing a ring gives up**, and it is [VIEW-2]'s
`requires head(vector) + len(vector) <= cap(vector)`: a `slice` is one contiguous
range and a wrapped window is two, so a run whose window has wrapped cannot be viewed
until it is returned to its origin. **`head` is an absorbing state** — no kernel row
republishes `head = 0` — so a ring that has served one front removal cannot be viewed
again until it is rebased. Two things carry it: the non-wrap premise, which an **empty**
run satisfies from the standing `head <= cap` alone; and **`DESIGN.md` 3.L.8's `rebase`,
a library function and not a kernel row** — a drain of the wrapped run front-to-back into
a fresh run of the same capacity, under the same `flat` invariant every construction loop
carries. [S29] proposed a kernel `seq_rebase` and is **withdrawn**, because L18 asks
whether a writer can express the effect and this program is the answer. **What it costs
is stated rather than hidden**: two runs of `n` slots live across the drain, so `E`
carries `2n` where a kernel rotate would carry `n`, plus the same O(len) copy the rotate
would have performed and a fresh spare per rebase. `DESIGN.md` Q18 puts the row back to
the owner if a real driver's `E` cannot afford it, and records the deeper point that a
real ring driver does not rotate at all — it hands the host two `iovec`s over the two
halves, and this language has no spelling for a view of two ranges.

### 3.3 The growable vector and its growth policy

**`Bytes`, `Grown`, `bs_new` and `bs_reserve` are declared and walked in `DESIGN.md`
3.L.5**, because a worked program may not call a function only a companion declares
and `DESIGN.md` 4.2 calls all four. This section carries only what that one does not.

**What the window bought, and it is the item that decides whether four boundary
operations are enough.** A growth policy must relocate a run's contents **in order**.
Draining from the **front** and appending at the **back** copies the run in order, so
there is no reversal to undo. The fifth draft's prefix drained from the end, so the
new run came out backwards and a second `@flip` loop had to put it back — a loop round
5 showed carrying two undischargeable [OP-4] obligations, because
`let mirror = count -wrap 1_u64 -wrap at;` is two operations in one expression (probe
`t13`), the wrapping form hands the checker a fresh atom with no ordering
(`docs/patterns.md` P8), and `half <= count` from `count / 2_u64` is a fact no premise
of [MSR-4] produces. All of that is gone rather than repaired.

**And the `+checked` route is gone with it.** The fifth draft computed
`count +checked additional` and claimed `want >= count` from the `Ok` arm; [ENT-3.S7]
2791 admits that only for a **constant** addend and probes `g1` and `g2` are the two
halves. 3.L.5 takes the **total** capacity as its parameter and states
`requires total >= len(s.v)`, so the caller does the addition where it has the facts
to prove it and no widening of S7 is proposed.

**Three things round 6 found and 3.L.5 repairs**, recorded here because a reader of
the sixth draft's version of this section will look for them: `bs_new` declared no
contract at all, so `bs_reserve`'s `requires` was undischarged at its only call site;
`bs_reserve` published a bound on a separate `u64` payload field that nothing related
to `room(ready.v)`; and its tail rested on a plain `replace`, whose commit [SET-2] 528
says establishes no fact and whose moved-from run's measures were already dead. The
repair for the third is the general sentence `DESIGN.md` 3.L.0 now carries: **a value
whose measures must survive is constructed into its owner, not replaced into it.**

`bs_shrink` is the same function with `total < count`, `requires total <= len(s.v)`
and the drain bounded by `total`. Its `dispose old;` then releases a run still holding
`count - total` elements, and that is **correct**: [PROV-6]'s walk visits a container's
elements before its backing, so the statement needs no emptiness premise. A writer
reading "drain then dispose" will assume otherwise, which is why it is written down. It is
also the one `dispose` the library writes: under D3 every other release in `byte_string.wf`
is compiler-derived, because every scope holding a `Bytes` holds the `Heap` by signature.

### 3.4 The block pool, and where the obligation goes

The fourth draft made a slot pool the third kernel store, with a provider, a lease, a
`PoolSlot`, a `PoolVector`, a `PoolExhausted`, six operation rows and its own seam
section. Under the minimality ruling it is a **run of runs**: the storage comes from
an arena once, and the recycling is ordinary value movement over the outer run.
**`Lease`, `BlockPool`, `pool_new`, `pool_take` and `pool_release` are declared and
walked in `DESIGN.md` 3.L.4**, because `DESIGN.md` 4.1 calls all of them.

**Where the linear obligation goes is the modelling decision**, and getting it wrong
is instructive. The obligation belongs on the value that is *handed out*, not on the
container of spares: `Lease` is `linear` by declaration, and `BlockPool`'s free list
holds bare runs, which are arena-backed and therefore affine, so the pool itself is an
ordinary value with an ordinary derived release. Had the free list held leases, the
pool would have been linear by ownership and an **empty** one would still have had no
route out — a run of a declaration-linear element type is linear whatever its length,
and neither `dispose` nor a destructuring consume reaches a run. `DESIGN.md` Q13
records that shape and §2.1's release row marks the notion open at it; this file
avoids it.

**And `pool_new`'s refusal arm is legal, which is round 7's repair to [PROV-6]'s
declaration obligation.** On that arm the partly-filled free list is live, holding up to
seven arena-backed runs; it is not moved out and not destructured, and the obligation
admits it because `'s`'s store class is fixed by the declaration's own `Arena<'s, ...>`
parameter, so every `Vector<'s, u8>` is affine and takes the ordinary derived release. An
earlier draft's version, quantified over "a value whose type names `'s`", refused this
function and every function with a `slice` parameter.

**And the release is the PROVED spelling, which is round 6's repair.** A checked
`pool_release` returns the lease inside an `Option` on refusal; `Lease` is linear so
the arm is mandatory, and the only thing the arm can do is destructure the lease and
drop the run — a legal consume that empties the pool in eight iterations with the same
observable the fifth draft's silent leak had. `linear` makes the discard **visible**,
which is what it buys, and not impossible. The proved form takes
`requires room(pool.free) > 0_u64`, which a caller discharges from `pool_take`'s own
published `when leased is Some(value: got): room(rest.free) >= 1_u64`; there is then no
refusal arm and the lease has one route on every path.

**Which regions survive, and why.** `'s` is written at every one of its occurrences
because it **relates** positions (3.K.0): the arena's type to the pool's, the pool's
to its field's, the pool's to the lease it hands out, and the lease's to its run.
Every loan region here relates nothing and is elided, and every call site elides `'s`
too because an operand determines it.

**What the pool does not publish, and what that costs.** `pool_take` cannot state
`room(got.run) >= 256_u64`, because a `Vector<'s, u8>` carries its capacity as a
measure and not in its type, so putting one into a `FixedVector` element and taking it
out loses the figure `pool_new` established. A caller that needs room reads it and
branches, once per lease. That is the honest price of the pool being library data
rather than a kernel store, and `DESIGN.md` 4.1 pays it in the open.

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

**One price, stated rather than discovered** [PROV-6]: when `T` is linear IN THIS SCOPE
the displaced `Option<T>` is too, so every `occupy` owes the caller a `match` on an arm
the writer can see is dead. Under D3 that is narrower than it was: a scope holding the
capability gets the derived release and the arm is only the writer's own bookkeeping,
while a scope without it must move the value out. It stays exactly as written for a `T`
linear by the **modifier**, and the pattern owed to `docs/patterns.md` should say so.
### 3.6 The convenience forms

The fourth draft's `update` statement and its `try` inventory rows are here, and none
of them needed a kernel rule.

```text
| fourth draft                            | this draft                                        |
|-----------------------------------------|---------------------------------------------------|
| update p by op(args);                   | set p = op(vector: move p, args);                 |
| update p by op(args) into x;            | set (p, x) = op(vector: move p, args);            |
| a swap of two whole places              | set (p, q) = move q, move p;                      |
| a swap of two elements of one run       | refused by [LIV-2]'s non-overlap condition;       |
|                                         |   `DESIGN.md` 3.L.2's three statements            |
| seq_try_place(vector, value)            | a library fn: branch on room, place or hand back  |
| seq_try_take(vector)                    | a library fn: branch on len, take or None         |
| seq_try_push(view, value)               | the same, value in and value out                  |
| seq_clear, seq_truncate                 | §3.1                                              |
| seq_take_at                             | `DESIGN.md` 3.L.2                                 |
| seq_exchange                            | `DESIGN.md` 3.L.2, in three statements            |
| seq_filled, seq_vacant                  | `DESIGN.md` 3.L.3                                 |
| seq_reserve_heap, seq_reserve_arena     | §3.3                                              |
| seq_shrink                              | §3.3, with total < count                          |
| seq_lease, seq_lease_proved             | §3.4 and `DESIGN.md` 3.L.4                        |
| FixedRing and its four rows             | nothing: a ring is a run [BLK-1]                  |
| seq_push, seq_pop, absorb               | `DESIGN.md` 3.L.4, value in and value out         |
```

The `try` rows are the interesting entry, because they are where "a convenience is
not a rule" is least obvious. Each is a branch on a measure and two returns, and
**both are declared and walked in `DESIGN.md` 3.L.4** because `DESIGN.md` 4.1 calls
them. Each rests on the [ENT-3.S6] generalization over the four measures, each is
written per element class where the body moves a `T` (probes `x14`, `x15`), and each
publishes a **two-sided** hand-back contract rather than an exact one, because a branch
joins two arms — which is [CALL-7]'s "a two-sided bound where the body establishes no
exact value" doing what it is for.

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

**A value that is linear *here* and reaches a scope exit.** Under D3 this fires only in
a scope that does **not** hold the capability, and its name is
`[PROV-6] LinearValueNotConsumed`. It names the binding, the leaf whose release needs a
capability, the store that leaf's brand names, and the fact that no binding of that
store's provider type is live in this scope — and its mechanical fix offers three routes:
move the value out, destructure it, **or take `heap: &uniq Heap` as a parameter of this
function, which makes the release compiler-derived on every leaving edge**. The third is
D3's own repair and it is why a hosted program carries no `dispose` ceremony: in
`byte_string.wf` a scope-blind criterion needed forty written statements in `main` alone,
and under D3 it needs none. The same diagnostic covers a value that is linear by the
**modifier**, which is linear in every scope; there it says so, and offers only the two
routes that remain.

**A `dispose` with no provider in scope.** `[PROV-6] DisposeHasNoProvider` fires when no
binding of the resolved store's provider type is live, and its mechanical fix names the
parameter the function needs — because the capability a `dispose` spends is determined by
the brand and is therefore never written. `dispose` survives D3 as the **early** release
a writer chooses: `DESIGN.md` 3.L.5's `bs_reserve` is the one place in the library that
writes it, and it is the difference between a peak of one run and a peak of two.

**A linear value taken apart.** `let page = move chunk.page;` on a value linear in this
scope reports `[PROV-6] LinearValuePartiallyConsumed`, names the residual leaf, and
points at `let Chunk(page: page, spare: spare) = move chunk;`. Probes `x4`, `g7` and
`p6_partial` are the program accepted today, and the third shows the residual freed by a
derived drop. The refusal is stated over the **consume**, so it reaches
`dispose chunk.page;` as well — **but it does not reach a [LIV-2] commit**: a consume of
a sub-place reinitialised at the same statement's commit is not a partial consume, which
is what admits `set (kept.v, total) = collect(...)` and `bs_reserve`'s drain.

**A confined type with no store.** `[BLK-4] ConfinedTypeWithoutStore` fires when a
nominal with no region parameter holds a store-backed value and the entry selects no
`command.heap`, so the elided brand resolves to an entry heap that does not exist. Its
fix is to give the nominal a region parameter or to give the program a heap.

**Two runs, one function.** Two stores in scope means both brands are written at every
position that names one, which is where the distinction is real; one store means none is
written anywhere, because [PROV-1]'s brand resolution sends an elided store brand at a
parameter or result position of `Heap` or heap-derived type to the entry heap. That
clause is what makes every hosted helper — `bs_reserve` included — declarable at all.
## 5. Evidence

Every probe cited here is in `DESIGN.md` 6.1 with its verdict. The five this file
rests on most: `x1c` and `x1d`, the two-invariant construction loop whose exit
ordering discharges a subscript with no equality anywhere, **accepted**; `g4` against
`g3`, the same three-term header invariant with and without one published relation,
**accepted then rejected**, which is why [BLK-0] requires every measure on every exit;
`x7`, the vacant table with two element-position `replace`s and a surviving `len`,
**accepted**; `r2`, a `set` at a live affine local, **rejected** by [STOR-1] with a
mechanical fix naming the field-by-field fold, which is the ceremony the one commit
rule removes; and `t1` against `t4`, a const generic and a named const in the same
three positions, **rejected then accepted**, which is why [MSR-6] is one of the nine.
