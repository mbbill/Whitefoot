v0 surface statements are exactly: `fn`, `let`, `match`, `region`, `set`, `check`, `doc`, `return`, `move` (plus `give` in a `let`-initializer `match` and `try` for `Result` propagation). Iteration is spelled per C3 (a protocol op with a conformer, or self-recursion). `loop`/`break` exist in the kernel grammar (GRAM-4) but are held out of the blessed catalog surface (R3-provisional); type aliases, line comments, block-expression `match`, and backslash continuations are not v0 at all, and a program using those is rejected. [M5-FIX-6] [M5R2-FIX-2]

## Card set (D18 decision 5, option 2 — PROVISIONAL pending round-4 data) [M5R3-PART2]

Eight cards are always loaded with full worked examples; the other seven are
demoted to stub entries (one-line promise + pointer), their full text held as
non-normative files under `examples/`. The 8/7 split is PROVISIONAL and is
re-decided on round-4 data.

Always loaded (KEEP, full examples):
- C3 iteration/conform — written below.
- C1 LRU bounded cache — written below.
- C2 FIFO/ring — written below.
- C4 sharded-mutex map — written below (structure only; concurrency ops unspellable in v0, flagged).
- C5 COW-republish — written below (pattern + measured constants; clone/publish unspellable in v0, flagged).
- C6 durability-ordering — written below.
- C7 tiny-map — written below.
- C8 validated-view — written below (validation + byte reads; at-par typed load unspellable in v0, flagged).

Demoted (STUB here; non-normative full text in `examples/`): interner, CSR
graph, pool-trees, intrusive links, timer wheel, prefetch-probe,
sort-by-key-permute — see the stub section at the end of this file.

## C1. Bounded cache with eviction (LRU/CLOCK) — sealed `pool<T>` + `table<K, hdl<T>>` + intrusive handle links

Problem: a bounded map with O(1) get, insert, and eviction — the canonical
COMPOSITION scenario (SCENARIO-DEMAND-MAP §9). Nodes need recency links; a
cache MUST recycle slots, and P2's append-only pool forbids exactly that
(STOR-1: recycled bare indices are well-typed UAF).
Pattern: one sealed `pool<Entry, links(prev, next)>` holds the nodes; one
sealed `table<u64, hdl<Entry>>` maps key to handle; recency is a circular
doubly-linked intrusive list threaded through two `hdl`-typed node fields and
a permanently-live sentinel handle minted at pool construction. Promote is
one splice row; evict is tail-read, take (which splices out first), table
remove. `hdl<T>` is the pool's u32-wide generational handle (index bits plus
generation bits under the pool's stated wrap contract): a copy VALUE, never a
borrow — it freezes nothing, and its safety is the generation witness, not
the freeze judgment. The `links(prev, next)` parameter is sealed-form
parameterization naming the two link fields of `Entry`; `pool_insert`
self-links a fresh slot, so unlink is total.

Node and the two structures:

```
struct Entry {
  key: u64;
  val: u64;
  prev: hdl<Entry>;
  next: hdl<Entry>;
}
```

The state owner holds `index`, `nodes`, `sent`, and `cap` as fields of its
own state struct and passes DISJOINT FIELD BORROWS down (P1/P4 posture): v0
has no reborrowing (T-A), so these helpers are written over the parts, never
over a borrowed cache-wrapper struct. Two field borrows of one owned struct
are disjoint places; nothing is relaxed.

Op rows this card leans on (sealed-form op tables; positional operands):

| row | signature | loan | effects | failure | facts -> / kills | codegen |
|---|---|---|---|---|---|---|
| `table_find<K, H>` | `(t: &'t table<K, H>, k: K) -> own Option<H>` | NONE | `reads('t)` | none — absence is a value | none | HOT: guaranteed-inline; one hash, one 16B group load, one key compare |
| `table_insert<K, H>` | `(t: &uniq 't table<K, H>, k: K, v: H) -> own Option<H>` (displaced value out) | NONE | `writes('t), allocates(heap), traps` | growth exhaustion traps (OP-9 / `buffer_new` precedent) | kills `present(t, *)` | — |
| `table_remove<K, H>` | `(t: &uniq 't table<K, H>, k: K) -> own Option<H>` | NONE | `writes('t)` | none | kills `present(t, *)` | — |
| `pool_insert<T>` | `(p: &uniq 'c pool<T>, v: own T) -> own hdl<T>` (own-in; self-links the slot) | NONE | `writes('c), allocates(heap), traps` | growth exhaustion traps | issues `live(p, h)` | — |
| `pool_take<T>` | `(p: &uniq 'c pool<T>, h: hdl<T>) -> own T` (splices out, vacates, bumps generation) | NONE | `writes('c), traps` | stale handle: deterministic trap | kills `live(p, *)` | — |
| `pool_entry<T>` | `('v, p: &uniq 'c pool<T>, h: hdl<T>) -> &'v T` | ISSUE entry loan; p frozen until `'v` ends | `reads('c), traps` | stale handle: deterministic trap | none; a proven `live(p, h)` may elide the generation check (proof-elision only, P8) | HOT: guaranteed-inline; one generation compare + one indexed load |
| `pool_entry_uniq<T>` | `('v, p: &uniq 'c pool<T>, h: hdl<T>) -> &uniq 'v T` | ISSUE entry loan (unique); p frozen until `'v` ends | `reads('c), traps` | stale handle: deterministic trap | kills value facts on the slot | — |
| `pool_move_front<T>` | `(p: &uniq 'c pool<T>, h: hdl<T>, s: hdl<T>) -> own unit` (total for live h, including h already front) | NONE | `writes('c), traps` | stale handle: deterministic trap | kills NOTHING — links only, no vacancy change; liveness facts survive promote | HOT: guaranteed-inline; exact lowering pinned at <=6 u32 stores, zero branches |
| `pool_link_prev<T>` | `(p: &uniq 'c pool<T>, h: hdl<T>) -> own hdl<T>` | NONE | `reads('c), traps` | stale handle: deterministic trap | none | — |
| `pool_len<T>` | `(p: &uniq 'c pool<T>) -> own u64` | NONE | `reads('c)` | none | none | — |
| `hdl_null<T>` | `() -> own hdl<T>` (never live; construction filler only) | NONE | `pure` | none | none | — |

