# Candidate x1 revision 4, targeted round two: reference-validity cells

Scope: the 18 cells 1-4, 1-10, 2-2, 2-7, 2-16, 3-3, 3-4, 3-7, 3-9, 3-10, 3-13, 4-12,
4-14, 5-15, 7-7, 8-9, 9-9, 15-15, re-derived against `../CANDIDATE-X1.md` revision 4
only. Theme: reference validity — the may-overlap prefix judgment, the index snapshot at
formation, scope-exit invalidation, move re-rooting, the variant-payload refinement fact,
and the static path shape, plus the `&[T]` range parameter type.

Round one's three memory-safety holes (unproved index distinctness in the prefix relation,
the index re-read after formation, the scope-exit release), its missing payload-path step,
and its unbounded target set for a self-extending rebinding are each checked here against
the revision-4 sentence that is supposed to close them. One new hole is reported, in 3-7:
revision 4 replaced the coarse `writes(buf)` of every shrinking window operation with
slot-precise parts, and the sentence that rescues an element reference across a `take_back`
is written for `r[i]` only, leaving a range reference `&r[lo..hi]` outstanding when the
window shrinks past it.

Notation is the rule file's: measures and window parts are members (`r.len`, `r.next`),
never `len_of`; there is no `par` statement — two adjacent statements are written adjacent
and the text says whether they may overlap.

---

### 1-4: Rule 1 and Rule 4 — no returned reference, no stored cursor, now under a static path shape

Question: with no reference returnable, storable or capturable, what does a position into a
Box-linked structure cost under revision 4, and does the static-path-shape sentence change
the fused form that round one relied on?

Round one: rejected-real-cost — all three cursor shapes die on Rule 4's single sentence, the
replay form costs a second full descent, and the only durable cursor was a handle under a
then-provisional Rule 16.

Strongest program:

```text
struct Node { key: Int, data: u64, left: Option<Box<Node>>, right: Option<Box<Node>> }

struct Cursor { at: &Node }                          // wanted: a position kept across unrelated work
fn find(root: &Node, k: Int) -> &Node                // wanted
fn finder(root: &Node) -> fn() -> &Node              // wanted

// round one's fused form, written as the loop a C++ programmer writes
fn find_update(root: &Node, k: Int, f: fn(v: own u64) -> own u64)  writes(root)
{
    p = root                                         // rebinding, not &root
    loop {
        if p.key == k { p.data = f(p.data); return }
        if k < p.key {
            match &p.left  { Some(b) => { p = &*b }   None => { return } }
        } else {
            match &p.right { Some(b) => { p = &*b }   None => { return } }
        }
    }
}
```

Trace:

- Rule 1 kills the stored cursor by its own example: "`struct Bad  { r: &Int }  // rejected`",
  and closes the wrapper route, "`Option<&T>  // rejected, whatever T is`".
- Rule 4 kills both function shapes in one sentence: a reference "cannot be assigned into any
  aggregate (Rule 1), cannot be returned, and cannot be captured by a function value that is
  stored or returned."
- Revision 4 first *helps*: the descent is spellable at all. Rule 2, "A payload step is
  available only under the refinement fact that the enum currently holds that variant, which
  a `match` or `if let` on the enum establishes in the selected arm and which any write to
  the enum invalidates." `match &p.left { Some(b) => ... }` binds `b` to `p.left.Some.0` and
  `&*b` extends it through the Box. Round one could not write this line at all.
- Revision 4 then refuses the loop: Rule 2, "A path has a static shape: a loop-carried
  rebinding may change only the index values inside the path, never extend the path through
  itself", illustrated by "`loop { p = &(*p.kids)[0] }  // rejected: the path would grow
  without bound; use recursion or indices`". Here `p = &*b` extends `p` by
  `.left.Some.0.*` every iteration, so `find_update` as written is rejected.
- The recursive form is accepted: each frame's path starts at its own parameter, Rule 2, "A
  path starts at a local variable or a parameter", so the shape is `n.left.Some.0.*` — depth
  four, constant. Rule 10: "Recursion is checked through contracts, never by unfolding
  bodies." The substituted row `writes(n.left.Some.0.*)` is covered by `writes(n)` under
  Rule 9's body check, since `n` is a prefix of it.
- Rule 16 is no longer provisional — "Confirmed by the owner on 2026-09-18" — so the handle
  cursor is a decided form, not a bet: `struct Cursor { id: u64, gen: u64 }` is owned data,
  which Rule 1 admits and Rule 15 lets a function return.

Rewrite and cost:

1. Fused, recursive: one descent, one call frame per level (`d` ≈ log n on a balanced tree),
   the frames off the dependent-load chain that dominates. Where `f` is a literal at the call
   site the body is monomorphic and this is the machine code C++ writes with a lambda; where
   the operation is genuinely dynamic it is one indirect call per hit, which the equivalent
   C++ (`std::function`, a virtual) also pays.
2. Replay: return `own Path` and re-descend. A second full chain of `d` dependent,
   frequently missing loads — this doubles the cost of the lookup, and it is the price of a
   position that must survive work unrelated to the container.
3. Handle into a `Box<Slots<Node>>` (Rule 16): re-forming `&(*pool.buf)[h]` is one
   shift-add plus Rule 7's bound test, "one compare and one predicted branch per
   data-determined index ... equal to safe Rust, real against C++". This is the only durable
   cursor, and it is now a confirmed form.

Rules consistent: yes — revision 4 makes the descent spellable and refuses only its
loop-carried rebinding, and books the refusal itself: "A rebinding of a reference that
extends its own path in a loop is refused; recursion or indices cost the same loads."
Task achievable: rewrite — fused-and-recursive for one-shot use, a handle for a durable
cursor; a stored pointer into a heap tree still does not exist.
Verdict: rejected-real-cost

---

### 1-10: Rule 1 and Rule 10 — the pairwise check meets index aliasing, and the bystander question round one had to withdraw

Question: with pointer aliasing gone and index aliasing left, what does one call that touches
several slots of one pool cost, and which of the caller's live references does it kill?

Round one: rejected-zero-cost, with one sub-claim withdrawn by the reviewer — the cell had
claimed a bystander `&g[t]` survives `writes(g[a])` for every `t`, which the frozen text did
not support and which, read permissively, admitted a read of freed storage.

Strongest program (three handles from a free list, so nothing is provable about them):

```text
g: Box<Slots<Node>>                                  // Rule 16: a pool is usage, not a type
fn link(g: &Box<Slots<Node>>, a: u64, m: u64, b: u64)
        writes((*g)[a]), writes((*g)[m]), writes((*g)[b])
    contract { requires a < (*g).len, m < (*g).len, b < (*g).len; }
{ (*g)[a].next = m; (*g)[m].prev = a; (*g)[m].next = b; (*g)[b].prev = m; }

m = pop_free(&freelist)                              // a handle from data
q = &(*g)[t]                                         // bystander at a data-determined slot
w = &(*(*g)[t].tag)                                  // bystander one level deeper: tag: Box<Tag>
link(&g, x, m, z)
```

Trace:

- Rule 9 admits the row: "Signatures never contain index expressions; an index enters an
  effect only through an argument, evaluated once at the call." `a`, `m`, `b` are arguments.
- Rule 10 clause 1 charges for it: "Two effects on overlapping paths where at least one is a
  write must be proved disjoint (different roots, or indices or ranges proved distinct);
  otherwise the call is rejected." Three write paths on one root need `x != m`, `m != z`,
  `x != z`, and Rule 11 has nothing to supply them with — the handles come from a free list
  and "There are no quantified facts over array elements". The call is rejected.
- The bystanders are now decided, which is the revision-4 change. Rule 3: "Whether one path
  is a prefix of another, and whether two paths overlap, is judged conservatively: two
  indexed positions on the same storage are taken to overlap unless their indices or ranges
  are proved distinct, exactly as in Rule 10." `q`'s path is `(*g)[t]`; the write paths are
  `(*g)[x]`, `(*g)[m]`, `(*g)[z]`, each an index position at the same depth, so each may be
  *the same place* as `q`'s path but none is a *proper* prefix of it. Rule 3's next sentence
  then governs: "Writing the storage at p's path or below it (a content write) does not
  invalidate p." `q` survives. `w`'s path is `(*g)[t].tag.*`, one step deeper, and `(*g)[x]`
  under the may-overlap judgment matches at the index position and is strictly shorter, hence
  a proper prefix: `w` is invalid after the call unless `t != x`, `t != m`, `t != z` are
  proved. Round one's withdrawn claim is exactly this distinction, and the rule file now
  makes it, with its own worked example: "`replace_at(&g, i, nb)  // writes (*g)[i]: overlaps
  (*g)[j] unless i != j is proved: q invalid`".
- Nothing here needed the permissive reading, so the cell's rewrite is untouched.

Rewrite and cost:

1. **Four ordinary statements in the caller.** Rule 10 opens "At a call, substitute the
   actual argument paths into the callee's row"; two adjacent *statements* are never compared
   pairwise for acceptance. Revision 4's Rule 13 only grants permission — "Two adjacent
   statements of one block *may* overlap when ..." — so failing its test costs overlap, never
   acceptance. The four stores stand in source order with the C++ meaning when handles
   coincide. Cost: zero, beyond the Rule 7 bound tests already charged per hop.
2. **Keep the helper, declare `writes(*g)`.** One effect, nothing to compare, accepted with
   no facts. Zero sequentially; it forfeits every overlap with any other access to `*g`, and
   it now also kills `q` (the container is a proper prefix of every slot path), where the
   fine row did not.
3. **Test the three disequalities.** Three compares, three predicted branches and a
   duplicated else arm per insert, against zero in C++. Strictly worse than 1 and 2.

Best: rewrite 1, zero runtime cost against the C++ insert. The recorded structure is the
asymmetry: the fine row buys overlap permission and charges distinctness proofs; the coarse
row is free sequentially and forfeits both the overlap and the caller's bystanders.

Rules consistent: yes — and more so than in round one, because Rule 3's may-overlap sentence
now decides the bystander question the same way Rule 10 clause 1 decides the argument
question.
Task achievable: rewrite, at zero cost.
Verdict: rejected-zero-cost

---

### 2-2: Rule 2 against itself — what a loop-carried rebinding may name, and how big its target set is

Question: when a reference is rebound on a back edge, what is its target set at the loop
header, and which rebindings does the rule admit?

Round one: undecided-rule-gap. The reviewer dismissed the cell's stated ground (an unbounded
target set) and re-grounded it on a different hole: Rule 2's path grammar had no
variant-payload element, so the walk could not be written at all.

Strongest program (an owned list walked iteratively, the shape every C++ and Rust programmer
writes):

```text
struct Node { value: Int, next: Option<Box<Node>> }

