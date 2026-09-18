# Candidate x1, round two (revision 4) — window-b: window parts, bulk operations, consuming moves, atomic update

Re-derivation of cells 6-6, 6-8, 6-9, 6-10, 6-12, 6-14, 6-15, 8-8, 8-10, 8-16,
9-12, 9-16, 10-15, 12-16 against `CANDIDATE-X1.md` revision 4 and only that file.
Round one's derivations are in `matrix-x1/rows-06-11.md`, `rows-08-09.md`,
`rows-07-10.md`, `rows-05-12.md`, its re-checks in the matching `verify-*` files.
"Round one" below is the corrected verdict (the verifier's, where it corrected).

Notation follows revision 4: measures and window parts are members (`r.len`,
`r.room`, `r.next`, `r.last`, `r.filled`, `r.free`); `(*b)[i]` and `(*b).len`
index through a Box; two adjacent statements are written adjacently and the trace
says whether they may overlap.

---

### 6-6: Rule 6 against itself — bulk growth, and the partial bulk move that is missing

Question: with `grow` and `append` in the operation table, what does an
order-preserving bulk movement of elements cost — for a whole window (Vector
growth) and for part of one (a B-tree node split)?

Round one: rejected-real-cost — growth of a window of affine elements had to be
written as a `swap_remove`/`pop` schedule costing about 2.5n element moves.

Strongest program (growth of a window of linear elements, then the split of a
node that must preserve order):

```text
struct Vector<T> { buf: Box<Slots<T>> }

fn push<T>(v: &Vector<T>, x: own T) -> Result<Unit, (Oom, T)>   writes(*v.buf)
{
    if (*v.buf).room == 0 {
        match grow(&v.buf, 2 * (*v.buf).cap + 1) {
            Err(o) => return Err((o, move x)),      // the payload comes back untouched
            Ok     => {}
        }
    }
    place_back(&*v.buf, move x)                     // requires room > 0
    Ok
}

// the split: move the upper half of src into the empty dst, order preserved
fn split(src: &Slots<T, N>, dst: &Slots<T, N>)   writes(src.last), writes(src.len),
                                                 writes(dst.next), writes(dst.len)
    contract { requires dst.room >= src.len; }
{
    m = src.len / 2
    k = src.len
    while k > m {
        invariant a: src.len == k;
        t = take_back(&src)                         // requires src.len > 0: from k > m >= 0
        place_back(&dst, move t)                    // requires dst.room > 0
        k = k - 1
    }
    reverse(&dst)                                   // dst received the half in descending order
}
```

Trace:

- Rule 6, operation table: `grow(&b, cap) writes(*b) // Box<Slots<T>> only; may
  reallocate in place`, `contract { requires cap >= (*b).cap; ensures (*b).cap ==
  cap, (*b).len == entry(*b).len; }`. Growth is now one built-in whose contract
  keeps `len` and raises `cap`; the element type never appears in it, so affine and
  linear elements are relocated by the implementation under Rule 1's "Consequence:
  any value can be relocated by copying its bytes (memmove, realloc), because
  nothing inside it points anywhere." Round one's element-by-element schedule is
  gone, and with it the 2.5n figure.
- `requires cap >= (*b).cap` is an affine comparison over measures (Rule 11), true
  of `2 * cap + 1`. `place_back`'s `requires r.room > 0` follows from
  `ensures (*b).cap == cap, (*b).len == entry(*b).len` and Rule 11's carry: "a fact
  known before the call about a measure of an argument survives as a fact about
  `entry(p)` of that argument".
- Failure: Rule 14, "Allocation returns a `Result` and never traps (Rule 5 for the
  payload)". The operation table prints `grow` without a result type; Rule 14 is
  the general normative sentence and there is no trap available, so `grow` is read
  as fallible, its `ensures` as the success arm. Rule 5's "A fallible allocation
  that takes a by-value payload hands it back on failure" is why `push` can return
  `(Oom, T)` and a linear `x` is never lost.
- `grow` declares `writes(*b)` and `*b` is a proper prefix of `(*b)[i]`, so Rule 3
  invalidates every reference into the window, in place or not. That is correct and
  is what C++ leaves to the writer.
- Whole-window transfer between two windows is `append(&dst, &src) // one memcpy`,
  `ensures dst.len == entry(dst).len + entry(src).len, src.len == 0`. It transfers
  payloads rather than releasing them, so it is available for affine and linear
  element types alike.
- The split has no operation. `append` moves all of `src` — its `ensures src.len ==
  0` is unconditional — and Rule 7's range reference cannot stand in for `src`,
  because `&[T]` is "a reference kind with measure `len`" with no `filled` part and
  no writable `len`, and `append`'s row is `writes(src.filled), writes(src.len)`.
  `insert_at`/`remove_at` move one element per call. So the only order-preserving
  schedule is the loop above.

Rewrite and cost: the growth path is free — one `realloc`-class call per doubling,
parity with Rust's `RawVec` and better than libstdc++'s `vector`, which always
copies. The split is the cost. `take_back` + `place_back` is two element moves and
two measure updates per element against one `memcpy` in C++ or Rust's
`Vec::split_off`, and the loop does not vectorise; the `reverse` adds `(n-m)/2`
swaps, three moves each. Best rewrite: hold the node in a `Ring` and drain the
*front* with `take_front`, which delivers the lower half in ascending order and
leaves the upper half in place, removing the reverse: 2 moves per element, about
2x the byte traffic of a `memcpy` plus per-element loop overhead, and `take_front`
declares `writes(r)` so every reference into the ring dies at each step. For a
B-tree with 4 KiB nodes that is roughly twice the split cost of the C++ form on a
hot path.

Rules consistent: yes — nothing in this cell needs two sentences ranked; the split
is simply not among the admitted operations.
Task achievable: rewrite — order-preserving growth is free, an order-preserving
partial move costs about 2x a `memcpy`.
Verdict: rejected-real-cost

---

### 6-8: window versus value classes — a window of linear elements, and who frees the emptied block

Question: with `truncate` demoted to library code and "No operation releases a
linear element" stated outright, how does a `Box<Slots<File>>` discharge its
obligations — including the block itself?

Round one: accepted-fine (corrected from undecided-rule-gap) — the drain loop was
unambiguous and the `truncate` contradiction did not decide the cell.

Strongest program (a growable container of resources with an early exit, plus
in-place substitution of one element):

```text
linear type File;  fn close(f: own File);  fn reopen(f: own File, flags: u32) -> File

files: Box<Slots<File>>                       // linear by containment (Rule 8)

(*files)[k] = reopen((*files)[k], flags)      // atomic update with an extra argument
old = replace(&(*files)[k], move fresh)       // own File out; `set` here would be rejected
close(move old)
swap(&(*files)[i], &(*files)[j])              // reordering resources: nothing is released

if error {
    while (*files).len > 0 {
        t = take_back(&*files)                // requires r.len > 0: the loop condition
        close(move t)
    }
    return Err(e)                             // (*files).len == 0 on this exit path
}
// ... and then? the block is still a linear-typed storage
```