Get with promote (hit path: one probe, one splice, one scoped read loan):

```
fn lru_get['i, 'c](idx: &'i table<u64, hdl<Entry>>, np: &uniq 'c pool<Entry, links(prev, next)>, sent: own hdl<Entry>, key: own u64) -> own Option<u64> reads('i), writes('c), traps {
  doc "Splice first, read second: the entry loan freezes the pool, so all link edits precede it.";
  let found: own Option<hdl<Entry>> = table_find<u64, hdl<Entry>>(idx, key);
  match found {
    None() => {
      return None();
    }
    Some(value: h) => {
      pool_move_front<Entry>(np, h, sent);
      region 'v {
        let e: &'v Entry = pool_entry<Entry, 'v>(np, h);
        let v: own u64 = deref(e).val;
        return Some(value: v);
      }
    }
  }
}
```

Insert with evict (fresh-key path; present keys take `lru_get` plus one
store through `pool_entry_uniq` instead):

```
fn lru_insert['i, 'c](idx: &uniq 'i table<u64, hdl<Entry>>, np: &uniq 'c pool<Entry, links(prev, next)>, sent: own hdl<Entry>, cap: own u64, key: own u64, val: own u64) -> own unit writes('i, 'c), allocates(heap), traps requires {
  check ige<u64>(cap, 1_u64) else trap "lru: zero capacity";
} {
  doc "At capacity the recency tail is evicted before the fresh node lands; occupancy never exceeds cap.";
  let used: own u64 = pool_len<Entry>(np);
  let full: own Bool = ige<u64>(used, cap);
  match full {
    True() => {
      let victim: own hdl<Entry> = pool_link_prev<Entry>(np, sent);
      let dead: own Entry = pool_take<Entry>(np, victim);
      let gone: own Option<hdl<Entry>> = table_remove<u64, hdl<Entry>>(idx, dead.key);
      match gone {
        None() => {
          check False() else trap "lru: victim key absent from index";
        }
        Some(value: prior) => {
        }
      }
    }
    False() => {
    }
  }
  let hnull: own hdl<Entry> = hdl_null<Entry>();
  let node: own Entry = Entry(key: key, val: val, prev: hnull, next: hnull);
  let h: own hdl<Entry> = pool_insert<Entry>(np, move node);
  pool_move_front<Entry>(np, h, sent);
  let displaced: own Option<hdl<Entry>> = table_insert<u64, hdl<Entry>>(idx, key, h);
  match displaced {
    None() => {
    }
    Some(value: dup) => {
      check False() else trap "lru: fresh-key contract violated";
    }
  }
  return unit;
}
```

The `requires` prologue closes the victim-is-sentinel edge structurally
(cap >= 1 and full implies a nonempty list), and its passed check feeds the
fact channel like any OP-5 fact. The construction filler `hnull` is inert:
`pool_insert` self-links the slot before it is reachable.

Performance contract: promote is one splice row — at most six u32 stores
(two splice-out, four splice-in), zero branches; the two node-local stores
land in the cache line the key compare already fetched, so off-node traffic
is the 2-4 stores the demand map prices. Links are u32 indices into the same
slab: no per-node allocation, no headers, no pointer-chase beyond lines the
probe touched. Evict is O(1): one link read, one splice-out-and-vacate, one
probe-and-remove. Par target: the hashbrown-backed `lru` crate 0.12,
get-with-promote ~20-30 ns at 100K entries. The composition test is
normative: this card reaching lru-crate parity is what keeps a dedicated
sealed cache form out of the catalog; missing it is a catalog FINDING (gap),
escalated through D16, not a writer error. Generation checks are retained by
default on every pool row; eliding one requires a proven `live(p, h)` fact —
proof-elision only, never a weakened check (P8).

Loan/freeze choreography, exactly: handles are copy values — holding any
number of `hdl<Entry>` freezes NOTHING, ever. Rows with loan NONE
(`table_find`, `pool_insert`, `pool_take`, `pool_move_front`,
`pool_link_prev`, `pool_len`, `table_insert`, `table_remove`) use their
container operand for the call only; `np` and `idx` are live again at the
next statement. `pool_entry`/`pool_entry_uniq` ISSUE an entry loan: while
`e: &'v Entry` is live, `np` is FROZEN — every pool row naming `np` is
rejected statically, citing the loan judgment — and the loan dies at region
`'v` exit; the example scopes it with an explicit `region 'v` so the freeze
window is visible in the source. Loans are per-binding: a pool entry loan
never freezes the table, and vice versa. The ordering rule the card teaches:
SPLICE FIRST, READ SECOND — all link edits happen before the read loan is
issued or after it dies.