fn sum(head: &Node) -> Int  reads(head)
{
    s = 0
    p = head                                  // rebinding a reference parameter: Rule 2
    loop {
        s = s + p.value                       // content read at p's path
        match &p.next { Some(b) => { p = &*b }   None => { return s } }
    }
}
```

The two rebindings that must be separated from it:

```text
q = p                                         // Rule 2: "q names the same path; both may be live"
r = &p.x                                      // Rule 2: "a reference extends a path"
&p                                            // Rule 2: "rejected: a reference variable is not storage"
w = if cond { &v1 } else { &v2 }              // target set { v1, v2 }
loop { i = next_i(i); p = &(*v)[i] }          // index-only rebinding: accepted, see below
```

Trace:

- Both of round one's grounds are closed by revision 4, in opposite directions.
- The payload hole is closed by admission. Rule 2: "A payload step is available only under
  the refinement fact that the enum currently holds that variant, which a `match` or `if let`
  on the enum establishes in the selected arm and which any write to the enum invalidates."
  `match &p.next { Some(b) => ... }` is now the rule file's own form, and `&*b` continues
  through the Box.
- The target-set hole is closed by rejection. Rule 2: "A path has a static shape: a
  loop-carried rebinding may change only the index values inside the path, never extend the
  path through itself." `p = &*b` extends `p`'s path by `.next.Some.0.*` every iteration, so
  `sum` is rejected at that line, with the file's own rejected example one line away:
  "`loop { p = &(*p.kids)[0] }  // rejected: the path would grow without bound`".
- What remains of the original question is now answerable. Two sentences bound the set. First
  the shape is fixed: a loop-carried rebinding may only change index values, so the *shapes* a
  reference may name are the finitely many written in the program text. Second the index is a
  value, not an expression to be re-read: "An index expression inside a path is evaluated when
  the reference is formed; the path records that value, and later assignments to the variables
  the expression used do not change it." So a target set at a join is the union over formation
  sites, and Rule 12's "a reference's target set is the union of its targets (Rule 2)" is a
  union of finitely many shapes with symbolic index values, each checked by Rule 2's "every
  check on it must hold for every member of the set". The index-only loop above is therefore
  ordinary: one shape, `(*v)[·]`, one check `i < (*v).len` per formation.
- Validity inside an iteration was never the problem and is unchanged: Rule 3, "Writing the
  storage at p's path or below it (a content write) does not invalidate p."

Rewrite and cost: for the walk, two forms.

(a) Recursion. `fn sum_from(n: &Node) -> Int reads(n) { match &n.next { Some(b) => n.value +
sum_from(&*b), None => n.value } }` — each frame re-roots at its parameter, so the shape is
static and Rule 10's contract-checked recursion applies. Cost against `while (p) p = p->next`:
one call and one return per element; the rule file promises no tail-call form, and it says
nothing about stack depth, so this form is not available for a list whose length is data.

(b) Index links in one window: `struct Node { value: Int, next: u64 }` inside a
`Box<Slots<Node>>`, `loop { s = s + (*v)[i].value; if (*v)[i].next == NIL { break }; i =
(*v)[i].next }`. The rebinding changes only an index value, so Rule 2 admits it. Cost: the
same one dependent load per hop (the address is `base + i*size`, one addressing mode), plus
Rule 7's test, since "every stored handle is in bounds" is a quantified fact Rule 11 excludes:
"One compare and one predicted branch per data-determined index ... equal to safe Rust, real
against C++."

Rules consistent: yes — revision 4 answers both of round one's grounds with quoted sentences,
and the answers agree: the set is finite because the shape is fixed and the index is a
recorded value.
Task achievable: rewrite — iterative pointer chasing over an owned list is refused; the
generally available replacement costs one compare and one predicted branch per hop.
Verdict: rejected-real-cost

---

### 2-7: Rule 2 against itself — what `[i]` in a formed path denotes after `i` changes

Question: is the index inside a formed reference's path the value at formation or the current
value of the index variable?

Round one: undecided-rule-gap, and the reviewer called it the sharpest gap in its sheet: the
two readings put the write in different slots and differ observably and unsafely.

Strongest program (the index is mutated between formation and use, and the mutation is large
enough to leave the window):

```text
p = &(*buf)[i]                       // requires i < (*buf).len
i = i + 1000000                      // an ordinary integer local
put_at(p, 7)                         // fn put_at(slot: &Int, x: own Int)  writes(slot)

part = &(*buf)[lo..hi]               // requires lo <= hi <= (*buf).len; part.len == hi - lo
hi = 0
k = part.len                         // the range's own measure
```

Trace:

- Closed by one sentence, Rule 2: "An index expression inside a path is evaluated when the
  reference is formed; the path records that value, and later assignments to the variables the
  expression used do not change it." The rule file's own lines say it twice:
  "`p = &(*v)[i]  // p names slot i of the window inside v, with i's value at this point`" and
  "`i = i + 1  // p still names the old slot`".
- So `put_at(p, 7)` writes the slot that was proved in bounds at formation, and the
  out-of-window value of `i` is irrelevant. The bounds obligation was discharged once, at
  formation, by Rule 7's "Every index must be proved in bounds"; there is no second index
  expression to discharge at the use.
- The same sentence settles the range endpoints: `part` records `lo` and `hi`, so
  `part.len == hi - lo` is a number fixed at formation and later assignments to `hi` do not
  move the range. This is what makes `&[T]` a usable parameter type in Rule 7 —
  "a reference kind with measure `len`, formed only from an indexable or from another range
  reference, never a stored value".
- Round one's unsafe reading is gone in both directions: under the snapshot the address is
  computed once and the write lands inside the window that was proved, and Rule 10's pairwise
  comparison substitutes recorded values rather than re-read variables.
- One consequence to state, because it is where a writer can still lose: distinctness between
  two snapshots taken from the same mutated variable is not automatic. Rule 11's vocabulary is
  "affine comparisons over measures and integer values", so `p = &(*r)[i]; i = i + 1; q =
  &(*r)[i]` gives two recorded values that a reader must be able to name in order to compare.
  Binding them, `i0 = i; p = &(*r)[i0]; i1 = i0 + 1; q = &(*r)[i1]`, makes `i0 != i1` an
  affine consequence. Failing to prove it is conservative — Rule 3's may-overlap judgment
  treats the two positions as overlapping — so the failure mode is a rejected call or a dead
  reference, never a silent aliased write.

Rewrite and cost: none needed. Where two snapshots of one variable must be proved apart, bind
a fresh integer per formation; the index values already exist in registers, so this is zero
runtime cost and gives the backend the same disjointness a C++ programmer asserts informally.

Rules consistent: yes — the snapshot reading is stated, and Rule 3's may-overlap judgment
makes the conservative direction the safe direction.
Task achievable: yes.
Verdict: accepted-fine

---

### 2-16: Rule 2 and Rule 16 — a reference into a pool slot across allocation and reset

Question: what happens to a reference into a pool slot when the pool allocates, when it is
reset, and when a stale handle is used again?

Round one: rejected-real-cost — `alloc` declared `writes(a.buf)`, a proper prefix of every
slot path, so every allocation killed every reference into the pool and the program re-formed
after each one; the stale handle was accepted and listed as an identity failure.

Strongest program (handles from data, a reference held across an allocation, a reset, and a
reuse of the same slot number):

```text
struct Arena<T> { buf: Box<Slots<T>> }

fn alloc<T>(a: &Arena<T>, x: own T) -> u64   writes((*a.buf).next), writes((*a.buf).len)
    contract { requires (*a.buf).room > 0;
               ensures result == entry(*a.buf).len; ensures (*a.buf).len == result + 1; }

id1 = alloc(&a, Node { next: NIL, data: 0 })
p   = &(*a.buf)[id1]                       // requires id1 < (*a.buf).len
id2 = alloc(&a, Node { next: id1, data: 1 })
work(p)                                    // accepted under revision 4
truncate(&a.buf, 0)                        // library code: a loop of take_back
use(p)                                     // rejected
id3 = alloc(&a, Node { next: NIL, data: 7 })   // id3 == id1 numerically
q   = &(*a.buf)[id1]                       // accepted: names the new occupant
```

Trace:

- The revision-4 change is the row, not the rule. Rule 6 gives `place_back` the slot-precise
  row "`place_back(&r, x)  writes(r.next), writes(r.len)`", and states the consequence
  directly: "a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`". The
  file writes the pool case out in full: "`fn add_node(g: &Graph, n: own Node) -> u64
  writes((*g.nodes).next), writes((*g.nodes).len)` ... `id = add_node(&g, node)  // p
  survives: (*g.nodes)[i] with i < len never overlaps .next`". Rule 6 also carries the
  formation fact across: "`place_back`'s `ensures` carries it across the call". So `work(p)`
  is accepted, and the re-formation round one paid after every allocation is gone.
- It is sound because the operation cannot move the block: `place_back`'s
  "`requires r.room > 0`" excludes a reallocation, and Rule 16's own `alloc` contract carries
  that requirement to the user function. A pool whose `alloc` grows instead declares
  `grow(&b, cap)  writes(*b)`, a proper prefix of every slot path, and correctly kills
  everything, which is exactly what a `realloc` does.
- The reset still kills. `truncate` is "Library code, all zero-cost compositions of the
  above", a loop of `take_back`, whose row is "`take_back(&r)  writes(r.last), writes(r.len)`"
  and whose effect on references Rule 6 states outright: "`take_back`'s does not, so p dies at
  a take_back." So `use(p)` after the reset is rejected, and the program cannot read a slot
  the reset emptied.
- The stale handle is Rule 16's confirmed ruling, not an accident: "A stale index that is
  still in bounds names the current occupant of that slot: a logic error, not a memory error."
  `q` is well formed, Rule 7's bound holds, and Rule 6 guarantees the slot holds a value —
  "no slot ever carries a tag, and no program point can observe a slot inside the window as
  empty". Memory-safe, identity-unsafe, by decision.
- Range references into a pool across a reset are *not* covered by the `take_back` sentence,
  which is written for `p = &r[i]`; see the gap recorded in 3-7. The program above uses only
  element references.

Rewrite and cost, against a C++ pool of raw pointers:

- Holding a reference across allocation is now free and is in fact stronger than the C++ form
  it replaces: a raw `Node*` into a `std::vector` is invalid after a `push_back` that
  reallocates, while here the row states which operations preserve it and which do not.
- Per traversal hop, where the next handle comes out of a node field, one bounds compare
  remains, since "every stored handle is in bounds" is quantified and Rule 11 excludes it:
  the known cost "One compare and one predicted branch per data-determined index ... equal to
  safe Rust, real against C++".
- Detecting reuse costs one generation word per slot and one compare per dereference, which
  Rule 16 books as the program's own cost: "Generation words for identity across slot reuse
  cost what a Rust slotmap costs."

Rules consistent: yes — the window parts make `place_back` preserving and `take_back`
destructive for exactly the right references, and Rule 16 states the residual identity
weakness rather than hiding it.
Task achievable: yes for the pool itself; the identity guarantee P2 asks for ("every old
pointer into b2 must be refused afterwards") is achieved only as a program-level generation
word, at one word and one compare.
Verdict: accepted-fine

---
### 3-3: Rule 3 against itself — repeated, chained and loop-carried reference formation

Question: when references are formed from each other, aliased, and re-formed from themselves
inside a loop, does Rule 3's validity fact stay decidable?

Round one: undecided-rule-gap — Rule 2 defined a reference's target set at a join but never
bounded it, and a self-re-formed reference made it unbounded; the reviewer's own stronger
example (`p = &(*p.kids)[0]` in a loop) showed the gap was independent of the missing payload
step.

Strongest program (the reviewer's example, in revision-4 spelling, with an aliased and a
chained reference live across the loop):

```text
struct Tree { n: Int, kids: Box<Slots<Tree>> }

