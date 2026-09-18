# Round two, revision 4: overlap permission, allocation hand-back, entry facts, confirmed pools

Eighteen cells re-derived against `CANDIDATE-X1.md` revision 4. Round one ran
against revision 2; each cell below states round one's corrected verdict (the
cell's own label as repaired by `REVIEW-X1-round1.md`), then re-derives from the
revision-4 text and names the sentence that moved it. Notation is revision 4's:
measures are members (`r.len`, `(*b).len`, `entry(r).len`), windows have the
named parts `r.next`, `r.last`, `r.filled`, `r.free`, and there is no `par`
statement — parallelism is written as two adjacent statements plus a judgment
about whether they may overlap, with sequential meaning.

One row is used by ten of these cells and is worth stating once. Rule 16's
`alloc` carries "the row: the append slot of `*a.buf` and `(*a.buf).len`", which
by Rule 6 is `writes((*a.buf).next), writes((*a.buf).len)` — the same row Rule 6
gives `place_back` and the same row Rule 6's own user function `add_node`
declares. In revision 2 that row was `writes(a.buf)`, a write of the whole
container. Every difference in the pool cells below descends from that one
change, through Rule 6's sentence: "a live `r[i]` (which has `i < r.len`) never
overlaps `r.next` or `r.free`, overlaps `r.last` unless `i != r.len - 1` is
proved, and always overlaps `r.filled`."

---

### 1-16: Rule 1 and Rule 16: the index is the only durable name, and what concurrent construction now costs

Question: when Rule 1 makes an index the only name that outlives a statement, what
does that name cost once several workers must build one pool at the same time?

Round one: `rejected-real-cost` — `alloc` wrote the whole pool, so every
outstanding reference died at each allocation, no allocation could overlap any
other pool access, and the rewrite was per-worker pools with a two-level handle
costing one extra dependent load on every hop for the life of the graph.

Strongest program: parallel construction of one compiler IR graph, where workers
create nodes and rewrite node data at the same time.

```text
struct Node  { op: u32, a: u32, b: u32, data: u64 }
struct Arena { nodes: Box<Slots<Node>> }

fn alloc(g: &Arena, x: own Node) -> u64   writes((*g.nodes).next), writes((*g.nodes).len)
    contract { requires (*g.nodes).room > 0;
               ensures result == entry(*g.nodes).len;
               ensures (*g.nodes).len == result + 1; }

fn set_data(g: &Arena, i: u64, d: u64)    writes((*g.nodes)[i])
    contract { requires i < (*g.nodes).len; }

p  = &(*g.nodes)[i]                       // under i < (*g.nodes).len
id = alloc(&g, n)                         // (a)
(*p).a = id                               // (b) does p survive?

id1 = alloc(&g, n1)                       // (c) adjacent allocations
id2 = alloc(&g, n2)                       // (d) may (c) and (d) overlap?

id3 = alloc(&g, n3)                       // (e)
set_data(&g, other, d)                    // (f) may (e) and (f) overlap?
```

Trace:

- (b) is now accepted. Rule 3 invalidates `p` only when "a proper prefix of p's
  path is written, moved out of, replaced, or freed". The call's write paths are
  `(*g.nodes).next` and `(*g.nodes).len`; neither is a prefix of
  `(*g.nodes)[i]`, and Rule 6 settles the overlap directly: "a live `r[i]`
  (which has `i < r.len`) never overlaps `r.next` or `r.free`". The bounds fact
  survives too, by Rule 6's "`place_back`'s `ensures` carries it across the call"
  together with Rule 11's "a fact known before the call about a measure of an
  argument survives as a fact about `entry(p)` of that argument": `i <
  entry(*g.nodes).len` and `(*g.nodes).len == entry(*g.nodes).len + 1` give
  `i < (*g.nodes).len`. Rule 6 prints this exact program: "`id = add_node(&g,
  node)  // p survives`".
- (e)/(f) may overlap. Rule 13: "Two adjacent statements of one block may overlap
  when the first's write paths are disjoint from the second's read and write
  paths and vice versa". `{(*g.nodes).next, (*g.nodes).len}` against
  `{(*g.nodes)[other]}`: the slot is live (`other < (*g.nodes).len` from
  `set_data`'s `requires`, discharged statically, so nothing reads the measure at
  run time), so by Rule 6's sentence it does not overlap `.next`, and Rule 6's
  "The measure `r.len` is itself a write target" makes `.len` a target distinct
  from any slot. Disjoint both ways: the permission holds. Allocation into a pool
  and rewriting of the pool's live nodes may now proceed at the same time.
- (c)/(d) may **not** overlap, and this is the whole residue. Both statements
  write `(*g.nodes).next` and `(*g.nodes).len`; identical paths cannot be "proved
  disjoint" under Rule 10 clause 1's "different roots, or indices or ranges proved
  distinct". Two workers cannot allocate from one window.
- Growth is worse and is unchanged: `grow(&b, cap)  writes(*b)`, so it overlaps
  everything, and by Rule 3's printed example ("`grow(&v, cap)` declares
  `writes(*v)`: `*v` is a proper prefix of `(*v)[i]`: p invalid") it kills every
  reference into the pool. A pool that is built concurrently must be reserved.

Rewrite and cost: keep one storage and make allocation a slot write in a range
that arithmetic proves disjoint.

```text
g.nodes: Box<Array<Node>>                       // Node is Copy; Array::filled(cap, NIL) — Rule 6: "zero-filled maps to calloc"
fn build(part: &[Node], base: u64, w: own Work) -> u64   writes(part)   // returns how many it filled
{ k = 0; ... part[k] = Node { .. }; k = k + 1 ...  // handles it stores are base + k
  return k }

n1 = build(&(*g.nodes)[0..mid], 0, w1)
n2 = build(&(*g.nodes)[mid..cap], mid, w2)       // adjacent ranges: may overlap (Rule 13's own line)
```

Rule 7 supplies the range reference and its parameter type `&[T]`; Rule 13 admits
"adjacent ranges passed to a helper" and prints
`kernel(&r[0..mid], &out[0..mid]); kernel(&r[mid..n], &out[mid..n])   // disjoint ranges: may overlap`.
Handles stay one level — `base + k` is an index into the single `Array`, so a hop
is one base (hoisted, one storage for the whole graph), one scaled add and the
Rule 7 bound compare that this candidate already records under known costs and
that cell 1-7 charges. Against a C++ parallel frontend with one bump arena per
thread and raw `Node*`: allocation is a local counter bump against C++'s head
bump, a hop is a scaled add against a pointer load, and there is no extra
dependent load anywhere. The reservation is memory, not runtime, and
`Array::filled` of a zero `NIL` maps to `calloc`, so the pages are lazy. Round
one's per-hop penalty is gone.

Two boundaries on that zero: the element type must be Copy for `Array::filled`
(a non-Copy node pays Rule 12's `Option` null check per hit, a cost the candidate
already records), and a worker that must *read* another worker's live nodes while
both are building is rejected, since `reads((*g.nodes)[hB])` against
`writes(partB)` has no distinctness proof. A frontend whose workers build
separate functions never needs that read.

Rules consistent: yes.   Task achievable: rewrite, at zero runtime cost —
concurrent construction is expressed as disjoint ranges of one pre-filled block,
and the durable name stays a single `u32`.
Verdict: rejected-zero-cost

What changed and why: Rule 16's `alloc` row became "the append slot of `*a.buf`
and `(*a.buf).len`", and Rule 6 added "a live `r[i]` ... never overlaps `r.next`
or `r.free`". Together they turn allocation from a container-wide write into a
two-path write, which restores held references and allocation/mutation overlap,
and leaves only allocation-against-allocation, which the range rewrite removes at
no per-hop price.

---

### 2-4: A reference captured by a function value has no declarable effect

Question: may a function value capture a reference when it is neither stored nor
returned, and if so, how is the captured access declared?

Round one: `rejected-zero-cost` (the cell wrote `undecided-rule-gap`; the
reviewer corrected it, because Rule 9's closing sentence decides the case the
cell called open).

Strongest program: a capturing function value passed to a helper that runs it on
two ranges which may overlap, so that an undeclared write would be a race rather
than a mistake.

```text
input:   Box<Slots<u8>>
scratch: Box<Slots<u8>>
out:     Box<Slots<u8>>

h = |part: &[u8]| { helper(part, &input) }          // captures &input: benign, both uses read
g = |part: &[u8]| { compress_into(part, &scratch) } // captures &scratch: both uses write

h(&(*out)[0..mid])                                  // declared type: fn(part: &[u8]) writes(part)
h(&(*out)[mid..n])                                  // adjacent ranges: the rows say they may overlap
```

Trace:

- Rule 4 permits the capture: a reference "cannot be captured by a function value
  that is stored or returned", and neither `h` nor `g` is stored or returned.
- Rule 9 then makes the capture unusable. Its grammar is closed — "An effect row
  lists `reads(path)` and `writes(path)` where each path starts at a reference
  parameter" — so neither `input` nor `scratch` can appear in the row of `h` or
  `g`. Its last sentence is what decides the cell: "A function body is checked
  against its own row: every statement's effect and every callee's substituted
  row must be covered by the declared row." `helper(part, &input)` substitutes to
  `writes(part), reads(input)`, and `reads(input)` is not covered by the declared
  `writes(part)`, so `h`'s body is rejected where it stands. `g` is rejected by
  the same clause on `writes(scratch)`.
- Nothing reaches Rule 10 or Rule 13, so the row the overlap judgment reads is
  never a row with a hidden path in it. Rule 4's permission survives as text but
  has no instances with an effect: any use of a captured reference is a `reads` or
  a `writes`, and neither can be covered.
- The ordinary forms are unaffected: a non-capturing function value, and Rule 4's
  own index protocol (`fn find(v: &Slots<Entry, N>, key: Int) -> Option<u64>`
  with `ensures when Some: result < v.len`, the caller forming `&v[i]`), both
  stand.

Rewrite and cost: make every captured reference a parameter.

```text
h = |part: &[u8], src: &Box<Slots<u8>>| { helper(part, src) }     // writes(part), reads(src)
h(&(*out)[0..mid], &input)
h(&(*out)[mid..n],  &input)      // reads(input) on both sides: read/read, still may overlap
```

The row is now complete, `reads(input)` in both statements is read/read which Rule
13 allows, and the `scratch` variant becomes `writes(scratch)` on both sides,
which Rule 13 refuses on the spot. Runtime cost against the C++ lambda: none —
the captured pointer becomes an argument register, and protocol item 7 excludes
extra parameters from cost.

Rules consistent: yes, with one dead letter — Rule 4's permitted capture has no
usable instance once Rule 9's body check applies.
Task achievable: rewrite, zero cost.
Verdict: rejected-zero-cost