Region and borrow discipline (normative, not merely the example choreography
above) [M5-FIX-3]: (a) every region name must be bound by a `fn` gparam bracket
or an enclosing `region 'x { }` block — there are no free or placeholder region
names; (b) v0 has no reborrowing and no uniq-to-shared coercion — a row spelled
`&'r` requires a shared binding, so a `&uniq` receiver must use the row's
`_uniq` variant, not the shared row; (c) only rows with a dedicated
result-region parameter issue loans — a single-region row such as
`tbl_get_uniq` returns its loan at the receiver's region, never a fresh one.

Calling a card fn [M5R3-FIX-1]. A user `fn` is called with NAMED arguments in
declared order and all region/type arguments explicit ([GRAM-11]/[TYPE-5]) —
unlike a table op, which is positional:

```
let hit: own Option<u64> = lru_get<'i, 'c>(idx: &'i index, np: &uniq 'c nodes, sent: sentinel, key: k);
```

`sent` (a copy `hdl`) and `key` (a copy `u64`) are bare; `idx`/`np` are borrow
atoms naming the regions passed in `<'i, 'c>`. Contrast the positional table op
`pool_len<Entry>(np)`.

Region-mint discipline (OWN-10/OWN-11) [M5R3-FIX-5]. A borrow `&'a p` of a place
rooted at an OWN binding — a local or an own parameter alike — requires `'a` to
be introduced WITHIN that binding's scope, never a caller-supplied region
(OWN-10). The default idiom is the local probe window `region 'e { let r: &'e T
= op(...); ...use r...; }`, exactly the `region 'v { }` scopes in this card.
Inside a `loop @l` (kernel-legal but held out of the blessed surface, see
M5-FIX-6), a `borrow_expr` may name only regions introduced inside `@l`'s body,
and outside bindings may not be moved in (copies exempt) — OWN-11:

```
loop @scan {
  region 'e {
    let e: &'e Entry = pool_entry<'e, Entry>(np, h);
    let seen: own u64 = deref(e).val;
  }
  break @scan;
}
```

The `region 'e { }` sits INSIDE the loop body, so the probe loan is fresh each
iteration and dead before the next; minting it around the loop (spanning all
iterations) is the OWN-11 rejection this rule prevents.

Borrow-minting, worked [M5R2-FIX-3]. A row spelled `&'r` or `&uniq 'r` never
accepts a bare owned binding (that is the OWN-1 hard error writers keep hitting):
mint the `borrow_expr` atom `&'r p` / `&uniq 'r p` at the call site, binding the
region with an enclosing `region 'r { }` when the value is a local. Borrows are
atoms (GRAM-9), so they need no `let` and no `move`:

```
fn demo_mint(v: own u32) -> own unit allocates(heap), traps {
  doc "Mint the borrow inline as the argument atom; the owned local is frozen only for the call.";
  region 'x {
    let buf: own seq<u32, 0> = seq_new<u32, 0>();
    seq_push(&uniq 'x buf, move v);
    let n: own u64 = seq_len(&'x buf);
    return unit;
  }
}
```

The `&uniq 'x buf` argument is the uniq-mode mint; `&'x buf` is the shared (`&`)
mode variant. If instead you bind the borrow (`let l: &uniq 'x seq<u32, 0> =
&uniq 'x buf;`), `l` is affine and must be passed with `move l` — the inline
atom above avoids that and is the blessed spelling.

Failure handling under the single failure principle: absence is a value
(`Option`), never a failure. Environmental failure on this card is
allocation growth exhaustion, which traps per the `buffer_new`/OP-9
precedent. Programmer error traps deterministically: a stale handle at any
pool row (generation mismatch), and cross-structure disagreement (the two
explicit `check ... else trap` arms — the table and pool disagreeing is a
broken invariant, not a recoverable outcome).

Misuse the checker rejects, and the trap that backstops the rest:

```
region 'v {
  let e: &'v Entry = pool_entry<Entry, 'v>(np, h);
  pool_move_front<Entry>(np, h, sent);
  let v: own u64 = deref(e).val;
}
```

The `pool_move_front` line is REJECTED at compile time: `np` is frozen by
the live entry loan `e` (loan judgment; the diagnostic names the issuing row
and the freezing binding). Also rejected statically: storing an entry loan
into a struct field (loans are confined, stack-only — structs store values,
not borrows), and minting an `hdl<Entry>` from an integer (no such row; the
handle type is opaque and nonforgeable — only `pool_insert` issues live
ones). The residual dynamic class is the stale handle: hold `h`, let another
path evict it, then `pool_entry<Entry, 'v>(np, h)` — the slot may already be
recycled, but the generation differs, so the row TRAPS deterministically,
citing itself. It is never a well-typed read of a recycled slot: this card
is the blessed recycler precisely because generations turn the STOR-1 UAF
class into a checked fact, which append-only P2 cannot offer a cache.