p = &t
r = &t.n                                   // chained: a reference below p's path
loop {
    if (*p.kids).len == 0 { break }
    p = &(*p.kids)[0]                      // re-formed from p's own path
}
```

Ordinary program (the shape the rule was written for, fully decided):

```text
p = &(*v)[i]
q = &p.x                                   // Rule 2: "a reference extends a path"
r = &(*v)[i]                               // a second name for the same path; both may be live
(*v)[j] = 5                                // content write: p and r stay valid whether or not i == j
bump(&(*v)[i])                             // writes((*v)[i]): p, r valid; q invalid (proper prefix)
use(q)                                     // rejected
q = &p.x                                   // "Validity is re-established only by forming the reference again"
grow(&v, cap)                              // writes(*v): a proper prefix of all three: p, q, r invalid
```

Trace:

- The loop is now rejected by a sentence, not left open. Rule 2: "A path has a static shape: a
  loop-carried rebinding may change only the index values inside the path, never extend the
  path through itself", with the file's own line "`loop { p = &(*p.kids)[0] }  // rejected: the
  path would grow without bound; use recursion or indices`" — the reviewer's program verbatim.
  Round one's unbounded target set cannot arise: the set of shapes is the set written in the
  text, and each index step records a value (Rule 2's formation sentence), so Rule 2's "every
  check on it must hold for every member of the set" is a check over finitely many shapes.
- Round one's second obstacle is also gone, in the other direction: the variant-payload
  descent that the earlier grammar could not spell is now Rule 2's "A payload step is
  available only under the refinement fact that the enum currently holds that variant". It
  does not rescue the loop, because a payload step extends the path like any other.
- The ordinary program is decided exactly as before, with one line now sharper. Rule 3's
  content clause keeps `p` and `r` alive across `(*v)[j] = 5`: "Writing the storage at p's path
  or below it (a content write) does not invalidate p", and the rule file's own annotation
  covers the aliasing case, "`p stays valid whether or not i == j`" — sound because `(*v)[j]`
  is an index position at `p`'s own depth and so is never a *proper* prefix of `p`'s path.
  `q = &p.x` is one step deeper, so `writes((*v)[i])` is a proper prefix and `q` dies; under
  revision 4 it would equally die from `bump(&(*v)[k])` for an unproved `k`, by Rule 3's
  "two indexed positions on the same storage are taken to overlap unless their indices or
  ranges are proved distinct".
- `grow(&v, cap)  writes(*v)` remains a proper prefix of every path into the block and kills
  all three, which is what a reallocation requires.

Rewrite and cost: carry an index, not a reference, with the nodes in one window:

```text
i = root_idx
loop {
    if (*nodes)[i].kid_count == 0 { break }
    i = (*nodes)[i].first_kid                 // only an index value changes: Rule 2 admits it
}
```

Against a C++ pointer chase this is the same one dependent load per hop — the address is
`base + i*size`, one addressing mode — plus Rule 7's test on a data-determined handle, "one
compare and one predicted branch", off the load's dependency chain and exactly what safe Rust
pays. The alternative rewrite, recursion, re-roots the path at each frame's parameter and is
checked by Rule 10's "Recursion is checked through contracts, never by unfolding bodies"; it
costs one call frame per hop, and the rule file promises no tail-call form and says nothing
about stack depth, so it is not the general answer for a structure whose depth is data.

Rules consistent: yes — the same two sentences that bound the target set also refuse the
program that made it grow, and Rule 3's prefix and content clauses decide every non-loop line.
Task achievable: rewrite — index-carried traversal is fully decidable, at one predicted
compare per hop.
Verdict: rejected-real-cost

---

### 3-4: Rule 3 and Rule 4 — validity across the return boundary references may not cross

Question: with references neither returned nor stored, what carries a located result from the
callee that found it to the caller that writes it, and what invalidates the carrier?

Round one: rejected-zero-cost — the index survives what a pointer would not, because
round one's append operation (`push_nogrow`, now `place_back`) re-established the bound; the deep case goes inward through a
function-typed parameter. The reviewer recorded two trace holes: the capacity precondition
was never discharged, and the inward form's indirect call is zero only under specialization
the rules do not promise.

Strongest program (deep structure, separate compilation, the callee has already walked to the
slot):

```text
// C++: Val* p = tree.find(k); *p += 1;            // one descent
fn lookup(t: &Tree, k: Int) -> Option<u64>   reads(t)                       // Rule 4: an index
fn lookup_update(t: &Tree, k: Int, f: fn(v: &Val) writes(v))  writes(t)     // the inward form
```

Ordinary program (flat window; the index outlives what a pointer would not, and under
revision 4 so does the reference):

```text
n = (*v).len
match find(&*v, k) {
    Some(i) => {                                  // ensures when Some: result < v.len  ->  i < n
        p = &(*v)[i]                              // formed under i < (*v).len
        if (*v).room > 0 {
            place_back(&*v, x)                    // writes((*v).next), writes((*v).len)
            bump(p)                               // accepted: p survives the append
        } else {
            grow(&v, 2 * (*v).cap)?               // writes(*v): p invalid here
            p = &(*v)[i]                          // re-formed; i < (*v).len from grow's ensures
            place_back(&*v, x); bump(p)
        }
    }
    None => ...
}
```

Trace:

- Rule 4 rejects the C++ shape at the signature: a reference "cannot be assigned into any
  aggregate (Rule 1), cannot be returned, and cannot be captured by a function value that is
  stored or returned", and prescribes the substitute: "A function that 'finds' something
  returns an index or other owned data; the caller forms the reference."
- Round one carried only the *fact* across the append. Revision 4 carries the *reference*:
  `place_back`'s row is "`writes(r.next), writes(r.len)`", and Rule 6 states the consequence,
  "a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`" and
  "`place_back`'s `ensures` carries it across the call". So `bump(p)` needs no re-formation.
  The fact route still works underneath it: Rule 11, "Across a call, a fact known before the
  call about a measure of an argument survives as a fact about `entry(p)` of that argument",
  with `ensures r.len == entry(r).len + 1`.
- The reviewer's first trace hole is discharged explicitly above: `place_back`'s
  "`requires r.room > 0`" is tested, and the growth arm calls `grow(&b, cap)  writes(*b)`,
  which is a proper prefix of `(*v)[i]` and correctly invalidates `p` — the one operation that
  may move the block is the one operation that kills the reference. `grow`'s
  "`ensures (*b).cap == cap, (*b).len == entry(*b).len`" re-establishes `i < (*v).len` for the
  re-formation.
- The deep case is unchanged: an index into a leaf does not name the leaf, and a path may not
  cross the return (Rule 1, Rule 4), so either the caller re-descends — four dependent loads
  again — or the update goes inward, which Rule 4 permits ("may be bound to a local, passed as
  a call argument") and Rule 10 types ("Function-typed parameters carry a full signature with
  its own row and contract, and a call through one uses that row").

Rewrite and cost: `lookup_update`. Memory traffic is identical to the C++ pointer form: one
descent, one write. On the reviewer's residual: where `f` is a literal at the call site the
instantiation is monomorphic and the call is direct; where the operation is genuinely dynamic,
the equivalent C++ (`std::function`, a virtual call) pays the same indirect call, so the cost
against the *equivalent* C++ program is zero in both regimes. For the flat container revision 4
removes even the re-formed address that round one charged.

Rules consistent: yes — Rule 4 removes the carrier, Rule 6's window parts now preserve the
reference itself across an append, and Rule 10 supplies the inward form for deep structures.
Task achievable: rewrite — find-then-write is expressible with no extra memory traffic.
Verdict: rejected-zero-cost

---

### 3-7: Rule 3 and Rule 7 — validity versus the bounds obligation, now that shrinking is slot-precise

Question: can validity and in-bounds-ness diverge — a valid reference naming a slot outside
the window, or a live in-bounds index that yields no valid reference?

Round one: accepted-fine. Its load-bearing argument was that "every shrink (`pop`,
`swap_remove`, `truncate`) declares `writes(buf)` ... which is a proper prefix of every
element path and invalidates by Rule 3. So validity implies in-bounds without a separate
check." The reviewer struck one general claim — that range and element references never
invalidate each other — and left the verdict.

Strongest program (a range reference held while the window shrinks past it; affine elements,
so the drained slots own heap objects):

```text
struct Entry { t: Box<Tag> }                  // affine
b: Box<Slots<Entry>>                          // say (*b).len == 10

part = &(*b)[0..2]                            // Rule 7: requires 0 <= 2 <= (*b).len; part.len == 2
while (*b).len > 0 {
    e = take_back(&*b)                        // writes((*b).last), writes((*b).len)
    consume(move e)
}
x = &part[1]                                  // Rule 7: requires 1 < part.len, and part.len == 2
read(x)                                       // slot 1 has left the window; its Entry was consumed
```

Second program (the direction that is decided, and the probe that is unchanged):

