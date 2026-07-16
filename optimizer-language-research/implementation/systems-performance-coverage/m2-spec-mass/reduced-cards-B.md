v0 surface statements are exactly: `fn`, `let`, `match`, `region`, `set`, `check`, `doc`, `return`, `move` (plus `give` in a `let`-initializer `match` and `try` for `Result` propagation). Iteration is spelled per C3 (a protocol op with a conformer, or self-recursion). `loop`/`break` exist in the kernel grammar (GRAM-4) but are held out of the blessed catalog surface (R3-provisional); type aliases, line comments, block-expression `match`, and backslash continuations are not v0 at all, and a program using those is rejected. [M5-FIX-6] [M5R2-FIX-2]

## C1. Bounded cache with eviction (LRU/CLOCK) — sealed `pool<T>` + `table<K, hdl<T>>` + intrusive handle links

Problem: a bounded map with O(1) get, insert, and eviction — the canonical
COMPOSITION scenario (SCENARIO-DEMAND-MAP §9). A cache MUST recycle slots, and
P2's append-only pool forbids exactly that (STOR-1: recycled bare indices are
well-typed UAF).
Pattern: one sealed `pool<Entry, links(prev, next)>` holds the nodes; one
sealed `table<u64, hdl<Entry>>` maps key to handle; recency is a circular
doubly-linked intrusive list threaded through two `hdl`-typed node fields and a
permanently-live sentinel handle minted at pool construction. Promote is one
splice row; evict is tail-read, take (which splices out first), table remove.
`hdl<T>` is the pool's u32-wide generational handle (index bits plus generation
bits under the pool's stated wrap contract): a copy VALUE, never a borrow — it
freezes nothing, and its safety is the generation witness, not the freeze
judgment. The `links(prev, next)` parameter is sealed-form parameterization
naming the two link fields of `Entry`; `pool_insert` self-links a fresh slot,
so unlink is total.

Node and the two structures:

```
struct Entry {
  key: u64;
  val: u64;
  prev: hdl<Entry>;
  next: hdl<Entry>;
}
```

The state owner holds `index`, `nodes`, `sent`, and `cap` as fields of its own
state struct and passes DISJOINT FIELD BORROWS down (P1/P4 posture): v0 has no
reborrowing (T-A), so these helpers are written over the parts, never over a
borrowed cache-wrapper struct. Two field borrows of one owned struct are
disjoint places; nothing is relaxed.

Op rows this card leans on are cited to the appendix pool and table forms, not
restated (Lever 2). Table: `table_find` (`-> own Option<H>`, absence is a
value), `table_insert`, `table_remove`. Pool: `pool_insert` (own-in; self-links
the slot; issues `live(p, h)`), `pool_take` (splices out, vacates, bumps
generation; kills `live(p, *)`), `pool_entry` / `pool_entry_uniq` (ISSUE entry
loan, p frozen until the loan ends; the uniq form kills value facts on the
slot), `pool_move_front` (total for live h, including h already front; links
only, no vacancy change, so liveness facts survive promote), `pool_link_prev`,
`pool_len`, `hdl_null` (never live; construction filler only). Generation checks
are retained by default on every pool row; eliding one requires a proven
`live(p, h)` fact — proof-elision only, never a weakened check (P8). Hot
lowerings the band rests on: `table_find` and `pool_entry` are guaranteed-inline
(one hash + one 16B group load + one key compare; one generation compare + one
indexed load); `pool_move_front` is pinned at <=6 u32 stores, zero branches.

Get with promote (hit path: one probe, one splice, one scoped read loan):

```
fn lru_get['i, 'c](idx: &'i table<u64, hdl<Entry>>, np: &uniq 'c pool<Entry, links(prev, next)>, sent: own hdl<Entry>, key: own u64) -> own Option<u64> reads('i), writes('c), traps
  -> teaching corpus: lru-get
```

Insert with evict (fresh-key path; present keys take `lru_get` plus one store
through `pool_entry_uniq`):

```
fn lru_insert['i, 'c](idx: &uniq 'i table<u64, hdl<Entry>>, np: &uniq 'c pool<Entry, links(prev, next)>, sent: own hdl<Entry>, cap: own u64, key: own u64, val: own u64) -> own unit writes('i, 'c), allocates(heap), traps requires {
  check ige<u64>(cap, 1_u64) else trap "lru: zero capacity";
}
  -> teaching corpus: lru-insert
```