CLOCK variant (drop the promote write): same composition plus a
`seen: Bool` field on `Entry`. Get performs no splice at all — one store
through `pool_entry_uniq` into the entry's own line (`set deref(e).seen =
True();`), eliminating the recency-list write entirely; evict holds a clock
hand as a stored copy handle and walks `pool_link_prev` from it, clearing
`seen` flags until a clear entry is found (second-chance rotation, bounded
by occupancy). Choose CLOCK when the promote store is the measured
bottleneck; the link discipline is the only thing that changes.

Replaces: hashbrown-plus-`Box` LRUs with pointer links (per-node allocation,
pointer-chase on every promote), hand-rolled Vec-index caches with free
lists (unchecked recycling — the exact well-typed-UAF shape STOR-1 rejects),
and `Rc<RefCell>` node webs (unrepresentable BY DESIGN).

## C2. FIFO/ring — the one blessed queue spelling

Problem: FIFO order with O(1) push and pop — worklists, BFS frontiers,
sliding windows, token buckets. The catalog blesses exactly ONE spelling;
the former two-spellings state (separate ring and two-stack cards) invited
band shopping and is dead.
Pattern (primary — copy payloads): a masked ring over a power-of-two
`buffer<T>` with MONOTONE u64 head/tail counters. `head` counts pops, `tail`
counts pushes; occupancy is the wrapping difference; every access index is
`counter & (cap - 1)`. The pow2 fact is minted at construction by an OP-5
check and restated as a `requires` prologue on every op, so no unchecked
state exists. Counters use `.wrap` under P8's structural-bound license —
`tail - head <= cap` always, so wrapping subtraction is exact modular
arithmetic, and the hot loop stays trap-free. The modulo spelling is
rejected twice over: `irem` costs a division in the hot loop, and its
`.trap` rung poisons totality for the whole tower (P8).

```
struct Ring {
  buf: buffer<u64>;
  head: u64;
  tail: u64;
}
```

```
fn ring_new(cap: own u64) -> own Ring allocates(heap), traps {
  doc "Capacity must be a power of two; the mask fact is minted here and re-checked in every op prologue.";
  let pc: own u32 = ipopcount<u64>(cap);
  let ok: own Bool = ieq<u32>(pc, 1_u32);
  check ok else trap "ring: capacity not a power of two";
  let buf: own buffer<u64> = buffer_new<u64>(cap, 0_u64);
  return Ring(buf: move buf, head: 0_u64, tail: 0_u64);
}
```

```
fn ring_push['q](q: &uniq 'q Ring, v: own u64) -> own Bool writes('q), traps requires {
  let qcap: own u64 = len<u64>(deref(q).buf);
  let qbit: own u32 = ipopcount<u64>(qcap);
  check ieq<u32>(qbit, 1_u32) else trap "ring: pow2 mask fact violated";
} {
  doc "Full is an expected outcome and returns a value (P9); it is never a contract trap.";
  let cap: own u64 = len<u64>(deref(q).buf);
  let used: own u64 = isub.wrap<u64>(deref(q).tail, deref(q).head);
  let full: own Bool = ieq<u64>(used, cap);
  match full {
    True() => {
      return False();
    }
    False() => {
      let mask: own u64 = isub.wrap<u64>(cap, 1_u64);
      let slot: own u64 = iand<u64>(deref(q).tail, mask);
      set index<u64>(deref(q).buf, slot) = v;
      let t2: own u64 = iadd.wrap<u64>(deref(q).tail, 1_u64);
      set deref(q).tail = t2;
      return True();
    }
  }
}
```

```
fn ring_pop['q](q: &uniq 'q Ring) -> own Option<u64> writes('q), traps requires {
  let qcap: own u64 = len<u64>(deref(q).buf);
  let qbit: own u32 = ipopcount<u64>(qcap);
  check ieq<u32>(qbit, 1_u32) else trap "ring: pow2 mask fact violated";
} {
  doc "Empty is an expected outcome and returns a value; the masked read mirrors the masked write.";
  let used: own u64 = isub.wrap<u64>(deref(q).tail, deref(q).head);
  let empty: own Bool = ieq<u64>(used, 0_u64);
  match empty {
    True() => {
      return None();
    }
    False() => {
      let cap: own u64 = len<u64>(deref(q).buf);
      let mask: own u64 = isub.wrap<u64>(cap, 1_u64);
      let slot: own u64 = iand<u64>(deref(q).head, mask);
      let v: own u64 = index<u64>(deref(q).buf, slot);
      let h2: own u64 = iadd.wrap<u64>(deref(q).head, 1_u64);
      set deref(q).head = h2;
      return Some(value: v);
    }
  }
}
```

Calling a card fn [M5R3-FIX-1]. `ring_push` is a user `fn`: named arguments in
declared order, region argument explicit — `let ok: own Bool = ring_push<'q>(q:
&uniq 'q ring, v: x);` (`v` is a copy `u64`, bare; `q` is a `&uniq 'q` borrow
atom). The table op inside it, `len<u64>(deref(q).buf)`, is positional.

The fact chain, stated honestly: the prologue's passed check mints
`pow2(len(q.buf))` on the dominated body (OP-5 stated-and-checked); deriving
`iand(x, mask) < len(q.buf)` from it — which is what retires the OP-4 bounds
check at the `index` sites — is fact extension F1 (pow2-mask domination),
which is GATED on its own hostile review plus the preregistered ring
disassembly demo. Until F1 lands, the bounds checks are RETAINED and the
card fails closed: correctness is never conditional on the elision, only
the band is. No row in this card resizes `buf`, so the fact survives the
body; a future grow row would carry an explicit kill column for every
length-derived fact on `q.buf`. Bulk traversal uses the two-slice view, not
per-element pops: unwrapped (`head&m < tail&m`) is the single slice
`[head&m .. tail&m]`; two slices only when wrapped.

Preregistered band (single, reconciled — no band shopping): push/pop cycle
at cap 4096 plus a BFS frontier over a 1M-edge graph, within
**[0.90x, 1.10x] of `VecDeque`**, with zero residual bounds branches in the
optimized loop (F1-dependent; failure fails closed to seal-or-extend, never
to a quiet miss). The affine variant is banded separately: pool-handle ring
vs `VecDeque<Box<T>>` on 64B tasks, same band; if the affine route exceeds
1.25x against an INLINE `VecDeque` on a workload the target set actually
contains, the deferred sealed inline-affine deque is escalated — through its
trigger, not around it.

Affine-T variant (pool-handle queue): values live in a `pool<T>`; the ring
is this same `Ring` shape instantiated at the u32-wide copy handle. Push is
`pool_insert` then `ring_push` of the handle; pop is `ring_pop` then
`pool_take` — ownership of T re-emerges at the boundary, one slab touch per
op is the honest extra cost, and stale-handle misuse traps via the pool's
generation check exactly as in C1. Worst case per op stays O(1).

Fallback (two-seq flip queue): two `seq<T>` stacks — push onto the in-stack;
pop from the out-stack, and when it is empty, flip the in-stack over in one
reversing drain. AMORTIZED-ONLY, documented as such: the flip is O(n) at the
moment it happens. It is the affine spelling of last resort, never for a
latency-bounded loop.

Which variant — three questions, in order (any cross-thread handoff exits
this card entirely: that is the sealed `conc_queue` form, not a composition):

1. Is the payload copy, or does it fit a u32/u64 handle or key? -> primary
   masked ring.
2. Is the payload affine AND does the queue need O(1) worst case per op or
   stable identity while enqueued? -> pool-handle queue.
3. Is the payload affine and amortized bounds are acceptable? -> two-seq
   flip queue, amortized-only.

Failure handling under the single failure principle: full on push and empty
on pop are expected outcomes and return values (`Bool`, `Option` — the P9
rule: an expected runtime outcome is never a contract trap); `buffer_new`
size overflow or exhaustion traps (OP-9); a non-power-of-two capacity is
programmer error and traps deterministically — at `ring_new` when built
honestly, or at the FIRST op's `requires` prologue when a writer
hand-constructs `Ring` around `ring_new`. There is no path on which the mask
fact is silently assumed.

Misuse: hand-constructing `Ring(buf: b, head: 0_u64, tail: 0_u64)` over a
capacity-12 buffer parses and checks — and the first push or pop traps
deterministically, citing the prologue's check string; the fact is checked,
never trusted. Forging the counters (`set deref(q).head = x` from other
code holding the `&uniq`) can corrupt queue CONTENTS but not memory: every
access is masked below the buffer length, so safety never rests on the
counters — only the band does. Rejected statically: pushing an affine T into
`buffer<T>` (copy-only domain; the checker routes you to variant 2), and
sharing `&uniq 'q Ring` across threads (no send/share row on this card;
that demand names `conc_queue`).