```text
p = &(*b)[i]                                  // formed under i < (*b).len
take_back(&*b)                                // Rule 6: "take_back's does not, so p dies at a take_back"
use(p)                                        // rejected

mask = (*tags).len - 1                        // len is a power of two by construction
i = h & mask
if i < (*tags).len { q = &(*slots)[i]; ... }  // Rule 7: "the test establishes the fact; one compare, no trap"
part = &(*buf)[lo..hi]; other = &(*buf)[hi..n]
fill(other)                                   // writes((*buf)[hi..n]): ranges proved distinct: part valid
grow(&buf, cap)                               // writes(*buf): a proper prefix of both: both invalid
```

Trace:

- The element direction is decided, and revision 4 decides it by a different sentence than
  round one used. The coarse `writes(buf)` is gone; `take_back`'s row is
  "`writes(r.last), writes(r.len)`". What rescues an element reference is Rule 6's own
  statement: "A reference `p = &r[i]` into a window is formed under the fact `i < r.len` and
  stays valid while that fact holds: `place_back`'s `ensures` carries it across the call;
  `take_back`'s does not, so p dies at a take_back." So for `r[i]`, validity still implies
  in-bounds, and the second program is decided in every line.
- The reviewer's struck claim is now restored in a narrowed and correct form by Rule 3:
  "Whether one path is a prefix of another, and whether two paths overlap, is judged
  conservatively: two indexed positions on the same storage are taken to overlap unless their
  indices or ranges are proved distinct, exactly as in Rule 10." `(*buf)[hi..n]` and
  `(*buf)[lo..hi]` are proved distinct by arithmetic, so `part` survives `fill`; had the
  ranges not been proved apart, `part` would be treated as overlapping the write.
- **The gap.** For the first program the same two sentences point opposite ways, and the
  difference is memory safety. `take_back`'s write path is `(*b).last`, defined in Rule 6 as
  "the last filled slot, at index `r.len - 1`", "interpreted at call entry". While
  `(*b).len - 1 >= 2` it is proved distinct from the range `[0..2)` and nothing happens. At
  the call where `(*b).len == 2` the written position is `(*b)[1]`, which lies inside the
  range and is not proved distinct. That position is an index step at the same depth as the
  range step, so it is not a *proper* prefix of `(*b)[0..2]`, and Rule 3's other sentence then
  applies literally: "Writing the storage at p's path or below it (a content write) does not
  invalidate p." Under that reading `part` is still valid after the drain, `part.len` is the
  number recorded at formation ("An index expression inside a path is evaluated when the
  reference is formed; the path records that value"), Rule 7 accepts `&part[1]` because
  "`part.len == hi - lo`", and the program reads a slot that has left the window — of which
  Rule 6 says only "slots `[0, len)` hold values, `[len, cap)` hold nothing". For an affine
  element the `Tag` behind it has already been released by `consume`. Under the other reading
  — that Rule 6's "stays valid while that fact holds" governs every reference into a window,
  with `lo <= hi <= r.len` as the range's formation fact — `part` dies at the `take_back` that
  first overlaps it, or at the first one at all, and the continuation re-forms at zero cost.
  The rule text states the first sentence generally and the second only for `p = &r[i]`, so it
  does not decide the range case, and the two readings differ by a use-after-free.
  Smallest fix: one clause extending Rule 6's sentence to range references, "a reference
  `&r[lo..hi]` stays valid while `hi <= r.len` holds", or equivalently making every operation
  that decreases `r.len` invalidate every reference into `r`.
- Unchanged and not a gap: the other direction of divergence is by design — an in-bounds index
  yields no reference until one is formed, because "Validity is re-established only by forming
  the reference again" (Rule 3) — and the mask identity is still outside the fact vocabulary,
  Rule 11 admitting only "affine comparisons over measures and integer values", with the
  bitmask fact explicitly "Deferred to a future concurrency and layout round".

Rewrite and cost: for the probe, bind `n = (*b).len` once and carry affine relations so that
counted loops need no test, leaving "one compare and one predicted branch per access" where
the index is genuinely data-determined — identical to safe Rust, one compare more than an
unchecked C++ array access. For the range-across-a-shrink program, the safe reading's rewrite
(re-form the range after the drain) is zero runtime cost, which is a reason to expect that
reading, not a reason to assume it.

Rules consistent: not for this case — Rule 3's content-write sentence and Rule 6's
"stays valid while that fact holds" give a range reference opposite fates across a `take_back`,
and only the second is memory-safe.
Task achievable: yes for every program that holds element references; the range-across-a-shrink
program cannot be traced either way.
Verdict: undecided-rule-gap

---

### 3-9: Rule 3 and Rule 9 — effect-row granularity decides which bystander references survive

Question: does the coarseness of a callee's declared row, and only that, decide which of the
caller's references die — and can a writer always declare finely enough?

Round one: corrected by the reviewer from accepted-fine to undecided-rule-gap. The cell had
argued that an argument-index-granular row leaves bystanders alone; with an affine element the
reviewer built a use-after-free from that reading (`replace_at(&g, i, nb)` freeing the Box that
`q = &*(g[j])` names) that the frozen text did not refuse.

Strongest program (the reviewer's construction, plus a payload-refinement bystander, all
across a separate-compilation boundary):

```text
struct Entry { kind: Option<Box<Tag>>, count: u64 }          // affine
s: Box<Slots<Entry>>

fn replace_at(s: &Box<Slots<Entry>>, i: u64, x: own Entry)   writes((*s)[i])
    contract { requires i < (*s).len; }
{ (*s)[i] = move x }          // Rule 6: "Assigning over any owned place releases the old value if it is affine"

j = probe(&*s, key)                                          // data-determined
p = &(*s)[j]                                                 // the slot itself
q = &(*s)[j].count                                           // one step below the slot
match &(*s)[j].kind {
    Some(c) => {                                             // c names (*s)[j].kind.Some.0
        replace_at(&s, i, ne)                                // writes (*s)[i], i data-determined
        use(p)                                               // accepted
        y = *q                                               // rejected unless i != j is proved
        z = (*c).id                                          // rejected unless i != j is proved
    }
    None => {}
}
```

Trace:

- The hole is closed by the sentence added to Rule 3: "Whether one path is a prefix of another,
  and whether two paths overlap, is judged conservatively: two indexed positions on the same
  storage are taken to overlap unless their indices or ranges are proved distinct, exactly as
  in Rule 10." The rule file then works the reviewer's own program: "`replace_at(&g, i, nb)  //
  writes (*g)[i]: overlaps (*g)[j] unless i != j is proved: q invalid`", and adds "with the
  fact `i != j`, q survives".
- So `q` dies: under the may-overlap judgment `(*s)[i]` matches `(*s)[j]` at the index step and
  is strictly shorter than `(*s)[j].count`, hence a proper prefix. `c` dies for the second
  reason in Rule 3's list, "a refinement fact that a payload step in p's path depends on is
  invalidated": the fact was established by the `match` on `(*s)[j].kind` and Rule 2 says it is
  invalidated by "any write to the enum", which `writes((*s)[i])` is, under the same
  conservative judgment, unless `i != j`.
- `p` survives, and that is sound rather than lucky: `(*s)[i]` is an index position at `p`'s own
  depth, so it is never a *proper* prefix of `(*s)[j]`, the write is a content write at a
  may-equal place, and whatever it leaves behind is a whole value of the element type — Rule 6,
  "no program point can observe a slot inside the window as empty", Rule 12, "every path is
  either wholly present or the program cannot name it".
- Granularity itself is what the cell is about, and revision 4 improves it twice. First, an
  appending callee can now be precise without naming an index: Rule 6's window parts are
  ordinary vocabulary — "This vocabulary is ordinary: the rows below use nothing a user
  function cannot write" — so `fn add(s: &Box<Slots<Entry>>, e: own Entry) writes((*s).next),
  writes((*s).len)` leaves every `&(*s)[k]` alive across an opaque call, which round one could
  only obtain by proposing a new `appends` effect kind. Second, the coarse case is unchanged
  and still forced in exactly one place: a callee that writes an element it *chooses itself* (a
  hash insert, a free-list pop) cannot name it, because Rule 9 says "Signatures never contain
  index expressions; an index enters an effect only through an argument, evaluated once at the
  call", so it declares `writes(*s)` and kills every reference into the block, `p` included.

Rewrite and cost: split the coarse helper into a `reads` search returning an index (Rule 4's
own prescription) and a fine `writes((*s)[i])` writer. The split repeats one address
computation at the caller — `base + i*size`, an addressing mode — and no copy, branch or lost
parallelism; it *gains* overlap permission, since `writes((*s)[i])` and `writes((*s)[k])` may
overlap under Rule 13 with a proved `i != k` where two `writes(*s)` never can. Where a
bystander below a slot must survive an unproved-index write, the caller re-forms it after the
call, which costs the same address computation.

Rules consistent: yes — the prefix relation, the may-overlap judgment and the payload
refinement fact now agree, and the reviewer's use-after-free is refused by the rule file's own
worked example.
Task achievable: yes.
Verdict: accepted-fine

---
### 3-10: Rule 3 and Rule 10 — the pairwise check and bystander invalidation together

Question: when two argument paths may be the same slot, do Rule 10's pairwise check and its
bystander clause combine without either admitting an aliased write or killing references that
nothing relocated?

Round one: corrected by the reviewer from accepted-fine to rejected-real-cost — the strongest
program is rejected by Rule 10 clause 1, the only rewrite that keeps P1's shape is a dominating
`if i != j`, and P1 explicitly requires "no runtime check", so the compare is unavoidable.

Strongest program (both indices data-determined, callee separately compiled, references live
across the call at three depths):

```text
struct Entry { child: Box<Sub>, count: u64 }          // affine
fn merge(dst: &Entry, src: &Entry)   writes(dst), reads(src)

i = find_a(&*v, ka)                                   // data-determined
j = find_b(&*v, kb)                                   // data-determined
p = &(*v)[m]                                          // bystander at a slot, m data-determined
r = &(*v)[m].count                                    // bystander below a slot
merge(&(*v)[i], &(*v)[j])                             // rejected without i != j
```

Adjacent statements, which under revision 4 are a different question from a call:

```text
bump(&(*v)[i]); bump(&(*v)[j])                        // accepted; may overlap only given i != j
swap(&(*v)[i], &(*v)[j])                              // accepted, aliasing included
swap(&b.left, &(*b.left).right)                       // rejected: overlapping, and not the same place
```

Trace:

- Rule 10 clause 1 rejects the call verbatim: "Two effects on overlapping paths where at least
  one is a write must be proved disjoint (different roots, or indices or ranges proved
  distinct); otherwise the call is rejected", with the file's own line
  "`two(&r[i], &r[j])  // accepted only with the fact i != j`". Rule 11 can supply the fact
  only "from a dominating branch" when the indices come from data.
- Clause 3 decides the bystanders, now against the explicit may-overlap judgment of Rule 3:
  `p`'s path `(*v)[m]` has no proper prefix among the write paths, because `(*v)[i]` is an
  index position at the same depth, so `p` survives whatever `m` is; `r`'s path is one step
  deeper, so `(*v)[i]` *is* a proper prefix under the conservative judgment and `r` dies unless
  `m != i` is proved. Round one asserted the first half and had to narrow the second; revision 4
  states both. Clause 3's second sentence still saves the arguments themselves: "A reference
  that is itself an argument is the thing being accessed, not a bystander."
- Two revision-4 changes move the cost, without moving the verdict.
  First, **there is no `par` statement**: Rule 13 now reads "Two adjacent statements of one
  block *may* overlap when the first's write paths are disjoint from the second's read and
  write paths and vice versa". Failing that test costs overlap permission, not acceptance, so
  the sequential half of P1 — "sequential legality by proof" — is free: `bump(&(*v)[i]);
  bump(&(*v)[j])` is two calls with one write path each, never compared with one another, and
  each accepted on its own. What still needs the fact is P1's other half, "parallel permission
  for the two writes".
  Second, the aliasing-tolerant operation exists, but only as a built-in: Rule 6,
  "`swap(p: &T, q: &T)  writes(p), writes(q)  // built in; p and q may be the same place, then
  nothing happens`", and Rule 10 clause 1, "The built-in `swap` is the one operation whose two
  arguments may be the same place", with the file naming the case it buys —
  "`swap(&r[i], &r[j])  // no i != j branch needed in a partition loop`". A user function
  cannot declare that tolerance; there is no marker for it. So the whole swap/partition/
  union family loses the compare that round one charged it, and everything else keeps it.
- Checked for a new hole: `swap` does not become a back door into nested places. In
  `swap(&b.left, &(*b.left).right)` the two paths overlap with one a proper prefix of the
  other, and they are not "the same place", so clause 1's general requirement applies and the
  call is rejected — the only exemption granted is for identical places.
- The container/element pair stays rejected outright: "`bad(&vv, &vv[0])  // rejected:
  writes(vv) and writes(vv[0]) overlap`", and clause 2 keeps a consumed by-value argument in
  the same comparison, which is what rejects "`put(slot, move b)  // rejected: move b writes
  the prefix b of slot's path`".

Rewrite and cost: for a user-written `merge` over two data-determined slots, three forms, all
with a cost.

1. The dominating test, `if i != j { merge(&(*v)[i], &(*v)[j]) } else { merge_self(&(*v)[i]) }`:
   one compare and one predicted branch per call against C++, which simply aliases; zero against
   Rust, whose `get_many_mut` performs the same test. In union-find `union`, partition and
   merge-two-buckets the test is already in the source for algorithmic reasons, so the usual
   case is free.
2. Two adjacent statements: `t = replace(&(*v)[j], empty)` then
   `(*v)[i] = merge_into((*v)[i], move t)`. Never compared pairwise, hence accepted with no
   fact, but it stores a placeholder into slot `j` — one extra store — and for a Copy element
   it is a full element copy through a local.
3. Pass indices and declare `writes(*v)`: no compare, and no overlap with any other access to
   `*v` under Rule 13, so it buys the instruction back with P1's parallelism.

P1 asks for both halves — parallel permission and no runtime check — and no form delivers both
when the indices are data. That cost is unavoidable and is the cell's content.

Rules consistent: yes — clause 3 is Rule 3 restated at a call, and both now use one
may-overlap judgment, so a sibling slot cannot kill a sibling slot and a deeper path cannot
survive its own prefix.
Task achievable: rewrite — sequential legality is free under revision 4; the parallel
permission costs one compare and one predicted branch, except for the swap family, which
revision 4 makes free.
Verdict: rejected-real-cost

---

### 3-13: Rule 3 and Rule 13 — references across statements that may overlap

Question: do references formed before and after two adjacent statements stay sound, and does
one statement's write invalidate the other's or the continuation's references?

Round one: corrected by the reviewer from accepted-fine to undecided-rule-gap. The cell had
noticed that Rule 3 did not say whether a *range* write invalidates an element reference under
it, called both readings sound, and moved on; the reviewer showed that for affine elements the
permissive reading reads freed storage.

Strongest program (the reviewer's construction, plus the adjacency questions, with no `par`
statement anywhere):

```text
out: Box<Slots<Box<Node>>>                       // affine elements
fn refresh(part: &[Box<Node>])  writes(part)     // body replaces elements: old Boxes released
fn kernel(src: &[Int], dst: &[Int])  reads(src), writes(dst)

q  = &(*(*out)[3]).value                         // path (*out)[3].*.value
lo = &(*out)[0..mid]                             // Rule 7: 0 <= mid <= (*out).len; lo.len == mid
hi = &(*out)[mid..n]
s  = &stats.total                                // a bystander under a different root

refresh(&(*out)[0..h])                           // h > 3, unproved
y = (*q).value                                   // rejected under revision 4

kernel(&(*src)[0..mid], lo); kernel(&(*src)[mid..n], hi)   // may overlap: ranges proved distinct
place_back(&*out, x); kernel(&(*src)[0..mid], lo)          // may overlap: see the trace
place_back(&*out, x); place_back(&*out, y)                 // may not overlap: both write out.next, out.len
a = build(&x)?; b = build(&y)?                             // may overlap: allocation is not an effect
*s = *s + 1                                                // valid: nothing wrote a prefix of stats.total
```

Trace:

- The hole is closed by Rule 3's added sentence: "two indexed positions on the same storage are
  taken to overlap unless their indices or ranges are proved distinct, exactly as in Rule 10."
  `q`'s path is `(*out)[3].*.value`; the write path `(*out)[0..h]` matches it at the index step
  under the conservative judgment and is strictly shorter, so it is a proper prefix and `q` is
  invalid — which is the right answer, since Rule 9's "`writes` covers writing, replacing,
  moving out of, and freeing the storage at the path" and the replaced elements' Boxes are
  released. With `h <= 3` proved the ranges are distinct and `q` survives. Round one could
  argue this either way; revision 4 cannot.
- The disjoint-range pair is the rule file's own accepted example: "`kernel(&r[0..mid],
  &out[0..mid]); kernel(&r[mid..n], &out[mid..n])  // disjoint ranges: may overlap`", resting on
  Rule 13's "using the same path-overlap and index/range-disjointness judgment as Rule 10".
