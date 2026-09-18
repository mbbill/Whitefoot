# Engineering task under candidate x1 revision 4 — mutable compiler IR graph

Derived against `CANDIDATE-X1.md` revision 4 and only that file. Round one's
derivation (`matrix-x1/task-mutable-graph.md`, against revision 2) was read first;
this file keeps its conclusions where revision 4 does not touch them and states in
§1 exactly what moved. Nothing under "Not in this candidate" is used: no `par`
statement, no permission markers, no regions, no channels or atomics, no
destructors, no traps, no returned references, no partial moves.

Notation is the rule file's: measures and window parts are members (`r.len`,
`r.cap`, `r.room`, `r.next`, `r.last`, `r.filled`, `r.free`); `len_of` and
`DynBox` do not appear; two adjacent statements are written adjacent and the text
says whether they may overlap. Runtime performance is the only cost counted
(protocol item 7): verbosity, extra parameters and duplicated code are not costs.

Task: nodes in a pool with edge lists; insert and delete nodes and edges; traverse
cycles; reuse deleted node slots; and hold a node reference across an append
performed by a user-level function that declares the window parts in its row.

---

## 1. What revision 4 changed for this task

Seven of the eight round-one conclusions that revision 4 touches move, and the
central cost does not. Ordered by how much each moves the numbers.

| # | Round one (revision 2) | Revision 4 | Runtime delta |
|---|---|---|---|
| 1 | A user-level append forced `writes(g.nodes)` whole, so every reference into the pool died at every node creation across a separate-compilation boundary (round one T5). | `add_node` declares `writes((*g.nodes).next), writes((*g.nodes).len)`; Rule 6 states "a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`". The reference survives. | −1 block-pointer load, −1 index multiply-add per held reference per node creation. This is the assignment's headline case and it now works. |
| 2 | The O(1) use-list splice for RAUW was refused (no partial move); the rewrite copied entry by entry with a capacity test per entry. | `swap(p, q)` applies "to any owned place, not only to window slots" and its two arguments "may be the same place", so two `Box<Slots<Edge>>` fields exchange in two word stores; `append(&dst, &src)` concatenates in "one memcpy". | −*u* capacity tests and −*u* branches per RAUW of a value with *u* uses; the splice becomes two stores (empty target) or one realloc plus one memcpy (non-empty target). |
| 3 | Ordered operand-list editing had no primitive: a shift loop of `replace` calls, two moves per shifted element. | `insert_at` / `remove_at`, each "one memmove". | −1 branch and −1 move per shifted element; an operand insert is now one memmove, as in C++. |
| 4 | `add_edge` allocated inside the mutation, so it carried an `Oom` arm and a half-edge rollback (`_ = pop(sl)`). | Growth is a separate `grow`, and `place_back`'s `requires r.room > 0` lets the caller hoist it; the mutator has no failure path. | −2 capacity tests, −1 error branch, −1 rollback path per edge insertion. |
| 5 | Whether a write to an inner list invalidated a fact about the pool's length was gap G2; the strict reading cost a length reload and a branch per drained edge. | Measures are named parts: "The measure `r.len` is itself a write target." A row that writes `(*g.nodes).filled` does not write `(*g.nodes).len`, so the length fact survives with no `ensures` clause. G2 is closed. | −1 length load, −1 compare and branch per drained edge and per worklist pop under the strict reading; 0 under the reading round one used. |
| 6 | Two edge insertions into distinct nodes could overlap only if written inline at the call site, because a row could not carry the index (gap G1). | Rule 9: paths "may continue through fields, `*`, payload steps, and whole-index or range positions supplied as arguments", and "an index enters an effect only through an argument". `add_edge`'s row is slot-precise. G1 is closed in the permissive direction. | 0 (round one already recovered the parallelism by inlining, which is not a cost), but the overlap now survives separate compilation. |
| 7 | A node carrying an owned lattice value could not be updated in place without a dummy value to `replace` with. | The atomic update `place = f(place, args...)` "on any owned place with extra args" commits in place, "no program point lies between". | −1 allocation per in-place update of an owned per-node value. New restriction: see CX2. |
| 8 | Per-hop bounds compare and branch on every edge traversal; the bitmask escape was open as gap G3. | Unchanged. Rule 11 still has no quantified facts, and the bitmask fact is listed under "Deferred to a future concurrency and layout round", so the escape is closed rather than open. | 0. This remains the dominant cost of the task (§8, CX1). |

