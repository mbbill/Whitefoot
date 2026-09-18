# Engineering task under candidate x1 — mutable compiler IR graph

Derived against the frozen rule set in `CANDIDATE-X1.md` (2026-09-18) and only that
file. The engineering task row ("Mutable compiler graph") and P3 are quoted from
`PROGRAMS.md` where they bear on a step. Nothing under "Not in this candidate" or
"Deferred" is used: no `with` blocks, no `uniq`/`mut` markers, no regions or store
brands, no `take`/`put`, no quantified facts, no channels or atomics, no destructors,
no traps, no returned references, no header-plus-tail blocks.

Cost convention from the protocol, item 7: runtime performance is the only cost.
Verbosity, extra parameters and duplicated code are not costs. Every cost below is
named as an instruction, load, store, copy, allocation, branch, or lost parallelism.

---

## 1. The question this task tests

Can candidate x1 represent a mutable, cyclic, pooled compiler IR — nodes with
bidirectional edge lists, node and edge insertion and deletion, slot reuse, cyclic
traversal, and a node handle kept live across arbitrary mutation — with its safety
facts discharged by the rules as written, and what per-operation runtime cost does the
resulting form carry against an intrusive-pointer C++ IR (LLVM's `Value`/`Use` shape)
and against a safe-Rust generational-arena IR (`slotmap`/`petgraph` shape)?

---

## 2. Why the representation is forced before any program is written

Three rules eliminate every alternative representation before a line is written, so
the pool-plus-handles form is not a design preference here; it is the only survivor.

- Rule 1: "Every struct, enum, tuple, array, slice-like value, Box, DynBox, and
  generic instantiation holds only owned values. This is recursive and closed under
  wrapping: there is no type parameter, wrapper, or variant payload through which a
  reference can be stored." An IR node cannot hold `next: &Node`, and the rule's own
  rejected line `struct Bad { r: &Int }` is exactly the shape a pointer-based IR
  needs.
- Rule 5 leaves `Box<T>` as the only heap pointer, and it "owns exactly one heap
  object". A cyclic graph cannot be built from owning single-owner Boxes: a cycle
  requires two owners of one object, and x1 has no shared-ownership form anywhere.
- Rule 4: "It cannot be assigned into any aggregate (Rule 1), cannot be returned, and
  cannot be captured by a function value that is stored or returned. A function that
  'finds' something returns an index or other owned data; the caller forms the
  reference."

So edges are indices into a pool, which is precisely Rule 16's first sentence: "A
pool is a DynBox plus indices used as handles". Section 11 states exactly which steps
depend on Rule 16 and what happens to this task if the owner refuses it.

---

## 3. Shared declarations

```text
struct Edge { peer: u64, mirror: u64 }     // Copy (two u64, Rule 8)
                                           // peer   = pool slot of the other endpoint
                                           // mirror = position of this edge's twin inside
                                           //          the peer's opposite list

struct Node {
    gen:   u64,                            // bumped on every delete; a stale Handle stops matching
    live:  u8,                             // occupancy as data (Rule 12)
    op:    u32,
    data:  u64,
    succs: DynBox<Edge>,                   // owned window (Rule 6), never a reference (Rule 1)
    preds: DynBox<Edge>,
}

struct Graph {
    nodes: DynBox<Node>,                   // the pool; the slot index is the handle (Rule 16)
    free:  DynBox<u64>,                    // dead slot indices, kept as data (Rule 12)
}

struct Handle { idx: u64, gen: u64 }       // Copy owned data: storable (Rule 1), returnable (Rule 15)
```

Value classes under Rule 8: `Edge`, `Handle`, `u64`, `u8`, `u32` are copy. `Node` is
affine because it contains two `DynBox` fields; `Graph` is affine for the same reason.
Nothing here is linear, so Rule 8's "at scope exit the compiler releases their memory
recursively (Box, DynBox) and runs no user code" disposes of the whole graph with no
user code and no destructors, which is the correct behavior for an IR arena.

Library `push` for a growable window, quoted from Rule 6 ("`Vector<T>` is library
code: a DynBox plus a `push` that, when `len_of == cap_of`, allocates a larger DynBox,
moves `[0, len_of)` across, and replaces the old one"):

```text
fn vpush<T>(buf: &DynBox<T>, x: own T) -> Result<Unit, Oom>   writes(buf)
    contract { ensures when Ok:  len_of(buf) == len_of(deref(entry(buf))) + 1;
               ensures when Err: len_of(buf) == len_of(deref(entry(buf))); }
```

`x: own T` carries no effect entry: Rule 9, "A by-value parameter has no effect entry:
the call site records the consumption (for `move`) or the read (for a copy) of the
argument's place."

---

## 4. Strongest program first

Per protocol item 2, the strongest program is the one where static analysis provably
cannot help. Every hard feature of this task is present at once: the index of the slot
being written is read out of the heap (data-determined), the node whose list must be
repaired is chosen inside a callee (so no effect row can name it), the operation is
behind a separate-compilation boundary, and a handle is held live across all of it.

### 4.1 Handle resolution across arbitrary mutation

```text
fn resolve(g: &Graph, h: Handle) -> Option<u64>   reads(g.nodes)
    contract { ensures when Some: result < len_of(g.nodes); }
{
    if h.idx < len_of(g.nodes) {                 // Rule 7: the test establishes the fact
        n = &g.nodes[h.idx]                      // Rule 2: a path through a field and an index
        if n.gen == h.gen {
            if n.live == 1 { return Some(h.idx) }
        }
    }
    return None
}
```

This is the whole answer to "hold a node handle across mutations". A `Handle` is two
`u64`s of owned copy data, so no rule can invalidate it: Rule 3 invalidates
*references*, and a `Handle` is not one. It survives pool growth, slot reuse, and
whole-graph rewriting, and it may be stored in an aggregate (Rule 1 permits owned
values) and returned (Rule 15, "A function returns owned values only").

### 4.2 Node creation, including reuse of a deleted slot

```text
fn new_node(g: &Graph, op: u32, data: u64) -> Result<Handle, Oom>
    writes(g.nodes), writes(g.free)
    contract { ensures when Ok: result.idx < len_of(g.nodes); }
{
    if len_of(g.free) > 0 {
        i = pop(&g.free)                          // Rule 6: requires len_of > 0, from the test
        if i < len_of(g.nodes) {                  // (*) see cost C-alloc: no fact exists for i
            n = &g.nodes[i]
            n.gen  = n.gen + 1                    // see gap G4 (overflow is not covered by x1)
            n.live = 1
            n.op   = op
            n.data = data
            truncate(&n.succs, 0)                 // Rule 6: requires 0 <= len_of; capacity retained
            truncate(&n.preds, 0)
            return Ok(Handle { idx: i, gen: n.gen })
        }
        return Err(Oom)                           // dead arm; no rule supplies the fact that kills it
    }
    i = len_of(g.nodes)
    fresh = Node { gen: 0, live: 1, op: op, data: data,
                   succs: DynBox::new<Edge>(0)?, preds: DynBox::new<Edge>(0)? }
    vpush(&g.nodes, move fresh)?                  // may replace the block; every reference into
                                                  // g.nodes dies here (Rule 3), handles do not
    return Ok(Handle { idx: i, gen: 0 })
}
```

The `Ok` postcondition on the append path comes from `vpush`'s `ensures when Ok:
len_of(buf) == len_of(deref(entry(buf))) + 1` together with `i == len_of(deref(entry(g.nodes)))`.
On the reuse path it comes from the branch test at `(*)`, a "refinement fact from a
dominating branch" (Rule 11).

### 4.3 Edge insertion

```text
fn add_edge(g: &Graph, s: u64, d: u64) -> Result<Unit, Oom>
    writes(g.nodes)
    contract { requires s < len_of(g.nodes);
               requires d < len_of(g.nodes);
               ensures len_of(g.nodes) == len_of(deref(entry(g.nodes))); }
{
    sl = &g.nodes[s].succs
    pl = &g.nodes[d].preds
    ms = len_of(pl)                               // position the twin will occupy
    md = len_of(sl)
    vpush(sl, Edge { peer: d, mirror: ms })?
    match vpush(pl, Edge { peer: s, mirror: md }) {
        Ok(_)  => { return Ok(unit) }
        Err(e) => { _ = pop(sl); return Err(e) }   // undo the half edge; pop's
                                                   // requires len_of > 0 comes from the
                                                   // first vpush's Ok ensures
    }
}
```

Note what makes this accepted without any disjointness obligation. The two write paths
are `g.nodes[s].succs` and `g.nodes[d].preds`. They diverge at the field selector, so
they are not overlapping paths at all, and Rule 10 clause 1 ("Two effects on
*overlapping* paths where at least one is a write must be proved disjoint") never
engages. A self edge `s == d` is therefore accepted with no proof, matching Rule 10's
`two(&a.x, &a.y)   // accepted: distinct fields`.

The declared row is `writes(g.nodes)` rather than the two field paths. That choice is
forced, not stylistic; section 10, gap G1 quotes the ambiguous sentence, and section 7
shows that the finer granularity is recoverable at the call site at zero runtime cost.

### 4.4 Edge removal with twin repair — the hardest point in the task

Removing edge `s -> d` must also remove its twin from `g.nodes[d].preds` in O(1). The
twin's position is `mirror`. `swap_remove` relocates the last entry of that list into
the vacated slot, and *that relocated entry's own twin* must have its `mirror` updated.
The node owning that twin is chosen by a value read out of the heap inside this
function. No effect row can name it: Rule 9 says "Signatures never contain index
expressions; an index enters an effect only through an argument, evaluated once at the
call", and this index is neither an argument nor known at the call.

```text
fn drop_pred_twin(g: &Graph, e: Edge)   writes(g.nodes)
    contract { ensures len_of(g.nodes) == len_of(deref(entry(g.nodes))); }
{
    if e.peer < len_of(g.nodes) {                      // C1
        pl = &g.nodes[e.peer].preds
        if e.mirror < len_of(pl) {                     // C2
            _ = swap_remove(pl, e.mirror)              // Rule 6: requires e.mirror < len_of(pl)
            if e.mirror < len_of(pl) {                 // C3: an entry was relocated into the hole
                m = pl[e.mirror]                       // Copy read; bound from C3
                if m.peer < len_of(g.nodes) {          // C4
                    ol = &g.nodes[m.peer].succs
                    if m.mirror < len_of(ol) {         // C5
                        ol[m.mirror].mirror = e.mirror // path: field, index, field, index, field
                    }
                }
            }
        }
    }
}

fn drop_succ_twin(g: &Graph, e: Edge)   writes(g.nodes)
    contract { ensures len_of(g.nodes) == len_of(deref(entry(g.nodes))); }
{   // the mirror image of drop_pred_twin: preds/succs exchanged.
    if e.peer < len_of(g.nodes) {
        ol = &g.nodes[e.peer].succs
        if e.mirror < len_of(ol) {
            _ = swap_remove(ol, e.mirror)
            if e.mirror < len_of(ol) {
                m = ol[e.mirror]
                if m.peer < len_of(g.nodes) {
                    pl = &g.nodes[m.peer].preds
                    if m.mirror < len_of(pl) { pl[m.mirror].mirror = e.mirror }
                }
            }
        }
    }
}
```

Duplicating the routine instead of parameterizing the field selector is deliberate and
free: a `bool`-selected field would force the row to `writes(g.nodes)` anyway, and
protocol item 7 makes duplicated code not a cost.

`C3` is real program logic and C++ pays it too. `C1`, `C2`, `C4`, `C5` are pure bounds
tests that exist only because `e.peer`, `e.mirror`, `m.peer` and `m.mirror` were loaded
out of a `DynBox`. Rule 11 is explicit that no fact can cover them: "There are no
quantified facts over array elements ('for all i ...') and no per-slot occupancy
facts; occupancy that is determined by data is stored as data (Rule 6, Rule 12)." This
is the single largest recurring cost in the task and it is priced in section 8.

### 4.5 Node deletion, draining both lists

```text
fn delete_node(g: &Graph, h: Handle) -> Result<Bool, Oom>
    writes(g.nodes), writes(g.free)
{
    match resolve(g, h) {
      None    => { return Ok(false) }              // stale or dead handle: detected, not a memory error
      Some(i) => {
        while len_of(g.nodes[i].succs) > 0 {
            sl = &g.nodes[i].succs                 // re-formed each iteration: see T5
            k  = len_of(sl) - 1                    // k < len_of(sl) from the loop condition
            e  = sl[k]                             // Copy out before the list is touched
            truncate(sl, k)                        // Rule 6: requires k <= len_of(sl)
            drop_pred_twin(g, e)                   // writes(g.nodes): kills sl (T5)
        }
        while len_of(g.nodes[i].preds) > 0 {
            pl = &g.nodes[i].preds
            k  = len_of(pl) - 1
            e  = pl[k]
            truncate(pl, k)
            drop_succ_twin(g, e)
        }
        n = &g.nodes[i]
        n.live = 0
        n.gen  = n.gen + 1                          // every outstanding Handle to i now fails resolve
        vpush(&g.free, i)?
        return Ok(true)
      }
    }
}
```

The node is never removed from `g.nodes`; `swap_remove` on the pool itself would
relocate the last node and silently change its handle, which the task's observable
forbids ("a relation to a deleted node must not silently become a relation to a
replacement node"). So the pool only ever grows, and dead slots return through
`g.free`. That is Rule 16's arrangement exactly.

### 4.6 Cyclic traversal

```text
fn dfs(g: &Graph, root: u64, mark: &DynBox<u8>, stack: &DynBox<u64>, order: &DynBox<u64>)
        -> Result<Unit, Oom>
    reads(g.nodes), writes(mark), writes(stack), writes(order)
    contract { requires root < len_of(g.nodes);
               requires len_of(mark) == len_of(g.nodes);
               ensures  len_of(mark) == len_of(deref(entry(mark))); }
{
    vpush(stack, root)?
    while len_of(stack) > 0 {                       // Rule 11: "no knowledge: the loop
        i = pop(stack)                              //  condition is the test"
        if i < len_of(mark) {                       // C6: i came out of storage
            if mark[i] == 0 {
                mark[i] = 1
                vpush(order, i)?
                sl = &g.nodes[i].succs              // needs i < len_of(g.nodes): from C6 and
                                                    // the requires-fact; see gap G2
                m  = len_of(sl)
                for k in 0..m {
                    invariant h: len_of(sl) == m;   // nothing in the body writes sl
                    vpush(stack, sl[k].peer)?       // k < m == len_of(sl): Rule 7 satisfied
                }                                   // with NO per-edge bounds test
            }
        }
    }
    return Ok(unit)
}
```

Cycles need nothing special. Termination is decided by `mark`, which is data, and the
loop is Rule 11's untested form whose "loop condition is the test". No rule in x1
imposes a termination measure on `dfs`, and none is invented here; the recursive
formulation is equally accepted by the quoted rules (Rule 10: "Recursion is checked
through contracts, never by unfolding bodies"), and the explicit stack is chosen only
because it is the faster shape, as it is in C++.

The inner loop is the good news of this derivation: iterating one node's edge list
costs no bounds test at all, because `m = len_of(sl)` is a measure on a bound reference
name and the counted loop gives `k < m` directly (Rule 11's `invariant` form, the same
shape as the rule file's own `invariant h: len_of(buf) == n - i;` example).

### 4.7 Ordinary case: build a small cyclic region

```text
fn build_loop(g: &Graph) -> Result<Handle, Oom>   writes(g.nodes), writes(g.free)
    contract { ensures when Ok: result.idx < len_of(g.nodes); }
{
    a = new_node(g, OP_PHI,  0)?
    b = new_node(g, OP_ADD,  0)?
    c = new_node(g, OP_BR,   0)?
    add_edge(g, a.idx, b.idx)?
    add_edge(g, b.idx, c.idx)?
    add_edge(g, c.idx, a.idx)?                     // closes the cycle: three index writes,
                                                   // no ownership question at all
    return Ok(a)
}
```

`a.idx < len_of(g.nodes)` at the `add_edge` call sites survives the intervening
`new_node` calls only because `new_node`'s own postcondition would have to carry a
length-monotonicity clause; as written above it does not, and the caller must re-test
or `new_node` must add `ensures len_of(g.nodes) >= len_of(deref(entry(g.nodes)))`. That
clause is an ordinary affine comparison over a measure (Rule 11) and costs nothing at
runtime, so the postcondition is the right fix; it is added here:

```text
fn new_node(g: &Graph, op: u32, data: u64) -> Result<Handle, Oom>
    writes(g.nodes), writes(g.free)
    contract { ensures len_of(g.nodes) >= len_of(deref(entry(g.nodes)));
               ensures when Ok: result.idx < len_of(g.nodes); }
```

---

## 5. Rule-by-rule trace at the interesting points

**T1 — no node may hold a pointer (Rule 1).** `struct Node { succs: DynBox<Edge> }`
contains only owned values, so it is accepted; `struct Node { succs: DynBox<&Node> }`
is rejected by "there is no type parameter, wrapper, or variant payload through which a
reference can be stored", which also rejects `Option<&Node>` explicitly. Rule 1's
consequence — "any value can be relocated by copying its bytes (memmove, realloc),
because nothing inside it points anywhere" — is what makes pool growth in §4.2 a plain
`memmove` of `Node` records with no fixup pass, even though each `Node` owns two heap
blocks.

**T2 — `add_edge`'s two writes need no disjointness proof (Rule 10 clause 1).** The
substituted effects are `writes(g.nodes[s].succs)` and `writes(g.nodes[d].preds)`.
Clause 1 governs "effects on overlapping paths"; these diverge at the field selector, so
they are not overlapping. Rule 10's own accepted example `two(&a.x, &a.y)   //
accepted: distinct fields` is this case with the index removed. Self edges are free.

**T3 — growth of one node's edge list does not conflict with another's (Rules 13, 14).**
`vpush` on a full list "allocates a larger DynBox, moves `[0, len_of)` across, and
replaces the old one" (Rule 6). The replacement is a write of `g.nodes[a].succs`, and
the allocation carries no effect: Rule 14, "Allocation and release carry no effect
entry and never make two parallel arms conflict." So two edge insertions on
proved-distinct nodes may run in `par` even when both grow.

**T4 — a reference into a node survives that node's edge-list mutation (Rule 3).**
With `p = &g.nodes[i]` live, a call declaring `writes(g.nodes[i].succs)` writes a path
*below* `p`. Rule 3: "Writing the storage at p's path or below it (a content write) does
not invalidate p." So `p` stays valid. Conversely `q = &g.nodes[i].succs[k]` is killed
by the same call, because `g.nodes[i].succs` is a proper prefix of `q`'s path — which is
exactly right, since the block may have been replaced.

**T5 — the coarse row kills the drained list reference, and why that is free.** In
§4.5, `drop_pred_twin` declares `writes(g.nodes)`. Rule 10 clause 3: "A live reference
outside the call whose path has a proper prefix among the call's write paths becomes
invalid after the call (Rule 3)." `g.nodes` *is* a proper prefix of `g.nodes[i].succs`,
so `sl` dies at every iteration and is re-formed at the top of the next one. The fact
`i < len_of(g.nodes)` would die too (Rule 11: "A fact that mentions a path is
invalidated when that path is written (by statement or call) unless the callee's
`ensures` re-establishes it"), which is precisely why `drop_pred_twin` carries
`ensures len_of(g.nodes) == len_of(deref(entry(g.nodes)))`. Re-forming `sl` is one
address computation and one load of the block pointer per edge. Inlining
`drop_pred_twin` into `delete_node` removes the call, hence the row, hence the
invalidation: statement-level Rule 3 then sees the writes `g.nodes[e.peer].preds` and
`g.nodes[m.peer].succs[m.mirror].mirror`, neither of which is a proper prefix of
`g.nodes[i].succs`, so `sl` survives the whole loop. **This is the sharpest lesson of
the task: x1's effect rows cannot express a write target chosen from heap data, so any
graph mutator behind a function boundary is `writes(g.nodes)` whole.** It costs nothing
when the mutator is in the same function, and costs the reload at a separate-compilation
boundary (priced in §8).

**T6 — `ol[m.mirror].mirror = e.mirror` is a legal assignment.** Rule 2's path grammar
is "fields, `*` (Box content), `[i]` (index, Rule 7), or `[lo..hi]`", applied in any
order, so `g.nodes[m.peer].succs[m.mirror].mirror` is a path. The assigned place is a
`u64`, which is copy, so Rule 6's caveat — "Assigning `buf[k] = x` where the old value
is affine releases the old value; where it is linear the assignment is rejected" —
does not engage. If `Edge` were affine the whole-slot form `ol[m.mirror] = Edge{..}`
would still be admitted, releasing the old entry.

**T7 — slot reuse never observes an empty slot (Rules 6, 12).** After `delete_node`,
slot `i` still holds a fully valid `Node` with `live == 0`, two truncated-but-allocated
`DynBox` fields, and a bumped `gen`. Rule 12: "Structures whose occupancy is decided by
data keep that occupancy as data", and its example line "a 'logically empty' slot holds
a valid value". Rule 6 agrees: "no program point can observe a slot inside the window as
empty." `new_node`'s reuse path overwrites the scalar fields and calls `truncate(...,
0)`, which retains the two blocks' capacity — the same behavior as `std::vector::clear`,
so a recycled node usually needs no allocation at all.

**T8 — `truncate(&n.succs, 0)` is not a linearity escape (Rule 8).** `Edge` is copy, so
`Node` is affine, so the truncated elements are simply released. Rule 6's warning "A
DynBox whose element type is linear is itself linear (Rule 8) and must be truncated to
zero by the program before it can go out of scope" does not bind here, and it also
tells us what an IR carrying linear payloads would owe: each deleted node would have to
be drained element by element rather than truncated. That variant is noted, not derived.

**T9 — no partial move out of a node (Rule 6, Rule 12).** The natural O(1) RAUW trick
— splice the old value's `preds` block onto the new value by swapping the two `DynBox`
fields — requires `t = move g.nodes[o].preds`, which Rule 6 forbids: "There is no
`take` operation and no partial move out of any place", reinforced by Rule 12's "No
place is ever partially moved: every path is either wholly present or the program cannot
name it." The rewrite copies entry by entry. Its cost is bounded, see §8 and §9.

**T10 — the free list index has no fact (Rule 11).** `i = pop(&g.free)` yields a `u64`
read out of storage. The needed fact is "every element of `g.free` is a valid slot
index", which is a quantified fact over array elements and is refused in terms by Rule
11. The test at `(*)` in §4.2 is therefore mandatory, and its `else` arm is dead code
that no rule can eliminate.

**T11 — stale handles are detected, not merely safe (Rule 16 + Rule 12).** Rule 16: "A
stale index that is still in bounds names the current occupant of that slot: a logic
error, not a memory error. Programs that need to detect it keep a generation number as
data." This task's observable requires detection, so the `gen` field is not optional
here; `resolve` turns the logic error into `None`. Memory safety is unconditional
regardless, because the slot always holds a valid `Node` (T7).

**T12 — edge entries themselves carry no generation in §3.** `Edge.peer` is a bare
slot index. The structural invariant "every stored `peer` names a live node" is
maintained by `delete_node` draining both lists, and no rule can state it (Rule 11, no
quantified facts). x1 guarantees that violating it is memory-safe; it does not detect
it. The detecting variant widens `Edge` to `{peer, gen, mirror}` and adds one load plus
one compare and branch per traversal hop; both variants are priced in §8.

---

## 6. Parallel opportunities the rules admit

All four use Rule 13's judgment: "`par { A; B }` is accepted when A's write paths are
disjoint from B's read and write paths and vice versa, using the same path-overlap and
index/range-disjointness judgment as Rule 10."

**PA1 — parallel map over all nodes writing node data.** This is P3's stated
requirement, "parallel map over all nodes writing data // must be permitted: nodes are
distinct".

```text
fn touch(part: &DynBox<Node>)  writes(part)
    contract { ensures len_of(part) == len_of(deref(entry(part))); }

mid = len_of(g.nodes) / 2
par { touch(&g.nodes[0..mid]); touch(&g.nodes[mid..len_of(g.nodes)]) }
```

Accepted by Rule 13's exhibited form `par { kernel(&v[0..mid], &out[0..mid]);
kernel(&v[mid..n], &out[mid..n]) }   // disjoint ranges: accepted`, with the range
requirement `lo <= hi <= len_of(buf)` from Rule 7 discharged by the arithmetic.
Each arm may read its own nodes' edge lists and write its own nodes' `data`.
**Zero cost against rayon's `par_chunks_mut`, which is the same split.**

**PA2 — parallel edge insertion on proved-distinct endpoints.**

```text
par { vpush(&g.nodes[a].succs, ea); vpush(&g.nodes[c].succs, ec) }   // needs a != c
par { vpush(&g.nodes[b].preds, eb); vpush(&g.nodes[d].preds, ed) }   // needs b != d
```

Accepted by Rule 13 via Rule 10's index-disjointness ground, the same shape as
`par { bump(&v[i]); bump(&v[j]) }   // needs i != j`. Growth inside an arm is fine by
T3. Note this is written at the call site, not behind `add_edge`: see §7.

**PA3 — parallel dataflow round with a separated data column.** With `data` lifted out
of `Node` into `cur, nxt: DynBox<u64>`:

```text
fn step(g: &Graph, cur: &DynBox<u64>, out: &DynBox<u64>, base: u64)
    reads(g.nodes), reads(cur), writes(out)

par { step(g, &cur, &nxt[0..mid], 0); step(g, &cur, &nxt[mid..n], mid) }
```

Accepted: `reads(g.nodes)`/`reads(g.nodes)` and `reads(cur)`/`reads(cur)` are
read/read ("Read/read overlap is allowed", Rule 13), and the two `writes` are disjoint
ranges. Each arm may follow arbitrary edges and read *any* node's `cur` value, which is
the operation PA1 cannot do. Ping-ponging the two columns between rounds is three local
`move`s of whole locals, which Rule 6 admits, so the swap is free.

**PA4 — parallel fill of a pre-extended pool region.** Extend `g.nodes` by `k` default
nodes sequentially, then `par` over disjoint ranges of `[len, len+k)`.

### Parallel opportunities the rules refuse

**PR1 — two `new_node` calls in parallel.** Both substitute to `writes(g.nodes)`,
`writes(g.free)`; identical roots, identical paths, refused by Rule 13, exactly as
Rule 13's own `par { push(&v, 1); stats(&v) }   // rejected`. Correct and unavoidable:
a shared bump allocator needs a lock in C++ too. Rewrite: PA4, or per-worker sub-pools
followed by a renumbering pass. PA4 costs one extra sequential pass writing `k` default
`Node` records; the sub-pool rewrite costs an O(nodes + edges) handle-renumbering pass.

**PR2 — `par { add_edge(g, a, b); add_edge(g, c, d) }` behind the function boundary.**
Both rows are `writes(g.nodes)`, refused. The call-site form PA2 recovers exactly the
same parallelism because the caller's argument paths carry the indices, and protocol
item 7 makes the extra source lines not a cost. **`rejected-zero-cost`.**

**PR3 — anything in parallel with a node deletion.** `delete_node` writes `g.nodes`
whole and must, by T5, because the repair target is data-chosen. No arm may run
alongside it, not even a pure reader of an unrelated range. Refused. A C++ IR is in the
same position without fine-grained locking, so the loss is against a lock-based design,
not against a lock-free one.

**PR4 — a parallel worklist, the standard shape of every real IR pass.** Workers pop
node indices from a shared worklist, rewrite nodes, and push newly dirty nodes. This
needs either atomics or a channel, and both are listed under "Not in this candidate:
... channels or atomics". Refused. The rule file already records the available rewrite
under "Known costs already recorded": "Lock-free rings are not expressible; batched
fork-join with two buffers is the available form." Cost is priced in §8 and this is a
candidate-level exclusion, not a Rule 16 consequence.

---

## 7. The effect-row granularity question and why it costs nothing here

Whether a row may write `writes(g.nodes[s].succs)` when `s` is a by-value parameter is
ambiguous in the rule text (gap G1, §10). It does not change the runtime outcome:

- Where the index is an argument (`add_edge`), the caller can always inline the two
  `vpush` calls and pass `&g.nodes[s].succs` directly, which is unambiguously legal
  under Rule 2 and Rule 9's `put_at(&buf[len_of(buf)], 3)   // the argument fixes the
  slot at the call; the row itself names no index`. PA2 is exactly this. Same
  instructions, same parallelism.
- Where the index is *not* an argument (`drop_pred_twin`'s `m.peer`), no reading of
  Rule 9 helps, because "an index enters an effect only through an argument". The
  coarse row is forced under both readings, and T5 shows inlining recovers the facts.

So G1 is recorded as a real textual gap with **zero runtime consequence for this task**.

---

## 8. Cost table

Baselines. **C++/LLVM** = `Value` objects individually allocated, an intrusive doubly
linked `Use` list (four pointers per `Use`), raw pointers as edges, `operator[]` with no
bounds check, use-after-free caught only by a sanitizer. **Rust** = a generational
arena (`slotmap`/`petgraph`), `Vec<EdgeIndex>` adjacency, bounds-checked indexing, a
generation compare per handle deref.

| Operation | x1 | vs C++/LLVM | vs safe Rust arena |
|---|---|---|---|
| Hold a handle across mutations | 16 B of copy data; no invalidation possible | +8 B/handle (16 B vs an 8 B pointer); 0 instructions | 0 (identical) |
| Resolve a handle (`resolve`) | 1 len load, 1 cmp/br; 1 addr calc; 1 `gen` load, 1 cmp/br; 1 `live` load, 1 cmp/br | +3 cmp/br, +3 loads, +1 mul-add (C++ dereferences a pointer) | 0; Rust's arena does the same bounds + generation pair |
| Iterate one node's edge list | 1 block-ptr load, 1 len load; **no per-edge bounds test** (T/§4.6 counted loop) | wash: LLVM chases 2 pointers per `Use`; x1 walks a contiguous array | −1 bounds test per element vs `Vec` indexing; Rust's iterator also elides it |
| Follow one edge to its target node | 1 handle load; C1-style bounds test = 1 len load + 1 cmp/br; 1 mul-add; 1 node load | **+1 cmp/br +1 load +1 mul-add per hop** — the dominant cost, see §9 | 0 (identical) |
| Follow one edge *with stale detection* (`Edge{peer,gen,mirror}`) | above, plus 1 `gen` load + 1 cmp/br; edge widens 16 B → 24 B | +2 cmp/br, +2 loads, +50 % edge memory; LLVM does not detect it at all | 0 |
| `add_edge` (both directions) | 2 len loads, 2 amortized `vpush` (cap test + store), 2×16 B stores | LLVM: 2 `Use` link-ins = ~4 stores, no cap test, but 1 malloc per `Use` in the general case. **x1 is ahead on allocation and locality, behind by 2 cap tests** | 0 |
| `add_edge` allocation on a fresh node | `DynBox::new<Edge>(0)` twice = 2 allocations per node | +2 allocations per node vs an empty `SmallVector`. Removable at zero cost: `succs: array<Edge,4> + n: u8 + spill: Option<DynBox<Edge>>`, Rule 12's `DynBox<Option<Entry>>` shape, which is what `SmallVector` already is → then **0** | 0 after the same fix |
| Remove one edge + twin repair | C1..C5: 4 spurious cmp/br + 4 len loads + 2 block-ptr loads + 2 mul-adds; `swap_remove` = 1×16 B copy + 1 len store; repair = 1 load + 1 u64 store | **+4 cmp/br, +6 loads, +2 mul-adds, +1 16 B copy** vs `Use::removeFromList()`'s 2 loads + 2 stores. Largest per-operation loss in the task | 0 |
| `new_node` reusing a dead slot | 1 `pop`, 1 spurious bounds test (T10), 4 scalar stores, 2 `truncate` (2 len stores, capacity kept) | +1 cmp/br +1 len load (T10); no malloc, vs LLVM's malloc per node → **x1 is ahead** | 0 |
| `new_node` appending | amortized `vpush`: cap test + `memmove` of `sizeof(Node)` records on growth, no fixup (Rule 1's relocation consequence) | LLVM never relocates but mallocs every node; x1 trades an amortized memmove for contiguity → **x1 ahead on traversal locality** | 0 |
| `delete_node` (degree d) | 2d edge removals (row above) + 1 `gen` store + 1 `live` store + 1 `vpush` | ~2d × (the edge-removal delta) | 0 |
| Same, with the mutator out of line (separate compilation) | + re-form `sl` each iteration: +1 block-ptr load, +1 mul-add per edge; every fact about `g.nodes` re-established only via the length `ensures` | small: an out-of-line C++ `removeEdge` also clobbers the caller's cached `data()` pointer | 0 |
| RAUW on a value with *u* uses | O(u) peer rewrites (unavoidable in both) + O(u) 16 B entry copies, because the O(1) list splice is refused by T9 | +u 16 B copies + u cap tests vs LLVM's O(1) splice, on top of the O(u) loop both pay. Constant factor, not asymptotic | 0 |
| Memory per edge | 16 B (`Edge`), or 24 B with detection; contiguous | **−16 B per edge** vs LLVM's 32 B `Use` (four pointers), and contiguous rather than malloc-scattered → **x1 ahead** | +8 B (two u64 vs one u32 index) |
| Parallel node map (PA1) | disjoint range refs, admitted | 0 | 0 |
| Parallel dataflow (PA3) | two columns + ping-pong of whole locals, no copy | 0 (a correct C++ parallel solver double-buffers too) | 0 |
| Parallel worklist (PR4) | **not expressible**; batched fork-join over all nodes per round | a round costs O(n) instead of O(dirty), plus one barrier per round; on a sparse fixpoint with dirty ≪ n this is the worst loss in the table | 0 against safe Rust without `crossbeam`; real against `crossbeam`-based Rust |
| Parallel node creation (PR1) | refused; PA4 rewrite | +1 sequential pass writing k default `Node` records | 0 |

---

## 9. The strongest counterexample against the candidate for this task

**Every edge traversal in the compiler pays a bounds branch that no fact in x1 can
remove, and the standard trick for removing it is also unavailable.**

The hot loop of a real IR pass is "load a handle out of an edge list, reach that node".
A compiler doing GVN or SCCP over a function with 10^5 nodes performs 10^6–10^7 such
hops. Under x1 each one costs a length load, a compare and a branch, because the handle
came out of a `DynBox` and Rule 11 states in terms: "There are no quantified facts over
array elements ('for all i ...') and no per-slot occupancy facts." The invariant that
would erase the branch — *every handle stored in any edge list is below
`len_of(g.nodes)`* — is a universally quantified fact over array elements and is
refused by name. P3 asks for exactly this invariant: "the invariant 'next/prev point to
live nodes' is stated once and reused." **x1 cannot state it once; it re-tests it at
every use.**

Three escapes were checked and all fail under the rules as written:

1. *Hoist the test out of the loop.* Impossible: the handle is different at every hop,
   and there is no fact form that covers a whole list.
2. *Make the pool full and power-of-two sized and mask the handle,* `i = e.peer & (cap - 1)`,
   which is what a C++ pool does to erase the branch. The fact needed is
   `(x & (c-1)) < c`, and Rule 11's fact language is "affine comparisons over measures
   and integer values". A bitwise AND is not an affine expression, so the masked index
   carries no bound and still needs the test. (Whether a `use` step could supply it is
   gap G3.)
3. *Give the branch nowhere to go.* x1 has no traps and no `unreachable`, so the
   `else` arm is live program text the writer must fill; it cannot be folded away.

The honest size of this loss: against safe Rust it is **zero**, because `slotmap` and
`petgraph` pay the identical compare. Against an LLVM-shaped C++ IR it is one
well-predicted, perfectly cached compare-and-branch per hop plus the index-to-address
multiply-add, partly repaid by x1's contiguous pool and 16-byte edges against LLVM's
malloc-scattered 32-byte `Use` records — x1 may well win on an L2-resident IR and lose
on a small one. It is a real cost, it is irreducible under the rules as written, and it
is the price the candidate charges for making a stale handle a logic error rather than
a use-after-free.

The runner-up counterexample is PR4: no parallel worklist, which moves a sparse
fixpoint from O(dirty) per round to O(n) per round. It is charged to the exclusion list
("channels or atomics"), not to Rule 16 or to the access model, and the rule file
already records it ("Lock-free rings are not expressible").

---

## 10. Rule gaps

Per protocol item 3, these are quoted and not resolved.

**G1 — may an effect row carry an index taken from a by-value parameter?**
Rule 9 says both of these:

> "An effect row lists `reads(path)` and `writes(path)` where each path starts at a
> reference parameter and may continue through fields, `*`, and whole-index or range
> positions supplied as arguments."

> "Signatures never contain index expressions; an index enters an effect only through an
> argument, evaluated once at the call."

The first permits `fn add_edge(g: &Graph, s: u64, d: u64) writes(g.nodes[s].succs)`;
the second, read together with the accompanying example `put_at(&buf[len_of(buf)], 3)
// the argument fixes the slot at the call; the row itself names no index`, suggests the
index must ride on a *reference* argument instead. **Status: `undecided-rule-gap`,
runtime cost 0** — §7 shows the call-site form recovers the same instructions and the
same parallelism under either reading.

**G2 — is a fact about `len_of(g.nodes)` invalidated by a write to `g.nodes[j].succs`?**
Rule 11 says:

> "A fact that mentions a path is invalidated when that path is written (by statement or
> call) unless the callee's `ensures` re-establishes it."

`g.nodes[j].succs` extends `g.nodes`; it does not write it. Rule 3 draws exactly this
distinction for references ("Writing the storage at p's path or below it (a content
write) does not invalidate p"), but Rule 11 states no granularity for facts. In §4.6
this decides whether `dfs` needs a second bounds test on `i` after each `vpush`, and in
§4.5 whether `i < len_of(g.nodes)` survives the truncations. **Status:
`undecided-rule-gap`, runtime cost under the strict reading: +1 len load and +1 cmp/br
per popped worklist item and per drained edge.** The derivation above uses the prefix
reading, which Rule 3's parallel sentence supports.

**G3 — what may an explicit `use` step cite?** Rule 11 lists among the fact forms:

> "explicit `use` steps inside an `invariant`"

and gives no grammar for them. This decides escape 2 in §9: whether a writer can
discharge `(x & (c - 1)) < c` and delete the per-hop branch on a power-of-two pool.
**Status: `undecided-rule-gap`, runtime cost of the unfavorable reading: the §9 cost,
one compare and branch per edge traversal, is permanent.** This is the highest-value gap
in the task.

**G4 — integer overflow on the generation counter.** The candidate contains no clause
on arithmetic overflow anywhere. `n.gen = n.gen + 1` in §4.2 and §4.5 therefore has no
stated obligation. If a proof or a written outcome is required (the shape P17 assumes:
"`q = a / d` // d a runtime value: proof d != 0 or a written outcome"), the writer adds
`if n.gen < MAX` with a dead arm. **Status: `undecided-rule-gap`, runtime cost either
way: at most +1 cmp/br per node deletion, negligible.**

Nothing else in this task was ambiguous. In particular Rule 10's disjointness grounds
read as exhaustive only for *overlapping* paths, which is how T2 is decided, and Rule
10's own `two(&a.x, &a.y)   // accepted: distinct fields` confirms that reading.

---

## 11. Dependence on Rule 16 — exactly where

Rule 16 is provisional and pending the owner's ruling. Its load in this task is
precise.

**Depends on Rule 16:**

1. **The representation itself.** `nodes: DynBox<Node>` with the slot index used as the
   node handle is Rule 16's sentence "A pool is a DynBox plus indices used as handles",
   and the rule states that adopting it "reverses the current design tree's rejection of
   the arena-index-pool pattern".
2. **"Reuse deleted node slots" (§4.2 reuse path, §4.5 free list).** This is Rule 16's
   scenario verbatim: "A stale index that is still in bounds names the current occupant
   of that slot: a logic error, not a memory error." Without that disposition the
   reuse path has no safety story: an outstanding `Handle` to a recycled slot is
   *in bounds* and reads a valid `Node`, and only Rule 16 says that this is acceptable.
3. **The `gen` field and `resolve`.** Rule 16: "Programs that need to detect it keep a
   generation number as data." The task's observable ("a relation to a deleted node must
   not silently become a relation to a replacement node", `PROGRAMS.md`) makes detection
   mandatory here, so Rule 16 converts from permission into obligation: the generation
   field and its compare in §4.1 and the optional widened `Edge{peer,gen,mirror}` in §8
   exist only because Rule 16 pushes the burden onto the program.
4. **`new_node`'s contract shape.** `ensures when Ok: result.idx < len_of(g.nodes)` is
   Rule 16's own exhibited contract pattern, `ensures result == len_of(deref(entry(a.buf)));
   ensures len_of(a.buf) == result + 1`.

**Does not depend on Rule 16:** edge lists as `DynBox<Edge>` inside `Node` (Rules 1 and
6); the per-hop bounds test and the impossibility of stating the structural invariant
(Rules 7 and 11); reference invalidation at pool growth and the T5 coarse-row effect
(Rules 3, 9, 10); every parallel result in §6 (Rules 13 and 14); the refusal of the
O(1) use-list splice (Rules 6 and 12); the counterexample in §9. Those hold whether or
not Rule 16 is adopted.

**If the owner refuses Rule 16, the task is not achievable at all under x1.** Rule 1
forbids a reference inside `Node`, Rule 5's single-owner `Box` cannot close a cycle,
Rule 4 forbids returning a reference, and x1 has no shared-ownership type. A pool of
indices is the only remaining representation of a cyclic mutable graph, and refusing
Rule 16 refuses it. Note the precise nature of the dependence: the *program text* above
still type-checks under Rules 1–15 alone, because no rule among them forbids indexing a
`DynBox` with a value read out of another `DynBox`. What Rule 16 supplies is the
*disposition* that the resulting stale-handle hazard is a logic error rather than a
defect the rule set must prevent. The derivation depends on Rule 16; the syntax does
not.

---

## 12. Rules consistent, and task achievable — recorded separately

**Rules consistent for this task: yes.** Every step above is decided by a quoted clause,
no two clauses used here contradict each other, and the four gaps in §10 are silences,
not conflicts. In particular the combination that looked most likely to break — Rule
1's no-references-in-aggregates against a cyclic structure, Rule 3's invalidation
against a long-lived handle, and Rule 6's no-partial-move against slot recycling —
composes cleanly once Rule 16 supplies the pool disposition: handles are data, so Rule
3 never touches them; slots are overwritten in place, so Rule 6's move restriction never
engages; the graph is a flat array, so Rule 1's recursion into aggregates is satisfied.

**Task achievable: yes, with cost.** All seven required capabilities are delivered:
nodes in a pool (§3), edge lists (§3, §4.3), node and edge insertion and deletion (§4.2,
§4.3, §4.4, §4.5), cyclic traversal (§4.6), reuse of deleted slots (§4.2), a node handle
held across mutations (§4.1), and P3's parallel node map (PA1). The task's identity
observable — a relation to a deleted node must not silently become a relation to a
replacement node — is delivered by the generation field and `resolve`, which is
*stronger* than the C++ baseline, which does not detect it at all.

The cost is concentrated in one place and is not removable by rewriting: one compare,
one branch and one length load per edge-traversal hop, plus four spurious bounds tests
per edge removal, all of them consequences of Rule 11's refusal of quantified facts over
array elements. Against safe Rust that cost is zero. Against an intrusive-pointer C++
IR it is a constant factor on the hottest loop, partly repaid by contiguous storage and
half-sized edge records. Two structural losses are charged elsewhere: the O(1) use-list
splice (Rule 6's no-partial-move, a bounded constant on top of a loop both forms pay)
and the parallel worklist (the candidate's exclusion of channels and atomics, the only
asymptotic loss in the table).

---

## 13. Verdict

- Overall task: **`accepted-fine`**, achievable **with cost**. Depends on Rule 16 for
  the representation, the slot-reuse safety story, and the generation discipline.
- `add_edge` / `delete_node` / slot reuse / cyclic traversal / handle across mutations:
  **`accepted-fine`**.
- P3's parallel node map (PA1), parallel edge insertion (PA2), parallel dataflow round
  (PA3): **`accepted-fine`**.
- `par { add_edge(...); add_edge(...) }` behind the function boundary (PR2), and the
  out-of-line mutator invalidating interior references (T5): **`rejected-zero-cost`** —
  the call-site and inlined forms give identical instructions and identical parallelism,
  and source duplication is not a cost.
- O(1) RAUW use-list splice by swapping two edge-list fields (T9):
  **`rejected-zero-cost`** at the asymptotic level, **`rejected-real-cost`** at the
  constant level: +1 16-byte copy and +1 capacity test per use, on top of a rewrite loop
  that both forms already pay.
- Parallel node creation (PR1): **`rejected-zero-cost`** via PA4, at one extra
  sequential pass writing defaults.
- Parallel worklist (PR4): **`rejected-real-cost`** — O(n) per round instead of
  O(dirty), plus a barrier per round. Charged to "Not in this candidate: channels or
  atomics", not to Rule 16.
- Effect-row index granularity (G1): **`undecided-rule-gap`**, zero runtime consequence.
- Fact invalidation granularity under a content write (G2): **`undecided-rule-gap`**,
  costing at most one compare and branch per drained edge and per worklist pop.
- Whether a `use` step can discharge a bitmask bound and delete the per-hop branch (G3):
  **`undecided-rule-gap`**, the highest-value open question for this task.
- Generation-counter overflow (G4): **`undecided-rule-gap`**, negligible either way.