- The append-beside-a-kernel pair is new in revision 4 and worth stating, because round one
  rejected its `par` form. `place_back`'s write paths are `out.next` and `out.len`; the kernel's
  is the range `(*out)[0..mid]` with `mid <= (*out).len`. Rule 6 defines the parts — "`r.next`
  (the append slot, at index `r.len`) ... `r.filled` (all slots below `r.len`)" — and states the
  overlap: "a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`", which
  the index/range judgment extends to a range inside the filled prefix. `r.len` is a separate
  write target ("The measure `r.len` is itself a write target") and the range reference carries
  its own recorded measure ("`part.len == hi - lo`"), so the kernel never reads `out.len`. The
  two statements are disjoint and may overlap. It is sound for the same reason the reference
  survives at all: "`requires r.room > 0`" excludes a reallocation.
- Two appends may not overlap, both writing `out.next` and `out.len` — the same refusal as the
  file's "`push(&v, 1); stats(&v)  // may not overlap: writes(*v) meets reads(*v)`".
- Rule 14 keeps the allocating pair independent: "Allocation and release carry no effect entry
  and never prevent two statements from overlapping."
- Nothing new is needed for the post-state, and that removes round one's other loose end:
  Rule 13, "Because the meaning is sequential, results, errors, early exits, and linear
  obligations are exactly those of the sequential program; nothing new is defined for the
  overlapped case."
- Not covered here: a statement that *shrinks* the window while a range reference is live. That
  is 3-7's gap; the programs above only append.

Rewrite and cost: none for the programs above. A statement that grows a shared container still
cannot overlap with anything touching that container, so parallel append remains the recorded
form: each worker appends into its own `Box<Slots<T>>` and the results are consumed in
sequence or merged with `append(&dst, &src)  // one memcpy`, whose cost against a single
sequential append is one extra buffer per worker and, when a merge is really needed, one
`memcpy` of the payload — against a shared C++ vector behind a mutex this is normally faster,
and the candidate has no atomics.

Rules consistent: yes — one may-overlap judgment now serves Rule 3, Rule 10 and Rule 13, and
the window parts make append-beside-compute disjoint by arithmetic rather than by assertion.
Task achievable: yes for maps, adjacent-range kernels, independent allocation, and
append-beside-compute; rewrite for parallel append into one container.
Verdict: accepted-fine

---

### 4-12: Rule 4 and Rule 12 — a rebound reference at a loop header

Question: with references neither returned nor stored, a linked walk must be a rebound local
reference; what is its target set at the loop header, and is it admitted?

Round one: undecided-rule-gap. The reviewer agreed and added a second gap in the same program:
the walk had to name the payload of an `Option<Box<Node>>`, which the grammar could not do, and
`content(cur) -> &Node` was flatly rejected by Rule 4.

Strongest program:

```text
cur = &head                                    // cur: &Option<Box<Node>>
loop {
    match cur {
        Some(b) => { touch(&*b); cur = &(*b).next }    // rebind, one link longer each iteration
        None    => { break }
    }
}
```

Ordinary programs (both decided, and both cheap):

```text
w = if cond { &v1 } else { &v2 }               // Rule 2: target set { v1, v2 }
bump(w)                                        // charged writes(v1) and writes(v2)

p = &a; q = &b                                 // double buffering
loop { step(p, q); t = p; p = q; q = t }       // rebinding between two roots: shapes unchanged

loop { i = (*nodes)[i].next; p = &(*nodes)[i] }   // only an index value changes: admitted
```

Trace:

- Rule 4 forces the shape: a reference "cannot be returned" closes `fn next_of(n: &Node) ->
  &Node`, and "cannot be assigned into any aggregate" closes a stored cursor, so the only
  remaining form is a reference rebound on a back edge.