Two further revision-4 sentences change the shape of the argument rather than a
number. Rule 16 is "Confirmed by the owner", so round one's §11 contingency ("if
the owner refuses Rule 16 the task is not achievable at all") collapses to a
single note: the pool-with-index-handles representation is now a decided part of
the candidate, and the stale-handle disposition ("a logic error, not a memory
error") is what the generation word in §3 is written against. And Rule 2's new
last clause — "a loop-carried rebinding may change only the index values inside
the path, never extend the path through itself" — refuses a Box-pointer walk
outright, so the pool is no longer merely the surviving representation, it is the
only one whose traversal loop is expressible.

---

## 2. The question this task tests

Can candidate x1 revision 4 carry a mutable cyclic pooled IR — bidirectional edge
lists, node and edge insertion and deletion, slot reuse, cyclic traversal, and a
live node reference held across a user-level append — with every safety fact
discharged by the rules as written, and what per-operation runtime cost does the
result carry against an intrusive-pointer C++ IR (LLVM's `Value`/`Use` shape) and
against a safe-Rust generational arena (`slotmap` / `petgraph`)?

---

## 3. Representation, and why it is forced

One paragraph, because revision 4 settles it. Rule 1 refuses `struct Node { next:
&Node }` ("there is no type parameter, wrapper, or variant payload through which a
reference can be stored"), Rule 5's `Box<T>` "owns exactly one heap object" and so
cannot close a cycle, Rule 4 refuses returning a reference, and Rule 2 now refuses
the traversal loop of a pointer graph as well ("`loop { p = &(*p.kids)[0] }` //
rejected: the path would grow without bound; use recursion or indices"). Rule 16
supplies what is left: "A pool is a `Slots` (or `Box<Slots<T>>`) plus indices used
as handles".

```text
struct Edge { peer: u64, mirror: u64 }      // Copy: two u64 (Rule 8)
                                            // peer   = pool slot of the other endpoint
                                            // mirror = index of this edge's twin inside
                                            //          the peer's opposite list

struct Node {
    gen:   u64,                             // bumped on every delete; a stale Handle stops matching
    live:  u8,                              // occupancy as data (Rule 12)
    op:    u32,
    data:  u64,
    succs: Box<Slots<Edge>>,                // an owned window, never a reference (Rule 1)
    preds: Box<Slots<Edge>>,
}

struct Graph {
    nodes: Box<Slots<Node>>,                // the pool; the slot index is the handle (Rule 16)
    free:  Box<Slots<u64>>,                 // dead slot indices, kept as data (Rule 12)
}

struct Handle { idx: u64, gen: u64 }        // Copy owned data: storable (Rule 1), returnable (Rule 15)
```

`Edge`, `Handle` and the scalars are copy; `Node` and `Graph` are affine by
containment of `Box` (Rule 8), and nothing is linear, so Rule 8's scope-exit
release frees the whole graph with no user code. Paths are spelled through the
Box: the pool storage is `*g.nodes`, a slot is `(*g.nodes)[i]`, and an edge is
`(*(*g.nodes)[i].succs)[k]`.

Revision 4 also makes the two-allocations-per-node problem removable, which round
one could only gesture at: with `succs: Array<Edge, 4>`, `nsucc: u8` and `spill:
Option<Box<Slots<Edge>>>`, the spill window is named by a payload path — Rule 2, "A
payload step is available only under the refinement fact that the enum currently
holds that variant, which a `match` or `if let` on the enum establishes in the
selected arm". Round one had no payload step and so could not write the
small-vector shape at all. The derivation below uses the plain two-Box form to
keep the traces short; §8 prices the small-vector variant.

---

## 4. The program

### 4.1 Handle resolution across arbitrary mutation

```text
fn resolve(g: &Graph, h: Handle) -> Option<u64>
    reads((*g.nodes).len), reads((*g.nodes).filled)
    contract { ensures when Some: result < (*g.nodes).len; }
{
    if h.idx < (*g.nodes).len {                    // Rule 7: the test establishes the fact
        n = &(*g.nodes)[h.idx]                     // Rule 2: field, `*`, index
        if n.gen == h.gen {
            if n.live == 1 { return Some(h.idx) }
        }
    }
    return None
}
```

A `Handle` is copy data, so no rule can invalidate it: Rule 3 invalidates
references, and this is not one. It survives growth, slot reuse and whole-graph
rewriting, may sit in an aggregate (Rule 1) and may be returned (Rule 15).
Unchanged from round one except for the measure spelling and the row, which is now
two named parts rather than the whole container — which is what lets §6 overlap a
resolve with an append.

### 4.2 Node creation — the reference-preserving append, and the reserve that is not

Revision 4 splits round one's single `new_node` into the two halves the window
parts distinguish. Only the first preserves caller references.

```text
fn add_node(g: &Graph, n: own Node) -> u64            // the append; separately compiled
    writes((*g.nodes).next), writes((*g.nodes).len)
    contract { requires (*g.nodes).room > 0;
               ensures result == entry(*g.nodes).len,
                       (*g.nodes).len == entry(*g.nodes).len + 1; }
{
    id = (*g.nodes).len
    place_back(&*g.nodes, move n)                     // writes(r.next), writes(r.len): covered
    return id
}

fn reserve_nodes(g: &Graph, want: u64) -> Result<Unit, Oom>
    writes(*g.nodes)                                  // `grow`'s own row; kills every reference
    contract { requires want >= (*g.nodes).cap;
               ensures (*g.nodes).cap == want, (*g.nodes).len == entry(*g.nodes).len; }
{
    grow(&g.nodes, want)?
    return Ok(unit)
}

fn recycle_node(g: &Graph, op: u32, data: u64) -> Option<Handle>
    writes((*g.free).last), writes((*g.free).len), writes((*g.nodes).filled)
    contract { ensures when Some: result.idx < (*g.nodes).len; }
{
    if (*g.free).len > 0 {
        i = take_back(&*g.free)                       // requires r.len > 0: from the test
        if i < (*g.nodes).len {                       // (*) no fact covers i: see T6
            n = &(*g.nodes)[i]
            n.gen  = n.gen + 1                        // gap G-d: no overflow clause in the candidate
            n.live = 1
            n.op   = op
            n.data = data
            truncate(&*n.succs, 0)                    // library (Rule 6): take_back loop; capacity kept
            truncate(&*n.preds, 0)
            return Some(Handle { idx: i, gen: n.gen })
        }
        return None                                   // dead arm; no rule supplies the fact that kills it
    }
    return None
}
```

`fresh_node` builds the payload; its allocations are the only ones on the creation
path, and revision 4 hands the payload back if the second one fails, so no window
is lost:

```text
fn fresh_node(op: u32, data: u64) -> Result<Node, Oom>
{
    s = Box::new(Slots::new<Edge>(4))?
    match Box::new(Slots::new<Edge>(4)) {
        Ok(p)        => { return Ok(Node { gen: 0, live: 1, op, data, succs: move s, preds: move p }) }
        Err((e, _))  => { return Err(e) }             // `s` is affine and is released at the exit
    }
}
```

Rule 5: "A fallible allocation that takes a by-value payload hands it back on
failure, so a linear payload is never lost." Here the payload is `Slots::new`'s
empty window, and the point that matters is the first `?`: if the second
allocation failed and `s` had been moved into a half-built `Node` that nobody
could name, `s`'s block would leak. Round one's `vpush(&g.nodes, move fresh)?`
had exactly that shape.

**The headline case.** A reference into the pool held across a user-level append:

```text
if (*g.nodes).room == 0 { reserve_nodes(&g, (*g.nodes).cap * 2 + 8)? }   // before p is formed
p  = &(*g.nodes)[i]                          // formed under i < (*g.nodes).len
n  = fresh_node(OP_ADD, 0)?
id = add_node(&g, move n)                    // separately compiled; row names only .next and .len
p.op = OP_NOP                                // accepted: p is still valid
add_edge(&g, i, id)                          // and the fact i < (*g.nodes).len still holds
```

### 4.3 Edge insertion

```text
fn add_edge(g: &Graph, s: u64, d: u64)
    writes((*(*g.nodes)[s].succs).next), writes((*(*g.nodes)[s].succs).len),
    writes((*(*g.nodes)[d].preds).next), writes((*(*g.nodes)[d].preds).len)
    contract { requires s < (*g.nodes).len, d < (*g.nodes).len,
                        (*(*g.nodes)[s].succs).room > 0,
                        (*(*g.nodes)[d].preds).room > 0;
               ensures (*(*g.nodes)[s].succs).len == entry(*(*g.nodes)[s].succs).len + 1,
                       (*(*g.nodes)[d].preds).len == entry(*(*g.nodes)[d].preds).len + 1; }
{
    ms = (*(*g.nodes)[d].preds).len          // the position the twin will occupy
    md = (*(*g.nodes)[s].succs).len
    place_back(&*(*g.nodes)[s].succs, Edge { peer: d, mirror: ms })
    place_back(&*(*g.nodes)[d].preds, Edge { peer: s, mirror: md })
}
```

No `Result`, no rollback: `place_back`'s `requires r.room > 0` is discharged by the
contract, and the caller hoists growth (`reserve_edges` below). A self edge `s ==
d` needs no proof (T2). Growth of one node's list:

```text
fn reserve_succs(g: &Graph, at: u64, want: u64) -> Result<Unit, Oom>
    writes(*(*g.nodes)[at].succs)
    contract { requires at < (*g.nodes).len, want >= (*(*g.nodes)[at].succs).cap;
               ensures (*(*g.nodes)[at].succs).cap == want; }
{ grow(&(*g.nodes)[at].succs, want)?; return Ok(unit) }
```

### 4.4 Edge removal with twin repair — the hardest point

Removing `s -> d` must remove the twin from `d`'s pred list in O(1). `swap_remove`
relocates the last entry into the hole, and that relocated entry's own twin must
have its `mirror` corrected. The node owning that twin is chosen by a value loaded
out of the heap inside this function, so no row can name it: Rule 9, "an index
enters an effect only through an argument, evaluated once at the call". The row
falls back to the coarsest part that covers a data-chosen slot, `(*g.nodes).filled`
— which is still strictly finer than round one's `writes(g.nodes)`, because it
leaves `(*g.nodes).len`, `.next`, `.free` and `.cap` alone.

```text
fn drop_pred_twin(g: &Graph, e: Edge)   writes((*g.nodes).filled)
{
    if e.peer < (*g.nodes).len {                          // C1
        pl = &*(*g.nodes)[e.peer].preds
        if e.mirror < pl.len {                            // C2
            last = pl.len - 1                             // C2 gives pl.len > 0
            swap(&pl[e.mirror], &pl[last])                // may be the same place: no branch
            _ = take_back(pl)
            if e.mirror < pl.len {                        // C3: an entry was relocated into the hole
                m = pl[e.mirror]                          // Copy read, bound from C3
                if m.peer < (*g.nodes).len {              // C4
                    ol = &*(*g.nodes)[m.peer].succs
                    if m.mirror < ol.len {                // C5
                        ol[m.mirror].mirror = e.mirror
                    }
                }
            }
        }
    }
}

fn drop_succ_twin(g: &Graph, e: Edge)   writes((*g.nodes).filled)
{   // the mirror image: succs and preds exchanged; duplication is not a cost (protocol item 7)
    if e.peer < (*g.nodes).len {
        ol = &*(*g.nodes)[e.peer].succs
        if e.mirror < ol.len {
            last = ol.len - 1
            swap(&ol[e.mirror], &ol[last])
            _ = take_back(ol)
            if e.mirror < ol.len {
                m = ol[e.mirror]
                if m.peer < (*g.nodes).len {
                    pl = &*(*g.nodes)[m.peer].preds
                    if m.mirror < pl.len { pl[m.mirror].mirror = e.mirror }
                }
            }
        }
    }
}
```

`C3` is program logic that C++ pays too. `C1`, `C2`, `C4`, `C5` exist only because
the four indices were loaded out of a window; Rule 11 forecloses the fact that
would remove them ("There are no quantified facts over array elements ('for all i
...') and no per-slot occupancy facts"). Revision 4 removes exactly one branch
here, the `i != j` test that round one's `swap_remove` needed before exchanging two
slots, because "`swap(&r[i], &r[j])` // no `i != j` branch needed in a partition
loop" and Rule 10 clause 1 names swap as "the one operation whose two arguments may
be the same place".

Ordered removal, for operand lists where position is meaning (a phi's operands):

```text
fn remove_operand(g: &Graph, at: u64, k: u64) -> Edge
    writes((*(*g.nodes)[at].succs).filled), writes((*(*g.nodes)[at].succs).len)
    contract { requires at < (*g.nodes).len, k < (*(*g.nodes)[at].succs).len;
               ensures (*(*g.nodes)[at].succs).len == entry(*(*g.nodes)[at].succs).len - 1; }
{ return remove_at(&*(*g.nodes)[at].succs, k) }          // "one memmove"
```

Every twin whose `mirror` was above `k` is now off by one, so the caller walks the
tail and decrements — O(d) fixups in any representation that stores twin positions,
including LLVM's. What revision 4 supplies is the shift itself, one memmove instead
of round one's per-element `replace` loop.

### 4.5 Node deletion, draining both lists

```text
fn delete_node(g: &Graph, h: Handle) -> Bool
    writes((*g.nodes).filled), writes((*g.free).next), writes((*g.free).len)
    contract { requires (*g.free).room > 0; }
{
    match resolve(&g, h) {
      None    => { return false }                         // stale or dead handle: detected
      Some(i) => {
        while (*(*g.nodes)[i].succs).len > 0 {
            sl = &*(*g.nodes)[i].succs                    // re-formed each iteration: see T4
            e  = take_back(sl)                            // requires r.len > 0: the loop condition
            drop_pred_twin(&g, e)                         // writes((*g.nodes).filled): kills sl
        }
        while (*(*g.nodes)[i].preds).len > 0 {
            pl = &*(*g.nodes)[i].preds
            e  = take_back(pl)
            drop_succ_twin(&g, e)
        }
        n = &(*g.nodes)[i]
        n.live = 0
        n.gen  = n.gen + 1                                // every outstanding Handle to i now fails resolve
        place_back(&*g.free, i)
        return true
      }
    }
}
```

Two revision-4 effects are visible. The function no longer returns `Result`: the
free-list push is a `place_back` under `requires (*g.free).room > 0`, so deletion
cannot fail halfway through a mutation. And `i < (*g.nodes).len` survives every
`drop_pred_twin` call without any `ensures`, because the callee's row does not name
`(*g.nodes).len` — round one had to carry a length-preservation clause on both
repair functions and, under the strict reading of its gap G2, pay a reload anyway.

The pool itself never shrinks; `swap_remove` on `(*g.nodes)` would relocate the
last node and silently change its handle, which the task's observable forbids.

### 4.6 Cyclic traversal

```text
fn dfs(g: &Graph, n: u64, mark: &Box<Slots<u8>>, stack: &Box<Slots<u64>>, order: &Box<Slots<u64>>)
    reads((*g.nodes).filled), writes(**mark), writes(**stack), writes(**order)
    contract { requires n <= (*g.nodes).len, (**mark).len == n,
                        (**stack).room >= n, (**order).room >= n; }
{
    while (**stack).len > 0 {                             // Rule 11: "the loop condition is the test"
        i = take_back(&**stack)
        if i < n {                                        // C6: i came out of storage
            if (**mark)[i] == 0 {
                (**mark)[i] = 1
                place_back(&**order, i)
                sl = &*(*g.nodes)[i].succs                // i < n <= (*g.nodes).len
                m  = sl.len
                for k in 0..m {
                    invariant h: sl.len == m;             // nothing in the body writes sl
                    place_back(&**stack, sl[k].peer)      // k < m == sl.len: no per-edge bounds test
                }
            }
        }
    }
}
```

Cycles need nothing special: termination is decided by `mark`, which is data.
Three details are revision-4 specific. The bound `n` is a by-value parameter rather
than a read of `(*g.nodes).len`, which keeps `.len` out of the row and lets a DFS
overlap with an append (§6, O4). The inner loop still costs no per-edge bounds test,
because `m = sl.len` is a measure read of a bound reference and the counted loop
gives `k < m` directly (Rule 11's `invariant` form). And the walk is written on
indices, which Rule 2's static-path-shape clause now *requires*: the pointer form

```text
loop { p = &*(*p.succs)[0].target }        // rejected: Rule 2, the path would grow without bound
```

is refused, and Rule 2 names the two admitted forms, "use recursion or indices".
The index form is the faster one anyway, so this is `rejected-zero-cost` for this
task, but it is the sentence that eliminates the last hypothetical alternative
representation.

### 4.7 RAUW — replace all uses of `o` by `n`

The operation round one had to give up on. `o`'s pred list (its users) must move
onto `n`, and every user's `peer` must be rewritten.

```text
fn transfer_uses(g: &Graph, o: u64, n: u64) -> Result<Unit, Oom>
    writes(*(*g.nodes)[o].preds), writes(*(*g.nodes)[n].preds)
    contract { requires o < (*g.nodes).len, n < (*g.nodes).len, o != n; }
{
    src = &*(*g.nodes)[o].preds
    dst = &*(*g.nodes)[n].preds
    if dst.len == 0 {
        swap(&(*g.nodes)[o].preds, &(*g.nodes)[n].preds)   // two word stores; the windows exchange
        return Ok(unit)
    }
    if dst.room < src.len { grow(&(*g.nodes)[n].preds, dst.len + src.len)? }
    append(&*(*g.nodes)[n].preds, &*(*g.nodes)[o].preds)   // one memcpy; src becomes empty
    return Ok(unit)
}
```

Then the O(*u*) fixup pass that LLVM pays as well — each moved user's `succs` entry
must point at `n`, and its `mirror` must point at its new position in `n`'s list:

```text
base = entry_len_of_dst                                   // captured before the append
k = base
while k < (*(*g.nodes)[n].preds).len {
    m = (*(*g.nodes)[n].preds)[k]
    if m.peer < (*g.nodes).len {                          // the per-hop test again (CX1)
        ul = &*(*g.nodes)[m.peer].succs
        if m.mirror < ul.len { ul[m.mirror].peer = n; ul[m.mirror].mirror = k }
    }
    k = k + 1
}
```

`o != n` in the contract is one compare and branch at the call, once per RAUW; it
is needed because `append`'s two arguments are compared by Rule 10 clause 1
(`writes(dst.free)` against `writes(src.filled)` on one root) and only `swap` is
exempt. The swap fast path needs no such proof at all.

---

## 5. Rule-by-rule trace at the interesting points

Only the points revision 4 touches or the task turns on. Round-one items T1
(no pointers in a node), T6 (`ol[m.mirror].mirror = e.mirror` is a legal path
assignment), T7 (a recycled slot always holds a valid `Node`, Rule 12's "a
'logically empty' slot holds a valid value") and T11 (stale handles are detected by
the generation word, not by the rules) stand verbatim and are not repeated.

**T1 — a reference survives a user-level append (the assignment's case).**
`p = &(*g.nodes)[i]`, formed under `i < (*g.nodes).len`. The call
`add_node(&g, move n)` substitutes `writes((*g.nodes).next)` and
`writes((*g.nodes).len)`. Rule 10 clause 3 asks whether p's path "has a proper
prefix among the call's write paths, under the same may-overlap judgment". Rule 6
answers both: "a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or
`r.free`", and `(*g.nodes).len` is "a runtime number stored with the block", a
distinct part. Neither is a prefix of `(*g.nodes)[i]`, so p stays valid. The fact
`i < (*g.nodes).len` survives too: Rule 6, "`place_back`'s `ensures` carries it
across the call", through Rule 11's `entry` carry ("a fact known before the call
about a measure of an argument survives as a fact about `entry(p)`"). Rule 6 shows
this exact program under `add_node`, and it is a user function, not a built-in:
"This vocabulary is ordinary: the rows below use nothing a user function cannot
write."

**T2 — `add_edge`'s two writes need no disjointness proof.** The substituted write
paths are `(*(*g.nodes)[s].succs).next` and `(*(*g.nodes)[d].preds).next`. Rule 10
clause 1 governs "effects on overlapping paths"; these diverge at a field selector
(`.succs` against `.preds`) below the possibly-equal index, so no instantiation of
`s` and `d` makes them overlap. Rule 10's own `two(&a.x, &a.y) // accepted:
distinct fields` is this case with the index removed. Self edges are free, exactly
as in round one.

**T3 — an edge-list mutation does not invalidate a reference to a node, whatever
the indices.** With `p = &(*g.nodes)[i]` live, a call writing
`(*(*g.nodes)[j].succs).next` cannot invalidate p under Rule 10 clause 3, and the
may-overlap judgment does not change that: p's proper prefixes are `(*g.nodes)[i]`,
`*g.nodes` and `g.nodes`, and the write path is not one of them under any relation
between `i` and `j`. If `j != i` the two are disjoint; if `j == i` the write is
*below* p, and Rule 3 says "Writing the storage at p's path or below it (a content
write) does not invalidate p". This is the one place in the task where the
conservative may-overlap rule costs nothing, because clause 3 asks about prefixhood
and no assignment of `j` makes a field of a slot a prefix of that slot.
Conversely `q = &(*(*g.nodes)[i].succs)[k]` does die at such a call, since
`*(*g.nodes)[j].succs` may-overlaps its prefix `*(*g.nodes)[i].succs` — correct,
since `grow` may have replaced the block.

**T4 — the coarse repair row kills the drained list reference, and it must.**
`drop_pred_twin` declares `writes((*g.nodes).filled)`, and Rule 6 states "a live
`r[i]` ... always overlaps `r.filled`", so by Rule 10 clause 3 `sl` is invalid
after the call and is re-formed at the top of the next iteration: one block-pointer
load and one index multiply-add per drained edge. Round one recorded this as an
artifact of coarse rows that inlining would remove; under revision 4's explicit
may-overlap judgment it is not an artifact. A self edge (`e.peer == i`, a phi that
references itself — ordinary in real IR) means the repair genuinely writes the very
list being drained, and no rule can prove `e.peer != i` because `e.peer` came out of
storage. The reload is honest work, not a rule tax.

**T5 — what the length fact costs now: nothing.** `drop_pred_twin`'s row names
`(*g.nodes).filled` and, through the path, the inner lists' `.len`. It does not
name `(*g.nodes).len`. Rule 11: "A fact that mentions a path is invalidated when
that path is written". Rule 6 makes `.len` a path of its own: "The measure `r.len`
is itself a write target." So `i < (*g.nodes).len` survives the whole drain loop
with no contract clause. This closes round one's gap G2 in the cheap direction.

**T6 — the free-list index still has no fact.** `i = take_back(&*g.free)` yields a
`u64` read out of storage; the needed fact is "every element of `(*g.free)` is a
valid slot index", a quantified fact over elements, refused by name in Rule 11. The
test at `(*)` in §4.2 is mandatory and its `else` arm is unreachable code that no
rule can delete (x1 has no traps and no `unreachable`). Unchanged by revision 4.

**T7 — no partial move, but now three ways around it.** `x = move (*g.nodes)[i].succs`
is refused twice over: Rule 6's "A move out of a window slot or an array element is
rejected", and the consuming-move clause, "A move out of a field or out of Box
content consumes the whole owner", which would consume the slot. Round one stopped
there. Revision 4 gives `swap` on any owned place (§4.7's fast path), `replace(&r[k],
x) -> own T`, and the atomic update `place = f(place, args...)`, which together cover
every use this task had for a take. The one case still refused is taking a node's
edge list *out* of the pool into a local — and no operation in the task needs it.

**T8 — the atomic update cannot read the pool it lives in.** Rule 6 requires that
"f's row must not overlap any prefix of place". For
`(*g.nodes)[i].data = meet((*g.nodes)[i].data, &g, i)`, `meet`'s row contains
`reads((*g.nodes).filled)`, which may-overlaps `(*g.nodes)[i]`, a prefix of the
place. Rejected. See CX2 for the rewrite and its cost.

**T9 — `truncate` on a recycled node is not a linearity escape.** `Edge` is copy,
so the truncated entries are released (Rule 8); capacity is kept, as
`std::vector::clear` does, so a recycled node usually allocates nothing. Rule 6's
"No operation releases a linear element" would bind an IR whose payload were
linear: each deleted node would drain element by element instead. Noted, not
derived.

---

## 6. Overlaps the rules admit and refuse

Rule 13, in revision 4's wording: "Two adjacent statements of one block may overlap
when the first's write paths are disjoint from the second's read and write paths
and vice versa ... read/read overlap is allowed ... a by-value consumption counts as
a write of the argument's place." No `par` statement exists; the pairs below are
written adjacent and the comment states the judgment.

Admitted:

```text
// O1 — parallel node map, P3's requirement: nodes are distinct
fn touch(part: &[Node])  writes(part)
mid = (*g.nodes).len / 2
touch(&(*g.nodes)[0..mid]); touch(&(*g.nodes)[mid..(*g.nodes).len])   // may overlap: disjoint ranges

// O2 — two edge insertions, now expressible behind a function boundary
add_edge(&g, a, b); add_edge(&g, c, d)      // may overlap given a != c and b != d
                                            // (the succ rows share .succs, the pred rows share .preds)

// O3 — an append into one node's list beside a field write on any node
place_back(&*(*g.nodes)[a].succs, e); (*g.nodes)[i].data = 7   // may overlap: no proof needed,
                                            // the paths diverge at .succs against .data (T3)

// O4 — growing the pool beside a read of the existing nodes
dfs(&g, n, &mark, &stack, &order); add_node(&g, move fresh)    // may overlap: dfs reads .filled,
                                            // add_node writes .next and .len; disjoint at entry
                                            // (this is why dfs takes its bound n by value)

// O5 — pool append beside free-list append: different roots
place_back(&*g.nodes, move fresh); place_back(&*g.free, k)     // may overlap

// O6 — two builds that both allocate
a = fresh_node(OP_ADD, 0)?; b = fresh_node(OP_MUL, 0)?         // may overlap: Rule 14, allocation
                                            // "carries no effect entry"
```

O3 and O4 are new in revision 4; round one's whole-container rows refused both. O2
was available in round one only by writing the two `place_back` calls inline at the
call site, which cost nothing but did not survive separate compilation.

Refused:

```text
// R1 — two node creations
add_node(&g, move x); add_node(&g, move y)        // may not overlap: both write .next and .len
// R2 — anything beside a deletion or a twin repair
delete_node(&g, h); touch(&(*g.nodes)[0..mid])    // may not overlap: .filled against .filled
// R3 — two appends to one node's list
place_back(&*(*g.nodes)[a].succs, e1); place_back(&*(*g.nodes)[a].succs, e2)   // same paths
// R4 — a parallel worklist
// not expressible at all: needs channels or atomics, both under "Not in this candidate"
```

R1 is the candidate's own recorded cost ("Pool allocation from one pool serializes
against every other access to that pool; per-worker pools are the standard form")
and a shared bump allocator needs a lock in C++ too. The zero-cost rewrite is to
extend the pool once and fill the new range in parallel: `reserve_nodes` then
`place_back` in a loop to publish the length, then overlapping range writes over
`[base, base + k)`. R2 is the real structural loss: every graph rewrite is
`.filled`-wide because its target index is data, so no pass may run beside it. R4
is charged to the exclusion list, not to the access model.

---

## 7. Rules consistent, task achievable — recorded separately

**Rules consistent for this task: yes.** Every step above is decided by a quoted
clause and no two clauses used here conflict. The three combinations most likely to
break compose: Rule 1's no-references-in-aggregates against a cyclic structure
(handled by indices), Rule 3's invalidation against a long-lived handle (handles are
data, so Rule 3 never touches them), and Rule 6's no-partial-move against slot
recycling and RAUW (handled by `swap`, `replace`, `append` and the atomic update).
Revision 4 removed two of round one's four gaps and left two silences, not
conflicts (§9).

**Task achievable: yes, with cost.** All seven required capabilities are delivered:
nodes in a pool (§3), edge lists (§3, §4.3), node and edge insertion and deletion
(§4.2–§4.5), cyclic traversal (§4.6), reuse of deleted slots (§4.2), a handle held
across arbitrary mutation (§4.1), and — new in revision 4 — a *reference* held
across a user-level append whose row names the window parts (§4.2, T1). The
identity observable, that a relation to a deleted node must not silently become a
relation to its replacement, is delivered by the generation word and `resolve`,
which is stronger than the C++ baseline, which does not detect it at all.

---

## 8. Cost table

Baselines. **C++/LLVM**: individually allocated `Value`s, an intrusive doubly
linked `Use` list (four pointers per `Use`), raw pointers as edges, unchecked
indexing, use-after-free caught only by a sanitizer. **Rust**: a generational arena
(`slotmap` / `petgraph`), `Vec` adjacency, bounds-checked indexing, one generation
compare per handle deref.

| Operation | x1 revision 4 | vs C++/LLVM | vs safe Rust | Δ vs round one |
|---|---|---|---|---|
| Hold a handle across mutations | 16 B copy data, no invalidation possible | +8 B per handle, 0 instructions | 0 | 0 |
| Hold a *reference* across a node append | 0 instructions; the reference stays live | 0 | 0 (Rust refuses it outright) | **−1 block-pointer load, −1 multiply-add per creation**; round one had to re-form |
| Hold a reference across `grow`/`reserve_nodes` | reference dies (`writes(*b)`) | +1 reload after growth | 0 | 0 |
| Hold a reference across slot recycling | reference dies (`.filled`) | +1 reload | 0 | 0 |
| `resolve` a handle | 1 len load, 1 cmp/br, 1 multiply-add, 1 gen load + cmp/br, 1 live load + cmp/br | +3 cmp/br, +3 loads, +1 multiply-add | 0 | 0 |
| Iterate one node's edge list | 1 block-pointer load, 1 len load, **no per-edge bounds test** | wash: LLVM chases 2 pointers per `Use`, x1 walks a contiguous array | −1 bounds test per element vs `Vec` indexing | 0 |
| Follow one edge to its node | 1 handle load, 1 len load, 1 cmp/br, 1 multiply-add, 1 node load | **+1 cmp/br, +1 load, +1 multiply-add per hop** — dominant, see CX1 | 0 | 0 |
| `add_edge`, both directions | 2 len loads, 4 × 8 B stores, 2 len stores, **no capacity test, no error branch** | LLVM: 2 `Use` link-ins ≈ 4 stores plus 1 malloc per `Use` in the general case; x1 ahead on allocation and locality | 0 | **−2 capacity tests, −1 error branch, −1 rollback path** |
| Allocation per fresh node | 2 (`succs`, `preds`), or **0** with `Array<Edge,4>` + `Option<Box<Slots<Edge>>>` spill | 0 after the small-vector shape (which is what `SmallVector` is) | 0 | the spill shape is **now writable**: revision 4 supplies the payload path |
| Remove one edge + twin repair | C1–C5: 4 spurious cmp/br, 4 len loads, 2 block-pointer loads, 2 multiply-adds; `swap` + `take_back`; 1 u64 repair store | **+4 cmp/br, +6 loads, +2 multiply-adds, +1 16 B copy** vs `Use::removeFromList()` | 0 | **−1 branch** (swap may alias) |
| Drain one edge inside `delete_node` | + re-form `sl`: 1 block-pointer load, 1 multiply-add | small: an out-of-line C++ `removeEdge` also clobbers a cached `data()` | 0 | **−1 len load, −1 cmp/br** (the length fact now survives; round one's G2) |
| Ordered operand insert/remove | 1 memmove + O(d) mirror fixups | 0 for the shift; the fixups are inherent to storing twin positions | 0 | **−1 branch, −1 move per shifted element** |
| RAUW, target list empty | `swap`: 2 word stores, O(1) | 0 — matches LLVM's splice | 0 | **−u copies, −u capacity tests** |
| RAUW, target list non-empty | 1 possible realloc + 1 memcpy of 16u B + O(u) fixups | +1 memcpy over LLVM's pointer splice; the O(u) fixups both pay | 0 | **−u capacity tests, −u branches** |
| `recycle_node` | 1 `take_back`, 1 spurious bounds test (T6), 4 scalar stores, 2 `truncate` | +1 cmp/br, +1 len load; no malloc against LLVM's per-node malloc → **x1 ahead** | 0 | 0 |
| `add_node` appending | amortized: room test at the caller, memmove of `Node` records on growth, **no fixup pass** (Rule 1's relocation consequence) | LLVM never relocates but mallocs every node; x1 trades an amortized memmove for contiguity | 0 | 0 |
| `delete_node`, degree d | 2d edge removals + 2 scalar stores + 1 `place_back`; **no failure path** | ≈ 2d × the edge-removal delta | 0 | −1 error branch |
| In-place update of an owned per-node value | atomic update: 0 extra | 0 | 0 | **−1 allocation** (round one needed a dummy to `replace` with) |
| Dataflow meet that reads predecessors | refused in place (T8); two columns + ping-pong of whole locals | 0: a correct parallel C++ solver double-buffers too | 0 | 0 |
| Memory per edge | 16 B, contiguous | **−16 B per edge** vs LLVM's 32 B `Use`, and contiguous rather than malloc-scattered | +8 B (two u64 vs one u32 index) | 0 |
| Parallel node map (O1) | disjoint `&[Node]` ranges | 0 | 0 | 0 |
| Append beside a reader (O4) | admitted | 0 | Rust refuses it (`&mut` against `&`) → **x1 ahead** | **newly admitted** |
| Parallel node creation (R1) | refused; reserve-then-fill | +1 sequential pass publishing k lengths | 0 | 0 |
| Parallel worklist (R4) | **not expressible**; batched fork-join | O(n) per round instead of O(dirty), plus a barrier | 0 against safe Rust without `crossbeam`; real against `crossbeam` | 0 |

---

## 9. The strongest counterexamples against the candidate for this task

**CX1 — every edge traversal pays a bounds branch that no fact can remove, and the
escape is now formally closed.** The hot loop of any IR pass is "load a handle out
of an edge list, reach that node"; GVN or SCCP over 10^5 nodes performs 10^6–10^7 of
them. Each costs a length load, a compare and a branch, because the handle came out
of a window and Rule 11 states "There are no quantified facts over array elements
('for all i ...')". The invariant that would erase it — every handle stored in any
edge list is below `(*g.nodes).len` — is exactly a quantified fact over elements.
Three escapes, all closed under revision 4: hoisting is impossible (a different
handle at every hop); masking a power-of-two pool (`i = e.peer & (cap - 1)`) needs
the fact `(x & (c - 1)) < c`, and revision 4 moved it to "Deferred to a future
concurrency and layout round", so it is not in the candidate rather than merely
unstated; and there is no trap or `unreachable` to give the branch nowhere to go,
so the dead arm is live program text. Against safe Rust this is **zero** — `slotmap`
pays the identical compare. Against an LLVM-shaped C++ IR it is one well-predicted
compare-and-branch plus a multiply-add per hop, partly repaid by contiguous storage
and half-sized edge records. Irreducible, and the price of making a stale handle a
logic error rather than a use-after-free.

**CX2 — the atomic update, revision 4's new mechanism, is exactly unavailable where
this task wants it.** A dataflow round wants
`(*g.nodes)[i].live_in = meet((*g.nodes)[i].live_in, &g, i)`, with `live_in` an
owned `Box<Slots<u64>>` bitset and `meet` reading the successors' sets. Rule 6
refuses it: "f's row must not overlap any prefix of place". `meet` must read the
pool, `reads((*g.nodes).filled)` may-overlaps `(*g.nodes)[i]`, and `(*g.nodes)[i]`
is a prefix of the place. The mechanism works only for a node-local update that
reads nothing else. The rewrite is the two-column form — lattice values lifted out
of `Node` into `cur, nxt: Box<Slots<Bitset>>`, with `step` reading `cur` and writing
a disjoint range of `nxt`, ping-ponged between rounds by two local moves. Runtime
cost: one extra column of memory, no extra pass, and the columns are what a parallel
solver wants anyway — so **`rejected-zero-cost` in time, at one column of space**.
The sharp part is not the cost but the shape: revision 4's most general new
operation cannot touch a pooled graph in place, because everything in a pool is
under one root.

**CX3 — the window parts help the axis an IR uses least.** The parts vocabulary
buys precision exactly where the write target is the append slot or a slot named by
an argument. Every *rewrite* in a compiler — twin repair, RAUW fixups, free-list
reuse, peephole rewiring — chooses its target from data loaded out of the graph, so
its row is `(*g.nodes).filled`, which Rule 6 says "always overlaps" every live slot
reference and which therefore serializes against every other access to the pool
(R2) and kills every interior reference. Revision 4 moved the *creation* axis from
whole-container to slot-precise and left the *mutation* axis where round one found
it. For a compiler IR, creation is the cold path and rewriting is the hot one. The
honest summary: the append result in T1 is real and is the assignment's case, and
it does not touch the two costs that dominate a real pass (CX1's per-hop branch and
R2's rewrite serialization).

**CX4 (runner-up, unchanged) — no parallel worklist.** Workers popping dirty nodes
from a shared queue need channels or atomics, both under "Not in this candidate".
Batched fork-join over all nodes per round turns a sparse fixpoint from O(dirty) to
O(n) per round plus a barrier. The only asymptotic loss in the table, and charged to
the exclusion list.

---

## 10. Rule gaps

Per protocol item 3, quoted and not resolved.

**Closed since round one.**

*G1 (index in an effect row) is closed in the permissive direction.* Rule 9: "each
path starts at a reference parameter and may continue through fields, `*`, payload
steps, and whole-index or range positions supplied as arguments", and "an index
enters an effect only through an argument, evaluated once at the call". `s` in
`add_edge(g, s, d)` is an argument, so `writes((*(*g.nodes)[s].succs).next)` names
no index expression. Rule 16's own `alloc` row and Rule 6's `add_node` row are the
same shape.

*G2 (does a write below a slot invalidate a fact about the container's length?) is
closed.* Rule 6: "The measure `r.len` is itself a write target", and the four parts
"paths and effect rows may name". A row that does not name `(*g.nodes).len` does not
invalidate a fact about it (Rule 11, "A fact that mentions a path is invalidated
when that path is written"). T5 uses this; the cost it removes is in the table.

**Still open.**

**G-c — the grammar of an explicit `use` step.** Rule 11 lists among the fact forms:

> "explicit `use` steps inside an `invariant`"

and gives no grammar for what such a step may cite. Round one called this the
highest-value gap for this task because it decided whether a masked index could
discharge its bound. Revision 4 makes the answer moot here without closing the gap:
the bitmask fact is listed under "Deferred to a future concurrency and layout
round", so no reading of `use` supplies it under this candidate. **Status:
`undecided-rule-gap`, runtime cost for this task 0** — the per-hop branch of CX1
stands either way.

**G-d — integer overflow on the generation counter.** The candidate contains no
clause on arithmetic overflow anywhere, so `n.gen = n.gen + 1` in §4.2 and §4.5 has
no stated obligation. If one is required, the writer adds `if n.gen < MAX` with a
dead arm. **Status: `undecided-rule-gap`, cost at most +1 cmp/br per node deletion.**

**G-e (new) — is a write covered by a declared write path that lies above it
through a `Box`?** `drop_pred_twin` declares `writes((*g.nodes).filled)` and writes
`(*(*g.nodes)[e.peer].preds)`, a separate heap block reached through a Box inside a
slot. Rule 9 says a row's path "may continue through fields, `*`, payload steps",
and that `writes` "covers writing, replacing, moving out of, and freeing the storage
at the path", but nowhere states that a write to a path *extending* a declared write
path is covered by it. Rule 10's `bad(&vv, &vv[0])` example assumes the analogous
coverage for the overlap test. The derivation reads coverage as extending downward
through `*`. **Status: `undecided-rule-gap`, runtime cost 0** — under the strict
reading the row is written as the pair `writes((*g.nodes).filled),
writes(*(*g.nodes)[peer].preds)` with `peer` promoted to an argument, which changes
no instruction.

---

## 11. Verdict

- Overall task: **`accepted-fine`**, achievable **with cost**. Revision 4 does not
  change the achievability, and Rule 16 being confirmed removes round one's
  contingency on it.
- A node reference held across a user-level append whose row names the window parts
  (§4.2, T1): **`accepted-fine`** — the assignment's case, and new in revision 4;
  round one was `rejected-real-cost` (a reload per creation).
- `add_edge` with slot-precise rows, including self edges, and two insertions on
  distinct nodes overlapping across a function boundary (O2): **`accepted-fine`**;
  round one `rejected-zero-cost` (inlining required).
- RAUW use-list transfer by `swap` or `append` (§4.7): **`accepted-fine`** for the
  empty-target fast path; **`rejected-zero-cost`** against LLVM's O(1) splice for the
  non-empty case, at one memcpy on top of a fixup pass both forms pay. Round one:
  `rejected-real-cost`.
- Ordered operand-list edit by `insert_at`/`remove_at`: **`accepted-fine`**; round
  one `rejected-real-cost` (a per-element shift loop).
- Node deletion, twin repair, slot reuse, cyclic traversal, handle across mutation:
  **`accepted-fine`**, at the costs in §8.
- A reference held across a rewrite, a recycle, or a `grow` (T4, CX3):
  **`rejected-real-cost`** — one block-pointer reload and one multiply-add per
  re-formation, unavoidable because the target index is data and, for a self edge,
  genuinely aliasing.
- In-place atomic update of a per-node lattice value that reads the graph (T8, CX2):
  **`rejected-zero-cost`** in time, at one extra column of memory.
- Per-hop bounds compare on edge traversal (CX1): **`rejected-real-cost`** against
  C++, zero against safe Rust. The dominant cost of the task, unchanged.
- Parallel node creation (R1): **`rejected-zero-cost`** via reserve-then-fill.
- Parallel worklist (R4): **`rejected-real-cost`**, O(n) per round instead of
  O(dirty). Charged to "Not in this candidate: ... channels or atomics".
- Gaps: G1 and G2 **closed**; G-c (grammar of `use`) **`undecided-rule-gap`**, zero
  cost here; G-d (generation overflow) **`undecided-rule-gap`**, negligible; G-e
  (coverage of a write below a declared path) **`undecided-rule-gap`**, zero cost.