Replaces: `VecDeque` in both its roles (Copy ring and boxed-affine queue),
hand-rolled head/tail rings spelled with modulo (a division plus a P8
totality poison in the hottest loop), linked-list queues (per-node
allocation, pointer-chase), and the catalog's own former two-spellings FIFO
state — the two-stack card survives only as the fallback subsection above,
documented amortized-only.

## C3. Iteration — the blessed spellings [M5R2-FIX-2]

Blessed v0 code spells iteration exactly two ways: (1) a protocol op
(`seq_for_each`/`seq_for_each_uniq`/`seq_drain`/`tbl_for_each`/`tbl_retain`/
`tbl_drain`) driving a conformer, or (2) self-recursion (FN-6). There is no
third blessed spelling. (`loop`/`break` exist in the kernel grammar (GRAM-4) but
are held out of the blessed catalog surface pending the R3 loop-form validation;
see the flag noted with M5-FIX-6.) The conformer half is what the earlier
statement-list fix left unwritable — here is the full spelling.

An env struct carries the accumulator; the visitor is a plain `fn` bound to the
contract member by `conform`; the protocol op drives it. Cross-element state
lives in the env (set through a `deref` place), so no loop counter is needed.

```
struct CountEnv { count: u64; }

fn count_visit['v](env: &uniq 'v CountEnv, item: &'v u32) -> own Bool reads('v), writes('v) {
  doc "Per-element visitor: bump the env counter; True() continues, False() stops.";
  let cur: own u64 = deref(env).count;
  set deref(env).count = iadd.wrap<u64>(cur, 1_u64);
  return True();
}

conform CountEnv : SeqVisit<u32> { visit = count_visit; }

fn count_all['r](s: &'r seq<u32, 0>) -> own u64 reads('r) {
  doc "Blessed iteration: the protocol op drives the conformer; no loop statement is written.";
  region 'v {
    let acc: own CountEnv = CountEnv(count: 0_u64);
    seq_for_each<CountEnv>(s, &uniq 'v acc);
    return acc.count;
  }
}
```