What changed and why: no revision-4 sentence changed the outcome; the reviewer's
correction stands unaltered because Rule 9's closing sentence is unchanged. What
revision 4 changes is the *stakes*: with no `par` statement, the two calls are
adjacent statements whose overlap permission is read straight off the declared
row (Rule 13: "using the same path-overlap and index/range-disjointness judgment
as Rule 10"), so Rule 9's body check is the only thing standing between a
capturing closure and a race. `&[T]` also makes the closure's declared type
writable, which revision 2 could not spell.

---

### 3-16: When long-lived identity must be an index, what does Rule 3 still protect?

Question: Rule 3 protects references, but Rules 1 and 4 make every long-lived link
an index — so what is left of the protection under revision 4, where a reference
also survives allocation?

Round one: `accepted-unsafe` — links are `u64` and get none of Rule 3's
protection; memory safety intact, identity silently wrong; generation words cost
one load and one compare per edge and grow an edge from 8 to 12 or 16 bytes.

Strongest program: a compiler graph in a pool, with a held reference carried
across allocation, across the program's own free, and across growth.

```text
struct Node  { op: Int, a: u64, b: u64, next_free: u64 }
struct Arena { buf: Box<Slots<Node>>, free_head: u64 }

p   = &(*a.buf)[id1]
id2 = alloc(&a, Node { .. })       // writes((*a.buf).next), writes((*a.buf).len)
(*p).a = id2                       // (1) accepted under revision 4; rejected under revision 2

free_node(&a, id1)                 // writes((*a.buf)[id1].next_free), writes(a.free_head)
id3 = alloc_reuse(&a, Node { .. }) // pops the free list: writes((*a.buf)[id3]); id3 == id1 at run time
x = (*p).op                        // (2) accepted: p is still valid, and it reads the NEW occupant

grow(&a.buf, bigger)               // writes(*a.buf)
y = (*p).op                        // (3) rejected
```

Trace:

- (1): Rule 3 invalidates on "a proper prefix of p's path"; `(*a.buf).next` and
  `(*a.buf).len` are siblings of `(*a.buf)[id1]`, not prefixes, and Rule 6 states
  the overlap outright ("a live `r[i]` ... never overlaps `r.next` or `r.free`").
  The bound is carried by Rule 16's `ensures (*a.buf).len == result + 1` read
  through Rule 11's `entry` sentence.
- (2): the reused slot is written by `alloc_reuse`, whose write path is
  `(*a.buf)[id3]`. That is not a prefix of `(*a.buf)[id1]` either — it is the same
  place under a different index — and Rule 3 says "Writing the storage at p's path
  or below it (a content write) does not invalidate p." So `p` survives its own
  target being reassigned to a different logical object, and Rule 16 endorses this
  in terms: "The same holds for a reference into a user-level pool or allocator:
  storage that still exists stays readable, and what the program considers 'freed'
  is the program's own bookkeeping."
- (3): `grow` declares `writes(*b)`, and `*a.buf` *is* a proper prefix of
  `(*a.buf)[id1]`. Rule 3's own printed example is this line. So the protection
  that remains is exactly: storage that ceases to exist, is replaced, is moved out
  of, or whose root leaves scope. Identity inside storage that still exists is
  outside Rule 3 by construction, for indices (Rule 1: "no aggregate ever contains
  a reference") and now for references too.
- Memory safety is untouched at every step: the slot is in bounds by Rule 7, holds
  a valid `Node` by Rule 6 ("no program point can observe a slot inside the window
  as empty") and Rule 12 ("a 'logically empty' slot holds a valid value"), and
  Rule 8 never releases a linear element behind the program's back.
- The task row's requirement — a relation to a deleted node must not silently
  become a relation to a replacement node — is refused by Rule 16 on purpose, and
  revision 4 refuses it in one more place than revision 2 did.

Rewrite and cost: generation words, which Rule 16 names ("Programs that need to
detect it keep a generation number as data").

```text
struct H    { idx: u32, gen: u32 }                       // Copy, storable (Rule 1 satisfied: no reference)
struct Node { gen: u32, op: Int, a: H, b: H }
fn live(a: &Arena, h: H) -> Bool  reads(a)
    contract { ensures when true: h.idx < (*a.buf).len; }
```

Cost against C++'s raw `Node*`: an edge grows from 8 to 12 or 16 bytes, which on a
two-edge IR node is 8 to 16 bytes of extra cache footprint per node on a
traversal-bound pass; one generation load (same cache line as the node, usually
free), one compare and one predicted branch per edge followed. Against a Rust
generational arena: exactly zero, same representation. A program that declines the
generation word pays nothing and gets the silent defect, which is why this is not
`accepted-fine`.

Rules consistent: yes — Rule 16 states its own consequence and Rule 3 is not
weakened, only narrowed to storage lifetime.
Task achievable: rewrite — graph, mutation and parallel maps are expressible;
deletion safety costs the generation word.
Verdict: accepted-unsafe

What changed and why: the slot-precise `alloc` row plus Rule 16's new sentence
about "a reference into a user-level pool or allocator". Revision 2 gave the
program an accidental protection — every reference died at every allocation, so a
reference could never outlive a free. Revision 4 removes that accident: the hole
is the same size for references as for indices, and the cell's answer to "what
does Rule 3 protect" narrows to storage lifetime alone.

---

### 4-6: Rule 4 and the window: an index outlives the element it named

Question: an index is the only locator Rule 4 lets cross a call boundary; what do
Rule 6's window operations do to it?

Round one: `accepted-unsafe` — `find` returns an index, `take_back` then
`place_back` restore the length, the bound is re-proved with no test, and slot `i`
holds a different element; the reviewer added that under revision 2 the
substitution was observable only at `i == r.len - 1`.

Strongest program: revision 4's `remove_at`/`insert_at`, which renumber a whole
suffix in one memmove while leaving every stored index in bounds.

```text
n = v.len
i = find(&v, key)?                  // Rule 4: ensures when Some: result < v.len, so i < n
y = remove_at(&v, 0)                // writes(v.filled), writes(v.len); ensures v.len == entry(v).len - 1
consume(move y)
insert_at(&v, 0, fresh)             // writes(v.filled), writes(v.next), writes(v.len)
                                    // ensures v.len == entry(v).len + 1, so v.len == n
charge(&v[i], amount)               // accepted with NO runtime test; slot i holds what was at i - 1
```

Trace:

- Rule 4 forces the locator to be owned data: `fn find(v: &Slots<Entry, N>, key:
  Int) -> Option<u64>` with `ensures when Some: result < v.len`.
- Rule 11's invalidation clause — "A fact that mentions a path is invalidated when
  that path is written (by statement or call) unless the callee's `ensures`
  re-establishes it" — kills `i < v.len` at `remove_at`. But `i < n` relates two
  integer locals and mentions no path, and revision 4 states the bridge that
  revision 2 left implicit: "Across a call, a fact known before the call about a
  measure of an argument survives as a fact about `entry(p)` of that argument."
  So `n == entry(v).len` at `remove_at`, and the two `ensures` compose affinely to
  `v.len == n`. Rule 7's `requires i < v.len` is discharged statically.
- The element at `i` is not the element `find` saw: `remove_at` is "one memmove"
  and shifts every slot above `0` down, `insert_at` shifts them back, and the
  suffix now sits one position off relative to the search's observation for
  *every* `i`, not only the last slot. One operation renumbers the whole tail.
- The reference form is protected and the index form is not, and revision 4 makes
  that contrast exact. `remove_at` declares `writes(r.filled)`, and Rule 6 says a
  live `r[i]` "always overlaps `r.filled`", so any live reference into the window
  dies at the call by Rule 3. Rule 4 is precisely what prevents the locator from
  being the thing that gets that protection.
- Memory safety holds: Rule 6 keeps every slot in `[0, len)` occupied, Rule 12
  keeps it a valid value, and Rule 7's bound is proved rather than trapped.

Rewrite and cost: an epoch beside the window, bumped by every operation that
renumbers.

```text
struct Log   { v: Box<Slots<Entry>>, epoch: u64 }
struct Stamp { i: u64, epoch: u64 }
fn find2(l: &Log, key: Int) -> own Option<Stamp>   reads(l)
    contract { ensures when Some: result.i < (*l.v).len; }

if s.epoch == l.epoch && s.i < (*l.v).len { charge(&(*l.v)[s.i], amount) }
```

Cost against C++ or Rust: one extra load (the epoch word, adjacent to the block
header and normally in the same line), one compare and one predicted branch per
use, and a behaviour the writer must invent for the stale arm. A C++ or Rust
program that also locates by index pays exactly this; a Rust program holding
`&mut Entry` across the mutation is rejected at compile time for free, and that
option is closed here because Rule 4 forbids the reference crossing the boundary
in the first place.

Rules consistent: yes — nothing in Rules 4, 6, 7, 11 or 12 is violated, which is
the finding.
Task achievable: yes for the computation, no for the identity guarantee without
the epoch check.
Verdict: accepted-unsafe

What changed and why: `insert_at`/`remove_at` ("one memmove") widen the
phenomenon from the last slot to every index above the edit point, and Rule 11's
new `entry` sentence removes the last step round one had to presuppose. Rule 6's
`writes(r.filled)` on both operations is also what makes the reference form die
where the index form survives, so the cell's asymmetry is now stated by the rules
rather than inferred.

---

### 4-16: Rule 4 and pools: an index is not invalidated by removal

Question: with Rule 16 confirmed, does the Rule 4 index recover what a reference
would have given for a mutable graph with slot reuse?

Round one: `accepted-unsafe` — a stale handle names the new occupant; the
successive-allocation distinctness proof through `entry` is sound; generation
words cost one load, one compare, and grow a link from 8 to 16 bytes.

Strongest program: an intrusive doubly linked list living in a pool, where a
node is unlinked, its slot recycled, and a stale `next` walks into the
replacement.

```text
struct Node  { next: u64, prev: u64, data: u64 }
struct Arena { buf: Box<Slots<Node>>, free_head: u64 }

m = alloc(&a, node)                      // ensures result == entry(*a.buf).len
link(&a, p, m); link(&a, m, q)           // each call a single writes((*a.buf)[k].next or .prev)
unlink(&a, m)                            // m leaves the list; its slot joins the free list
k = alloc_reuse(&a, other)               // reuses slot m
walk(&a, start)                          // a node still holding next == m walks into k
```

Trace:

- Rule 1 forces the links to be data — "there is no type parameter, wrapper, or
  variant payload through which a reference can be stored" — and Rule 4 keeps a
  reference from being returned or stored, so nothing in the structure is under
  Rule 3's regime at all.
- Rule 16 states the outcome without hedging, and it is now confirmed rather than
  provisional: "A stale index that is still in bounds names the current occupant
  of that slot: a logic error, not a memory error." `walk` is accepted with no
  diagnostic. P3's "removal invalidates every path to m" is not delivered.
- The opposite direction is equally sharp and is a genuine strength: successive
  allocations are provably distinct with no runtime check. `i1 = alloc(...)` gives
  `i1 == entry(*a.buf).len` and `(*a.buf).len == i1 + 1`; Rule 11's "a fact known
  before the call about a measure of an argument survives as a fact about
  `entry(p)`" ties the second call's `entry` term to the first call's `ensures`,
  so `i2 == i1 + 1` and Rule 10 clause 1's "indices or ranges proved distinct" is
  discharged. That is what makes the independent-block case of 13-16 free.
- Revision 4 adds a trap that Rule 16 does not warn about. `insert_at` and
  `remove_at` are now available on any `Slots`, and each is "one memmove" that
  renumbers every slot above `k`. Applied to a pool, one `remove_at` silently
  renumbers every handle in the entire graph — the same defect as 4-6 but across
  a structure whose handles are stored in its own nodes. A pool must edit slots
  only through `replace`, `swap` and `take_back`; Rule 6 offers the shifting
  operations on the same storage and Rule 16 does not exclude them.
- Memory safety survives every step: Rule 7's bound, Rule 6's full window, Rule
  12's valid occupant.

Rewrite and cost: generation numbers, exactly as Rule 16 prescribes.

```text
struct H    { idx: u64, gen: u64 }
struct Node { gen: u64, next: H, prev: H, data: u64 }
if h.idx < (*a.buf).len && (*a.buf)[h.idx].gen == h.gen { ... }
```

Cost per hop against C++ with raw pointers and slot reuse: one extra load (the
generation word, normally the same cache line as the node), one compare and one
predicted branch on top of the Rule 7 bound compare, and every stored link grows
from 8 to 16 bytes — which halves the edges per cache line and is the dominant
term in a pointer-chasing traversal. Against C++ with the same reuse and no
detection, safety is equal and the word is pure overhead; against a Rust
`Vec<Node>`-plus-index pool it is identical; against Rust references the graph is
not expressible at all, so there is no cheaper safe baseline to lose to.

Rules consistent: yes — Rule 16 states this outcome deliberately.
Task achievable: rewrite, at one load, one compare and 8 extra bytes per link.
Verdict: accepted-unsafe

What changed and why: Rule 16 is confirmed ("Confirmed by the owner on
2026-09-18, knowingly reversing the design tree's earlier rejection"), so this is
a decided position rather than a provisional one; and the new shifting operations
make the hole reachable through an ordinary window operation, not only through a
free list.

---

### 5-16: Box versus pool handles for shared and cyclic structure

Question: a mutable compiler graph needs back edges, slot reuse and parallel work;
what does moving from `Box` edges to Rule 16 handles cost under revision 4?

Round one: `rejected-real-cost` — handles are the only rendering; per-hop bound
compare, mandatory generation word, and parallel allocation lost, because
`alloc` wrote the whole container while `Box::new` is exempt from every effect
comparison.

Strongest program: a doubly linked pool graph, built while it is traversed, then
mapped in parallel.

```text
struct Node  { data: u64, next: u64, prev: u64, gen: u32 }
struct Arena { buf: Box<Slots<Node>> }

i = alloc(&a, n1)                                   // (A)
j = alloc(&a, n2)                                   // (B) may (A) and (B) overlap?
p = &(*a.buf)[cur];  d = (*p).data;  i2 = alloc(&a, n3)   // (C) may the read and the allocation overlap?

h = (*a.buf)[cur].next                              // a hop: the handle is data
if h < (*a.buf).len { if (*a.buf)[h].gen == want { use(&(*a.buf)[h]) } else { stale() } }

map(&(*a.buf)[0..mid]);  map(&(*a.buf)[mid..n])     // adjacent ranges: may overlap
```

Trace:

- `Box` cannot express the structure. Rule 1 is recursive and closed, so a back
  edge cannot be a reference, and Rule 5's "`Box<T>` owns exactly one heap object"
  makes a second owning edge a double owner. A graph is not an ownership forest,
  so Rule 16's handles are the only rendering. That part is unchanged.
- (A)/(B): still may not overlap. Both declare `writes((*a.buf).next)` and
  `writes((*a.buf).len)`; identical paths admit no distinctness proof. The `Box`
  form of the same two allocations *does* overlap, by Rule 14's "Allocation and
  release carry no effect entry and never prevent two statements from
  overlapping". Choosing handles for shared structure still gives up
  allocation-against-allocation overlap — but only that.
- (C) now overlaps, which revision 2 refused. The read path `(*a.buf)[cur]` is a
  live slot; Rule 6: it "never overlaps `r.next` or `r.free`", and `.len` is a
  separate target. Note the exact boundary: if the reader instead writes `if cur <
  (*a.buf).len { ... }`, that is `reads((*a.buf).len)` against the allocation's
  `writes((*a.buf).len)` and the permission is withdrawn. Proving the bound
  statically buys the overlap; testing it at run time spends it.
- The hop still tests. Rule 11: "There are no quantified facts over array elements
  ("for all i ...") and no per-slot occupancy facts", so nothing discharges
  `h < (*a.buf).len` once and for all, and Rule 7 prescribes the test: "when the
  proof is unavailable the program tests the measure, which is ordinary data."
  The bitmask fact that would remove it is listed under "Deferred to a future
  concurrency and layout round".
- Staleness still costs the generation word, per Rule 16, because the task
  requires that a relation to a deleted node not become a relation to its
  replacement.
- The parallel node map is gained, exactly as in revision 2: adjacent ranges over
  one contiguous pool, which a scattered `Box` graph could never express, since
  Rule 4 forbids collecting the node references a worker split would need.

Rewrite and cost against a C++ pointer graph with `new` per node: per hop, one
compare and one not-taken branch (bounds) plus one load and one compare
(generation), against a bare pointer dereference; the address computation
`base + h * size` replaces a pointer load, which is a wash or better for
locality. Per node, four bytes of generation against zero. For parallel
construction, 1-16's pre-filled `Box<Array<Node>>` with per-worker ranges now
recovers the allocation overlap with one-level handles and no extra dependent
load, so round one's "per-worker arenas plus a rebasing pass or a two-level
handle" term is withdrawn. Against a correct safe C++ or Rust rendering (a
generational slot map) everything above except the bounds compare is what that
rendering also pays.

Rules consistent: yes.
Task achievable: rewrite — the graph is expressible with handles; the only thing
not recoverable in the `Slots` form is two allocations overlapping, and the
pre-filled block recovers even that.
Verdict: rejected-real-cost

What changed and why: Rule 16's slot-precise `alloc` row and Rule 6's window-part
overlap sentence. The parallel-allocation term of round one's cost shrinks to a
single case (two appends to one window) and the per-hop penalty of the two-level
handle disappears; what remains real against C++ is the bounds compare (Rule 11's
refusal of quantified facts) and the generation word (Rule 16's stated position).

---

### 7-16: Bounds proofs over pool and arena handles

Question: what does a handle dereference cost when handles are stored in the
structure itself, and can a builder keep a reference to the parent while it
allocates the child?

Round one: `rejected-real-cost` — one compare and one branch per hop, not
hoistable; and the builder's `p = &g.buf[parent]` died at every `alloc`, so every
allocation forced a re-formation.

Strongest program: a builder that holds the parent while allocating, in a pool
that may also need to grow.

```text
struct Node  { next: u64, prev: u64, data: u64 }
struct Arena { buf: Box<Slots<Node>> }

// traversal: every hop dereferences a handle that came out of memory
h = start
loop {
    if h >= (*g.buf).len { break }            // the test Rule 7 demands, once per hop
    acc = acc + (*g.buf)[h].data
    h = (*g.buf)[h].next                      // the next handle is data
}

// builder
p  = &(*g.buf)[parent]
id = alloc(&g, Node { next: 0, prev: 0, data: d })   // writes((*g.buf).next), writes((*g.buf).len)
(*p).next = id                                        // (1) accepted under revision 4

grow(&g.buf, bigger)                                  // writes(*g.buf)
(*p).prev = id                                        // (2) rejected
```

Trace:

- (1) is the revision-4 change, and it is Rule 6's printed example. The write
  paths are `(*g.buf).next` and `(*g.buf).len`; Rule 3 invalidates only on "a
  proper prefix of p's path", and Rule 6 settles the overlap: "a live `r[i]`
  (which has `i < r.len`) never overlaps `r.next` or `r.free`". The parent's bound
  survives by Rule 6's "`place_back`'s `ensures` carries it across the call"
  together with Rule 11's `entry` sentence. There is no re-formation and no
  compare: the builder holds the parent across an unbounded number of
  allocations, exactly as a C++ builder holds a `Node*`.
- (2) is the price of that: `alloc` can only be that cheap because it cannot
  reallocate — Rule 16's contract carries `requires (*a.buf).room > 0`. A pool
  that grows calls `grow(&b, cap)  writes(*b)`, `*g.buf` *is* a proper prefix of
  `(*g.buf)[parent]`, and Rule 3's own example rejects the next use. So the
  builder reserves capacity up front, or re-forms after every growth. Reserving
  is what a C++ builder does too.
- The traversal is unchanged and is the residual cost. `h` came from
  `(*g.buf)[h_prev].next`, so Rule 11's "There are no quantified facts over array
  elements ("for all i ...")" leaves nothing to discharge `h < (*g.buf).len`, and
  P3's stated requirement that "the invariant next/prev point to live nodes is
  stated once and reused" is not expressible. The test cannot be hoisted, because
  the fact it establishes is about an `h` that is reassigned from memory each
  iteration and the new `h` carries no fact. A power-of-two capacity does not
  help: masking is not an affine comparison, and the bitmask fact is listed under
  "Deferred to a future concurrency and layout round".
- Identity is Rule 16's declared position, unchanged: "A stale index that is still
  in bounds names the current occupant of that slot".

Rewrite and cost:

- Traversal, as written, with the bound test in the loop. Against the C++ `Node*`
  chase: one compare and one well-predicted branch per hop plus one
  writer-supplied dead arm. The chase is latency-bound so the compare usually
  hides under the dependent load, but the branch still blocks software-pipelining
  two hops. Against safe Rust with `arena[h as usize]`: zero.
- Builder: none needed — the reference simply survives, where round one paid an
  address recomputation per allocation. Where the pool may grow, hoist the
  reservation; where it cannot be hoisted, re-form after `grow`, one scaled add
  that folds into the following store.
- Identity, where the task needs it: one generation word per slot and per handle,
  one load, one compare, one branch per dereference and a second dead arm.

Rules consistent: yes — Rule 16 composes with Rules 3, 6, 7 and 11 without
contradiction; it declines a guarantee rather than breaking one.
Task achievable: rewrite — pool code works at one compare per handle hop, plus a
load and a compare where stale-handle detection is required.
Verdict: rejected-real-cost

What changed and why: Rule 16's row, read through Rule 6's window parts, plus
Rule 11's `entry` sentence. The builder cost that round one charged is gone
entirely; the per-hop compare, which is the cell's real finding, is untouched
because Rule 11's exclusion of quantified facts is unchanged.

---

### 8-14: A linear payload and an allocation that fails

Question: Rule 14 makes every allocation fallible and Rule 8 makes a linear
value's consumption an obligation; when a linear value is moved into an
allocation that returns `Err`, who owes the `close`?

Round one: `accepted-unsafe` — `Box::new`'s error payload was `Oom` alone, so the
argument was not handed back, Rule 9 recorded the consumption at the call, and the
`File` was lost with no rule violated. The reviewer added that the cell's
suggested rewrite was itself a rejection, since `?` on the allocation left `f`
live on the early-exit path.

Strongest program: a linear resource installed into a freshly allocated node, on
the failure arm, with a second linear value already live.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, n: u64 }          // linear by containment (Rule 8)

fn install(f: own File, g: own File, n: u64) -> Result<Box<Conn>, (Oom, File, File)>
{
    match Box::new(Conn { f: move f, n: n }) {          // Result<Box<Conn>, (Oom, Conn)>
        Ok(b)        => Ok(b),                           // g still owed on this path
        Err((e, c))  => {
            let Conn { f, .. } = move c                  // Rule 6: several fields at once, `..` covers the rest
            return Err((e, move f, move g))
        }
    }
}
```

Trace:

- Rule 5 closes round one's hole in one sentence: "A fallible allocation that
  takes a by-value payload hands it back on failure, so a linear payload is never
  lost", with the shape printed beside it — `b = Box::new(Node { ... })?  //
  Result<Box<Node>, (Oom, Node)>: on Err the Node comes back`.
- Rule 6 supplies the way back out of the returned aggregate: "A move out of a
  field or out of Box content consumes the whole owner: the owner ceases to exist,
  its other affine parts are released, and a remaining linear part rejects the
  move (take it in the same destructuring)", with `let Conn { f, g, .. } = move c`
  as the printed form. The `u64` is affine and is released; the `File` leaves as an
  owned value.
- Rule 8's obligation is now discharged by an operation rather than by an
  accounting artefact: "Linear values must be consumed by an explicit operation on
  every exit path; the compiler never releases them." On the `Ok` path the `File`
  is inside the returned `Box`; on the `Err` path it is inside the returned tuple;
  on neither does it vanish.
- Rule 9 still records the consumption at the call ("the call site records the
  consumption (`move`) ... of the argument's place"), and that is now *true* rather
  than a fiction: the allocator either keeps the value or returns it.
- One rejection remains and is correct: `?` cannot be used while an unconsumed
  linear value is live. `cell = Box::new(Slots::new<Conn>(1))?` with `g` live
  manufactures an exit path on which `g` is not consumed, so Rule 8 rejects it.
  The repair is the `match` above, with `close(move g)` on the failure arm — the
  branch a C++ program writes anyway.
- Rule 15 carries the payload out: "A function returns owned values only", and a
  linear value is an owned value.

Rewrite and cost: none needed for the allocation itself. Round one's
allocate-then-populate workaround — a one-slot `Box<Slots<Conn>>` so that no
linear value was ever in flight across a fallible boundary — is unnecessary, and
with it goes its 16 bytes of block header per linear cell and the cache pressure
of `16n` bytes across a tree of `n` linear nodes. What remains is the `match` in
place of `?` wherever a linear value is live, which costs one branch on a path
that must do the work anyway. Against C++ (`new Conn{std::move(f), n}` whose
`bad_alloc` path unwinds `~File()`): equal, and WF needs no destructor and no
unwinder. Against Rust `Box::try_new`: equal. The one standing cost is the
candidate's already-recorded "construction into the append slot moves one element
where C++ constructs in place; usually elided by the backend, not guaranteed".

Rules consistent: yes — Rules 5, 6, 8, 9 and 15 now close over the failure arm
with no value unaccounted for.
Task achievable: yes, at zero runtime cost.
Verdict: accepted-fine

What changed and why: Rule 5's sentence "A fallible allocation that takes a
by-value payload hands it back on failure, so a linear payload is never lost",
together with Rule 6's whole-owner destructuring, which gives the returned
aggregate a legal way to be taken apart. Round one's finding was the sharpest
unsafety in the matrix; revision 4 removes it and removes its workaround's cost
with it.

---

### 10-11: Contracts across the call boundary: which facts die and which are rebuilt

Question: after a call, which of the caller's bounds and measure facts survive,
and can a callee hand back the facts its writes destroyed?

Round one: `accepted-fine` — `writes` is a total demolition of the caller's
knowledge about that path and `ensures` is the only rebuilding tool; Rule 1 makes
the invalidation set exactly the declared paths, which beats a C++ compiler
across an opaque call. The reviewer confirmed it and noted one presupposed step:
the survival of a pre-call fact as an `entry(p)` fact.

Strongest program: an in-place partition whose caller must re-derive both
endpoints, beside a writer that must keep a caller's fact about a *slot* across an
append made in another translation unit.

```text
fn partition(part: &[Int], pivot: Int) -> u64   writes(part)
    contract { ensures result <= part.len; }

fn qsort(part: &[Int])   writes(part)
{
    n = part.len
    if n > 1 {
        mid = partition(part, choose(part))       // writes(part): kills every fact mentioning part
        qsort(&part[0..mid])                      // (A)
        qsort(&part[mid..n])                      // (B) adjacent, disjoint ranges: may overlap
    }
}

// the facts that must survive untouched, across a separately compiled callee
m  = other.len
k  = key_slot                                     // k < (*meta.index).len proved earlier
v0 = (*r)[j].tag                                  // a fact about the CONTENTS of slot j
place_back(&r, x)                                 // writes(r.next), writes(r.len)
use(&other[m - 1]); use(&(*meta.index)[k])        // still proved: different paths
assert v0 == (*r)[j].tag                          // still proved: .next never overlaps a live r[j]
```

Trace:

- Rule 11's invalidation clause is the engine: "A fact that mentions a path is
  invalidated when that path is written (by statement or call) unless the callee's
  `ensures` re-establishes it." `n == part.len` mentions `part`, `partition`
  declares `writes(part)`, so the fact dies. It comes back for free here because a
  range reference's measure is fixed at formation (Rule 7: "a reference kind with
  measure `len`, formed only from an indexable or from another range reference"),
  so `part.len` cannot have changed; for a `&Box<Slots<Int>>` parameter the callee
  must publish `ensures (*b).len == entry(*b).len` or the caller cannot form
  `&(*b)[mid..n]` at all.
- With `mid <= part.len` and `0 <= mid`, both sub-ranges are formable with no
  runtime test (Rule 7's `requires lo <= hi <= r.len`), and Rule 13 admits the
  overlap of (A) and (B) by its own printed line for disjoint adjacent ranges.
- `entry(p)` is what makes a writing callee's promise statable, and revision 4
  states the step round one presupposed: "Across a call, a fact known before the
  call about a measure of an argument survives as a fact about `entry(p)` of that
  argument, which is how an `ensures` of the shape `r.len == entry(r).len + 1`
  connects to what the caller knew." Without that sentence, `place_back`'s
  contract is a relation between two unknowns.
- The second half is where this rule set beats a C++ compiler, and revision 4
  widens the margin. `writes(part)` cannot reach `other`, because Rule 1
  guarantees "no aggregate ever contains a reference" — there is no pointer inside
  `part` through which `other` could be touched — and it cannot reach the sibling
  field `meta.index`, because those are non-overlapping paths. New in revision 4:
  a row may name a *part* of a window, so `place_back`'s `writes(r.next)` leaves
  the caller's fact about the contents of live slot `j` intact, by Rule 6's "a
  live `r[i]` ... never overlaps `r.next` or `r.free`". Under revision 2 an
  appending callee declared the container and the caller lost every slot fact it
  had. A C++ compiler must reload after an opaque call unless it proves no-alias;
  here the guarantee is structural and now slot-precise.
- Field granularity is unchanged (Rule 9's `fn update(o: &Obj, c: Bool)
  writes(o.a), writes(o.b)` leaves facts about `o.c` alive), and Rule 10's
  function-typed clause extends the reasoning to callbacks: "Function-typed
  parameters carry a full signature with its own row and contract, and a call
  through one uses that row." The caveat round one inherited from a capturing
  callback lapses: Rule 9's body check rejects such a callback outright (2-4,
  10-13), so a row the caller trusts is a row that was checked.
- One shape stays unavailable: a callee whose writes are conditional on a runtime
  value can only route its promise through its result, since Rule 11 offers
  "`ensures when Variant:` for result-routed relations". A function that sometimes
  grows a buffer returns the outcome and the caller branches — the same branch C++
  writes.

Rewrite and cost: none needed. The discipline is to publish a measure relation on
every writing signature and to name the narrowest window part the function
actually touches. Cost against C++: zero, and strictly better wherever an opaque
call would otherwise force a reload of a length or a re-check of a bound.

Rules consistent: yes — `writes`, `entry` and `ensures` form a closed loop, and
Rule 1 is what makes the invalidation set exactly the declared paths.
Task achievable: yes.
Verdict: accepted-fine

What changed and why: Rule 11's new sentence on `entry(p)` persistence supplies
the step round one had to assume, and Rule 6's window parts let an appending
callee leave the caller's slot facts standing. The verdict is unchanged; its
ground is now complete and its margin over C++ is larger.

---

### 10-13: Rows, overlap permission, and the function-typed parameter

Question: does the overlap judgment see everything two adjacent statements can
reach, given that a statement may be a call through a function-typed parameter?

Round one: `rejected-zero-cost` (the cell wrote `accepted-unsafe`, reporting a
real race between two `par` arms through a closure's captured reference; the
reviewer corrected it, because Rule 9's body check rejects the capturing closure
before Rule 13 is ever consulted).

Strongest program: a helper that runs a passed function on two ranges, where the
passed function writes something no row names.

```text
fn run2(f: fn(s: &[u8]) writes(s), x: &[u8], y: &[u8])   writes(x), writes(y)
{
    f(x)                               // rows: writes(x)
    f(y)                               // rows: writes(y); disjoint by the caller's proof: may overlap
}

scratch: Box<Slots<u8>>
g = |s: &[u8]| { compress_into(s, &scratch) }       // captures &scratch
run2(g, &(*out)[0..mid], &(*out)[mid..n])           // (1)
touch(&scratch)                                      // (2) may (1) and (2) overlap?
```

Ordinary program, where everything a statement touches is an argument:

```text
kernel(&(*v)[0..mid], &(*out)[0..mid]);  kernel(&(*v)[mid..n], &(*out)[mid..n])   // may overlap
s1 = stats(&v);  s2 = stats(&v)                                                    // read/read: may overlap
a = build(&x)?;  b = build(&y)?                                                    // both allocate: may overlap
sink(move p);    use(&p)                                                           // may not: consumption is a write
place_back(&v, 1);  s = stats(&v)                                                  // may not: writes(*v) meets reads(*v)
```

Trace:

- Rule 13's test is stated over paths — "Two adjacent statements of one block may
  overlap when the first's write paths are disjoint from the second's read and
  write paths and vice versa, using the same path-overlap and
  index/range-disjointness judgment as Rule 10" — and a statement that is a call
  contributes its substituted row, by Rule 10's first sentence.
- The ordinary program is settled line by line: adjacent ranges are disjoint;
  "read/read overlap is allowed"; allocation is exempt by Rule 14's "Allocation
  and release carry no effect entry and never prevent two statements from
  overlapping"; and `place_back` beside `stats` is Rule 13's own printed
  rejection. Revision 4 also settles by-value consumption inside Rule 13 itself —
  "a by-value consumption counts as a write of the argument's place" — which
  round one had to import from Rule 10 clause 2 as an inference.
- The closure is where round one found a race, and Rule 9 is what forecloses it.
  `g` is passed where the declared type is `fn(s: &[u8]) writes(s)`. Rule 9: "A
  function body is checked against its own row: every statement's effect and every
  callee's substituted row must be covered by the declared row." The substituted
  row of `compress_into(s, &scratch)` is `writes(s), writes(scratch)`, and
  `writes(scratch)` is not covered by `writes(s)`, so `g`'s body fails its own
  check and (1) never reaches Rule 13. A benign capturing closure that only reads
  is rejected by the same clause, which is the right answer, because nothing
  distinguishes the two to the checker.
- The closing observation of round one survives and is now load-bearing: a capture
  is the only route by which two statements could reach a common path that no row
  names, because Rule 1 removes references from every aggregate, so without
  captures a statement can touch only what its arguments name. Closing that route
  by Rule 9 is what makes the row the whole truth, and therefore what makes Rule
  13's judgment sound.
- A bystander reference needs no special clause: `p = &(*v)[i]; grow(&v, cap);
  use(p)` is rejected twice over — by Rule 3 for the prefix write, and by the
  pairwise test if the two statements are offered for overlap.

Rewrite and cost: make the captured reference a parameter of the function-typed
parameter.

```text
fn run2(f: fn(s: &[u8], aux: &Box<Slots<u8>>) writes(s), reads(aux),
        x: &[u8], y: &[u8], aux: &Box<Slots<u8>>)   writes(x), writes(y), reads(aux)
{ f(x, aux)
  f(y, aux) }                       // reads(aux) on both sides: read/read, may overlap
```

The unsafe variant becomes `writes(aux)` in both rows, which Rule 13 refuses on
the spot rather than silently permitting. Cost against the C++ lambda: zero — the
captured pointer becomes an argument register, and protocol item 7 excludes extra
parameters from cost.

Rules consistent: yes — Rule 4 permits the capture, Rule 9 makes any effectful use
of it fail the body check, and Rule 13 therefore reads a row that is complete.
Task achievable: yes at zero cost.
Verdict: rejected-zero-cost

What changed and why: no rule moved the verdict — the reviewer's correction stands
on Rule 9's unchanged closing sentence. Revision 4 changes the setting in two
ways: with no `par` statement the question is whether two adjacent statements may
overlap, judged from the same rows, so the hole would have been a race in ordinary
straight-line code rather than inside a special construct; and Rule 13 now states
the by-value consumption clause itself, closing round one's complaint that the
clause was an inference rather than a statement.

---

### 10-14: Allocation failure at a call, and the heap that is not an effect

Question: how does a failing allocation inside a called statement interact with
the call-site rule, with linear obligations, and with a statement that may overlap
it?

Round one: `undecided-rule-gap` — Rule 14's own example put `?` inside both `par`
arms while Rule 13 defined `par` only by an acceptance test, leaving three things
unstated: whether the sibling completes, which error is produced when both fail,
and whether a completed arm's bindings exist on the propagating path.

Strongest program: two allocating calls that may overlap, with a linear resource
already held and an early exit in flight.

```text
fn build(src: &Box<Slots<Int>>) -> Result<Box<Slots<Int>>, Oom>   reads(src)

f  = open(path)                       // linear, live across both calls
ra = build(&x)                        // (A) reads(x)
rb = build(&y)                        // (B) reads(y): disjoint from (A): may overlap
match (ra, rb) {
    (Ok(a), Ok(b))   => { use(&a, &b); close(move f); Ok(()) }
    (Ok(a), Err(e))  => { close(move f); Err(e) }      // a is affine: released at scope exit (Rule 8)
    (Err(e), Ok(b))  => { close(move f); Err(e) }
    (Err(e), Err(_)) => { close(move f); Err(e) }      // the writer picks, deterministically
}
```

Trace:

- The call-site side is immediate and unchanged. Rule 14: "Allocation and release
  carry no effect entry and never prevent two statements from overlapping", so
  `build`'s row is `reads(src)` alone, the two statements read different
  containers, and Rule 13 grants the permission. "Allocation returns a `Result`
  and never traps", so there is no hidden control flow at the allocation itself.
- Release *through a path* remains an effect, by Rule 9's "`writes` covers
  writing, replacing, moving out of, and freeing the storage at the path". Rule
  6's `grow` frees the old block and declares `writes(*b)`, so it conflicts as
  expected. The two rules are consistent: Rule 14 exempts the heap, not the
  program's storage.
- Round one's gap is closed by one sentence of Rule 13: "Because the meaning is
  sequential, results, errors, early exits, and linear obligations are exactly
  those of the sequential program; nothing new is defined for the overlapped
  case." All three unstated items get the same answer at once. There is no arm to
  abandon — (A) has completed when control reaches (B), and if a `?` on (A) leaves
  the function then (B) never runs, exactly as written. The error reported is the
  one the sequential order produces. The bindings on any path are the sequential
  bindings.
- The linear side is decided by Rule 8, "Linear values must be consumed by an
  explicit operation on every exit path; the compiler never releases them". `?`
  manufactures an exit path, so `a = build(&x)?` is rejected while `f` is live and
  unconsumed, and the writer writes the `match` above. That is the branch C++
  writes too, so the cost is zero. Round one could not decide this inside a `par`
  arm because "exit path" had no meaning there; with sequential statements it is
  the ordinary rule.
- Rule 5's hand-back settles what an `Err` carries when the failing allocation took
  a payload: "A fallible allocation that takes a by-value payload hands it back on
  failure, so a linear payload is never lost" (8-14). `build` takes only a
  reference, so its `Err` is a bare `Oom`.
- One obligation lands on the trusted base rather than on the program, and it is
  worth naming precisely. Under a memory budget, which of (A) and (B) receives the
  last block depends on the order they reach the "internally synchronized"
  allocator, and Rule 13 requires the observed result to be the sequential one.
  Rule 14's defence covers only addresses — "Addresses are not observable, so
  allocator concurrency does not affect program determinism" — so the guarantee
  rests on Rule 13's sentence, which constrains when an implementation may take
  the overlap permission. Nothing about the program's meaning is left open by
  this; it is a requirement on the base.

Rewrite and cost: none needed. The `match` replaces `?` wherever a linear value is
live, which is one branch on a path that must do that work anyway; against a C++
version that collects two futures and handles `bad_alloc`, the same branch. If the
writer prefers `?`, it is available once no linear value is live.

Rules consistent: yes — Rule 13's sequential-meaning clause, Rule 14's exemption
and Rule 8's every-exit-path obligation now compose without a remainder in the
program; the only open item is an implementation obligation on the allocator.
Task achievable: yes, at zero runtime cost.
Verdict: accepted-fine

What changed and why: the deletion of the `par` statement plus Rule 13's sentence
"Because the meaning is sequential, results, errors, early exits, and linear
obligations are exactly those of the sequential program; nothing new is defined
for the overlapped case." Round one's three unanswerable questions were artefacts
of a construct that no longer exists.

---

### 10-16: Calls over a pool: allocation as a window write, and the parallel node map

Question: under the confirmed pool rule, what do the call-site checks cost a
program that allocates while it traverses, and can node work still overlap?

Round one: `rejected-real-cost` — (i) a two-write link edit needed `a != b`, fixed
for free by splitting into two single-effect calls; (ii) a reference to the parent
died at every allocation; (iii) two work statements on data-determined handles
could not overlap, recovered only by a dense-range map that visits free slots.

Strongest program: build and edit a graph in a pool, then map over its nodes.

```text
struct Node  { next: u64, prev: u64, data: u64 }
struct Arena { buf: Box<Slots<Node>> }
fn link(from: &Node, to: &Node, id_from: u64, id_to: u64)   writes(from), writes(to)

link(&(*g.buf)[a], &(*g.buf)[b], a, b)       // (i) rejected unless a != b is proved

p  = &(*g.buf)[parent]
id = alloc(&g, Node { next: 0, prev: 0, data: d })
(*p).data = id                               // (ii) accepted under revision 4

work(&(*g.buf)[h1])                          // (iii) h1, h2 come from node fields
work(&(*g.buf)[h2])                          //      may these overlap?
```

Ordinary program, addressing the pool as a dense range:

```text
map_range(&(*g.buf)[0..mid],  &(*live)[0..mid])
map_range(&(*g.buf)[mid..n],  &(*live)[mid..n])     // may overlap; each arm skips slots whose liveness byte is 0
id = alloc(&g, fresh)                                // and this may overlap either of them
```

Trace:

- (i) is Rule 10 clause 1 verbatim: two write paths sharing the root `*g.buf`,
  separated only by "indices or ranges proved distinct". In a doubly linked insert
  the endpoints are distinct by construction, but that construction is data. The
  repair is the statement form, not a comparison: two sequential calls,
  `set_next(&(*g.buf)[a], id)` then `set_prev(&(*g.buf)[b], id)`, each carrying a
  single effect, never enter the pairwise comparison at all. Rule 10's test is per
  call. Revision 4 adds two more free forms: Rule 6's generic `swap(p, q)`, whose
  "p and q may be the same place, then nothing happens", so a free-list splice
  needs no `i != j` branch; and the atomic update `place = f(place, args...)`,
  which edits a link in place with no reference and no prefix conflict. All are
  pure source changes with no runtime test.
- (ii) flips. `alloc`'s write paths are `(*g.buf).next` and `(*g.buf).len`; Rule 3
  invalidates only on "a proper prefix of p's path", and Rule 6 states that a live
  `r[i]` "never overlaps `r.next` or `r.free`". So the reference survives, the
  bound survives through Rule 16's `ensures (*a.buf).len == result + 1` read with
  Rule 11's `entry` sentence, and round one's per-allocation re-formation cost is
  withdrawn. It only returns at `grow(&b, cap)  writes(*b)`, which really does
  move the block.
- (iii) is unchanged and is the cell's residual cost: handles pulled out of node
  fields carry no distinctness fact, and Rule 11 supplies none ("There are no
  quantified facts over array elements"). P3's "parallel data writes are permitted
  by distinctness of nodes" is not met through traversal order.
- The dense-range map recovers the overlap by giving up traversal order: adjacent
  ranges are disjoint by arithmetic, which Rule 13 admits, and the liveness column
  is data, as Rule 12 requires ("occupancy that is determined by data is stored as
  data"). New in revision 4: the allocation may overlap the map as well, since
  `.next` and `.len` are disjoint from every live slot — with one caveat that
  should be stated because it is easy to lose. A map written as `for k in 0..
  (*g.buf).len` *reads* the measure, and `reads((*g.buf).len)` against the
  allocation's `writes((*g.buf).len)` withdraws the permission. Passing the range
  reference instead keeps it. Proving the bound statically buys the overlap;
  testing it at run time spends it.
- Identity remains the declared hole: a stale handle that is still in bounds reads
  the replacement node, memory-safely.

Rewrite and cost, per item:

- (i) one write per call, or `swap`, or the atomic update: zero.
- (ii) nothing: the reference survives. Where the pool must grow, reserve, or
  re-form after `grow` — one scaled add that folds into the following store.
- (iii) dense range map plus a liveness byte: one load, one compare and one
  predictable branch per slot, and the map visits free slots too, so the work is
  proportional to `(*g.buf).cap` rather than to the live count. Against a C++
  parallel walk of a free-list-managed pool this is about the same test; against a
  C++ walk over a compact live array it is the fragmentation factor, which the
  writer controls by compacting.
- Identity, where the task needs it: a generation word per slot and per handle —
  one load, one compare, one branch per dereference and 4 to 8 bytes per node and
  per handle.

Rules consistent: yes — Rule 16 composes with Rules 3, 6, 10, 11 and 13 without
contradiction.
Task achievable: rewrite — the graph, its edits and a parallel node map are all
expressible; traversal-ordered overlap is lost and stale-handle detection is paid
in data.
Verdict: rejected-real-cost

What changed and why: the slot-precise `alloc` row removes item (ii)'s cost
entirely and lets allocation overlap a node map, and Rule 6's `swap` and atomic
update give item (i) two more zero-cost forms. Item (iii), which is the cell's
real cost, is untouched: Rule 11's exclusion of quantified facts over elements is
unchanged.

---

### 12-13: Joins feeding statements that may overlap

Question: when a statement's argument comes out of a join, the target set
multiplies the disjointness obligations — does that cost the overlap permission?

Round one: `accepted-fine` — all four pairs discharge and the branch can always be
hoisted at zero cost. The reviewer corrected two details: `cols.a` versus `cols.b`
is "distinct fields", not "different roots"; and "the rewrite is always available"
is too strong, since a target set built from a data-determined index cannot be
enumerated by duplication.

Strongest program: a double-buffered stencil whose two range references are
selected by the same runtime condition, inside a loop that rebinds them.

```text
for t in 0..steps {
    p = if flip(t) { &(*cols.a)[lo..mid] } else { &(*cols.b)[lo..mid] }
    q = if flip(t) { &(*cols.b)[mid..hi] } else { &(*cols.a)[mid..hi] }
    kernel(p, &src)                  // writes(p), reads(src)
    kernel(q, &src)                  // may these two overlap?
}
```

Trace:

- Rule 2 gives `p` the target set `{(*cols.a)[lo..mid], (*cols.b)[lo..mid]}` and
  `q` the set `{(*cols.b)[mid..hi], (*cols.a)[mid..hi]}`, with "every check on it
  must hold for every member of the set".
- Rule 13 requires the two statements' paths to be disjoint "using the same
  path-overlap and index/range-disjointness judgment as Rule 10". That is four
  pairs. Two are distinct fields of one root, which Rule 10 accepts by its printed
  line `two(&a.x, &a.y)  // accepted: distinct fields` — not "different roots",
  since the root is `cols` for both. The other two are `[lo..mid]` against
  `[mid..hi]` on the same storage, "ranges proved distinct" from `lo <= mid <=
  hi`, which is Rule 13's own printed adjacent-range line. The permission holds
  with the target sets in place: the join multiplied the obligations and every
  member discharged them.
- Two revision-4 sentences make the loop itself legal, and round one could not
  appeal to either. Rule 2: "An index expression inside a path is evaluated when
  the reference is formed; the path records that value, and later assignments to
  the variables the expression used do not change it" — so a later `mid = mid + 1`
  does not move either target, and the four pairs stay the four pairs. Rule 2
  again: "A path has a static shape: a loop-carried rebinding may change only the
  index values inside the path, never extend the path through itself" — so the
  loop-carried rebinding of `p` keeps a two-member set across every iteration
  instead of growing one, which was round one's open question in 2-2.
- The failing variant is still instructive. Make `q`'s else arm
  `&(*cols.a)[lo..mid]`. At run time the two statements never coincide, because
  both conditionals test the same `flip(t)`, but that correlation is not a fact the
  candidate can carry: Rule 11's forms are affine comparisons over measures and
  integer values, and Rule 12 intersects each `if` independently at its own join,
  keeping nothing that relates the two. Rule 2's "every member" clause then
  withdraws a permission that always would have been sound.
- Round one flagged a non-decisive ambiguity about whether the end of a `par` is a
  Rule 12 join. It is gone: there is no `par`, the two statements are ordinary
  statements, and the facts after the second are the sequential ones (Rule 13).

Rewrite and cost: hoist the branch — duplicate the two statements into each arm of
one conditional.

```text
if flip(t) { kernel(&(*cols.a)[lo..mid], &src)
             kernel(&(*cols.b)[mid..hi], &src) }
else       { kernel(&(*cols.b)[lo..mid], &src)
             kernel(&(*cols.a)[mid..hi], &src) }
```

Each statement now names a literal path with no target set and the same
disjointness proof succeeds. Cost: one branch the original already executed,
duplicated source that the criteria do not charge for, and the full overlap
permission retained — zero.

The claim must be bounded as the reviewer required: the hoist is available for a
*source-level* branch join, which is always duplicable (a loop header by peeling
or unrolling by two), and Rule 4 guarantees that a target set can only arrive from
such a join, since a reference "cannot be returned" and "cannot be assigned into
any aggregate". It is not available for a target set built from a data-determined
index, `p = &(*buf)[hit]`, which no duplication enumerates; that case is priced in
13-16 as the scatter, not here.

Rules consistent: yes.
Task achievable: yes — a source-level join never costs the overlap permission.
Verdict: accepted-fine

What changed and why: Rule 2's index-snapshot sentence and its static-path-shape
sentence. The verdict is unchanged, but the loop form in the strongest program is
now licensed rather than assumed, and the `par`-join ambiguity round one parked has
been removed by deleting the construct.

---

### 13-14: Two statements that both allocate and both can fail

Question: when two adjacent statements both allocate, both carry a linear payload
and either may fail, what is the joint outcome?

Round one: `undecided-rule-gap` — revision 2's Rule 14 example wrote `par { a = build(&x)?; b =
build(&y)? }` and defined nothing about control flow leaving an arm; the cell also
observed that Rule 14's determinism sentence argues about addresses and does not
cover which arm receives `Err` under a budget.

Strongest program: two allocations that each swallow a linear resource, adjacent,
with no linear value left unowed on any path.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, n: u64 }

f1 = open(p1)
f2 = open(p2)
r1 = Box::new(Conn { f: move f1, n: 1 })      // Result<Box<Conn>, (Oom, Conn)>
r2 = Box::new(Conn { f: move f2, n: 2 })      // may these two statements overlap?

match (r1, r2) {
    (Ok(a), Ok(b))           => { serve(move a); serve(move b); Ok(()) }
    (Ok(a), Err((e, c)))     => { serve(move a); let Conn { f, .. } = move c; close(move f); Err(e) }
    (Err((e, c)), Ok(b))     => { serve(move b); let Conn { f, .. } = move c; close(move f); Err(e) }
    (Err((e, c1)), Err((_, c2))) => { let Conn { f: fa, .. } = move c1
                                      let Conn { f: fb, .. } = move c2
                                      close(move fa); close(move fb); Err(e) }
}
```

Trace:

- The overlap permission holds. Rule 13 counts "a by-value consumption ... as a
  write of the argument's place", so statement one writes `f1`'s place and
  statement two writes `f2`'s place — distinct locals, different roots, disjoint
  under Rule 10 clause 1. Rule 14 removes the allocator from the comparison
  entirely: "Allocation and release carry no effect entry and never prevent two
  statements from overlapping." Two allocating, linear-payload-carrying statements
  may overlap.
- Round one's three unanswered questions are answered by Rule 13's own sentence:
  "Because the meaning is sequential, results, errors, early exits, and linear
  obligations are exactly those of the sequential program; nothing new is defined
  for the overlapped case." There is no arm to abandon; both statements have
  completed when the `match` runs; the error the program reports is the one the
  writer's `match` selects, not the one a scheduler selects; and both bindings
  exist on every path.
- The failure arms are writable at all only because of Rule 5: "A fallible
  allocation that takes a by-value payload hands it back on failure, so a linear
  payload is never lost", printed as `Result<Box<Node>, (Oom, Node)>: on Err the
  Node comes back`. Under revision 2 this program leaked one `File` per failing
  allocation with no rule violated (8-14); the two failing together leaked both.
- Rule 6 supplies the exit from the returned aggregate: "A move out of a field or
  out of Box content consumes the whole owner ... (take it in the same
  destructuring)", so `let Conn { f, .. } = move c` extracts the `File` and
  releases the `u64`.
- Rule 8 then checks every path: "Linear values must be consumed by an explicit
  operation on every exit path; the compiler never releases them." All four arms
  consume both `File`s. `?` is unavailable here, precisely because it would create
  an exit path on which the other `Conn` is not destructured — which is the correct
  answer, not a gap.
- The residue is the same one 10-14 names and it is not about this program's
  meaning: under a budget, which of the two statements receives the last block is
  decided inside the "internally synchronized" heap, and Rule 13 requires the
  observed outcome to be the sequential one, while Rule 14's defence covers only
  addresses. That constrains the implementation's use of the permission, not the
  program.

Rewrite and cost: none needed. The `match` is exactly what a C++ version with two
`new`s and a `bad_alloc` path writes, minus the destructor and the unwinder; the
WF form does the same allocations with the same permission to overlap and one
post-join branch. Zero.

Rules consistent: yes.
Task achievable: yes, at zero runtime cost, with every linear obligation
discharged on every arm.
Verdict: accepted-fine

What changed and why: three revision-4 sentences, each closing one of round one's
unstated items — Rule 13's "results, errors, early exits, and linear obligations
are exactly those of the sequential program"; Rule 5's payload hand-back; and Rule
6's whole-owner destructuring, which is what lets the handed-back payload be taken
apart. The construct whose semantics round one could not find has been deleted.

---

### 13-16: Overlap and pools: allocating from one arena, and scattering into one pool

Question: may two statements allocate from a shared pool, and may they write nodes
they select by data?

Round one: `rejected-real-cost` — the scatter is rejected three ways over; P2's
three independent block fills are accepted for free through `alloc`'s `entry`
contract; parallel allocation from one arena is rejected.

Strongest program: a parallel scatter over a pool (histogram, PageRank push).

```text
fn push_edges(nodes: &Box<Slots<Node>>, part: &[Edge])   reads(part), writes((*nodes).filled)
                                                          // the written index comes from the edge data
push_edges(&nodes, &(*edges)[0..m])
push_edges(&nodes, &(*edges)[m..n])          // may these overlap?
```

Ordinary programs, both of which revision 4 admits:

```text
i1 = alloc(&a, blank);  i2 = alloc(&a, blank);  i3 = alloc(&a, blank)
fill(&(*a.buf)[i1]);    fill(&(*a.buf)[i2]);    fill(&(*a.buf)[i3])      // each adjacent pair may overlap

part = &(*dst)[0..m]                          // formed before
append(&dst, &src)                            // writes(dst.free), writes(dst.len), writes(src.filled), writes(src.len)
kernel(part, &cfg)                            // writes(part): may overlap the append
```

Trace:

- The scatter is still rejected, and for the same three reasons. Both statements
  declare `writes((*nodes).filled)` — identical paths, so Rule 13's "the first's
  write paths are disjoint from the second's read and write paths" fails. Rule 9
  cannot narrow the row, because the written index is read out of the edge data
  rather than supplied at the call: "an index enters an effect only through an
  argument, evaluated once at the call". Rule 11 cannot prove the scattered targets
  distinct — "There are no quantified facts over array elements" — and no atomic
  exists to make the conflict legal ("channels or atomics" are listed under "Not in
  this candidate").
- Two allocations from one pool are still rejected: both write `(*a.buf).next` and
  `(*a.buf).len`, identical paths. Under revision 2 the reason was that `alloc`
  wrote the whole container; the reason is now narrow, and the narrowing is what
  the two ordinary programs exploit.
- P2's three independent fills are accepted with no runtime check, and the proof is
  worth spelling out because it is Rule 16 earning its place. `alloc` gives
  `ensures result == entry(*a.buf).len` and `ensures (*a.buf).len == result + 1`;
  Rule 11's "a fact known before the call about a measure of an argument survives
  as a fact about `entry(p)` of that argument" ties each call's `entry` term to the
  previous call's `ensures`, so `i2 == i1 + 1` and `i3 == i2 + 1`, all affine. Rule
  10 clause 1's "indices or ranges proved distinct" is then discharged for each
  pair. Round one needed a nested `par`, and had to argue that nesting was
  permitted; revision 4 needs only three statements and the pairwise test.
- The second ordinary program is new in revision 4 and is the strongest positive
  result here: a bulk producer and a consumer may overlap on one window.
  `append(&dst, &src)` declares `writes(dst.free)`, and Rule 6 says a live `r[i]`
  "never overlaps `r.next` or `r.free`", while `part` is a range of live slots
  formed before the call. `dst.len` is a separate target, and the consumer holds a
  range reference rather than reading the measure. Also note that `part` stays
  valid: `append`'s `ensures dst.len == entry(dst).len + entry(src).len` only grows
  the window, so the formation fact `m <= dst.len` survives. A build phase and a
  process phase on one storage no longer have to be separated in time.
- The mirror of that is unchanged: a statement that reads `(*a.buf).len` at run
  time to bound itself may not overlap an allocation, since `reads` meets `writes`
  on the measure. And a consumer beside `truncate` (library code: a loop of
  `take_back`, writing `r.last` and `r.len`) is rejected, so Rule 16's stale-index
  hazard can never become a data race — it stays a sequential logic error.

Rewrite and cost, for the scatter: (a) privatize — one accumulator array per
worker and a reduction afterwards, costing `P * (*nodes).len * 8` bytes and a merge
pass, which for a graph with millions of vertices exceeds the traversal itself; or
(b) transpose — build the in-edge list once and pull instead of push, costing
`|E| * 8` extra bytes held permanently plus one transposition pass, after which the
statements write disjoint node ranges and are admitted with no check. Against C++
with one atomic add per edge, (a) costs the reduction and the memory and (b) costs
the transposed structure; both are real and neither disappears with tuning. For
allocation, the rewrite is one pool per worker, or 1-16's pre-filled block with
per-worker ranges, both zero per-hop cost.

Rules consistent: yes.
Task achievable: rewrite for independent block filling, for build-beside-process,
and for regular per-node work; no for concurrent scatter.
Verdict: rejected-real-cost

What changed and why: the slot-precise `alloc` row and Rule 6's `append` row over
`dst.free` open build-beside-process overlap that revision 2 refused, and deleting
`par` removes round one's nesting argument for the three-block case. The scatter,
which is the strongest program and the source of the verdict, is untouched: Rule
11's exclusion of quantified facts and the absence of atomics are unchanged.

---

### 14-16: One heap plus pools-as-usage against a byte arena with a free list

Question: with one heap, no second store and arenas expressed as window usage, can
several independently usable blocks be filled at once, one released and its bytes
reused, the others unaffected?

Round one: `rejected-real-cost` — the bump-allocated prefix works and is cheap;
the reuse case fails, because the invariant that every offset on the free list is
disjoint from every live block is a quantified fact over the contents of the free
list; rewrites are k(k-1)/2 static disjointness tests or per-worker sub-arenas.

Strongest program: the byte arena, written out under revision 4.

```text
struct Arena { buf: Box<Array<u8>>, head: u64, free: Box<Slots<Blk>> }

fn alloc_bytes(a: &Arena, n: u64) -> Result<u64, Oom>   writes(a.head), reads(*a.buf)
    contract { ensures when Ok: result == entry(a).head;
               ensures when Ok: result + n <= (*a.buf).len; }

o1 = alloc_bytes(&a, 64)?;  o2 = alloc_bytes(&a, 64)?;  o3 = alloc_bytes(&a, 64)?
                                            // o2 == o1 + 64, o3 == o2 + 64 from the contracts: affine

fill(&(*a.buf)[o1..o1+64])
fill(&(*a.buf)[o2..o2+64])                  // each adjacent pair: ranges proved distinct, may overlap
fill(&(*a.buf)[o3..o3+64])

free_bytes(&a, o2, 64)                      // writes(a.free ...), writes(a.head)
o4 = alloc_bytes_reuse(&a, 32)?             // comes off the free list: o4's value is data
read(&(*a.buf)[o1..o1+64])                  // still legal

fill(&(*a.buf)[o1..o1+64]);  fill(&(*a.buf)[o4..o4+32])     // may NOT overlap
```

Trace:

- Rule 14 fixes the shape before anything else: one heap, and "There are no store
  or region parameters anywhere". So the arena is a block on that heap, its blocks
  are ranges into it, and there is no second storage whose disjointness could come
  from the type. `Box<Array<u8>>` is the right shape: every slot always holds a
  value, so no window boundary is involved and `a.head` is ordinary data the
  program maintains.
- The bump-allocated prefix works and is cheap. The three offsets are related
  affinely through the contracts, carried across the calls by Rule 11's "a fact
  known before the call about a measure of an argument survives as a fact about
  `entry(p)`", so Rule 13's requirement is met through Rule 10 clause 1's "ranges
  proved distinct". Three adjacent statements, three pairwise-checked pairs; round
  one had to argue that a two-armed `par` could be nested, and that argument is no
  longer needed.
- The reuse case is where it fails, and nothing in revision 4 touches the reason.
  Once `o4` comes off the free list its value is data. Rule 11 admits "affine
  comparisons over measures and integer values, refinement facts from a dominating
  branch, loop-header invariants ..., explicit `use` steps ..., and callee
  contracts", and states "There are no quantified facts over array elements ... and
  no per-slot occupancy facts". The invariant that actually holds — every offset on
  the free list is disjoint from every live block — is a quantified fact over the
  contents of `a.free`, so it cannot be stated, and no contract on
  `alloc_bytes_reuse` can produce a disjointness relation against an arbitrary
  caller-held `o1`. The permission is withheld for a pair that is always disjoint
  at run time.
- Rule 16 covers the identity half, and revision 4 makes it worse rather than
  better. Under revision 2 an old reference into the freed block could not survive,
  because Rule 3 killed it at the `writes(a.head)` of any subsequent call — but
  `a.head` is not a prefix of `(*a.buf)[o2..o2+64]`, so on a careful reading it
  never did, and revision 4 removes the doubt in the other direction: "storage that
  still exists stays readable, and what the program considers 'freed' is the
  program's own bookkeeping." A saved `o2`, and a range reference formed from it,
  both still read bytes that now belong to the reused block — "a logic error, not a
  memory error". Detection costs one generation word per block and one compare per
  use (3-16).
- Reset is the candidate's win: for a `Box<Array<u8>>` it is one store to
  `a.head`, matching a C++ bump-arena reset exactly; for a window of affine
  elements it is the same recursive release C++ destructors would run, and for
  linear elements Rule 6 refuses to release them at all ("No operation releases a
  linear element"), so `truncate` hands each one back and the caller must consume
  it — which closes round one's silent-discard hole for `Slots<File>`.

Rewrite and cost, two options:

1. A dominating disjointness test: `if o4 + 32 <= o1 || o1 + 64 <= o4 { fill(...)
   fill(...) } else { ... }` supplies the refinement fact (Rule 11) at two compares
   per pair. Pairwise permission means k blocks need k(k-1)/2 statically written
   tests, and for a worker count that is a runtime value the form does not exist.
   Usable at k = 2 or 3, not for a worker pool.
2. Partition the arena into k contiguous sub-arenas by arithmetic and give each
   worker its own, so every range is proved disjoint from the arithmetic and each
   worker bump-allocates inside its own slice. Rule 13 keeps the counted-loop forms
   — "per-element maps, adjacent ranges passed to a helper, admitted reductions ...
   with the ranges written as range references" — so this scales to a runtime `k`
   where option 1 cannot. Costs: each slice is sized for its worst case, so peak
   memory rises by roughly k times the per-worker imbalance, and a block freed by
   one worker cannot be reused by another without a sequential rebalancing pass —
   real fragmentation, not source awkwardness. Against a C++ shared arena with an
   atomic bump pointer, which this candidate cannot express, the trade is an atomic
   for memory; against C++ per-thread arenas, the common production choice, the
   cost is zero.

Rules consistent: yes — every rejection above is a stated rule doing its job.
Task achievable: rewrite with a policy change. A free list feeding blocks that are
then filled concurrently is not achievable; contiguous per-worker sub-arenas
achieve the concurrent fill, and generation words achieve stale-handle refusal.
Verdict: rejected-real-cost

What changed and why: the verdict and its ground are unchanged, because Rule 11's
fact vocabulary is unchanged. What revision 4 changes is the k-block form — three
adjacent statements instead of a nested `par`, with the counted-loop forms carrying
a runtime worker count — and the identity side, where Rule 16's new sentence about
references into a user-level allocator states plainly what revision 2 left to
inference.

---

### 15-16: Owned returns as pool handles

Question: does "positions come back as indices with a bounds relation" compose with
the confirmed pool rule well enough to build and traverse a mutable graph at C++
cost?

Round one: `rejected-real-cost` — the interface is right and the capacity re-test
is free, but Rule 3 cost one reference re-formation per allocation and every
traversal hop costs a bounds compare. The reviewer added one repair: a recursive
`build` needs its own monotonicity `ensures`, since nothing otherwise states that
it does not shrink the window.

Strongest program: a recursive builder over one pool, whose caller holds a
reference to an already-built node across the whole recursion.

```text
struct Arena { buf: Box<Slots<Node>> }
struct Node  { left: u64, right: u64, data: u64 }         // handles, never references (Rule 1)

fn alloc(a: &Arena, x: own Node) -> u64   writes((*a.buf).next), writes((*a.buf).len)
    contract { requires (*a.buf).room > 0;
               ensures result == entry(*a.buf).len;
               ensures (*a.buf).len == result + 1; }

fn build(a: &Arena, d: u64) -> Result<u64, Full>   writes((*a.buf).next), writes((*a.buf).len)
    contract { ensures (*a.buf).len >= entry(*a.buf).len;         // monotonicity: an affine comparison
               ensures when Ok: result < (*a.buf).len; }
{
    if d == 0 { return Ok(alloc(a, leaf())) }
    l = build(a, d - 1)?
    r = build(a, d - 1)?
    if (*a.buf).room > 0 { return Ok(alloc(a, Node { left: l, right: r, data: 0 })) }
    return Err(Full)
}

root = &(*a.buf)[known]            // formed before the recursion
id   = build(&a, depth)?
(*root).data = id                  // accepted under revision 4: build's row names only .next and .len
```

Trace:

- Rule 15 and Rule 4 fix the interface: "Positions found by a search are returned
  as indices with a bounds relation in `ensures`", so `build` and `alloc` return
  `u64` and the caller forms `&(*a.buf)[h]`. The one address computation the caller
  pays is the one a C++ callee performed before returning its pointer.
- Rule 16 supplies the pool and the `alloc` row, now stated as the append slot and
  the length, which by Rule 6 is exactly `place_back`'s row. The consequence that
  matters for this cell: `build`, a user function that only appends, declares that
  same row — Rule 6 prints the pattern, "A user function that only appends declares
  the same row as `place_back`" — so the caller's `root` survives an entire
  recursive construction. Round one charged one reference re-formation per
  allocation; that charge is withdrawn in full.
- Rule 10's recursion clause, "Recursion is checked through contracts, never by
  unfolding bodies", means `alloc`'s `requires (*a.buf).room > 0` must be re-proved
  at each site. After two recursive calls the only facts about the window are
  `build`'s own `ensures`, and the precondition that would make the test
  unnecessary — `(*a.buf).len + 2^d <= (*a.buf).cap` — is not an affine comparison
  over measures, so Rule 11 does not admit it. The writer therefore tests capacity
  before each allocation, which is exactly what a C++ bump allocator does, and
  returns a typed failure. Zero cost.
- The reviewer's repair is admitted by the rules as stated: `ensures (*a.buf).len
  >= entry(*a.buf).len` is "an affine comparison over measures", and Rule 11's
  `entry` sentence is what lets the caller compose it with what it knew. Without it
  the second recursive call destroys the first's bound and `l < (*a.buf).len`
  cannot be recovered.
- Failure needs no payload hand-back here, and that is worth recording as a
  contrast with `Box::new`: `alloc` cannot fail, because `requires (*a.buf).room >
  0` is a precondition rather than a `Result`, so a linear payload entering a pool
  is never in flight across a fallible boundary (8-14). A pool that must grow calls
  `grow(&b, cap)  writes(*b)`, which does kill every live reference, so the builder
  reserves.
- Rule 16's identity statement is where the requirement is not met: "A stale index
  that is still in bounds names the current occupant of that slot: a logic error,
  not a memory error." After a reset the bounds facts die with the write, so a
  stale handle is refused while it is outside the window; once new allocations pass
  it again the same handle silently names a new occupant.

Rewrite and cost against a C++ pointer graph:

- Per traversal hop: one scaled add on a base that hoists — and now hoists across
  allocations too, since nothing invalidates it — plus one bounds compare, because
  the next handle comes out of a node field and "every stored handle is in bounds"
  is a quantified fact over elements that Rule 11 excludes. That is the cell's
  residual real cost. The power-of-two trick `h & (cap - 1)` is not an affine
  comparison and the bitmask fact is deferred.
- Per allocation: one capacity compare, which C++ pays too, and nothing else.
- Per identity check, only where the program needs it: one generation word per slot
  and one compare per dereference — what a generational C++ or Rust arena pays.
- Parallel node work stays available in the admitted shapes: adjacent ranges
  `map(&(*a.buf)[0..mid]); map(&(*a.buf)[mid..len])` may overlap, and under
  revision 4 an append may overlap them as well (13-16), while a map that follows
  handles is the scatter case.

Rules consistent: yes — the residual cost comes from Rule 11's exclusion of
quantified facts, not from Rule 15 or Rule 16.
Task achievable: rewrite — the graph is built, traversed and reset with owned
handles, with weaker identity than the task asks for.
Verdict: rejected-real-cost

What changed and why: Rule 6's "A user function that only appends declares the same
row as `place_back`" plus the window-part overlap sentence remove round one's
per-allocation re-formation entirely, and Rule 11's `entry` sentence grounds the
monotonicity repair the reviewer asked for. The per-hop bounds compare, which is
the reason for the verdict, is unchanged.

---

### 16-16: Rule 16 with itself: slot reuse meets pool reset

Question: when two pool usages compose — per-slot reuse through a free list and
whole-pool reset — does a handle still name what the program thinks it names, and
what does detection cost?

Round one: `accepted-unsafe` — the composition is accepted and memory-safe, the
only check that ever fires is the bounds test, and it fires non-deterministically
with input size; the generation-plus-epoch rewrite costs two loads, two compares
and one to two branches per dereference and 12-byte handles instead of 8.

Strongest program: a reset between compiler passes, with handles surviving in a
side table, slot-level reuse inside each pass, and — new under revision 4 — a live
reference held across the program's own free.

```text
struct Arena { buf: Box<Slots<Node>>, free_head: u64, epoch: u32 }
side: Box<Slots<u64>>                   // handles kept across the reset: owned data, entirely legal

// pass 1
h  = alloc(&a, n1)                      // h == 7
side[t] = h                             // Rule 1 permits this: a handle is a value
p  = &(*a.buf)[h]                       // valid while 7 < (*a.buf).len
free_slot(&a, h)                        // free list: the program's own bookkeeping
h2 = alloc_reuse(&a, n2)                // returns 7 again: ABA within one pass
x  = (*p).data                          // (1) p is still valid and reads the NEW occupant
...
truncate(&a.buf, 0)                     // library code: a loop of take_back
y  = (*p).data                          // (2) rejected

// pass 2
for .. { alloc(&a, ...) }               // (*a.buf).len climbs back past 7
z = (*a.buf)[side[t]].data              // (3) in bounds, initialized, correctly typed, wrong object
```

Trace:

- Rule 16 states the acceptance outright, and it is now confirmed rather than
  provisional: "A stale index that is still in bounds names the current occupant of
  that slot: a logic error, not a memory error. Programs that need to detect it
  keep a generation number as data." The cell's job is to price the composition,
  not to dispute the ruling.
- (1) is new in revision 4 and widens the hole from indices to references. Rule 3
  invalidates on "a proper prefix of p's path"; `alloc_reuse` writes
  `(*a.buf)[h2]`, which is the same place under a different index, and Rule 3 says
  "Writing the storage at p's path or below it (a content write) does not
  invalidate p." Rule 16 endorses it in terms: "The same holds for a reference into
  a user-level pool or allocator: storage that still exists stays readable, and
  what the program considers 'freed' is the program's own bookkeeping." Under
  revision 2 the reference died at the next allocation and the hole was
  index-only; that accidental protection is gone.
- (2) shows the one place the composition does catch itself, and it is the window
  boundary rather than identity. `truncate` is library code, "a loop of
  `take_back`", and `take_back` writes `r.last` and `r.len`; Rule 6 says a live
  `r[i]` "overlaps `r.last` unless `i != r.len - 1` is proved", and more decisively
  its `ensures r.len == entry(r).len - 1` does not carry the formation fact, so
  Rule 6's own sentence applies: "`take_back`'s does not, so p dies at a
  `take_back`." The reference is refused for the rest of the program, since
  "Validity is re-established only by forming the reference again."
- (3) is the round-one finding, unchanged: immediately after the reset the handle
  is caught by the Rule 7 bound test; once pass 2 has allocated eight nodes the
  same handle passes the same test and reads a live node of the wrong identity.
  Detection is load-dependent — it fires on small inputs and stops firing on large
  ones, the worst possible shape for a latent defect, and a direct product of Rule
  16 composed with itself.
- Memory safety survives intact and the audit is worth restating precisely. Rule 6:
  every slot in `[0, len)` holds a value and "no program point can observe a slot
  inside the window as empty", so there is no uninitialized read. Rule 7's test
  keeps the access in bounds. Rule 12: "every path is either wholly present or the
  program cannot name it", so there is no hole. Rule 8 survives too, and revision 4
  strengthens it: `truncate` built from `take_back` returns each element to the
  caller, and Rule 6 states "No operation releases a linear element", so resetting
  a `Slots<File>` forces the caller to close every file — round one's silent-
  discard hole is closed by construction. A stale handle used with `swap_remove`
  (now `swap` then `take_back`) removes the *wrong* value, never the same one
  twice, so nothing is duplicated or leaked.
- The self-interaction proper is unchanged: the two reuse mechanisms need different
  detectors and one erases the other's data. Per-slot generation counters live in
  the slots, and a reset releases the slots, taking the counters with them; a
  pool-level epoch survives the reset but says nothing about slot reuse inside a
  pass. Neither alone suffices once both usages are present, and Rule 16 does not
  warn the writer.
- Nesting is sound: a `Box<Slots<Arena>>` relocates inner headers on growth, which
  Rule 1's consequence clause makes safe ("any value can be relocated by copying
  its bytes"), and inner handles are indices, so only the identity question
  compounds.

Rewrite and cost: carry both detectors.

```text
struct Slot  { gen: u32, node: Node }
struct Arena { buf: Box<Slots<Slot>>, epoch: u32, next_gen: u32 }
struct H     { idx: u32, gen: u32, epoch: u32 }        // owned: storable, returnable

fn get(a: &Arena, h: H) -> Option<u64>   reads(a)
    contract { requires h.idx < (*a.buf).len; }
{
    s = (*a.buf)[h.idx]
    if s.gen == h.gen && a.epoch == h.epoch { Some(s.node.data) } else { None }
}
// reset: truncate(&a.buf, 0); a.epoch = a.epoch + 1     — still O(1)
```

- Reset stays O(1) because the epoch is one counter; bumping a per-slot generation
  across the whole pool would make reset O(cap), which is the naive fix and much
  worse.
- Per dereference against C++'s raw `Node*`: one generation load (same cache line
  as the node, no extra miss), one epoch load (pool header, cache-hot), two
  compares, one branch, plus the caller's handling of the `Option` — a second
  branch on the hot path or a `None` arm that must invent behaviour. The Rule 7
  bound compare is already counted in 1-7.
- Handle footprint grows from 8 bytes to 12, which costs real bandwidth in a large
  side table or edge list.
- Against safe Rust: identical — this is what a slotmap does and it is the only
  safe form available there either. Against C++ that also wants detection:
  identical. Against C++ raw pointers: two loads, two compares and one to two
  branches per dereference that C++ does not pay — C++ pays instead with undefined
  behaviour on precisely the same mistake, where WF has a well-defined read of the
  wrong object.

Rules consistent: yes — Rule 16 composes with itself without contradiction. Both
the free list and the reset are "usage", nothing forbids using both, and the result
is well defined at every step.
Task achievable: yes for a memory-safe pool at the costs derived in 1-16 and 7-16;
rewrite for the identity guarantee, since "removal invalidates every path to m" is
not delivered by the rules and must be delivered by the program's own data.
Verdict: accepted-unsafe

What changed and why: Rule 16 is confirmed, and its new sentence about a reference
into a user-level pool extends the accepted identity hole from indices to
references, which the slot-precise `alloc` row makes reachable. Against that, Rule
6's "No operation releases a linear element" plus `truncate` as a `take_back` loop
close the resource half of the composition. The verdict is unchanged; its scope is
wider and its resource leak is gone.

---

## Sheet summary

| Cell | Round one (corrected) | Revision 4 | Moved by |
|---|---|---|---|
| 1-16 | rejected-real-cost | rejected-zero-cost | Rule 16's slot-precise `alloc` row; Rule 6's live-slot/`r.next` overlap sentence |
| 2-4 | rejected-zero-cost | rejected-zero-cost | unchanged; Rule 9's body check still decides it |
| 3-16 | accepted-unsafe | accepted-unsafe | Rule 16's new reference sentence widens the hole |
| 4-6 | accepted-unsafe | accepted-unsafe | `insert_at`/`remove_at` widen it from one slot to a whole suffix |
| 4-16 | accepted-unsafe | accepted-unsafe | Rule 16 confirmed; shifting operations add a second route |
| 5-16 | rejected-real-cost | rejected-real-cost | the parallel-allocation term shrinks; the per-hop compare stands |
| 7-16 | rejected-real-cost | rejected-real-cost | the builder's re-formation cost is withdrawn; the per-hop compare stands |
| 8-14 | accepted-unsafe | accepted-fine | Rule 5's payload hand-back; Rule 6's whole-owner destructuring |
| 10-11 | accepted-fine | accepted-fine | Rule 11's `entry` sentence grounds the presupposed step |
| 10-13 | rejected-zero-cost | rejected-zero-cost | unchanged verdict; Rule 13 now states the consumption clause itself |
| 10-14 | undecided-rule-gap | accepted-fine | Rule 13's sequential-meaning clause; `par` deleted |
| 10-16 | rejected-real-cost | rejected-real-cost | item (ii) now free; the data-indexed overlap still refused |
| 12-13 | accepted-fine | accepted-fine | Rule 2's index-snapshot and static-path-shape sentences |
| 13-14 | undecided-rule-gap | accepted-fine | Rule 13's sequential-meaning clause; Rule 5; Rule 6's destructuring |
| 13-16 | rejected-real-cost | rejected-real-cost | build-beside-process overlap gained; the scatter unchanged |
| 14-16 | rejected-real-cost | rejected-real-cost | k-block form simplified; the free-list fact still unstatable |
| 15-16 | rejected-real-cost | rejected-real-cost | the per-allocation re-formation is withdrawn |
| 16-16 | accepted-unsafe | accepted-unsafe | hole widened to references; the linear-reset leak closed |

All three cells this sheet inherited as `undecided-rule-gap` (2-4 as the cell wrote
it, 10-14, 13-14) are decided under revision 4, and every one of them was decided
by the removal of the `par` statement or by a sentence revision 3/4 added.

Three text items the owner may want, none of which blocked a derivation:

1. Rule 16's `alloc` comment reads "see the open proposal", but "Proposed
   additions awaiting owner ruling" now says "None. The window-parts vocabulary was
   adopted into Rule 6 in revision 4." The row is determinate from Rule 6; the
   cross-reference is stale.
2. Rule 13 grants the permission to "Two adjacent statements of one block". For
   three or more heterogeneous statements that pairwise qualify, the text does not
   say whether all of them may overlap at once. The arithmetic cases in 13-16 and
   14-16 escape through Rule 13's counted-loop forms, so no cell is blocked, but a
   k-way fan-out of *different* statements has no stated licence.
3. Rule 13 says "results, errors, early exits ... are exactly those of the
   sequential program" while Rule 14 exempts the allocator from the comparison and
   defends determinism only for addresses ("Addresses are not observable"). Under a
   memory budget these meet: which of two overlapping allocating statements
   receives the last block decides which returns `Err`. The program's meaning is
   not in doubt — Rule 13 fixes it — but the obligation this places on the trusted
   base is not written down anywhere.