The `requires` prologue closes the victim-is-sentinel edge structurally
(cap >= 1 and full implies a nonempty list), and its passed check feeds the
fact channel like any OP-5 fact. The filler `hnull` is inert: `pool_insert`
self-links the slot before it is reachable.

Performance contract: promote is one splice row — at most six u32 stores (two
splice-out, four splice-in), zero branches; the two node-local stores land in
the cache line the key compare already fetched, so off-node traffic is the 2-4
stores the demand map prices. Links are u32 indices into the same slab: no
per-node allocation, no headers, no pointer-chase beyond lines the probe
touched. Evict is O(1): one link read, one splice-out-and-vacate, one
probe-and-remove. Par target: the hashbrown-backed `lru` crate 0.12,
get-with-promote ~20-30 ns at 100K entries. The composition test is normative:
reaching lru-crate parity is what keeps a dedicated sealed cache form out of the
catalog; missing it is a catalog FINDING (gap), escalated through D16, not a
writer error.

Loan/freeze choreography: handles are copy values — holding any number of
`hdl<Entry>` freezes NOTHING, ever. Rows with loan NONE use their container
operand for the call only; `np` and `idx` are live again at the next statement.
`pool_entry`/`pool_entry_uniq` ISSUE an entry loan: while `e: &'v Entry` is
live, `np` is FROZEN — every pool row naming `np` is rejected statically, citing
the loan judgment — and the loan dies at region `'v` exit; the example scopes it
with an explicit `region 'v`. Loans are per-binding: a pool entry loan never
freezes the table, and vice versa. The ordering rule the card teaches: SPLICE
FIRST, READ SECOND — all link edits happen before the read loan is issued or
after it dies.

Region and borrow discipline (normative, not merely the example choreography
above) [M5-FIX-3]: (a) every region name must be bound by a `fn` gparam bracket
or an enclosing `region 'x { }` block — there are no free or placeholder region
names; (b) v0 has no reborrowing and no uniq-to-shared coercion — a row spelled
`&'r` requires a shared binding, so a `&uniq` receiver must use the row's
`_uniq` variant, not the shared row; (c) only rows with a dedicated
result-region parameter issue loans — a single-region row such as
`tbl_get_uniq` returns its loan at the receiver's region, never a fresh one.

Borrow-minting, worked [M5R2-FIX-3]. A row spelled `&'r` or `&uniq 'r` never
accepts a bare owned binding (the OWN-1 hard error writers keep hitting): mint
the `borrow_expr` atom `&'r p` / `&uniq 'r p` at the call site, binding the
region with an enclosing `region 'r { }` when the value is a local. Borrows are
atoms (GRAM-9): no `let`, no `move`. `&uniq 'x buf` is the uniq-mode mint,
`&'x buf` the shared variant; a bound borrow (`let l: &uniq 'x ... = &uniq 'x
buf;`) is affine and needs `move l`.

```
fn demo_mint(v: own u32) -> own unit allocates(heap), traps
  -> teaching corpus: borrow-mint
```

Failure handling under the single failure principle: absence is a value
(`Option`), never a failure. Environmental failure here is allocation growth
exhaustion, which traps per the `buffer_new`/OP-9 precedent. Programmer error
traps deterministically: a stale handle at any pool row (generation mismatch),
and cross-structure disagreement (the two explicit `check ... else trap` arms —
the table and pool disagreeing is a broken invariant, not a recoverable
outcome).

Misuse the checker rejects, and the trap that backstops the rest:

```
-> teaching corpus: lru-misuse
```

The `pool_move_front` line is REJECTED at compile time: `np` is frozen by the
live entry loan `e` (the diagnostic names the issuing row and the freezing
binding). Also rejected statically: storing an entry loan into a struct field
(loans are confined, stack-only — structs store values, not borrows), and
minting an `hdl<Entry>` from an integer (no such row; the handle type is opaque
and nonforgeable — only `pool_insert` issues live ones). The residual dynamic
class is the stale handle: hold `h`, let another path evict it, then
`pool_entry<Entry, 'v>(np, h)` — the slot may already be recycled, but the
generation differs, so the row TRAPS deterministically, citing itself. It is
never a well-typed read of a recycled slot: generations turn the STOR-1 UAF
class into a checked fact, which append-only P2 cannot offer a cache.