- Round one's second gap is closed by admission: Rule 2 now has the payload step, "A payload
  step is available only under the refinement fact that the enum currently holds that variant,
  which a `match` or `if let` on the enum establishes in the selected arm", so the `Some(b)`
  arm names `cur.Some.0` and `&(*b).next` continues through the Box.
- Round one's first gap is closed by rejection, and the rejection is quoted: Rule 2, "A path has
  a static shape: a loop-carried rebinding may change only the index values inside the path,
  never extend the path through itself." `cur = &(*b).next` extends `cur`'s path by
  `.Some.0.*.next` per iteration, so the walk is rejected.
- With that, Rule 12's join clause becomes finite and means something: "a reference's target set
  is the union of its targets (Rule 2)" is a union over the shapes written in the program, each
  with recorded index values, and Rule 2's "every check on it must hold for every member of the
  set" is a finite check. The three ordinary programs show the whole admitted range: a branch
  union of two roots; an alternation between two roots across a back edge, whose shapes never
  grow; and an index-only rebinding, which is the one loop-carried form that survives.
- The branch-selected `w` costs nothing at runtime — the proof is erased and the selection is a
  `cmov` exactly as in C++ — but charges `bump(w)` to both members, which removes the overlap
  permission for any adjacent statement touching `v2`. Duplicating the call into both arms
  restores precision at zero runtime cost, since Rule 12 intersects per edge.

Rewrite and cost for the walk: (a) recursion, where each frame's reference "starts at a local
variable or a parameter" so the shape is constant and Rule 10's "Recursion is checked through
contracts, never by unfolding bodies" keeps checking finite — one call and return per hop, with
no tail-call form and no stack-depth statement anywhere in the file, so it is not the general
answer for a data-determined length; (b) a `Box<Slots<Node>>` with `u64` links, where nothing is
rebound except an index — one compare and one predicted branch per hop, the file's first known
cost, "equal to safe Rust, real against C++". Against `for (Node* p = head; p; p = p->next)`
either rewrite adds about one instruction per hop.

Rules consistent: yes — revision 4 answers the join question by making the set finite and
answers the walk question by refusing it, and books the refusal: "A rebinding of a reference
that extends its own path in a loop is refused; recursion or indices cost the same loads."
Task achievable: rewrite, at roughly one instruction per hop.
Verdict: rejected-real-cost

---

### 4-14: Rule 4 and Rule 14 — ownership escapes where a reference may not, and what scope exit does to a reference

Question: does the no-escape rule interfere with a helper that allocates, with allocation in
two adjacent statements, or with a reference into a heap object whose Box goes out of scope?

Round one: accepted-fine. Two loose ends: what `?` inside a parallel arm meant for the sibling
arm was booked to another cell, and the reviewer separately recorded that a linear payload
handed to a failing allocation was lost.

Strongest program (the escape, the failure arm with a linear payload, and the scope-exit case
that round one's reviewer found unrefused):

```text
fn build(src: &Src) -> own Box<Node>  reads(src)     // ownership leaves; no reference does

a = build(&x)?; b = build(&y)?                       // Rule 14's own pair: may overlap
holder.child = move a                                // Rule 5: a Box in a struct field
p = &(*holder.child).left                            // the caller forms its own reference

// a linear payload on the failure path
match Box::new(Conn { f: move f, n }) {
    Ok(bx)        => { use_conn(bx) }
    Err((_, c))   => { let Conn { f, .. } = move c;  close(move f) }
}

// the scope-exit case
q: &Int
{
    bx = Box::new(Node { value: 7 })?                // affine: `?` releases nothing the writer owes
    q = &(*bx).value
}                                                    // bx's scope ends; the compiler releases the Box
*out = *q                                            // rejected
```

Trace:

- Rule 14 keeps the two adjacent allocations independent: "Allocation and release carry no
  effect entry and never prevent two statements from overlapping", so the pair compares only
  `reads(x)` against `reads(y)` and Rule 13 grants the overlap, with "`a = build(&x)?; b =
  build(&y)?  // both allocate: may overlap`" in the rule file itself.
- Rule 4 restricts references, not ownership, and Rule 15 admits the owned `Box`, so Rule 5 is
  the escape Rule 4 leaves open, and the returned Box is the same value a C++ factory returns.
  The caller then forms whatever reference it likes locally.
- Round one's `?`-inside-an-arm question does not exist any more: there are no arms. Rule 13,
  "The program's meaning is its sequential meaning ... results, errors, early exits, and linear
  obligations are exactly those of the sequential program; nothing new is defined for the
  overlapped case." An early exit at the first statement is simply an early exit.
- The linear payload is no longer lost: Rule 5, "A fallible allocation that takes a by-value
  payload hands it back on failure, so a linear payload is never lost", with
  "`b = Box::new(Node { ... })?  // Result<Box<Node>, (Oom, Node)>: on Err the Node comes back`".
  The `Err` arm names the `Conn` and Rule 6's destructuring takes the `File` out —
  "`let Conn { f, g, .. } = move c  // several fields at once`" — so Rule 8's obligation is
  discharged on the failure path, with no destructor and no runtime flag. On the success path
  nothing extra is computed.
- The scope-exit hole is closed by the clause added to Rule 3's invalidation list: validity is
  invalidated when "a proper prefix of p's path is written, moved out of, replaced, or freed, by
  a statement, by a call, or by **the compiler-derived release at scope exit**; the scope of the
  local variable at which p's path starts ends". The rule file gives round one's exact program
  as its example: "`{ b = Box::new(...)?; p = &(*b).value }` ... `use(p)  // rejected: p's root
  has gone out of scope`". Both halves of the clause fire here, which is the right degree of
  redundancy for a release nobody wrote.

Rewrite and cost: none needed for the shape. Two costs worth naming, neither new. `Box::new(v)?`
takes the payload by value, so a large inline `T` is materialized in a local and then moved into
the heap — one copy of `sizeof(T)` that C++'s placement-new avoids and that Rust also pays; the
zero-cost form is to allocate the block directly, `Box::new(Array::filled(n, v))?`, which Rule 6
maps to `calloc` for a zero fill. And the returned payload on the failure path costs one move of
the payload, on the failure path only.

Rules consistent: yes — and the two round-one loose ends are closed in opposite ways: the `par`
question by deleting the construct, the lost payload by changing the signature.
Task achievable: yes.
Verdict: accepted-fine

---

### 5-15: Rule 5 and Rule 15 — returning owned Boxes, find-then-mutate, and what a move does to a live reference

Question: with references unreturnable, how does a program reach a node it just found in a Box
structure, and what happens to a reference into a value that is then moved out as an owned
return?

Round one: rejected-zero-cost — Rule 4 rejects the returned reference at the signature, the
recursive callback does the work at the point of discovery with no second walk, and the only
exposure is an unspecialized indirect call.

Strongest program (the C++ shape, and a live reference across the move that the owned-return
discipline forces):

```text
p = find(&*root, k)                 // wants to return &Node: rejected
(*p).value = v

n = &(*b).left                      // b: Box<Node>
c = move b                          // the heap object does not move
use(n)                              // rejected
w = move v                          // v: a large owned local
m = &w.field                        // the only way back in: form from the new root
```

Ordinary program (the accepted forms, both one descent):

```text
// (a) do the work where the node is found
fn update(t: &Node, k: Int, f: fn(x: &Val) writes(x))  writes(t)
{
    if t.key == k { f(&t.value) }
    else { match &t.left { Some(b) => update(&*b, k, f)   None => () } }      // and right
}

// (b) revision 4's atomic in-place update, with no function-typed parameter at all
fn insert(t: own Option<Box<Node>>, k: Int) -> own Option<Box<Node>>
node.left = insert(node.left, k)      // Rule 6's own example
```

Trace:

- Rule 4 rejects the first line at the signature — a reference "cannot be returned" — and
  Rule 15 states the same positively, "A function returns owned values only." Rule 4's
  substitute, an index, works for a window but a Box structure has no index space, so the owned
  surrogate would be a path word plus a second walk: O(depth) extra dependent loads.
- Form (a) is licensed by Rule 10, "Function-typed parameters carry a full signature with its
  own row and contract, and a call through one uses that row", and by Rule 9's body check, which
  covers `writes(t.value)` and the recursive `writes(t.left.Some.0.*)` under `writes(t)`. The
  payload step it needs is now in Rule 2; round one had to assume it.
- Form (b) is new in revision 4 and removes even the indirect call: Rule 6, "`place =
  f(place, args...)  writes(place)  // atomic in-place update: the old value enters f by value,
  f's result is committed, no program point lies between`", with the constraint "f's row must
  not overlap any prefix of place" — `insert` takes its argument by value and has no reference
  parameters, so it has no row at all. This matters because revision 4 also closed the other
  route: "A move out of a field or out of Box content consumes the whole owner", so
  `node.left = insert(move node.left, k)` is not available and the atomic update is the form
  that keeps the owner alive.
- The move case is decided by the sentence added at the end of Rule 3: "Validity is
  re-established only by forming the reference again. A move never re-roots an existing
  reference: after `w = move v`, references formed from `v` are invalid and are not
  reinterpreted as references into `w`." The rule file's Box example is the sharp one —
  "`c = move b  // b is a proper prefix of *b: p invalid, even though the Node did not move`" —
  and the reason it is right is that the alternative would make validity depend on a layout
  accident rather than on a path.
- Rule 15's other clause is unaffected: "Passing a large value in and back out is expressed as a
  reference parameter with a `writes` entry, not as move-and-return", and a Box is pointer-sized,
  so returning `own Box<Big>` is the candidate's zero-copy way to hand back a large owned value.

Rewrite and cost:

- Form (a): one walk, one write, no re-derivation. Where `f` is a literal at the call site the
  instantiation is monomorphic; where the operation is genuinely dynamic, the equivalent C++
  pays the same indirect call, so the cost against the equivalent program is zero.
- Form (b): one walk, no indirect call, but the commit writes `node.left` at every level on the
  way back up — `d` stores to already-hot lines where an in-place C++ insert writes one link —
  and it forecloses a tail call. A writer picks (a) when the callback is static and (b) when it
  is not.
- For the move: re-form from the new root. For a Box this is literally the same address, so the
  re-formation is zero instructions beyond the register move the `move` already is.

Rules consistent: yes.
Task achievable: rewrite — do the work at the point of discovery, or rebuild the spine with the
atomic update; the returned reference is not expressible and is not needed.
Verdict: rejected-zero-cost

---
### 7-7: Rule 7 against itself — indexing on indexing, ranges of ranges, and narrowing a range in a loop

Question: does the bounds obligation compose when an index feeds another index and when a range
reference is subdivided again — and does a subdivision that is carried by a loop rather than by
recursion survive Rule 2's path shape?

Round one: accepted-fine. The reviewer left the verdict and recorded two defects: `mid = n / 2`
was used as if `0 <= mid <= n` were free, and a range reference was passed to a parameter typed
`&Slots<f64>` with no rule saying a range may bind there.

Strongest program (the idiomatic narrowing loop — Rust writes `s = &s[..mid]`, C++ narrows a
pointer/length pair):

```text
part = &(*b)[0..n]                         // &[Int]
while part.len > 1 {
    mid = part.len / 2
    if key < part[mid] { part = &part[0..mid] } else { part = &part[mid..part.len] }
}                                          // rejected
```

Ordinary programs (ragged two levels, and recursive subdivision with the range parameter type):

```text
vv: Box<Slots<Box<Slots<f64>>>>            // rows of different runtime lengths

fn row_work(part: &[f64])  writes(part)
{
    if part.len > 1 {
        mid = part.len / 2
        if mid <= part.len {                                  // Rule 7's stated fallback
            row_work(&part[0..mid]); row_work(&part[mid..part.len])   // may overlap: ranges distinct
        }
    } else if part.len == 1 { part[0] = kern(part[0]) }
}

for i in 0..(*vv).len {
    m = (*(*vv)[i]).len                    // a measure of an indexed path
    for j in 0..m { (*(*vv)[i])[j] = kern((*(*vv)[i])[j]) }
    row_work(&(*(*vv)[i])[0..m])
}
```

Trace:

- The narrowing loop is rejected by Rule 2: "A path has a static shape: a loop-carried rebinding
  may change only the index values inside the path, never extend the path through itself." Each
  iteration appends another `[lo..hi]` step to `part`'s own path, which is the growth the
  sentence forbids; the file's rejected example, "`loop { p = &(*p.kids)[0] }`", differs only in
  which step is appended. This is the one place in this cell where revision 4 changed an answer,
  and it changed it against the most idiomatic form of the program.
- Everything else composes, and the reviewer's second defect is fixed by a type. Rule 7: "A range
  reference `&x[lo..hi]` has the parameter type `&[T]`: a reference kind with measure `len`,
  formed only from an indexable or from another range reference, never a stored value", with the
  worked lines "`fn kernel(part: &[Int]) writes(part) { for k in 0..part.len { part[k] += 1 } }`"
  and "`sub = &part[a..b]  // requires a <= b <= part.len`". So a range binds to `&[T]`, a range
  of a range is a range, and Rule 1 never sees a stored reference because a range reference is
  "never a stored value".
- Both index levels are proved as before. Rule 7, "Every index must be proved in bounds";
  `i < (*vv).len` comes from the counted loop, and `j < m` with `m == (*(*vv)[i]).len` is an
  affine comparison over measures, which Rule 11 admits. The inner write does not disturb `m`:
  Rule 11 invalidates a fact "when that path is written", the path written is `(*(*vv)[i])[j]`,
  and Rule 3 states in general that a write below a path is not a write of it — "Writing the
  storage at p's path or below it (a content write) does not invalidate p". The row length is
  loaded once per row, as in C++.
- The reviewer's first defect survives revision 4 and is a cost, not a gap: `mid = part.len / 2`
  is not an affine relation, and Rule 11's vocabulary is "affine comparisons over measures and
  integer values", so `mid <= part.len` is not derivable and the program does what Rule 7 tells
  it to do — "when the proof is unavailable the program tests the measure, which is ordinary
  data." One compare and one predicted branch per subdivision node.
- Recursion is checked by Rule 10's "Recursion is checked through contracts, never by unfolding
  bodies", and the two recursive calls may overlap under Rule 13's "same path-overlap and
  index/range-disjointness judgment as Rule 10", the subranges being adjacent by arithmetic.

Rewrite and cost: keep the endpoints as integers and re-form the range, or index the storage
directly:

```text
lo = 0; hi = n
while hi - lo > 1 {
    mid = lo + (hi - lo) / 2
    if mid < (*b).len { if key < (*b)[mid] { hi = mid } else { lo = mid } } else { break }
}
```

The rebinding now changes only index values, which Rule 2 admits explicitly. Against the rejected
form the machine code is identical — the same halving, the same one dependent load per iteration,
the same bound test, because the rejected form had to discharge `mid <= part.len` too. So the
rejection itself costs nothing; what remains is Rule 7's standard test on an index the affine
vocabulary cannot reach, which is charged once, here and in every other data-determined access.
Nested ragged iteration and recursive subdivision need no rewrite and run at C++ cost, one row
length loaded per row.

Rules consistent: yes — the path grammar, the `&[T]` measure and the range arithmetic compose at
every level, and the loop form is refused by a sentence rather than left open.
Task achievable: yes — every shape in the cell is expressible; only the loop-carried narrowing
must be written with indices, at the same machine code.
Verdict: rejected-zero-cost

---

### 8-9: Rule 8 and Rule 9 — consuming a contained linear value, and what a whole-owner move does to a live reference

Question: can a helper that receives a reference consume one linear element chosen by a
data-determined index, what does the row say about it, and what happens to references into an
owner that a field move consumes?

Round one: accepted-fine, on a reading the reviewer endorsed twice: "moving out of" in an effect
row cannot mean a callee leaving a hole in a place the caller still owns. Separately the reviewer
recorded that `truncate` on a window of linear elements silently discarded them and that Rule 8's
own `close(move c.f)` example used an operation Rule 6 did not have.

Strongest program (a data-determined index, a linear element removed through a separate helper,
an owner with two linear parts, and a reference into that owner):

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, g: File, n: u64 }          // linear by containment (Rule 8, Rule 1)
pool: Box<Slots<Option<File>>>                    // linear by containment