The conformer's declared row `reads('v), writes('v)` is contained in `SeqVisit`'s
ceiling `reads('v), writes('v), traps` (subsumption, [FN-7]/[M5R3-FIX-7]) and
equals what the body exhibits (one `deref` read, one `set` write) —
[CAT-5a]/[EFF-2]. `count_all`'s own row is just `reads('r)`: the visitor's `'v`
effects are confined to the internal `region 'v { }` block ([CAT-5a](ii)), and
`seq_for_each`'s `+ join(CountEnv)` uses `count_visit`'s actual row (which does
not trap), not the ceiling, so no `traps` reaches `count_all`.

A `tbl_retain` predicate is the same shape; `keep_big` declares `reads('v)`,
contained in `TblRetain`'s ceiling `reads('v), traps` (the `traps` there lets a
different predicate use a checked `index`; `keep_big` does not, so its row omits
it):

```
struct KeepBig { min: u64; }

fn keep_big['v](env: &uniq 'v KeepBig, key: u64, value: &'v u64) -> own Bool reads('v) {
  doc "Retain predicate: keep entries whose value >= env.min; False() drops the entry.";
  let lo: own u64 = deref(env).min;
  let v: own u64 = deref(value);
  return ige<u64>(v, lo);
}

conform KeepBig : TblRetain<u64, u64> { keep = keep_big; }