CLOCK variant (drop the promote write): same composition plus a `seen: Bool`
field on `Entry`. Get performs no splice — one store through `pool_entry_uniq`
into the entry's own line (`set deref(e).seen = True();`), eliminating the
recency-list write entirely; evict holds a clock hand as a stored copy handle
and walks `pool_link_prev` from it, clearing `seen` flags until a clear entry is
found (second-chance rotation, bounded by occupancy). Choose CLOCK when the
promote store is the measured bottleneck; only the link discipline changes.

Replaces: hashbrown-plus-`Box` LRUs with pointer links (per-node allocation,
pointer-chase on every promote), hand-rolled Vec-index caches with free lists
(unchecked recycling — the well-typed-UAF shape STOR-1 rejects), and
`Rc<RefCell>` node webs (unrepresentable BY DESIGN).

## C2. FIFO/ring — the one blessed queue spelling

Problem: FIFO order with O(1) push and pop — worklists, BFS frontiers, sliding
windows, token buckets. The catalog blesses exactly ONE spelling; the former
two-spellings state (separate ring and two-stack cards) is dead.
Pattern (primary — copy payloads): a masked ring over a power-of-two
`buffer<T>` with MONOTONE u64 head/tail counters. `head` counts pops, `tail`
counts pushes; occupancy is the wrapping difference; every access index is
`counter & (cap - 1)`. The pow2 fact is minted at construction by an OP-5 check
and restated as a `requires` prologue on every op, so no unchecked state exists.
Counters use `.wrap` under P8's structural-bound license — `tail - head <= cap`
always, so wrapping subtraction is exact modular arithmetic, and the hot loop
stays trap-free. The modulo spelling is rejected twice over: `irem` costs a
division in the hot loop, and its `.trap` rung poisons totality for the whole
tower (P8).

```
struct Ring {
  buf: buffer<u64>;
  head: u64;
  tail: u64;
}
```

```
fn ring_new(cap: own u64) -> own Ring allocates(heap), traps
  -> teaching corpus: ring-new
```

```
fn ring_push['q](q: &uniq 'q Ring, v: own u64) -> own Bool writes('q), traps requires {
  let qcap: own u64 = len<u64>(deref(q).buf);
  let qbit: own u32 = ipopcount<u64>(qcap);
  check ieq<u32>(qbit, 1_u32) else trap "ring: pow2 mask fact violated";
}
  -> teaching corpus: ring-push
```

```
fn ring_pop['q](q: &uniq 'q Ring) -> own Option<u64> writes('q), traps requires {
  let qcap: own u64 = len<u64>(deref(q).buf);
  let qbit: own u32 = ipopcount<u64>(qcap);
  check ieq<u32>(qbit, 1_u32) else trap "ring: pow2 mask fact violated";
}
  -> teaching corpus: ring-pop
```

The fact chain: the prologue's passed check mints `pow2(len(q.buf))` on the
dominated body (OP-5 stated-and-checked); deriving `iand(x, mask) < len(q.buf)`
from it — which retires the OP-4 bounds check at the `index` sites — is fact
extension F1 (pow2-mask domination), GATED on its own hostile review plus the
preregistered ring disassembly demo. Until F1 lands, the bounds checks are
RETAINED and the card fails closed: correctness is never conditional on the
elision, only the band is. No row in this card resizes `buf`, so the fact
survives the body; a future grow row would carry an explicit kill column for
every length-derived fact on `q.buf`. Bulk traversal uses the two-slice view,
not per-element pops: unwrapped (`head&m < tail&m`) is the single slice
`[head&m .. tail&m]`; two slices only when wrapped.

Preregistered band (single, reconciled): push/pop cycle at cap 4096 plus a BFS
frontier over a 1M-edge graph, within **[0.90x, 1.10x] of `VecDeque`**, with
zero residual bounds branches in the optimized loop (F1-dependent; failure fails
closed to seal-or-extend, never to a quiet miss). The affine variant is banded
separately: pool-handle ring vs `VecDeque<Box<T>>` on 64B tasks, same band; if
the affine route exceeds 1.25x against an INLINE `VecDeque` on a workload the
target set actually contains, the deferred sealed inline-affine deque is
escalated — through its trigger, not around it.

Affine-T variant (pool-handle queue): values live in a `pool<T>`; the ring is
this same `Ring` shape instantiated at the u32-wide copy handle. Push is
`pool_insert` then `ring_push` of the handle; pop is `ring_pop` then `pool_take`
— ownership of T re-emerges at the boundary, one slab touch per op is the honest
extra cost, and stale-handle misuse traps via the pool's generation check
exactly as in C1. Worst case per op stays O(1).