Trace:

- Rule 8: "Linear values must be consumed by an explicit operation on every exit
  path; the compiler never releases them", and linearity "propagates through every
  aggregate of Rule 1: a struct, enum, tuple, Array, Slots, Ring, or Box containing
  a linear part is linear."
- Rule 6 now states the element discharge and the prohibition together: "No
  operation releases a linear element: a storage whose element type is linear is
  itself linear (Rule 8) and the program must take every element out and consume
  it." Round one's contradiction (a built-in `truncate` that dropped `File`s) is
  gone: `truncate` and `clear` are listed under "Library code, all zero-cost
  compositions of the above", and a composition of `take_back` on a linear element
  type must hand each element back to its caller.
- `set(&r[k], x) // old value: affine released, linear rejected` and "Assigning
  over any owned place releases the old value if it is affine and is rejected if it
  is linear" leave exactly two in-place transitions for a linear element, and
  revision 4 supplies both: `replace(&r[k], x) -> own T // the old value is
  returned`, and the atomic update, which revision 4 extends to "any owned place
  with extra args" (`place = f(place, args...)`), so `reopen` may take `flags`.
- `swap(p: &T, q: &T)` releases nothing, so reordering a window of linear elements
  is legal and costs three moves.
- The open point is the block. Rule 8 says the compiler never releases a linear
  value, and `Box<Slots<File>>` is linear by containment. Rule 8's shown discharge
  for a Box of a linear value is unboxing — "`f = move *b; close(move f)` //
  `Box<File>`: unbox, then close" — and that route does not exist here: Rule 5
  confines `Slots<T>` to "the content of a Box: never inline in another value and
  never as a local variable", so `move *files` has nowhere to land. The only
  candidate discharge is the one Rule 6 states, "the program must take every
  element out and consume it", after which the block is memory with nothing linear
  in it. Reading (a): the class is a property of the type, so the emptied box is
  still linear, no operation consumes it, and `Box<Slots<File>>` is a type no
  program can bring to an exit path. Reading (b): emptying *is* the discharge and
  Rule 5's "at scope exit the compiler releases its memory recursively" then frees
  an ordinary block. The file states the obligation and never states its
  completion, and the protocol forbids me choosing.

Rewrite and cost: under reading (b) nothing is rejected — per element one
relocation of `sizeof(File)` and one measure update against a C++
`vector<unique_fd>` destructor that closes in place, which is noise beside the
syscall. Under reading (a) the rewrite is a chain of `Box<Conn>` nodes, each
discharged by `move *b`, or a constant-capacity inline `Slots<File, N>` whose scope
exit frees no memory: one allocation per resource and one dependent load per hop
instead of one block and a stride, i.e. the candidate's own recorded
header-plus-tail cost, and a fixed ceiling in the inline case.

Rules consistent: yes for the elements; unranked for the emptied storage.
Task achievable: yes under reading (b) at negligible cost; under reading (a) only
by a linked representation at one allocation and one dependent load per element.
Verdict: undecided-rule-gap

---

### 6-9: window versus effect rows — an append-precise row, and a helper that names the container and a slot

Question: can a user function declare "I only append" precisely enough that a
caller's reference into the window survives the call, and can one row name the
container and a slot at the same time?

Round one: rejected-zero-cost (corrected from undecided-rule-gap) — the row had to
name the whole container, so container-plus-slot helpers were rejected and the
index-parameter rewrite carried the case.

Strongest program (a graph builder that appends a node while a reference into the
node window is live, and a helper that appends a copy of an existing slot):

```text
struct Graph { nodes: Box<Slots<Node>>, edges: Box<Slots<Edge>> }

fn add_node(g: &Graph, n: own Node) -> u64
    writes((*g.nodes).next), writes((*g.nodes).len)
    contract { requires (*g.nodes).room > 0;
               ensures result == entry(*g.nodes).len;
               ensures (*g.nodes).len == result + 1; }
{ id = (*g.nodes).len;  place_back(&*g.nodes, move n);  id }

fn duplicate(r: &Slots<Node, N>, i: u64)                  // container part and slot in one row
    reads(r[i]), writes(r.next), writes(r.len)
    contract { requires i < r.len, r.room > 0; }

p = &(*g.nodes)[i]                      // under i < (*g.nodes).len
id = add_node(&g, node)                 // p survives
q = &(*g.nodes)[id]                     // the new node, no test
use(p)
```

Trace:

- Rule 6's window-parts paragraph is the whole cell: "a window has four named parts
  that paths and effect rows may name ... `r.next` (the append slot, at index
  `r.len`)", and "This vocabulary is ordinary: the rows below use nothing a user
  function cannot write." The rule then shows precisely this program, with the
  comment "p survives: `(*g.nodes)[i]` with `i < len` never overlaps `.next`".
- Rule 10 clause 3 is what has to be checked: `p`'s path is `(*g.nodes)[i]` and the
  call's write paths are `(*g.nodes).next` and `(*g.nodes).len`; neither is a proper
  prefix of it, and Rule 6's overlap paragraph states "a live `r[i]` (which has
  `i < r.len`) never overlaps `r.next` or `r.free`". So `p` is not a bystander over
  a written prefix. The fact `i < (*g.nodes).len` survives too: Rule 6, "`place_back`'s
  `ensures` carries it across the call", via Rule 11's `entry` carry.