fn prune['e](t: &uniq 'e table<u64, u64, fold>, floor: own u64) -> own unit writes('e) {
  doc "Drop small entries in place; tbl_retain drives the KeepBig conformer.";
  region 'v {
    let env: own KeepBig = KeepBig(min: floor);
    tbl_retain<KeepBig>(t, &uniq 'v env);
    return unit;
  }
}
```

Accumulators and `set`. A `set` target is any place [GRAM-5]: a `deref(p).field`
(as above), an `index<T>(...)` slot, or a bare local IDENT. So a straight-line
accumulator on a local is legal — `set acc = iadd.wrap<u64>(acc, x);` where
`acc` is a copy-typed local — even though *cross-element* accumulation lives in
the visitor's env as shown. What a bare local can never do in blessed code is
carry state across iterations by itself, because there is no blessed loop to
iterate it; that role is the env field (protocol-op path) or a recursion
parameter (self-recursion path).

Calling these fns [M5R3-FIX-1]: `count_all` and `prune` are user fns —
`let n: own u64 = count_all<'r>(s: &'r data);` and `prune<'e>(t: &uniq 'e cache, floor: 8_u64);`
(named args, explicit region args, positional table ops only inside the bodies).

User-fn call with explicit type AND region args [M5R3-FIX-4]. TYPE-5 requires
every type/region/const argument at a user-fn call site (the CAT-1a suppression
is table-ops-only). Given `fn nth<T>['r](s: &'r seq<T, 0>, i: own u64) -> own T reads('r), traps`:

```
let x: own u32 = nth<u32, 'r>(s: &'r data, i: 3_u64);
```

The type arg `u32` and region arg `'r` share one `<...>` list in
generics-then-regions order (the `fn` declaration order `<generics>['regions]`);
value arguments are named; `i` (a copy `u64`) is bare.

## C4. Sharded-mutex map — concurrent keyed state [CARD-1]

Problem: get/insert/remove from many threads with no writer starvation; per-key
atomic get-or-insert; mixed read/write. Par target: dashmap 6 (64 shards) ~20-40
ns uncontended, ~50-100M mixed ops/s across 16 threads; the `RwLock<HashMap>`
read plateau is the fail line (demand-map scenarios 11/42).

Pattern: `N = 2^k` shards, each a sealed `table<K, V, h>` behind its own mutex;
the shard is `hash(key) & (N-1)`, so lock and data contention are confined to
1/N of traffic. This is composition (an array of locked blessed maps + a
hash-based shard pick), not a new primitive.

> **UNSPELLABLE IN v0 [CARD-1 FLAG].** The concurrency half of this card has no
> v0 spelling and is NOT invented here. Kernel CAP-1: "v0 defines no thread
> construct"; `Shareable`/`Sendable` are reserved vocabulary with nothing to
> bind. There is no mutex, lock, guard, or cross-thread share form. What IS
> v0-expressible: the per-shard `table<K, V, h>` and the shard pick
> (`iand<u64>(h, mask)` over a power-of-two shard count, the C2 mask idiom).
> What is NOT: acquiring a shard lock, sharing the shard array across threads,
> and the atomic get-or-insert under the lock. This card becomes a full worked
> card when the concurrency layer lands (a `mutex`/`guard` sealed form whose
> guard is a confined loan token — exactly the M1 loan-judgment shape — plus
> `Shareable` binding for the shard array). Until then it is a structural
> placeholder; the single-threaded per-shard `table` operations are the C-table
> rows in S.2.

Performance note: sharding divides contention linearly; a 16-shard/8-thread
mixed workload takes an uncontended shard lock (~15 ns) plus one SwissTable
probe. The read-mostly regime where readers must write nothing is a different
shape — see C5 (COW-republish).

## C5. COW-republish read-mostly map [CARD-2]

Problem: read-mostly keyed state read millions of times/second where the read
path must write NO shared cache line (else read scaling dies); occasional whole
-table update. Par target: arc-swap snapshot load ~2 ns/read, linear read scaling
(demand-map scenario 42, read-mostly arc).

Pattern: hold an immutable `table<K, V, h>` behind one shared cell; a reader
loads the current snapshot (a pure read, zero shared writes); a writer CLONES
the whole table, mutates the clone, and atomically republishes it (copy-on-write
republish); the old snapshot is reclaimed when its last reader releases it.

Clone-is-publish constant (measured, `evidence/microbench/RESULTS.md`): the
whole-table clone IS the publish cost. POD / handle-valued tables clone at ~1-2
ns/entry — ~1-2 ms per 1,000,000 entries — so COW republish is viable at modest
update rates. KB-value tables clone at ~360-3060 ns/entry — ~3 s per 1,000,000
entries — so republish is non-viable except at very low update rates; that
regime belongs to C4 (sharded-mutex) or a deferred in-place concurrent table.
The falsifier MUST pick its constant by the target scenario's value type; the
old ~50-100 ns/entry blanket figure is wrong in both directions (25-100x high
for POD, 3-30x low for KB values).

> **UNSPELLABLE IN v0 [CARD-2 FLAG].** Two gaps, neither invented here. (1) The
> atomic-pointer publish and cross-thread reader sharing are concurrency — CAP-1
> gives no thread construct and no atomic cell. (2) There is NO `tbl_clone` op:
> the S.2 `table` form declares no whole-table clone row, so even the
> single-threaded clone-modify step has no v0 spelling. This card is therefore a
> pattern + measured viability rule; it becomes writable when the table form
> gains a `tbl_clone` row (its cost the constants above) and the concurrency
> layer provides a shareable atomic snapshot cell. Do not spell either today.

## C6. Durability-ordering — WAL fsync discipline [CARD-3]

Problem: "these bytes are on stable storage before that state becomes valid."
Log append then ONE `fdatasync` per commit batch; checkpoint = write-new,
`fsync`, `rename`, directory `fsync`; checksummed frames so a torn tail is
detectable; recovery truncates at the first bad checksum. Par target: SQLite
3.45 WAL mode ~20-50k commits/s batched on NVMe vs ~100-250/s naive per-commit
fsync (demand-map scenario 49). SOTA wins by minimizing fsync COUNT (group
commit), not write speed — `fdatasync` is the ~20-100 us wall.

The rows this card leans on are the enumerated io-file rows (`IO-ROW-ENUMERATION.md`),
cited not restated, exactly as C1 cites the pool rows: `pwrite` (row 5,
positional append at an explicit offset), `fdatasync` (row 7), `fsync` (row 8),
`fsync`-on-directory-handle (row 9), `rename` (row 10). Failure is a value
(`Result`, errno class); `try` propagates it.

```
fn wal_commit['h, 'f](log: &uniq 'h io_file, frame: slice<'f, u8>, off: own u64) -> own Result<unit, IoError> reads('f), writes('h) {
  doc "Append the checksummed frame, then one fdatasync makes the commit durable; data is written before the record it validates.";
  let wrote: own u64 = try pwrite(log, frame, off);
  let synced: own unit = try fdatasync(log);
  return Ok(value: unit);
}
```

> **DARWIN DURABILITY PIN [CARD-3 FLAG].** On macOS, plain `fdatasync`/`fsync`
> return before the drive cache is flushed; the durable spelling is
> `fcntl(fd, F_FULLFSYNC)` (`IO-ROW-ENUMERATION.md` rows 7/8/9 and §3(b)). A
> WAL-durability test on the dev machine is void unless `fdatasync`/`fsync`
> lower to `F_FULLFSYNC` on Darwin; `F_BARRIERFSYNC` is ordering-only and does
> NOT satisfy a commit. Also pending: the io-file rows are enumerated but not yet
> a sealed form in S.0-S.3, so `io_file`/`IoError` here are cited like C1's
> `pool` — a pending-catalog handle, not yet an optables row.

Checkpoint (write-new + `fsync` + `rename` + directory `fsync`) publishes a new
generation atomically: the rename is durable only after the directory `fsync`
(F_FULLFSYNC on the dir fd, Darwin). Recovery replays frames via a `pread` loop
to the first checksum mismatch, then truncates there — a torn tail never
corrupts a committed prefix. This is a checked-library shape over the io rows;
sealing buys no mechanism (the `fdatasync` cost dominates by 2-3 orders of
magnitude), which is why durability is a card, not a form.

## C7. Tiny-map — inline linear scan [CARD-4]

Problem: a map whose n is statically tiny (<= ~8-16), built and dropped millions
of times, so construction cost matters as much as lookup and no heap allocation
is tolerated at tiny n. Par target: LLVM `SmallDenseMap<K,V,8>` beats `DenseMap`
~3-4x on the build-4/lookup-4/drop microcycle; the blessed form must beat a plain
SwissTable on "millions of 4-entry maps" by >=2x (demand-map scenario 7).

Pattern: a `seq<Pair, N>` with inline capacity `N` (frame-resident while
unspilled, SEQ-2 — zero heap touch at n <= N); lookup is a linear `for_each`
scan comparing keys; no hashing at tiny n (a quality hash of an 8-byte key costs
about as much as the whole scan). Insert appends a fresh `Pair` (or overwrites on
a key hit). It is nearly a knob on the sequence, not a new container.

```
struct Pair { key: u64; val: u64; }

struct FindEnv { want: u64; found: Bool; val: u64; }

fn find_visit['v](env: &uniq 'v FindEnv, item: &'v Pair) -> own Bool reads('v), writes('v) {
  doc "Stop (False) at the first key match, recording the value into env.";
  let k: own u64 = deref(item).key;
  let hit: own Bool = ieq<u64>(k, deref(env).want);
  match hit {
    True() => {
      set deref(env).val = deref(item).val;
      set deref(env).found = True();
      return False();
    }
    False() => {
      return True();
    }
  }
}

conform FindEnv : SeqVisit<Pair> { visit = find_visit; }

fn tiny_get['r](m: &'r seq<Pair, 8>, want: own u64) -> own Option<u64> reads('r) {
  doc "Linear scan of an inline pair sequence; no hashing, no heap at n <= 8.";
  region 'v {
    let env: own FindEnv = FindEnv(want: want, found: False(), val: 0_u64);
    seq_for_each<FindEnv>(m, &uniq 'v env);
    let ok: own Bool = env.found;
    match ok {
      True() => {
        return Some(value: env.val);
      }
      False() => {
        return None();
      }
    }
  }
}
```

`find_visit` declares `reads('v), writes('v)`, contained in `SeqVisit`'s ceiling
and exactly what it exhibits ([CAT-5a]). `tiny_get`'s own row is `reads('r)`: the
`'v` effects are confined to the `region 'v { }` block, and the join uses
`find_visit`'s non-trapping row. Performance contract: at n <= 8 the scan touches
1-2 cache lines with a perfectly predicted loop and zero allocation, versus a
freshly-allocated cold SwissTable's hash + group load; on the millions-of-tiny-
maps compiler workload this is the >=2x win. Insert is `seq_push(move Pair(...))`
(append) or a scan-then-`seq_set` overwrite; spill past `N` is one-way to heap
(SEQ-2), at which point a SwissTable is the better shape (the knob's threshold).

Replaces: a heap-allocated `HashMap` per tiny scope (a malloc + cold hash per
build) and hand-rolled parallel key/value arrays.

## C8. Validated-view — typed view over borrowed bytes [CARD-5]

Problem: overlay typed, endian-explicit accessors on a borrowed byte buffer after
one length/shape validation, so each subsequent field access is a plain load with
no residual checks; no copy; view lifetime tied to the buffer. Par target:
zerocopy 0.8 field access is codegen-identical to a raw C load (1 load [+1 bswap],
verified by asm diff); demand-map scenario 27.

Pattern: validate the buffer length ONCE, then read fields. Reads use
`len<u8>`/`index<u8>` (both from the kernel op table) over a `buffer<u8>` or
`slice<'r, u8>`.