fn discard_one(pool: &Box<Slots<Option<File>>>, k: u64)   writes((*pool)[k])
    contract { requires k < (*pool).len; }
{ (*pool)[k] = drop_file((*pool)[k]) }            // Rule 6's atomic in-place update

fn drop_file(x: own Option<File>) -> own Option<File>     // by value: no effect entry
{ match x { Some(f) => close(move f);  None => () }   None }

k = probe(&*pool, key)                            // data-determined, returned as an index
if k < (*pool).len { discard_one(&pool, k) }

p = &c.n                                          // a reference into an owned local
x = move c.f                                      // rejected: g is linear and would be released
let Conn { f, g, n } = move c                     // accepted: every linear part is taken
close(move f); close(move g)
use(p)                                            // rejected: c was consumed

fn steal(p: &File)  writes(p)                     // can a callee consume the caller's File?
while (*buf).len > 0 { f = take_back(&*buf); close(move f) }   // draining a Slots<File>
```

Trace:

- Rule 9 admits the row: "each path starts at a reference parameter and may continue through
  fields, `*`, payload steps, and whole-index or range positions supplied as arguments", and the
  index "enters an effect only through an argument, evaluated once at the call". No `consumes`
  entry is needed, since `writes` "covers writing, replacing, moving out of, and freeing the
  storage at the path".
- The consumption is invisible to the row and legal only in the atomic-update shape. Rule 9: "A
  by-value parameter has no effect entry: the call site records the consumption (for `move`) or
  the read (for a copy) of the argument's place." Rule 6: "`place = f(place, args...)`
  `writes(place)` // atomic in-place update: the old value enters f by value, f's result is
  committed, no program point lies between", and "Assigning over any owned place releases the old
  value if it is affine and is rejected if it is linear" — so the atomic update is the only form
  that empties and refills a linear slot, which is what `discard_one` does.
- Revision 4 settles round one's field-move question in the direction that keeps linearity, and
  the sentence is new: Rule 6, "A move out of a field or out of Box content consumes the whole
  owner: the owner ceases to exist, its other affine parts are released, and a remaining linear
  part rejects the move (take it in the same destructuring)", with "`let Conn { f, g, .. } = move
  c  // several fields at once; `..` covers the rest`". So `x = move c.f` is rejected here
  because `g` is linear, and the destructuring is the only way to finish a two-resource `Conn` —
  Rule 8's old example, which Rule 6 did not support, has been replaced by one it does.
- The reference is killed by the same move, and by two clauses at once: Rule 3's list includes "a
  proper prefix of p's path is written, **moved out of**, replaced, or freed", and `c` is a proper
  prefix of `c.n`; and "A move never re-roots an existing reference", so `p` is not reinterpreted
  as a reference into anything the destructuring produced. Rule 12 agrees at a join: "a local
  consumed on one incoming edge is consumed after the join."
- `steal` still cannot consume the caller's `File`. Rule 6's account of how a value leaves storage
  is closed — the whole-owner move, the window operations, the atomic update — and a callee holding
  `p: &File` cannot name the owner in order to consume it, while Rule 12 forbids the alternative:
  "No place is ever partially moved: every path is either wholly present or the program cannot
  name it." So "moving out of" in a row describes the window operations and the transient half of
  an atomic update, never a hole in a place the caller still owns; the unbounded reading would let
  Rule 8's obligation be discharged twice.
- Round one's `truncate` hole is closed by a sentence and by construction. Rule 6: "No operation
  releases a linear element: a storage whose element type is linear is itself linear (Rule 8) and
  the program must take every element out and consume it", and `truncate` is now listed among
  "Library code, all zero-cost compositions of the above", i.e. a loop of `take_back`, which hands
  each value back to the caller, exactly as the drain above does.

Rewrite and cost: none needed. Against C++, `discard_one` costs one branch on the `Option`
discriminant plus a store of the whole `Option<File>` back into the slot, where C++ calls
`~File()` in place and clears a bitmap bit — one branch and one byte of state either way, and with
a null niche (Rule 12, "using a null niche where the type has one") the tag is free. The drain
relocates `sizeof(File)` per element against an in-place destructor loop. For ordered removal
revision 4 adds "`remove_at(&r, k) -> own T ... // one memmove`", which is the same memmove
`std::vector::erase` performs and which, unlike a release, hands the linear element back.

Rules consistent: yes — and on firmer ground than in round one: the whole-owner sentence now says
what a field move does, and the no-release-of-a-linear-element sentence says what the compiler
will not do behind the writer's back.
Task achievable: yes — removal of a linear element at a data-determined index costs at most one
branch, and a two-resource owner is taken apart in one destructuring.
Verdict: accepted-fine

---

### 9-9: Rule 9 against itself — composing rows through a call chain, and two entries in one row