- `duplicate` is legal under Rule 9: "each path starts at a reference parameter and
  may continue through fields, `*`, payload steps, and whole-index or range
  positions supplied as arguments", and `i` is an argument, so `reads(r[i])` names
  no index expression. At the call, Rule 10 clause 1 compares `reads(r[i])` with
  `writes(r.next)` — disjoint by the same overlap sentence — and with
  `writes(r.len)`, which is a distinct part of the storage ("The window boundary
  `r.len` is a runtime number stored with the block"). Accepted. Round one could
  not write this function at all.
- What is still rejected is a helper that may *grow*: its row must contain
  `writes(*g.nodes)` (that is `grow`'s row), which is a proper prefix of every slot
  path, so no slot may appear beside it and every caller reference dies. Rule 10's
  own `bad(&vv, &vv[0])` line is that case.
- A user function cannot bypass the built-ins: measures are "read-only pseudo-fields
  ... A program reads them like fields and can never assign them; only the
  operations below change them", so `add_node` declares `writes(r.len)` and effects
  it by calling `place_back`.

Rewrite and cost: for the growing helper, either hoist the capacity check out of
the loop (`requires r.room > 0` on the appending helper, one `grow` before the
loop) or re-form the reference after the call, one shift-add. Both are free. The
accepted programs cost exactly what C++ costs — one store and one measure
increment per append — and the reference-survival that C++ leaves to the writer's
knowledge of `push_back` is checked here.

Rules consistent: yes.
Task achievable: yes — an append-precise row, a container-part-plus-slot row, and
surviving references, all at C++ cost.
Verdict: accepted-fine

---

### 6-10: window versus the call-site rule — exchanging two elements

Question: can two slots of one window be exchanged, with data-determined indices
and an affine or linear element type, and what does Rule 10's pairwise check
charge?

Round one: rejected-real-cost — an interior exchange had to be composed from three
swap-with-last operations, 9 moves and 6 measure stores against `std::swap`'s 3.

Strongest program (Hoare-style partition of a window of owning elements, indices
decided by the data, `T = Box<Record>`):

```text
fn partition(r: &Slots<T, N>, lo: u64, hi: u64) -> u64
    writes(r.filled)
    contract { requires lo <= hi, hi <= r.len; ensures lo <= result, result < r.len; }
{
    i = lo
    j = lo
    while j < hi - 1 {
        invariant a: lo <= i;
        invariant b: i <= j;
        if less(&r[j], &r[hi - 1]) {
            swap(&r[i], &r[j])          // i == j on most iterations: no branch, no fact
            i = i + 1
        }
        j = j + 1
    }
    swap(&r[i], &r[hi - 1])
    i
}
```

Trace:

- Rule 6: "`swap(p: &T, q: &T)  writes(p), writes(q)  // built in; p and q may be
  the same place, then nothing happens`", with the line `swap(&r[i], &r[j]) // no
  i != j branch needed in a partition loop`. Rule 10 clause 1 exempts it in terms:
  "The built-in `swap` is the one operation whose two arguments may be the same
  place." Round one's whole cost disappears: three moves, no distinctness proof, no
  branch.
- Nothing is released, so the element type may be affine or linear; `swap` is not
  one of the operations that could drop a value.
- Bounds: `r[i]` and `r[j]` need `i < r.len` and `j < r.len` (Rule 7), which the
  invariants give from `hi <= r.len`; no runtime test, the same code C++ emits.
- The row `writes(r.filled)` covers both swaps, because Rule 6's overlap paragraph
  says a live `r[i]` "always overlaps `r.filled`", and Rule 9's body check asks only
  that "every callee's substituted row must be covered by the declared row". The row
  does not name `r.next` or `r.free`, so a caller's reference to the append slot,
  and its `room` fact, survive a sort.
- Rule 10 clause 3 leaves caller references alone as well: `r[i]` is not a *proper
  prefix* of `r[k]`, and Rule 3 says "Writing the storage at `p`'s path or below it
  (a content write) does not invalidate `p`." So a reader holding `&r[k]` across a
  sort keeps a valid reference to a slot whose contents changed — which is what C++
  gives and what Rust forbids.
- One neighbouring form is not decided by the text. Merging two slots in place,
  `r[k] = merge(r[k], &r[m])`, is an atomic update whose constraint reads "f's row
  must not overlap any prefix of `place`". Literally, `reads(r[m])` overlaps the
  prefix `r` of `r[k]`, so no other slot of the same window may ever be an argument;
  purposively, the clause stops `f` from destroying the place's owner and `k != m`
  would be enough. This does not touch the cell's case — the exchange is `swap`, and
  a merge has the rewrite below — so it is recorded as a gap rather than a verdict.

Rewrite and cost: none needed for the exchange. If the restrictive reading of the
atomic-update clause holds, an in-place merge of two slots is written
`x = swap_remove(&r, m); r[k] = merge(r[k], move x)` (Rule 6 lists `swap_remove` as
library code), costing one extra element move and one measure update per merge, and
requiring `k != m` and `m != r.len - 1`.

Rules consistent: yes for the exchange; the atomic-update argument clause has two
readings that this cell does not depend on.
Task achievable: yes — in-place sort and partition of owning elements at
`std::swap` cost.
Verdict: accepted-fine

---

### 6-12: window versus joins and the absence of holes

Question: what survives a join whose arms move the window in opposite directions,
and what happens to an owned value consumed on one arm only?

Round one: accepted-fine, with three program defects noted (an uncovered row entry,
an undischarged probe bound, and the arity of the atomic update).

Strongest program (arms that remove, insert, and hand the payload back, with a
linear element type so no arm may simply drop it):

```text
fn apply(r: &Slots<Conn, N>, k: u64, cmd: Cmd, spare: own Conn)
        -> Result<Unit, (Busy, Conn)>
    writes(r.filled), writes(r.next), writes(r.len)
    contract { requires k < r.len; requires r.room > 0; }
{
    n = r.len
    match cmd {
        Drop => { c = remove_at(&r, k)          // ensures r.len == entry(r).len - 1
                  close_conn(move c)
                  close_conn(move spare) }
        Ins  => { insert_at(&r, k, move spare) } // ensures r.len == entry(r).len + 1
        Busy => { return Err((Busy, move spare)) }
    }
    // join of the two falling-through arms
    use(&r[m])                                   // needs m < r.len
    Ok
}
```

Trace:

- Rule 12: "At a join, facts are intersected (a fact survives only if it holds on
  every incoming edge)". The `Drop` edge carries `r.len == n - 1`, the `Ins` edge
  `r.len == n + 1`; the intersection in the affine-comparison lattice is
  `r.len >= n - 1`, which is what Rule 12's own worked example does for
  `if cond { place_back(&r, x) }`. So `use(&r[m])` is free for `m < n - 1` and costs
  one compare for `m` in `[n - 1, n + 1)`. C++ pays the same compare, because it too
  must know whether the erase happened.
- Holes: Rule 12's "No place is ever partially moved: every path is either wholly
  present or the program cannot name it" is free here because `remove_at` and
  `insert_at` are single operations that also move the boundary, and Rule 6
  guarantees "no program point can observe a slot inside the window as empty".
  Neither arm can leave a hole for the other to inherit.
- The linear payload is the interesting half. `spare: own Conn` is linear, so Rule 8
  demands consumption "on every exit path": the `Drop` arm closes it, the `Ins` arm
  moves it into the window, and the `Busy` arm hands it back inside the error
  payload, which is the shape Rule 5 uses for a failed allocation. Rule 12's "a
  local consumed on one incoming edge is consumed after the join" then makes `spare`
  unnameable after the join on every edge, with no drop flag: each consumption point
  is static.
- The row covers all three arms: `remove_at` substitutes `writes(r.filled),
  writes(r.len)`, `insert_at` adds `writes(r.next)`, and Rule 9's body check is
  satisfied by the declared union. Round one's arity defect is gone: revision 4
  states the atomic update as `place = f(place, args...)`, so a merge arm
  `r[k] = merge(r[k], move e)` is a shown form rather than an extrapolation.
- Rule 11's "There are no quantified facts over array elements ... and no per-slot
  occupancy facts" costs nothing, because occupancy that is decided by data is data
  (Rule 12's `tags`/`slots` representation), and the window itself has no occupancy
  question.

Rewrite and cost: none needed. One predicted compare after a length-moving join if
the program then indexes into the top two slots; zero otherwise.

Rules consistent: yes.
Task achievable: yes.
Verdict: accepted-fine

---

### 6-14: window versus one global heap and fallible allocation

Question: with `grow` allowed to reallocate in place and a failed allocation
handing the payload back, what does the heap cost a growable window — for `Slots`,
for `Ring`, and for a linear payload?

Round one: rejected-real-cost — no resize operation existed, so every doubling
copied the whole window where Rust's `RawVec` reaches `realloc`.

Strongest program (a growable container of resources: the payload is linear, the
growth may fail, and two independent builders run adjacently):

```text
fn push(v: &Vector<File>, f: own File) -> Result<Unit, (Oom, File)>   writes(*v.buf)
{
    if (*v.buf).room == 0 {
        match grow(&v.buf, 2 * (*v.buf).cap + 1) {
            Err(o) => return Err((o, move f)),     // the File is still owned by the caller
            Ok     => {}
        }
    }
    place_back(&*v.buf, move f)
    Ok
}

build(&src_l, &out_l)?                              // each call may grow its own Vector
build(&src_r, &out_r)?                              // disjoint roots: may overlap
```

Trace:

- Rule 6: `grow(&b, cap) writes(*b) // Box<Slots<T>> only; may reallocate in place`,
  with `ensures (*b).cap == cap, (*b).len == entry(*b).len`. The "may reallocate in
  place" clause is the change: the implementation is permitted to extend the mapping,
  so a large buffer grown by repeated pushes is `realloc` parity with Rust's `RawVec`
  and strictly better than libstdc++'s `vector`, which allocates and copies on every
  doubling. Round one's unbounded extra copy is gone.
- Rule 5: "A fallible allocation that takes a by-value payload hands it back on
  failure", and Rule 14: "Allocation returns a `Result` and never traps". With a
  linear `File` in flight this is not a convenience — without it the `Err` arm would
  hold a `File` nobody can name, and Rule 8's "must be consumed ... on every exit
  path" would be unsatisfiable. Round one recorded this as an open signature
  question; it is now a rule.
- Rule 14: "Allocation and release carry no effect entry and never prevent two
  statements from overlapping." Rule 13 then judges the two `build` calls on
  `out_l`/`out_r` and `src_l`/`src_r` alone — different roots — so they may overlap
  while both grow on the one internally synchronised heap.
- Two residues. First, `grow` declares `writes(*b)`, so on both the success and the
  failure path every reference into the window dies (Rule 3, proper prefix) and
  every fact about `(*b).len` is invalidated (Rule 11) — on the `Err` path no
  `ensures` re-establishes it, so a caller that held `(*b).len == n` re-reads the
  measure, one load, on the failure path only. Second, `grow` is annotated
  "`Box<Slots<T>>` only", so a `Ring` cannot be grown by it; a `Deque` grows by
  allocating a new ring and calling `append(&dst, &src) // one memcpy`, which is
  what Rust's `VecDeque` does on reallocation, and which C++'s `std::deque` avoids
  entirely at the price of one indirection per element access.

Rewrite and cost: none is needed; the costs named are a failure-path measure reload
and, for a ring, one bulk copy per growth that Rust also pays. The candidate's
recorded cost "Construction into the append slot moves one element where C++
constructs in place" still applies to `place_back` and is unchanged by revision 4.

Rules consistent: yes — the one text defect is that the operation table prints
`grow` without its `Result`, which Rule 14 supplies.
Task achievable: yes, at Rust-`Vec` parity.
Verdict: accepted-fine

---

### 6-15: window versus owned returns

Question: what does "return owned values only" cost when the value is an element
taken out of a window, a position inside one, or a measure of one?

Round one: accepted-fine, with one named cost — a dependent load per measure read
where Rust's `Vec` keeps the length in a register.

Strongest program (ordered removal that hands the element back, over a container of
linear resources, with the search returning an index):

```text
fn find(r: &Slots<Conn, N>, key: Int) -> Option<u64>   reads(r.filled)
    contract { ensures when Some: result < r.len; }

fn evict(r: &Slots<Conn, N>, k: u64) -> own Conn       writes(r.filled), writes(r.len)
    contract { requires k < r.len; ensures r.len == entry(r).len - 1; }
{ remove_at(&r, k) }                                   // one memmove

fn detach(c: own Conn) -> File { let Conn { f, n } = move c;  f }

match find(&r, key) {
    Some(i) => { c = evict(&r, i);  close(move detach(move c)) }
    None    => {}
}

fn scan(part: &[Conn]) -> u64   reads(part)            // part.len travels with the reference
{ k = 0;  s = 0;  while k < part.len { s = s + weight(&part[k]);  k = k + 1 };  s }
s = scan(&r[0..r.len])
```

Trace:

- Rule 15: "A function returns owned values only. Positions found by a search are
  returned as indices with a bounds relation in `ensures` (Rule 4)." The `ensures
  when Some: result < r.len` is a Rule 11 result-routed fact, so the caller indexes
  with no test; Rule 4 prices the residue, "the caller re-derives, one address
  computation".
- `remove_at(&r, k) -> own T` makes the owned return a real program rather than an
  illustration: the element leaves the window as an owned value, which is the only
  way a linear element can reach `close`, and the return type carries it. `detach`
  is revision 4's whole-owner destructuring — "A move out of a field or out of Box
  content consumes the whole owner ... (take it in the same destructuring)" — so a
  function may return one part of an owned parameter without leaving a hole.
- Rule 15's other sentence, "Passing a large value in and back out is expressed as a
  reference parameter with a `writes` entry, not as move-and-return", keeps large
  aggregates off the return path, where no zero-copy ABI is assumed.
- The measure cost is now avoidable at a call boundary. Rule 6 keeps `r.len` "a
  runtime number stored with the block", one dependent load per read; Rule 7's range
  reference is "a reference kind with measure `len`, formed only from an indexable
  or from another range reference, never a stored value", so `&r[0..r.len]` hands
  the callee a length that travels with the reference, exactly like a Rust slice.
  Inside a loop that does not write the window, the load is hoistable anyway (the
  round-one 6-11 correction: an element write does not invalidate a measure fact).

Rewrite and cost: none needed. One memmove per ordered eviction, parity with
`std::vector::erase`; one shift-add per re-derived reference; the measure load,
removable by passing a range reference.

Rules consistent: yes.
Task achievable: yes.
Verdict: accepted-fine

---

### 8-8: value classes against themselves — two linear parts in one aggregate, and recursion

Question: can an aggregate with two linear parts, and a recursive linear structure,
discharge every obligation when no partial move is allowed?

Round one: undecided-rule-gap — Rule 8's `close(move c.f)` and the no-partial-move
sentences of Rules 6 and 12 gave opposite answers, and the verifier confirmed the
gap covered every program in the cell.

Strongest program (two linear parts per node, a recursive chain, and a window of
children):

```text
linear type File
struct Conn  { f: File, log: File, id: u64 }              // two independent linear parts
struct Chain { c: Conn, next: Option<Box<Chain>> }        // recursive, linear

fn close_conn(c: own Conn)
{ let Conn { f, log, id } = move c;  close(move f);  close(move log) }

fn drain(head: own Option<Box<Chain>>)
{
    cur = move head
    while is_some(&cur) {
        match &cur {
            Some(b) => {
                t = move *b                               // unbox: consumes the owner chain
                let Chain { c, next } = move t
                close_conn(move c)
                cur = move next
            }
            None => {}
        }
    }
}
```

Trace:

- Rule 6 replaces the sentence the round-one gap turned on: "A move out of a field
  or out of Box content consumes the whole owner: the owner ceases to exist, its
  other affine parts are released, and a remaining linear part rejects the move
  (take it in the same destructuring)", with the form `let Conn { f, g, .. } = move
  c`. `close_conn` names both `File`s in one destructuring, so neither is "a
  remaining linear part"; `id` is Copy and needs nothing. The gap is closed and the
  answer is the permissive one, with a syntactic requirement instead of a
  prohibition.
- Rule 12's "No place is ever partially moved" is still satisfied: the destructuring
  takes the owner apart in one step, so no program point sees `c` with a hole. The
  same sentence decides how far "the whole owner" reaches in the chain walk: `*b` is
  Box content whose Box is the payload of `cur`, so consuming it would leave `cur`
  partially moved unless the consumption propagates to `cur` itself. It does, and
  `cur` is unnameable until it is rebound.
- The payload path is available because of Rule 2's revision-4 clause: "A payload
  step is available only under the refinement fact that the enum currently holds that
  variant, which a `match` or `if let` on the enum establishes in the selected arm
  and which any write to the enum invalidates." The `Some` arm gives `b`; the move
  invalidates the fact, and `b` is not used afterwards.
- Rule 5: "`move *b` consumes the Box, yields its content, and frees the cell", so
  each node's cell is freed as the walk passes it; Rule 8's "the compiler never
  releases them" is satisfied because every `File` reaches `close` explicitly on
  every exit path, and the loop has exactly one exit.
- Depth is not a constraint: the walk is iterative and Rule 2's "a loop-carried
  rebinding may change only the index values inside the path, never extend the path
  through itself" is not engaged, because `cur` is a *value* local rebound to an
  owned value, not a reference whose path grows.

Rewrite and cost: none needed. Each destructuring and unbox is a byte move of a
value the backend can keep in registers (Rule 1's consequence makes them plain
copies with no fixup, and Rule 14 makes addresses unobservable, so the copies are
elidable); the chain walk is one dependent load per hop, parity with the C++
pointer walk, and each `close` is the same syscall C++ performs in its destructor.

Rules consistent: yes — revision 4 ranks the sentences round one could not.
Task achievable: yes.
Verdict: accepted-fine

---

### 8-10: value classes against the call-site rule — a by-value argument is a write of its place

Question: which ownership transfers does Rule 10 clause 2 reject, and is the
rewrite free now that the atomic update takes extra arguments?

Round one: undecided-rule-gap (corrected from rejected-zero-cost) — the splice
rewrite assigned over a linear place, and the repair depended on the unranked
partial-move question.

Strongest program (installing a node in the middle of a chain whose payload is
linear, where the allocation may fail):

```text
struct Node { f: File, next: Option<Box<Node>> }          // linear by containment
fn put<T>(slot: &T, value: own T)   writes(slot)

b: Box<Node>
slot = &(*b).next
put(slot, move b)                                          // rejected

// the rewrite: the atomic update, on the place that must change
fn link(old: own Option<Box<Node>>, f: own File) -> Option<Box<Node>>
{
    match Box::new(Node { f: move f, next: move old }) {
        Ok(nb)      => Some(move nb),
        Err((o, n)) => { let Node { f, next } = move n;  close(move f);  move next }
    }
}

(*p).next = link((*p).next, move file)                     // one commit, no point in between

// the by-value copy case
place_back(&r, r[0])                                       // T Copy: accepted under revision 4
```

Trace:

- Rule 10 clause 2: "A by-value argument contributes a consumption (`move`) or a
  read (copy) of its place to this comparison." In the first call the consumed place
  is `b` and the other argument's substituted path is `(*b).next` — Rule 2, a
  reference "names a path; it is not storage of its own" — so clause 1 rejects, and
  the rule prints the same line: "`put(slot, move b)` // rejected: `move b` writes
  the prefix `b` of slot's path (Rule 3)".
- The repair round one could not take is now a shown form. Rule 6: "`place =
  f(place, args...)  writes(place)  // atomic in-place update: the old value enters
  `f` by value, `f`'s result is committed, no program point lies between", with the
  example `node.left = insert(node.left, k)`. The old chain enters `link` by value,
  so nothing is released and nothing is assigned over a linear place — which is what
  the verifier's objection to round one's `head.next = Some(...)` was about
  ("Assigning over any owned place ... is rejected if it is linear"). `f`'s row is
  empty (by-value parameters have no effect entry, Rule 9), so the constraint "f's
  row must not overlap any prefix of `place`" is satisfied trivially.
- Failure inside an atomic update has to be encoded in the place's type, because "f
  is total and returns the place's type" — Rule 6's own comment, "failure is an enum
  in the place". Here `link` returns the chain unchanged on `Oom`, after closing the
  `File` it was handed; Rule 5's payload return (`Err((o, n))`) is what makes that
  `File` nameable at all, so no linear obligation is lost on the failure path.
- `place_back(&r, r[0])` changes verdict for a revision-4 reason. Round one compared
  a read of `buf[0]` with `writes(buf)` and rejected. The row is now
  `writes(r.next), writes(r.len)`, and Rule 6 states "a live `r[i]` (which has `i <
  r.len`) never overlaps `r.next` or `r.free`"; `r.len` is a distinct part of the
  block. So the call is accepted with no self-aliasing guard — Rust rejects
  `v.push(v[0])` and C++ pays a library guard for it.
- A two-window transfer with one parameter per window, `transfer(&p, &p)`, is still
  rejected by clause 1 (one root, nothing to prove distinct), and the rewrite is a
  one-parameter helper; duplicated code is not a cost.

Rewrite and cost: zero. The atomic update commits exactly the stores the C++
pointer splice performs, because Rule 1's consequence makes a `Box` move a pointer
copy; `place_back(&r, r[0])` needs no rewrite at all; the single-window helper
performs the same operations as the rejected two-parameter call.

Rules consistent: yes — clause 2, Rule 3, Rule 6's assignment rule and the atomic
update all point the same way now.
Task achievable: rewrite — every rejection has a same-instruction replacement.
Verdict: rejected-zero-cost

---

### 8-16: value classes against pools and arenas — resetting and reusing slots that hold resources

Question: with Rule 16 confirmed, what does a pool of linear resources cost when
slots are reused through a free list and the pool must be torn down?

Round one: rejected-real-cost (corrected from accepted-fine) — an index popped from
the free list carries no bound, so every acquisition needs a test and a dead arm;
and releases serialise on the free list.

Strongest program (reuse with detection, a linear payload, and a teardown):

```text
struct Pool { slots: Box<Slots<Option<Conn>>>, gen: Box<Array<u32>>, free: Box<Slots<u64>> }

fn acquire(p: &Pool, c: own Conn) -> Result<u64, (Full, Conn)>
    writes((*p.slots).filled), writes((*p.free).last), writes((*p.free).len)
{
    if (*p.free).len == 0 { return Err((Full, move c)) }
    k = take_back(&*p.free)                       // own u64, with no bound of any kind
    if k < (*p.slots).len {
        old = replace(&(*p.slots)[k], Some(move c))   // the old value is returned
        consume_none(move old)                        // a match that closes a Some, if any
        Ok(k)
    } else {
        Err((Full, move c))                       // unreachable, but c must go somewhere
    }
}

fn release(p: &Pool, k: u64)
    writes((*p.slots)[k]), writes((*p.gen)[k]), writes((*p.free).next), writes((*p.free).len)
    contract { requires k < (*p.slots).len, k < (*p.gen).len, (*p.free).room > 0; }
{
    (*p.slots)[k] = retire((*p.slots)[k])         // atomic update: consumes the Conn, stores None
    (*p.gen)[k] = (*p.gen)[k] + 1
    place_back(&*p.free, k)
}
```

Trace:

- Rule 16 is confirmed, not provisional: "A pool is a `Slots` (or `Box<Slots<T>>`)
  plus indices used as handles ... A stale index that is still in bounds names the
  current occupant of that slot: a logic error, not a memory error. Programs that
  need to detect it keep a generation number as data."
- The rejection is unchanged by revision 4 and is the cell's cost. `k` comes out of
  `(*p.free)` as ordinary data; the fact `k < (*p.slots).len` would have to hold of
  every element of the free list, and Rule 11 forecloses it: "There are no
  quantified facts over array elements ("for all i ...") and no per-slot occupancy
  facts." So `replace(&(*p.slots)[k], ...)` cannot discharge Rule 7's obligation
  without the written test, and the `else` arm is code that never runs. The deferred
  list names the only escape — "A bitmask fact (`x & (c - 1) < c` for a power-of-two
  `c`) to remove the per-probe bounds compare" — and it is deferred.
- Revision 4 does improve the body. `replace(&r[k], x) -> own T // the old value is
  returned` gives a legal in-place substitution for a linear element where `set`
  "linear rejected"; the atomic update "on any owned place with extra args" makes
  `retire` a shown form; and `release`'s slot-precise row is legal because `k` is an
  argument (Rule 9, "whole-index or range positions supplied as arguments").
- Two releases still cannot overlap. Both substitute `writes((*p.free).next)` and
  `writes((*p.free).len)` on one root, so Rule 13's "the first's write paths are
  disjoint from the second's read and write paths" fails however distinct `k1` and
  `k2` are. Two *fills* of distinct slots do overlap, which is what the slot-precise
  row buys. This is the candidate's recorded cost, "Pool allocation from one pool
  serializes against every other access to that pool".
- Teardown: Rule 6's "No operation releases a linear element ... the program must
  take every element out and consume it" makes the constant-time arena reset
  (`truncate(0)`) unavailable for a linear element type; the drain loop is the
  reset. The residual question of who frees the emptied block is cell 6-8's gap and
  is not re-derived here.

Rewrite and cost: the rewrite is the written test plus the unreachable arm, one
predicted compare and one branch per acquisition against an unsafe C++ free-list
pool that has none — no memory traffic, but a real instruction on the allocation
path, and a signature forced to return the payload so the dead arm can discharge
`c`. Against a correct Rust `slotmap` this is parity. The generation word costs
four bytes per slot and one load plus one compare per handle dereference, again
`slotmap` parity, and buys the compiler-graph requirement that a relation to a
deleted node must not silently become a relation to its replacement.

Rules consistent: yes.
Task achievable: rewrite — the pool works, with one compare per acquisition that no
form inside this candidate removes.
Verdict: rejected-real-cost

---

### 9-12: effect rows against the absence of holes — the append slot is now a name

Question: revision 4 lets a path name `r.next`; does that make the slot outside the
window writable, and if not, who may name it?

Round one: rejected-zero-cost (corrected from undecided-rule-gap) — Rule 9's
example `put_at(&buf[len_of(buf)], 3)` demanded `len_of < len_of`, so it was a
defect in the example and `push_nogrow` was the only append.

Strongest program (an appending helper for a linear element type, beside the
program that would leak if the append slot were an ordinary place):

```text
fn adopt(r: &Slots<File, N>, f: own File)
    writes(r.next), writes(r.len)
    contract { requires r.room > 0;  ensures r.len == entry(r).len + 1; }
{ place_back(&r, move f) }                        // the only body that discharges this row

fn leak(r: &Slots<File, N>, f: own File)   writes(r.next)
{ set(&r.next, move f) }                          // is this a program?
```

Trace:

- Rule 6 makes the row legal and precise: "a window has four named parts that paths
  and effect rows may name, all interpreted at call entry: `r.next` (the append
  slot, at index `r.len`)", and "A user function that only appends declares the same
  row as `place_back`". Round one's rejected `&buf[len_of(buf)]` is not needed: the
  append slot has a name that is not an out-of-bounds index, and Rule 7's bound
  obligation is never raised, because `r.next` is not an index expression.
- `adopt`'s body can only be `place_back`. Measures are "read-only pseudo-fields ...
  only the operations below change them", so no user statement can move the
  boundary, and Rule 9's body check ("every statement's effect ... must be covered
  by the declared row") accepts the built-in's substituted row exactly.
- `leak` is the question the new name raises. `set(&r[k], x)` is stated for a slot
  `r[k]`, and assignment is stated for "any owned place". A slot at or above `r.len`
  is not an owned place: Rule 6 says slots "[len, cap) hold nothing", so there is no
  old value for the assignment rule to release or reject, and `r.free` is defined as
  those same slots. If `leak` were a program, the stored `File` would sit outside the
  window where no read can reach it, where scope-exit release does not go (release
  covers "the slots inside the window"), and where Rule 8's obligation could never be
  discharged — and a writable slot outside the window is exactly the "`take`/`put`
  holes" that the "Not in this candidate" list excludes by name. So `r.next` is a
  name for rows and for the overlap judgment, not an assignable place, and the only
  writer of the append slot is an operation that also moves `r.len`.
- With that settled, Rule 12's "every path is either wholly present or the program
  cannot name it" survives contact with the new vocabulary: `r.next` is nameable in a
  row but names nothing that a program can read or assign.

Rewrite and cost: none needed. `adopt` costs one store and one measure increment,
identical to `place_back` and to a C++ `push_back` with spare capacity, and its
precision is what lets a caller's reference into the same window survive the call
(cell 6-9).

Rules consistent: yes, once the exclusion list is read as normative over the
inference that "paths may name `r.next`" makes it assignable.
Task achievable: yes.
Verdict: accepted-fine

---

### 9-16: effect rows against pools and arenas — allocation no longer writes the whole block

Question: an arena allocation appends; with `r.next` in the vocabulary, can
allocation and filling of an earlier block overlap, and do references into the
arena survive an allocation?

Round one: accepted-fine, with two recorded losses — allocation serialised against
every fill in the same arena, and every reference into the pool died at every
allocation.

Strongest program (P2/P3: blocks allocated from one arena, filled while further
blocks are allocated, with a reference into an earlier block held across the
allocation):

```text
struct Arena<T> { buf: Box<Slots<T>> }

fn alloc<T>(a: &Arena<T>, x: own T) -> u64
    writes((*a.buf).next), writes((*a.buf).len)
    contract { requires (*a.buf).room > 0;
               ensures result == entry(*a.buf).len;
               ensures (*a.buf).len == result + 1; }

fn fill(a: &Arena<Block>, id: u64, seed: u64)   writes((*a.buf)[id])
    contract { requires id < (*a.buf).len; }

i1 = alloc(&a, empty_block())
i2 = alloc(&a, empty_block())
fill(&a, i1, s1);  fill(&a, i2, s2)          // may overlap: i1 != i2 from the contracts
fill(&a, i1, s1);  i3 = alloc(&a, blk)       // may overlap: a live slot never overlaps .next

p = &(*a.buf)[i1]
i4 = alloc(&a, node)                         // p survives; its bound survives too
(*a.buf)[i1].next = i4                       // link writes are paths, edges are indices (Rule 1)
use(p)
```

Trace:

- Rule 16's `alloc` is the arena's allocation and revision 4 gives it the row this
  cell needed: "the append slot of `*a.buf` and `(*a.buf).len`". Rule 10 clause 1
  then compares `writes((*a.buf)[i1])` with `writes((*a.buf).next)` and
  `writes((*a.buf).len)`, and Rule 6 settles both: "a live `r[i]` (which has `i <
  r.len`) never overlaps `r.next` or `r.free`", and the boundary is a separate
  number "stored with the block". Rule 13 reuses the same judgment, so a fill and an
  allocation may overlap. Round one's serialisation was caused by the row having to
  name the whole block, and it is gone.
- References survive too. Rule 10 clause 3 invalidates a bystander only when its
  path "has a proper prefix among the call's write paths"; `(*a.buf).next` and
  `(*a.buf).len` are not prefixes of `(*a.buf)[i1]`. Rule 6 states the conclusion in
  its own example, "p survives". The bound survives by the contracts: `i1 <
  entry(*a.buf).len` and `ensures (*a.buf).len == result + 1` give `i1 <
  (*a.buf).len` (Rule 11's `entry` carry). Round one's "one base reload per hop in a
  traversal that allocates" is removed for a reserved arena.
- Distinctness of two handles is still what makes parallel fills legal: `ensures
  result == entry(*a.buf).len` plus `ensures (*a.buf).len == result + 1` give `i1 !=
  i2` by affine comparison, which is what "indices or ranges proved distinct" asks
  for.
- The boundary of the result: an arena whose `alloc` may `grow` declares
  `writes(*a.buf)`, a proper prefix of every slot, so that arena's references die at
  every allocation and no fill may overlap an allocation. Reserving capacity —
  `requires (*a.buf).room > 0` on `alloc`, one `grow` outside the round — is what
  buys the property, and it is checked rather than assumed.
- Rule 16's stale-handle position is unchanged: a reused slot's index is a logic
  error, detected with a generation word as data.

Rewrite and cost: none needed. Allocation is one store plus one measure increment;
a handle dereference is a shift-add on a 32- or 64-bit index, one dependent load
like a pointer and half the edge footprint; the batching round one prescribed is no
longer required. One scope note carried over: a `Slots<T>` has uniform typed slots,
so P2's differently sized blocks carved from one reservation are still not this
cell's program.

Rules consistent: yes. (Rule 16's comment still says "see the open proposal" while
"Proposed additions awaiting owner ruling: None" — a stale cross-reference, not an
ambiguity.)
Task achievable: yes, with the overlap and the surviving references that round one
had to give up.
Verdict: accepted-fine

---

### 10-15: owned results, container-level rows, and the reader who was not in the call

Question: when a helper returns an owned index rather than a reference, does the
caller's reference to another slot survive the call that produced it?

Round one: rejected-real-cost — a container-wide row killed a reader on an
untouched slot, and the slot-granular rewrite cost one compare per fill.

Strongest program (P11: a write-once cache with many readers and one lazy writer,
where the writer may also append and may grow, and the keys come from data):

```text
cache: Box<Slots<Entry>>
tags:  Box<Array<u8>>

fn fill_slot(slot: &Entry, tag: &u8, key: Int) -> u64   writes(slot), writes(tag)
    contract { ensures result == key; }

fn append_entry(c: &Box<Slots<Entry>>, e: own Entry) -> u64
    writes((*c).next), writes((*c).len)
    contract { requires (*c).room > 0;  ensures result == entry(*c).len; }

fn refill(c: &Box<Slots<Entry>>, key: Int) -> u64   writes(*c)      // may grow

p = &(*cache)[i]                                  // i data-determined
j = append_entry(&cache, e)                       // p survives, no comparison at all
if i != k { fill_slot(&(*cache)[k], &(*tags)[k], key);  read(p) }    // one compare
m = refill(&cache, key);  read(p)                 // rejected: *cache is a proper prefix
q = &(*cache)[i];  read(q)                        // re-formed, bound re-proved
```

Trace:

- Rule 15 fixes the shape: "A function returns owned values only. Positions found by
  a search are returned as indices with a bounds relation in `ensures`." Each helper
  hands back an index and the caller forms its own reference.
- Rule 10 clause 3 is then read against three different rows, and revision 4 adds
  the first. (i) An appending writer names `(*c).next` and `(*c).len`; neither is a
  proper prefix of `(*cache)[i]` and Rule 6 says a live slot "never overlaps
  `r.next`", so the reader survives with no fact about `i` at all — this case cost
  one compare in round one and now costs nothing. (ii) A slot-precise writer
  substitutes `writes((*cache)[k])`, which overlaps `(*cache)[i]` "unless their
  indices or ranges are proved distinct" (Rule 3's may-overlap paragraph), so the
  writer supplies `i != k`, one compare per fill. (iii) A writer that may grow
  declares `writes(*c)`, a proper prefix, and every reader dies; that is correct,
  since `grow` "may reallocate".
- A fourth case is worth stating because the rule states it: a reader survives a
  `take_back` only with the stronger fact. `take_back` writes `r.last` and `r.len`,
  neither a proper prefix of `r[i]`, so the *reference* is not invalidated, but the
  bound is: from `i < entry(r).len` and `ensures r.len == entry(r).len - 1` one gets
  only `i <= r.len`. Rule 6 says it flatly: "`take_back`'s does not, so p dies at a
  `take_back`" — unless the program already knows `i != r.len - 1`, which is the
  same fact Rule 6 requires for `r.last` not to overlap `r[i]`.
- Rule 10 clause 2 keeps the value transfer clean: `fill_slot(..., move e)`
  contributes a consumption of `e`'s place, compared against `writes(cache[k])` —
  distinct places, accepted.

Rewrite and cost: the rewrite is to declare the narrowest true row and to relate the
two indices. Against a C++ cache that keeps an `Entry*` across another slot's fill:
zero for an appending writer, one predicted compare per fill when the writer's slot
is unrelated to the reader's, and one address recomputation plus a re-proved bound
after any call that may grow — where the C++ program is either correct by the
writer's knowledge or silently wrong. The compare is per fill, not per read.

Rules consistent: yes — clause 3 is exactly as coarse as the row it reads.
Task achievable: rewrite — the reader keeps its reference across appends for free
and across foreign fills for one compare; only a growing writer forces re-derivation.
Verdict: rejected-real-cost

---

### 12-16: joins over pool allocation — free list, generations, surviving handles

Question: a pool allocates from a free list or by appending; do the handles it hands
out stay proved in bounds across every join and every later allocation, and what does
reuse cost?

Round one: accepted-fine, with the verifier correcting the decisive step — the
caller's bound does *not* survive a later `obtain` by Rule 12's example; the row
writes the container, so the contract must carry a monotonicity clause.

Strongest program (a pool with an affine payload, a free list, a generation word,
and handles used after further allocations):

```text
struct Slot { data: Payload, gen: u32, next_free: u64 }     // Payload affine
struct Pool { buf: Box<Slots<Slot>>, free_head: u64 }

fn obtain(p: &Pool, x: own Payload) -> Result<u64, (Full, Payload)>
    writes((*p.buf).filled), writes((*p.buf).next), writes((*p.buf).len), writes(p.free_head)
    contract { ensures when Ok: result < (*p.buf).len;
               ensures (*p.buf).len >= entry(*p.buf).len; }
{
    if p.free_head != NONE {
        h = p.free_head
        if h < (*p.buf).len {                        // the free list is data, not a fact
            p.free_head = (*p.buf)[h].next_free
            (*p.buf)[h] = recycle((*p.buf)[h], move x)   // atomic update with an extra argument
            return Ok(h)
        }
        return Err((Full, move x))                   // unreachable arm; x must be discharged
    }
    if (*p.buf).room == 0 { return Err((Full, move x)) }
    id = (*p.buf).len
    place_back(&*p.buf, Slot { data: move x, gen: 0, next_free: NONE })
    Ok(id)
}

i = obtain(&pool, n1)?
j = obtain(&pool, n2)?                               // another write of the container
use(&(*pool.buf)[i])                                 // i still proved in bounds
```

Trace:

- The internal join is intersected correctly: the reuse edge proves `h < (*p.buf).len`
  from the written test ("refinement facts from a dominating branch", Rule 11), the
  append edge from `id == entry(*p.buf).len` and `place_back`'s `ensures r.len ==
  entry(r).len + 1`. Rule 12: "a fact survives only if it holds on every incoming
  edge", and `result < (*p.buf).len` does, so the `ensures when Ok` is provable and
  the caller needs no test of its own.
- Handle survival across the *second* call is the step round one got wrong and
  revision 4 makes textual. `obtain` writes the container, so Rule 11 invalidates the
  caller's `i < (*p.buf).len`; the repair is the monotonicity clause `ensures
  (*p.buf).len >= entry(*p.buf).len`, provable on both edges, combined with Rule 11's
  "a fact known before the call about a measure of an argument survives as a fact
  about `entry(p)` of that argument". Zero runtime cost, and it is now a stated rule
  rather than an inference from Rule 12's example.
- Revision 4 also supplies the step round one had to leave without a shape. With an
  affine payload the free step must consume the old occupant, and the atomic update
  is now "on any owned place with extra args", so `(*p.buf)[h] = recycle((*p.buf)[h],
  move x)` installs the replacement in the same commit. `f`'s row is empty (both
  arguments are by value), so "f's row must not overlap any prefix of `place`" holds.
- Occupancy stays data: Rule 6's "no program point can observe a slot inside the
  window as empty" plus Rule 12's "Structures whose occupancy is decided by data keep
  that occupancy as data". A freed slot holds a valid `Slot` whose `next_free` links
  the list.
- The cost is the same one cell 8-16 isolates, and it is the reason this cell's label
  moves. `h` comes out of the free list as data; the fact that every list member is in
  bounds is a quantified fact over elements, and Rule 11 has none. So the natural
  program — the C++ free-list pop with no test — is rejected, and the rewrite is the
  written test plus an arm that never runs. Round one priced this compare but still
  labelled the cell accepted because its own program had the test written in; under
  protocol step 4 the rejected form plus a rewrite that adds an instruction is a
  real-cost rejection, and 8-16 reaches the same conclusion from the same sentence.
- Rule 16 keeps the staleness story: a stale but in-bounds handle "names the current
  occupant of that slot: a logic error, not a memory error", detected with the
  generation word as data.

Rewrite and cost: the free-list bound test, one predicted compare and one dead arm
per allocation, plus the payload-returning signature that lets the dead arm discharge
a linear or affine `x`. The capacity compare before `place_back` is one C++'s
`push_back` also makes, so it is not a delta. The generation word is four bytes per
slot and one load plus one compare per dereference, `slotmap` parity. The in-bounds
fact itself costs nothing at any dereference, which is the property the contract
chain buys.

Rules consistent: yes.
Task achievable: rewrite — a reusing pool with stable, provably in-bounds handles and
detection works; one compare per allocation is not recoverable in this candidate
(the bitmask fact that would remove it is deferred).
Verdict: rejected-real-cost