Fallback (two-seq flip queue): two `seq<T>` stacks — push onto the in-stack;
pop from the out-stack, and when it is empty, flip the in-stack over in one
reversing drain. AMORTIZED-ONLY: the flip is O(n) at the moment it happens. It
is the affine spelling of last resort, never for a latency-bounded loop.

Which variant — three questions, in order (any cross-thread handoff exits this
card entirely: that is the sealed `conc_queue` form, not a composition):

1. Is the payload copy, or does it fit a u32/u64 handle or key? -> primary
   masked ring.
2. Is the payload affine AND does the queue need O(1) worst case per op or
   stable identity while enqueued? -> pool-handle queue.
3. Is the payload affine and amortized bounds are acceptable? -> two-seq flip
   queue, amortized-only.

Failure handling under the single failure principle: full on push and empty on
pop are expected outcomes and return values (`Bool`, `Option` — the P9 rule: an
expected runtime outcome is never a contract trap); `buffer_new` size overflow
or exhaustion traps (OP-9); a non-power-of-two capacity is programmer error and
traps deterministically — at `ring_new` when built honestly, or at the FIRST
op's `requires` prologue when a writer hand-constructs `Ring` around `ring_new`.

Misuse: hand-constructing `Ring(buf: b, head: 0_u64, tail: 0_u64)` over a
capacity-12 buffer parses and checks — and the first push or pop traps
deterministically, citing the prologue's check string; the fact is checked,
never trusted. Forging the counters (`set deref(q).head = x` from other code
holding the `&uniq`) can corrupt queue CONTENTS but not memory: every access is
masked below the buffer length, so safety never rests on the counters — only the
band does. Rejected statically: pushing an affine T into `buffer<T>` (copy-only
domain; the checker routes you to variant 2), and sharing `&uniq 'q Ring` across
threads (no send/share row on this card; that demand names `conc_queue`).

Replaces: `VecDeque` in both its roles (Copy ring and boxed-affine queue),
hand-rolled head/tail rings spelled with modulo, linked-list queues (per-node
allocation, pointer-chase), and the catalog's own former two-spellings FIFO
state — the
two-stack card survives only as the fallback subsection above, documented
amortized-only.

## C3. Iteration — the blessed spellings [M5R2-FIX-2]

Blessed v0 code spells iteration exactly two ways: (1) a protocol op
(`seq_for_each`/`seq_drain`/`tbl_for_each`/`tbl_retain`/`tbl_drain`) driving a
conformer, or (2) self-recursion (FN-6). No third blessed spelling. (`loop`/
`break` exist in kernel GRAM-4 but are held out of the blessed surface pending
R3 loop-form validation; see M5-FIX-6.) The conformer half — a `conform` binds a
visitor `fn` to the contract member; the op drives it; cross-element state lives
in the env (set through a `deref` place). Signatures below; bodies in the corpus:

```
struct CountEnv { count: u64; }
fn count_visit['v](env: &uniq 'v CountEnv, item: &'v u32) -> own Bool reads('v), writes('v)
  -> teaching corpus: iter-count-visit
conform CountEnv : SeqVisit<u32> { visit = count_visit; }
fn count_all['r](s: &'r seq<u32, 0>) -> own u64 reads('r)
  -> teaching corpus: iter-count-all
```

The conformer's row `reads('v), writes('v)` equals `SeqVisit`'s member row and is
exactly what the body exhibits ([CAT-5a]/[EFF-2]); `count_all`'s row is just
`reads('r)` (the `'v` effects are confined to its internal `region 'v { }`). A
`tbl_retain` predicate is the same shape with a read-only member row:

```
struct KeepBig { min: u64; }
fn keep_big['v](env: &uniq 'v KeepBig, key: u64, value: &'v u64) -> own Bool reads('v)
  -> teaching corpus: iter-keep-big
conform KeepBig : TblRetain<u64, u64> { keep = keep_big; }
```

Accumulators and `set`. A `set` target is any place [GRAM-5]: `deref(p).field`,
`index<T>(...)`, or a bare local IDENT — so `set acc = iadd.wrap<u64>(acc, x);`
on a copy-typed local is legal. A bare local cannot carry state across
iterations (no blessed loop); that role is the env field or a recursion
parameter.