```
fn header_kind['r](buf: &'r buffer<u8>) -> own Option<u32> reads('r), traps {
  doc "Validate length once, then read the big-endian u16 at offset 0 by byte assembly (v0 has no typed load).";
  let n: own u64 = len<u8>(deref(buf));
  let ok: own Bool = ige<u64>(n, 2_u64);
  match ok {
    False() => {
      return None();
    }
    True() => {
      let b0: own u8 = index<u8>(deref(buf), 0_u64);
      let b1: own u8 = index<u8>(deref(buf), 1_u64);
      let hi: own u32 = cvt<u8, u32>(b0);
      let lo: own u32 = cvt<u8, u32>(b1);
      let hishift: own u32 = ishl.wrap<u32>(hi, 8_u32);
      let kind: own u32 = ior<u32>(hishift, lo);
      return Some(value: kind);
    }
  }
}
```

> **NOT AT PAR IN v0 [CARD-5 FLAG].** The par target is 1 load per field; the code
> above is byte assembly — 4-8 instructions per multi-byte field (`index<u8>`
> reads ONE byte; a u16/u32 is shifted and OR-ed together). Two gaps, neither
> invented here: (1) there is no byte->typed load op — `reinterpret` is
> scalar<->scalar (i32<->u32, {i,u}32<->f32, ...), never bytes->u32 out of a
> slice; (2) even the retained per-byte `index` bounds checks (the `traps` in the
> row) are only elidable by a length-fact-dominates-bounds extension, the same
> gated F-extension the C2 ring's pow2 mask needs. At-par validated views require
> a future typed-view form (validate-once, then endian-explicit unaligned loads
> with the bounds fact discharged). Until then this card fails closed to correct-
> but-not-at-par byte assembly; correctness is never conditional on the missing
> form, only the band is.

Replaces (once the view form lands): `memcpy`-then-parse and per-field re-
validation; the value is that the one-time length/shape check licenses every
later field load.

## Demoted cards (stubs; non-normative full text in `examples/`) [M5R3-PART2]

Each promise is one line; the full worked card is non-normative and lives at the
pointer. Demotion is PROVISIONAL (re-decided on round-4 data).

- Interner (value -> stable u32 id): bump arena + span-keyed `table` + id vector, one taught composition. -> `examples/interner.md`
- CSR graph (frozen adjacency scan): two-phase build (Vec-of-adjacency) then compressed sparse row over u32 ids. -> `examples/csr-graph.md`
- Pool-trees (navigable node soup): `pool` slab with u32 parent/child/sibling handle links. -> `examples/pool-trees.md`
- Intrusive links (O(1) unlink from within an element): u32 prev/next fields threaded through a slab. -> `examples/intrusive-links.md`
- Timer wheel (O(1) insert/cancel timeouts): array of buckets + embedded-link lists + masked index. -> `examples/timer-wheel.md`
- Prefetch-probe (memory-level parallelism): compute-ahead batched prefetch over a probe loop. -> `examples/prefetch-probe.md`
- Sort-by-key-permute (parallel key sort): `par.for_chunks` partition then permutation gather. -> `examples/sort-by-key-permute.md`