Question: when one function's row must cover its callees' substituted rows, and when a single row
carries two index-carrying entries, what does Rule 9 force the writer to declare and what does the
declaration cost the callers?

Round one: accepted-fine, with two defects the reviewer recorded: `swap_two`'s body was itself
rejected without a `requires i != j`, and the recursion example was not expressible because the
path grammar had no variant-payload step.

Strongest program (two index-carrying entries, a coarse twin, a precise row that names no index
at all, and a recursion that descends through a payload):

```text
fn swap_two(v: &Box<Slots<Int>>, i: u64, j: u64)   writes((*v)[i]), writes((*v)[j])
    contract { requires i < (*v).len, j < (*v).len; }
{ swap(&(*v)[i], &(*v)[j]) }                    // body accepted with no distinctness fact

sort_step(&v, a, b)                             // a != b provable from a comparison network
swap_two(&v, p, p)                              // still rejected at the call

fn swap_any(v: &Box<Slots<Int>>, i: u64, j: u64)   writes(*v)
    contract { requires i < (*v).len, j < (*v).len; }
swap_any(&v, p, p)                              // accepted
swap_any(&v, a, b); swap_any(&v, c, d)          // accepted; may not overlap

fn add_all(s: &Box<Slots<Entry>>, xs: &[Entry])   writes((*s).next), writes((*s).len), reads(xs)
    contract { requires (*s).room >= xs.len; }   // precise without naming an index

fn walk(n: &Node, d: u64)   writes(n)   contract { requires d <= 64; }
{ n.mark = 1;  match &n.child { Some(b) => walk(&*b, d + 1)   None => () } }
```

Trace:

- Rule 9's grammar now includes the payload step: "each path starts at a reference parameter and
  may continue through fields, `*`, payload steps, and whole-index or range positions supplied as
  arguments". So the recursive call substitutes `writes(n.child.Some.0.*)`, `n` is a prefix of it,
  and Rule 9's body clause is satisfied — "every statement's effect and every callee's substituted
  row must be covered by the declared row". Round one could not write this call; revision 4 makes
  it the ordinary form, and Rule 10 keeps the check finite: "Recursion is checked through
  contracts, never by unfolding bodies."
- The refinement fact survives the recursive call, and the two sentences that decide it must be read
  together. Rule 2 says the fact is one "which any write to the enum invalidates"; the write path
  here is `n.child.Some.0.*`, below the payload, and Rule 3 states in general that "Writing the
  storage at p's path or below it (a content write) does not invalidate p", while Rule 11
  invalidates a fact only "when that path is written". Reading "a write to the enum" to include a
  write below the payload would make Rule 3's sentence dead letter and the payload step unusable —
  the binding it creates could never be written through — and Rule 2's own example gives the
  invalidating write at the enum's own path, "`n.left = None  // the fact is gone`". So the fact
  dies on a write at or above `n.child`, including a may-overlapping write at a sibling index under
  Rule 3's conservative judgment, and survives writes below the payload.
- Round one's body defect is gone, because the operation is now a built-in with an aliasing clause:
  Rule 6, "`swap(p: &T, q: &T)  writes(p), writes(q)  // built in; p and q may be the same place,
  then nothing happens`", and Rule 10 clause 1, "The built-in `swap` is the one operation whose two
  arguments may be the same place". So `swap_two`'s body needs no `requires i != j`.
- The price of the precise row still appears at the caller, not in the body: Rule 10 clause 1
  compares "the substituted effects pairwise", so `swap_two(&v, p, p)` puts `writes((*v)[p])`
  against itself, indices not distinct, and is rejected although the body would be harmless. The
  coarse twin has the mirror defect: `writes(*v)` never conflicts with itself inside one call, so
  aliasing callers are fine, but Rule 13 then refuses overlap for any two such statements. Nothing
  in Rule 9 lets one function have both rows; the writer ships two functions with one body, and the
  caller picks the row it can discharge.
- Revision 4 removes the case where precision was impossible rather than merely verbose. The window
  parts are ordinary vocabulary — Rule 6, "This vocabulary is ordinary: the rows below use nothing
  a user function cannot write", and Rule 9, "The rows of the built-in operations of Rule 6 use
  only this vocabulary plus the window parts of Rule 6, which any user function may use too" — so
  `add_all` declares what it touches without naming an index, and every `&(*s)[k]` in the caller
  survives it across a separate-compilation boundary. What remains coarse is a callee that picks
  its own slot, since "Signatures never contain index expressions".

Rewrite and cost: shipping both rows duplicates source, which the protocol excludes from the cost
account, and each caller selects the row it can discharge, so no call site executes an instruction
it would not execute in C++ or Rust. For the recursion, `writes(n)` over a whole subtree is as
coarse as C++'s implicit "may touch anything reachable", so nothing is lost against the baseline,
and the contract check is erased before lowering. The one shape still unavailable is a caller that
must alias *and* overlap, which needs a distinctness fact no row can supply.

Rules consistent: yes — the path grammar, the body-covering clause, Rule 10's pairwise comparison
and the payload refinement fact compose without contradiction.
Task achievable: yes — precision is a per-function choice, and both precisions are available at
zero runtime cost.
Verdict: accepted-fine

---

### 15-15: Rule 15 against itself — composing owned returns across a move that kills references

Question: what does "returns owned values only" cost when owned returns feed owned returns, and
what happens to a reference into a value that one of those returns moves?

Round one: rejected-zero-cost. The reviewer added a rewrite stronger than the cell's four —
duplicate the branch at the *use* site, which is free under the protocol — and confirmed that the
callback inversion and its indirect call are never forced.

Strongest program (the selection C++ writes as `Big& choose(Big&, Big&, bool)`, with a reference
live across it and a linear element type):

```text
fn choose(a: own Big, b: own Big, c: Bool) -> own Big
{ if c { return move a } else { return move b } }        // both consumed; the loser released

q = &a.header                            // live across the selection
r = choose(move a, move b, c)            // Rule 10 clause 2: each by-value argument consumes its place
use(q)                                   // rejected, and not re-rooted into r
s = &r.header                            // the only way back in

linear type Handle;  struct BigL { h: Handle, ... }      // a linear Big
r2 = choose_l(move a2, move b2, c)       // rejected: the loser cannot be released
```

Ordinary programs (composition, which is where the rule is cheap):

```text
fn a_into(out: &Big)  writes(out) { b_into(out) }        // one destination threaded down the chain
fn rewrite(t: own Box<Tree>) -> own Box<Tree>            // pointer-sized return
fn f(out: &Big) -> u64  writes(out)                      // large part by reference, small part returned
```

Trace:

- Rule 15, "A function returns owned values only", with Rule 4, a reference "cannot be returned":
  the zero-copy C++ selection is unavailable, both candidates are consumed, and the loser is
  released by Rule 8 as an affine value. With a linear part the program is rejected outright, since
  "Linear values must be consumed by an explicit operation on every exit path; the compiler never
  releases them" — `choose` would have to hand the loser back as well.
- The reference is killed by the move, and revision 4 says so in a sentence round one had to infer:
  Rule 3, "A move never re-roots an existing reference: after `w = move v`, references formed from
  `v` are invalid and are not reinterpreted as references into `w`." Rule 10 clause 2 is what
  brings the consumption into the call's comparison in the first place: "A by-value argument
  contributes a consumption (`move`) or a read (copy) of its place to this comparison", the shape
  the file illustrates with "`put(slot, move b)  // rejected: move b writes the prefix b of slot's
  path`".
- The composition cases are accepted: the in-out form chains because `out` is a reference parameter
  that may be "passed as a call argument" (Rule 4), and recursion composes because each frame
  returns owned data and Rule 10 checks recursion "through contracts, never by unfolding bodies".

Rewrite and cost:

1. Put the candidates in one indexable place and return the position, which is Rule 4's and
   Rule 15's own prescription: `fn choose_at(v: &Box<Slots<Big>>, c: Bool) -> u64  reads(v)
   contract { ensures result < (*v).len; }`, then `p = &(*v)[k]` at the caller. Zero copies, one
   shift-add the C++ callee performed anyway, nothing consumed — so this is also the form that
   works for a linear element type.
2. Duplicate the branch at the use site, `if c { use(&a) } else { use(&b) }`; behind an abstraction
   boundary the callee returns an owned selector and the caller branches. Duplicated code is not a
   cost under the protocol, so the callback inversion and its indirect call are never forced.
3. Thread one destination through the chain with `writes(out)`, so no level copies; this matches
   C++'s out-parameter chain and beats a return the ABI does not elide.
4. Where the selected value must afterwards leave storage, revision 4 has the operations and they
   are the same moves C++ makes: `swap_remove` is "swap with the last slot, then `take_back`", one
   element move, exactly swap-and-`pop_back`; `remove_at(&r, k) -> own T` is "one memmove", exactly
   `erase`. A `Box`-rooted return is pointer-sized, so a recursive tree transformation carries no
   copy cost at all.
5. For the killed reference: re-form from the new root. The address is recomputed from a value
   already in a register, and for a `Box` it is literally the same address.

Rules consistent: yes.
Task achievable: rewrite, at zero cost — and the container-plus-index form is strictly more
general than the C++ shape it replaces, because it also works when `Big` is linear.
Verdict: rejected-zero-cost

---

## Closing note on the three round-one holes

A1, the prefix relation over unproved indices, is closed by one sentence in Rule 3 ("two indexed
positions on the same storage are taken to overlap unless their indices or ranges are proved
distinct, exactly as in Rule 10") plus the file's own worked `replace_at` example. It turns 3-9
from undecided to accepted and narrows 1-10, 3-10 and 3-13 to the exact statements round one could
not make.

A2, the index re-read after formation, is closed by Rule 2's formation sentence, which also fixes
the endpoints of a range reference and thereby makes `&[T]` usable (2-7, 7-7).

A3, the scope-exit release, is closed by the clause added to Rule 3's invalidation list, with
round one's own program as the rule file's example (4-14).

The payload step (2-2, 3-3, 4-12, 5-15, 9-9) and the static path shape (1-4, 2-2, 3-3, 4-12, 7-7)
close the remaining two findings in opposite directions: the descent becomes writable, and its
loop-carried form becomes rejected.

One new hole is opened by the replacement of the coarse `writes(buf)` with slot-precise window
parts: the sentence that kills an element reference at a `take_back` is written for `p = &r[i]`
and leaves a range reference `&r[lo..hi]` decided only by Rule 3's content-write clause, under
which it survives the window shrinking past it. Recorded in 3-7 with the smallest fix.
