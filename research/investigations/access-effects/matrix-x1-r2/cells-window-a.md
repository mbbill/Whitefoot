# Candidate x1 revision 4, targeted round two: window parts, bulk operations, consuming moves, atomic update (rows 1-5 side)

Sixteen cells re-derived against `CANDIDATE-X1.md` revision 4 only. For each cell
the round-one derivation in `matrix-x1/rows-NN-MM.md` and its verification in
`matrix-x1/verify-rows-NN-MM.md` were read first; "Round one" states the
verdict as corrected by the verifier, not the verdict the cell originally
carried. Measures and window parts are spelled as members (`r.len`, `r.next`);
`len_of` and `par` do not appear. Runtime performance is the only cost counted.

Three revision-4 sentences do most of the work in this batch and are quoted in
full once here so the cells can cite them briefly:

- Rule 6, window parts: "Overlap follows from the definitions: a live `r[i]`
  (which has `i < r.len`) never overlaps `r.next` or `r.free`, overlaps `r.last`
  unless `i != r.len - 1` is proved, and always overlaps `r.filled`. The measure
  `r.len` is itself a write target."
- Rule 6, consuming moves: "A move out of a field or out of Box content consumes
  the whole owner: the owner ceases to exist, its other affine parts are
  released, and a remaining linear part rejects the move (take it in the same
  destructuring)."
- Rule 6, general places: "Two operations apply to any owned place, not only to
  window slots: `swap(p: &T, q: &T)  writes(p), writes(q)` ... `place =
  f(place, args...)  writes(place)`".

---

### 1-3: Rule 1 and Rule 3: what makes invalidation a local question

Question: with no aggregate able to hold a reference, is invalidation decided by
walking local reference variables against a statement's write paths, and what
does that decision still refuse that C++ keeps?

Round one: `rejected-real-cost` (verifier's correction of the cell's own
`rejected-zero-cost`). The cost was the block-base reload forced by giving every
window operation the row `writes(buf)`, so an append killed every element
reference even across separate compilation.

Strongest program: a reference held across a separately compiled call that
appends, and across a move of the Box that owns the path.

```text
struct Graph { nodes: Box<Slots<Node>> }

fn add_node(g: &Graph, n: own Node) -> u64            // separately compiled
    writes((*g.nodes).next), writes((*g.nodes).len)
    contract { requires (*g.nodes).room > 0;
               ensures result == entry(*g.nodes).len,
                       (*g.nodes).len == entry(*g.nodes).len + 1; }

p  = &(*g.nodes)[i]                  // under i < (*g.nodes).len
id = add_node(&g, n)                 // writes the append slot and the measure
(*p).out = (*p).out + 1              // p still names slot i

b = Box::new(Node { key: 1, left: None })?
q = &(*b).left
c = move b                           // b is a proper prefix of *b
use(q)                               // rejected, although the heap Node did not move
```

Trace:

- Rule 9 no longer forces a whole-container row: "The rows of the built-in
  operations of Rule 6 use only this vocabulary plus the window parts of Rule 6,
  which any user function may use too." `add_node` therefore declares exactly
  what `place_back` declares, and the row is its interface across separate
  compilation.
- Rule 10 clause 3 asks whether a bystander's path has "a proper prefix among
  the call's write paths, under the same may-overlap judgment". The call's write
  paths are `(*g.nodes).next` and `(*g.nodes).len`. Rule 6: a live `r[i]` "never
  overlaps `r.next` or `r.free`", and `r.len` is a measure member, not a prefix
  of `r[i]`. So `p` is not a bystander of either write.
- The fact `i < (*g.nodes).len` survives by Rule 11's entry clause plus the
  `ensures`: `i < entry(*g.nodes).len` and `(*g.nodes).len == entry(...) + 1`
  give `i < (*g.nodes).len` by affine arithmetic. Rule 6 states the conclusion
  outright: "`place_back`'s `ensures` carries it across the call".
- The move half is unchanged: Rule 3, "a proper prefix of p's path is written,
  moved out of, replaced, or freed", and "A move never re-roots an existing
  reference".
- The Rule 1 point of the cell survives intact: because no aggregate holds a
  reference (Rule 1) and none escapes (Rule 4), the live reference set at any
  point is the set of live reference variables of the current function, so the
  check above is a walk over local bindings against one row. Nothing in
  revision 4 widens it.

Rewrite and cost: only the move case needs one, `c = move b; q = &(*c).left`.
The move is a register copy and the re-formed path computes the same address
from the same register: zero instructions. The append case needs no rewrite at
all, so round one's per-iteration base reload is gone. `grow(&b, cap)` still
declares `writes(*b)` and still kills every interior reference, which is what
C++ `vector` realloc does too.

Rules consistent: yes.   Task achievable: yes for the append; rewrite at zero
cost for the move.
Changed by: Rule 6's window-part overlap sentence and the `add_node` example
("p survives: `(*g.nodes)[i]` with `i < len` never overlaps `.next`").
Verdict: rejected-zero-cost

---

### 1-8: Rule 1 and Rule 8: emptying a container of linear resources

Question: when linear resources are reachable only by containment, what empties
the container, and does any built-in release them?

Round one: `undecided-rule-gap` (verifier agreed). `truncate` was a built-in
whose contract mentioned only the length, so it either released linear elements
(against Rule 8) or left them outside the window where nothing could name them.

Strongest program: an early exit from the middle of a fill loop, with the
container still holding an unknown number of open files.

```text
linear type File;   fn close(f: own File)
bufs: Box<Slots<File>>                       // linear by containment (Rule 8)

if error {
    while (*bufs).len > 0 { close(take_back(&*bufs)) }     // the whole obligation
    return Err(e)
}
```

Trace:

- Rule 8: "Linearity is declared on external-resource types ... and propagates
  through every aggregate of Rule 1: a struct, enum, tuple, Array, Slots, Ring,
  or Box containing a linear part is linear."
- Rule 6, Release: "No operation releases a linear element: a storage whose
  element type is linear is itself linear (Rule 8) and the program must take
  every element out and consume it." That is both the prohibition round one
  could not find and the statement of what discharges the obligation, so no
  reading survives in which a built-in silently drops a `File`.
- `truncate` is no longer a built-in with a length-only contract. Rule 6 lists
  it under "Library code, all zero-cost compositions of the above", i.e. a loop
  of `take_back`, which returns `own T`; for a linear `T` the caller must
  consume each returned value, so a discarding `truncate` cannot be written.
  Rule 6's assignment clause blocks the other escape: "Assigning over any owned
  place releases the old value if it is affine and is rejected if it is linear."
- `take_back(&r) -> own T  writes(r.last), writes(r.len)  contract { requires
  r.len > 0; ... }`, and Rule 11's own example supplies the precondition from
  the loop condition: "while `r.len > 0` { `take_back(&r)` } // no knowledge:
  the loop condition is the test".
- The emptied block itself is ordinary memory: Rule 6, "at scope exit the
  compiler releases the slots inside the window recursively and frees the
  block". With the window empty there are no slots to release, so the affine
  free and the linear obligation do not collide.

Rewrite and cost: none needed. The drain loop emits one length compare, one slot
load and one `close` per element, which is byte for byte what `~std::vector<File>`
and Rust's `Drop` emit.

Rules consistent: yes.   Task achievable: yes, at zero cost.
Changed by: Rule 6's "No operation releases a linear element ... the program
must take every element out and consume it", plus moving `truncate` into library
code.
Verdict: accepted-fine

---

### 1-12: Rule 1 and Rule 12: getting two linear parts out of one aggregate

Question: when one owned aggregate holds more than one linear part, how is each
part consumed, given that no place is ever partially moved?

Round one: `undecided-rule-gap` (verifier agreed and suggested whole-local
destructuring as an unstated third reading). Round one's fallback was to split
the aggregate into parallel containers.

Strongest program: two linear parts plus an affine part, with only one of the
linear parts consumed on one branch.

```text
linear type File;   fn close(f: own File)
struct Pair { in_f: File, out_f: File, scratch: Box<Slots<u8>>, n: u64 }

fn finish(c: own Pair, only_in: Bool) -> u64 {
    if only_in {
        let Pair { in_f, out_f, n, .. } = move c    // scratch released by `..`
        close(move in_f)
        close(move out_f)
        return n
    }
    x = move c.in_f                                  // rejected here
    close(move x)
    return 0
}
```

Trace:

- Rule 6: "A move out of a field or out of Box content consumes the whole owner:
  the owner ceases to exist, its other affine parts are released, and a
  remaining linear part rejects the move (take it in the same destructuring)."
  `move c.in_f` leaves `out_f`, a linear part, in an owner that has ceased to
  exist, so the second branch is rejected by name — no reading is left open.
- The same sentence's parenthesis names the admitted form, and Rule 6 spells it:
  "`let Conn { f, g, .. } = move c`   // several fields at once; `..` covers the
  rest". Both linear parts are bound in one destructuring, `scratch` is affine
  and is released by the same statement, `n` is Copy.
- Rule 12 is satisfied literally: "No place is ever partially moved: every path
  is either wholly present or the program cannot name it." After the
  destructuring `c` is not nameable at all; there is no hole, which is what
  "Not in this candidate" excludes.
- Rule 8's obligation, "consumed by an explicit operation on every exit path",
  is discharged by the two `close` calls on the accepted branch.
- The drop-flag-free result round one recorded still holds and is worth keeping:

```text
if cond { close(move a); s = move b } else { close(move b); s = move a }
use(&s);  close(move s)
```

  Rule 12's "At a join, facts are intersected" makes both edges leave exactly
  `s` live; C++ needs a destructor and, in general, a runtime drop flag, and WF
  emits neither.

Rewrite and cost: replace `x = move c.in_f` with the destructuring above. The
destructuring is a compile-time split of an owned local into its parts: the
`File` handles end up in the same registers or stack slots they already occupied
and `scratch`'s free is the free C++ would run. Zero. Round one's fallback
(splitting the aggregate into two parallel windows, paying an extra index
computation per access) is no longer needed.

Rules consistent: yes.   Task achievable: yes, at zero cost.
Changed by: Rule 6's consuming-move sentence and the `let Conn { f, g, .. } =
move c` form.
Verdict: accepted-fine

---

### 2-6: A window operation and the references into the container

Question: which references into a window survive which window operation, now
that the operations name parts rather than the whole container?

Round one: `rejected-real-cost` (verifier agreed, sharpening the cost to one
bounds compare per re-formed access). Every window operation declared
`writes(buf)`, so every operation killed every element reference.

Strongest program: a reference to a data-determined slot held across a removal
from the back and across a bulk append from another window.

```text
p = &dst[i]                              // i data-determined, i + 1 < dst.len known
v = take_back(&dst)                      // writes(dst.last), writes(dst.len)
use(p)                                   // accepted
append(&dst, &src)                       // writes(dst.free), writes(dst.len),
                                         // writes(src.filled), writes(src.len)
use(p)                                   // accepted
q = &src[j]
append(&dst, &src)
use(q)                                   // rejected: src.filled overlaps every live src[j]
```

Trace:

- `take_back` writes `dst.last` and `dst.len`. Rule 6: a live `r[i]` "overlaps
  `r.last` unless `i != r.len - 1` is proved". The pre-fact `i + 1 < dst.len`
  gives `i != dst.len - 1`, so the write paths and `dst[i]` are disjoint and
  Rule 10 clause 3 does not fire.
- The bound is re-proved for free. Rule 11: "a fact known before the call about
  a measure of an argument survives as a fact about `entry(p)`". With
  `i + 1 < entry(dst).len` and `ensures dst.len == entry(dst).len - 1`, affine
  arithmetic gives `i < dst.len`. No compare is emitted; Rule 6's note that
  "`take_back`'s [ensures] does not [carry the fact]" applies only when the
  caller knew nothing stronger than `i < r.len`.
- `append` writes `dst.free` and `dst.len` on the destination side. Rule 6: a
  live `r[i]` "never overlaps `r.next` or `r.free`", and `ensures dst.len ==
  entry(dst).len + entry(src).len` keeps `i < dst.len`. So every reference into
  the destination's existing elements survives a bulk move-in.
- On the source side `append` writes `src.filled`, which "always overlaps" a
  live `src[j]`, so `q` dies — correctly, since `ensures src.len == 0` empties
  the window.
- `insert_at` and `remove_at` both write `r.filled` and therefore kill every
  element reference, which is the memmove they perform.
- The refused variant is the one where the caller knows only `i < dst.len`
  before a `take_back`: then `i != dst.len - 1` is unprovable, `p` may name the
  slot being emptied, and the reference dies.

Rewrite and cost: for that refused variant, guard and re-form:
`v = take_back(&dst); if i < dst.len { p = &dst[i]; use(p) }`. That is one
compare and one address computation. The C++ program it is compared against
holds a pointer to element `i` across `pop_back` without knowing `i < n - 1`,
which is undefined behaviour when `i == n - 1`; correct C++ emits the same
compare. Cost against correct C++ and against safe Rust: zero. For
`insert_at`/`remove_at` a surviving reference below the edit point is re-formed
with one scaled add (the bound follows affinely from `i < k` and
`r.len == entry(r).len - 1`), where C++ keeps a register: one ALU operation, no
load, no branch.

Rules consistent: yes.   Task achievable: yes; the strongest program is accepted
unchanged.
Changed by: the part-precise rows of `take_back` and `append` plus Rule 6's
overlap sentence.
Verdict: accepted-fine

---

### 2-8: Consuming a linear value that lives behind a Box

Question: how is a linear value consumed when it sits in a field of a heap
object rather than in a whole local, and what happens to a reference into the
same object?

Round one: `undecided-rule-gap` (verifier agreed and widened it: the same
conflict appeared for a struct field and for Box content). Rule 8 asserted
`Box<File>` "must be taken apart" while Rule 6's enumeration provided no
operation that could do it.

Strongest program: the linear part consumed straight out of the heap object,
with a reference to the sibling field live across the consumption.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, n: u64 }

b = Box::new(Conn { f: move file, n: 0 })?     // Err arm hands (Oom, Conn) back
p = &(*b).n
x = *p                                          // read before the consuming move
close(move (*b).f)                              // consumes *b: b gone, cell freed
y = *p                                          // rejected
```

Trace:

- Rule 6: "A move out of a field or out of Box content consumes the whole owner:
  the owner ceases to exist, its other affine parts are released, and a
  remaining linear part rejects the move". The owner of `(*b).f` is `*b`; `n` is
  Copy, so no remaining linear part rejects the move, and the move is admitted.
  Rule 5's unbox line fixes what "ceases to exist" costs: "`n = move *c` //
  unbox: c is consumed, n is the Node, the cell is freed."
- Rule 8's example now has an operation behind it: "`f = move *b;  close(move f)`
  // `Box<File>`: unbox, then close."
- Rule 3 decides `p`: the consuming move "moved out of" `*b`, a proper prefix of
  `(*b).n`, so `p` is invalid afterwards and `y = *p` is rejected. Reading `x`
  before the move is an ordinary read at `p`'s own path.
- Rule 5's failure arm closes the other half of the question: "A fallible
  allocation that takes a by-value payload hands it back on failure, so a linear
  payload is never lost", `Result<Box<Conn>, (Oom, Conn)>`. The `Err` arm binds
  an owned `Conn` local and discharges it with the Rule 6 destructuring.
- The window case is unchanged and remains the cheap one:
  `c = remove_at(&pool, k)` yields `own Conn`, a whole local.

Rewrite and cost: hoist the read of the sibling field above the consuming move,
as written. Against C++ `unique_ptr<Conn>` whose destructor closes the file: the
WF form loads `n` from the heap object, loads `f`, calls `close` and frees the
cell — the same loads, the same call, the same free. Zero. There is no
whole-object copy: `close(move (*b).f)` needs no `c = move *b` step, so the
`sizeof(Conn)` memcpy that round one's fallback implied never appears.

Rules consistent: yes.   Task achievable: yes, at zero cost.
Changed by: Rule 6's consuming-move sentence, Rule 8's `f = move *b` line, and
Rule 5's payload-return sentence.
Verdict: accepted-fine

---

### 2-9: How an index reaches an effect row, and what the append slot is called

Question: may a reference name the slot at `r.len`, and how does a set-valued
reference substitute into a callee's row?

Round one: `undecided-rule-gap` (verifier agreed). Rule 9's own example formed
`&buf[len_of(buf)]`, contradicting Rule 7's formation requirement.

Strongest program: an attempt to name the append slot as a slot, and a
set-valued argument colliding with a second argument.

```text
fn put_at(slot: &Int, x: own Int)  writes(slot)
put_at(&r[r.len], 3)                  // rejected
place_back(&r, 3)                     // the sanctioned form

fn kernel(dst: &Slots<Int, N>, src: &Slots<Int, N>)  writes(dst), reads(src)
w = if c { &a } else { &b }
kernel(w, &third)                     // accepted: writes({a, b}) against reads(third)
kernel(w, &a)                         // rejected: the member pair (a, a) is write/read
```

Trace:

- Rule 7: "`r: Slots<Int, N>;  r[i]  // requires i < r.len`". For `i = r.len`
  that is `r.len < r.len`, false, so the formation is refused. Rule 6 agrees
  about what is there: "slots `[0, len)` hold values, `[len, cap)` hold
  nothing".
- The contradicting example is gone. Rule 9 now reads
  "`place_back(&r, 3)  // appending: writes(r.next), writes(r.len)`", so the
  append slot enters effect rows as the named part `r.next`, not as an index
  expression, which is consistent with Rule 9's own "Signatures never contain
  index expressions; an index enters an effect only through an argument".
- Naming the part in a row is not the same as publishing a value through it.
  Rule 6: measures are read-only — "A program reads them like fields and can
  never assign them; only the operations below change them" — and the only
  operation that raises `r.len` is `place_back`, which writes `r.next` from its
  own by-value argument. So a user function that writes `r.next` cannot make the
  slot live; the cost of that limitation is priced in 4-9, not here.
- The set-valued half is unchanged and decided: Rule 10 clause 1, "Two effects on
  overlapping paths where at least one is a write must be proved disjoint
  (different roots, or indices or ranges proved distinct)", with Rule 2's "every
  check on it must hold for every member of the set" making the member pair
  `(a, a)` decide `kernel(w, &a)`.

Rewrite and cost: `place_back(&r, 3)` is one store into the slot at `r.len` plus
one increment of the length word — the same two instructions a C++ `push_back`
into reserved capacity performs; zero. For the colliding call, hoist the branch
above it so each arm passes a singleton-target reference; zero, and duplicated
code is not a cost.

Rules consistent: yes.   Task achievable: rewrite, at zero cost.
Changed by: the replacement of Rule 9's `&buf[len_of(buf)]` example by
`place_back(&r, 3)`, and Rule 6's named part `r.next`.
Verdict: rejected-zero-cost

---

### 2-14: Allocation under live references, with a linear payload and an overlap question

Question: does allocation, which carries no effect, disturb a live reference or
the overlap permission of an adjacent statement — and is a linear payload safe
on the failure edge?

Round one: `accepted-fine` (verifier agreed, noting that the cell's `grow` lines
were a Rule 9/Rule 3 rejection rather than an allocation effect, and that the
cold-path re-formation might need one compare).

Strongest program: a live reference across a failing allocation of a linear
payload, next to an independent allocation.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, n: u64 }

p = &(*g.nodes)[i]
a = Box::new(Node { .. })?;   d = Box::new(Node { .. })?   // adjacent: may overlap
match Box::new(Conn { f: move file, n: 0 }) {
    Ok(bc)             => { use(&bc) }
    Err((_, back))     => { let Conn { f, .. } = move back; close(move f) }
}
place_back(&(*g.nodes), n2)                  // writes .next and .len
(*p).out = (*p).out + 1                      // p still valid
grow(&g.nodes, cap)?                         // writes(*g.nodes)
use(p)                                       // rejected on both arms of grow
```

Trace:

- Rule 3's invalidation list is closed over writes, moves, replacements and
  frees of a *named* prefix. An allocation writes no path the program can name,
  so `p` crosses both `Box::new` calls. Rule 14 states the same for scheduling:
  "Allocation and release carry no effect entry and never prevent two statements
  from overlapping", and its own example `a = build(&x)?; b = build(&y)?` is the
  adjacent pair above. Rule 13 supplies the meaning: "The program's meaning is
  its sequential meaning", so the `?` exits are the sequential ones.
- Rule 5 closes the linear-payload hole round one's neighbours reported:
  "A fallible allocation that takes a by-value payload hands it back on failure,
  so a linear payload is never lost". The `Err` arm binds an owned `Conn` and
  Rule 6's destructuring discharges the `File`, so Rule 8's "every exit path" is
  satisfied without the allocator ever owning a linear obligation.
- `place_back` is no longer a bystander threat (see 1-3), so the only remaining
  invalidation in this program is `grow`, which declares `writes(*g.nodes)` —
  a proper prefix of `(*g.nodes)[i]` — and Rule 9's "`writes` covers writing,
  replacing, moving out of, and freeing the storage at the path" makes the row
  hold for the whole call including its failure arm.
- The non-relocation obligation on the trusted base is still implied rather than
  stated, and is still not a contradiction: Rule 2 makes a reference "a local
  name for a path", so a relocating allocator would be a lowering bug, not a
  rule reading.

Rewrite and cost: after `grow`, re-form `p = &(*g.nodes)[i]`; `grow`'s
`ensures (*b).len == entry(*b).len` carries `i < (*g.nodes).len`, so the
re-formation is one base load and one scaled add with no compare, on the
reallocation path where C++ `vector` has just copied the whole block. Negligible
and amortized.

Rules consistent: yes.   Task achievable: yes.
Changed by: Rule 5's payload-return sentence, Rule 6's part-precise `place_back`
row, and the removal of the `par` statement in favour of adjacent statements.
Verdict: accepted-fine

---

### 3-6: Window operations versus element references: the run-length loop

Question: can the loop that C++ writes with a cached `Run* last` across
`push_back` be written with the same instruction count?

Round one: `accepted-fine` (verifier agreed) but with a priced residue: every
`push_nogrow` killed `last`, so each iteration re-formed the reference and
reloaded the length word, one scaled add plus one header load that C++ does not
pay.

Strongest program: append while updating the element appended last, with the
reference held across the append.

```text
for i in 0..m {
    if runs.len > 0 {
        last = &runs.last                      // the last filled slot
        if (*last).v == src[i] {
            (*last).n = (*last).n + 1          // content write: last stays valid
            continue
        }
    }
    place_back(&runs, Run { v: src[i], n: 1 }) // writes(runs.next), writes(runs.len)
}
```

Trace:

- Rule 6 makes `runs.last` an ordinary path: "a window has four named parts that
  paths and effect rows may name, all interpreted at call entry ... `r.last`
  (the last filled slot, at index `r.len - 1`)". Forming it needs `r.len > 0`,
  which the dominating branch establishes (Rule 11, "refinement facts from a
  dominating branch").
- Rule 2 fixes what the reference names afterwards: "An index expression inside
  a path is evaluated when the reference is formed; the path records that value."
  So `last` keeps naming the slot that was last at formation; it does not follow
  the window.
- The write through `last` is a content write. Rule 3: "Writing the storage at
  p's path or below it (a content write) does not invalidate p."
- `place_back` writes `runs.next` and `runs.len`. Rule 6: a live `r[i]` "never
  overlaps `r.next`", and the index recorded in `last` is `r.len - 1` at
  formation, strictly below the appended index, so `last` survives the append as
  well — it simply names the previous run, which is what the sequential meaning
  says. The loop re-forms at the top of the next iteration because it wants the
  new last, not because a rule forced it.
- Rule 12 at the join of the `if` no longer has to intersect a validity fact,
  since neither edge invalidates; only the binding differs, and the loop rebinds
  at the header, which Rule 2 allows ("a loop-carried rebinding may change only
  the index values inside the path, never extend the path through itself").
- The counterexamples in the same family: `take_back` writes `r.last`, which
  overlaps `last` by definition, so it dies there; `insert_at` and `remove_at`
  write `r.filled`, which "always overlaps" a live slot; `grow` writes `*b`.

Rewrite and cost: none needed. Per iteration the loop emits one length compare,
one address computation for `runs.last`, the value compare, and either one
increment or one store plus one length increment — the same sequence as C++ with
`reserve` and a cached `Run* last`. Round one's extra header load and scaled add
per iteration are gone, because the only thing that changes between iterations is
which slot is last, which C++ recomputes too.

Rules consistent: yes.   Task achievable: yes, at zero cost.
Changed by: the named part `r.last` plus `place_back`'s part-precise row.
Verdict: accepted-fine

---

### 3-8: Validity while a linear part is consumed out of an aggregate

Question: when a linear field is consumed, what happens to a live reference into
the rest of the same aggregate?

Round one: `undecided-rule-gap` (verifier agreed and escalated: under the strict
reading no linear value inside an aggregate could ever be consumed).

Strongest program: a reference into a heap block owned by a sibling field, held
across the consumption of the linear field.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, scratch: Box<Slots<u8>>, n: u64 }

fn finish(c: own Conn) -> u64 {
    p = &(*c.scratch)[0]
    close(move c.f)                       // rejected use of p afterwards
    x = *p
    return x
}
```

Trace:

- Rule 6: the move out of the field "consumes the whole owner: the owner ceases
  to exist, its other affine parts are released". `c` is `p`'s root and `c` is a
  proper prefix of `(*c.scratch)[0]`, so Rule 3's "a proper prefix of p's path
  is written, moved out of, replaced, or freed" invalidates `p`, and the release
  of `scratch` frees the very block `p` names. The rejection is not merely
  bookkeeping: it is the one that keeps the program memory-safe.
- The consumption itself is now admitted. `f` is the only linear part, so the
  clause "a remaining linear part rejects the move" does not fire.
- The admitted form that keeps the block alive is the destructuring, which binds
  the sibling as a local root:

```text
fn finish(c: own Conn) -> u64 {
    let Conn { f, scratch, n } = move c   // c ceases to exist; scratch is now a local
    close(move f)
    q = &(*scratch)[0]                    // same heap block, new root
    return *q + n
}
```

  Rule 6 spells the form ("`let Conn { f, g, .. } = move c`"), Rule 12's "No
  place is ever partially moved" is satisfied because `c` is wholly gone, and
  Rule 3's "A move never re-roots an existing reference" is why `q` must be
  formed again rather than `p` being reinterpreted.

Rewrite and cost: the destructuring above. `scratch` is a Box: the destructuring
moves one pointer word into a local and the heap block never moves, so `q`
computes the same address `p` did. One register move that the backend coalesces;
against C++'s member access through `this`, zero.

Rules consistent: yes.   Task achievable: rewrite, at zero cost.
Changed by: Rule 6's consuming-move sentence, which turned round one's
contradiction into a decided rejection with a stated remedy.
Verdict: rejected-zero-cost

---

### 3-12: Joins, the absence of holes, and reaching the payload of a variant

Question: can the representation Rule 12 prescribes for non-Copy payloads,
`Option<Entry>` in a fully initialized block, be updated in place?

Round one: `rejected-real-cost` (verifier's correction of the cell's
`undecided-rule-gap`). With no payload step in the path grammar, every update
went through the whole-slot atomic update: for a 32-byte `Entry` about eight
extra loads and stores per hit.

Strongest program: a hash-table hit that updates two fields of the payload.

```text
slots: Box<Array<Option<Entry>>>

if i < (*slots).len {
    match &(*slots)[i] {
        Some(e) => {
            (*e).count = (*e).count + 1        // first write through the payload path
            (*e).stamp = t                     // second write: is e still valid?
        }
        None => { (*slots)[i] = Some(fresh) }
    }
}
```

Trace:

- Rule 2 now gives the step: a path "continues through fields, `*` (Box
  content), `[i]` (index, Rule 7), `[lo..hi]` (range, Rule 7), or the payload of
  an enum variant. A payload step is available only under the refinement fact
  that the enum currently holds that variant, which a `match` or `if let` on the
  enum establishes in the selected arm". Rule 2's own example binds `child` the
  same way. So `e` names `(*slots)[i].Some.0` and the first write is a content
  write at `e`'s own path, which Rule 3 explicitly preserves.
- The second write is where the text stops deciding. The sentence that governs
  the fact is, verbatim:

  > "A payload step is available only under the refinement fact that the enum
  > currently holds that variant, which a `match` or `if let` on the enum
  > establishes in the selected arm and which any write to the enum invalidates."

  Read narrowly — "a write to the enum" is a write of the enum place, the only
  write that can change the variant — `e` survives its own field writes and the
  program above is accepted with two stores. Read broadly — any write to storage
  inside the enum, which includes `(*e).count` — the fact dies at the first
  write and the second is rejected. Rule 3 is drafted with exactly this
  distinction available ("Writing the storage at p's path or below it (a content
  write) does not invalidate p") but states the payload case only as "a
  refinement fact that a payload step in p's path depends on is invalidated",
  without saying what invalidates it. Rule 11's "A fact that mentions a path is
  invalidated when that path is written" does not say whether writing below a
  path writes it. Resolving this is outside a derivation cell.
- The `None` arm and the join half are decided and unchanged: `(*slots)[i] =
  Some(fresh)` is a whole-place write (Rule 6's assignment clause), and a
  reference whose target set spans a place consumed on one edge is rejected by
  Rule 12's "facts are intersected" plus Rule 2's "every check on it must hold
  for every member of the set", with a zero-cost rewrite that duplicates the use
  into both arms.

Rewrite and cost: under the narrow reading, none needed — the loop is one
discriminant test and two stores, matching Rust's `if let Some(e) = &mut
slots[i]` exactly and matching C++'s `std::optional` with a niche; the "one null
check per hit versus hashbrown" already recorded under Known costs is the whole
difference. Under the broad reading, either re-match before each further write
(one extra discriminant load, compare and predicted branch per additional write)
or write the payload once as a whole value, `(*e) = Entry { count: (*e).count +
1, stamp: t, .. }` (one `sizeof(Entry)` store instead of two field stores).
Either way round one's whole-slot round trip is gone.

Rules consistent: undecided for a second write through a payload reference.
Task achievable: yes under either reading; the cost differs by one compare per
extra write, not by an order of magnitude.
Changed by: Rule 2's payload-step sentence, which removed the whole-slot round
trip and created this narrower question in its place.
Verdict: undecided-rule-gap

---

### 4-5: Rule 4 and Box: rewriting a subtree that no returned reference may name

Question: with no reference allowed to leave the frame that formed it, how does
a program replace a child deep in a heap-linked tree without re-descending?

Round one: `undecided-rule-gap` (verifier agreed and sharpened the fallback: if
the atomic update were window-only, even rebuilding by value was blocked, so the
only form left was a Rule 16 pool with one compare per hop).

Strongest program: find and mutate in one descent, with the mutation happening at
a depth the caller never names.

```text
struct Node { key: Int, left: Option<Box<Node>>, right: Option<Box<Node>> }

fn insert(x: own Option<Box<Node>>, k: Int) -> own Option<Box<Node>>
    contract { ensures true; }
{
    match x {
        None      => Some(Box::new(Node { key: k, left: None, right: None })?),
        Some(b)   => {
            if k < (*b).key { (*b).left  = insert((*b).left,  k) }
            else            { (*b).right = insert((*b).right, k) }
            Some(b)
        }
    }
}

root = insert(root, k)
```

Trace:

- Rule 4 still refuses the C++ shape: a `find_slot` returning
  `&Option<Box<Node>>` "cannot be returned", and Rule 4's remedy (return an
  index) does not reach a Box chain, which Rule 7 does not list among indexable
  things. So the mutation must happen in the frame that owns the path.
- Rule 6 now supplies that frame's operation without qualification: "Two
  operations apply to any owned place, not only to window slots: ...
  `place = f(place, args...)  writes(place)  // atomic in-place update: the old
  value enters f by value, f's result is committed, no program point lies
  between`", with the example `node.left = insert(node.left, k)`. A struct field
  of a heap object is an owned place, so the question round one could not decide
  is answered by the sentence's first clause.
- The side conditions hold: "`f` is total and returns the place's type; `f`'s row
  must not overlap any prefix of `place`". `insert` has no reference parameter at
  all, so its substituted row is empty and Rule 10 clause 1 has nothing to
  compare. Allocation failure is routed as the enum in the place (`Option`), not
  as `?` through the update, which is what Rule 6 requires ("failure is an enum
  in the place").
- `Some(b)` in the match arm binds the payload under Rule 2's payload step, and
  the two field updates are writes below `b`, so Rule 3 leaves `b` valid; the
  arm returns the same Box it received, so nothing is copied.
- Recursion is checked through the contract, not by unfolding (Rule 10), and the
  path never grows through itself, so Rule 2's static-shape clause is not
  touched — which is exactly why the loop form (`p = &(*p.kids)[0]`) stays
  refused and recursion is the form.

Rewrite and cost: none needed for acceptance. The residual cost is the one the
atomic update commits: on the unwind, each level stores the returned
`Option<Box<Node>>` back into the field it came from. Against C++'s
pointer-to-pointer descent (`Node** p = &root; ... *p = n`), which writes one
pointer once, the WF form writes one pointer word per level: about `depth`
stores to lines already hot from the descent, roughly 20 stores per insert into
a balanced million-node tree. It is identical to idiomatic safe Rust
(`node.left = insert(node.left, k)`), and the Known-costs entry "Find-then-mutate
on a Box-linked tree re-descends once unless fused" is avoided here because the
descent and the mutation are the same pass.

Rules consistent: yes.   Task achievable: yes; one pointer store per level more
than the C++ double-pointer idiom, zero against safe Rust.
Changed by: "Two operations apply to any owned place, not only to window slots",
with `node.left = insert(node.left, k)` as the stated example.
Verdict: accepted-fine

---

### 4-8: Rule 4 and linearity: servicing and removing a resource found by a search

Question: can a linear value located by an index be serviced in place, removed
in order, and discharged on every exit path, with nothing ever moved out through
a reference?

Round one: `accepted-fine` (verifier agreed) but the derivation had to steer
around two sentences that pulled against each other: `truncate` on a linear
element type, and Rule 8's `close(move c.f)`.

Strongest program: an order-preserving removal of a linear resource from the
middle of the pool, plus in-place recycling of another slot, plus the drain.

```text
linear type File;   fn close(f: own File)
pool: Box<Slots<Option<File>>>              // linear by containment

i = find_idle(&*pool)?                      // Rule 4: an index, not a reference
(*pool)[i] = recycle((*pool)[i])            // atomic update at an owned place
fn recycle(x: own Option<File>) -> own Option<File>
    { match x { Some(f) => { close(move f); None }   None => None } }

j = find_dead(&*pool)?
s = remove_at(&*pool, j)                    // own Option<File>, one memmove
match s { Some(f) => close(move f)   None => {} }

while (*pool).len > 0 {
    t = take_back(&*pool)
    match t { Some(f) => close(move f)   None => {} }
}
```

Trace:

- Rule 6: `remove_at(&r, k) -> own T  writes(r.filled), writes(r.len)  contract
  { requires k < r.len; ... }`, and the returned value is a whole local, so
  `move f` out of it is the ordinary destructuring case. Nothing releases the
  element on the way out — Rule 6, "No operation releases a linear element" —
  so the linear obligation travels with the value and is discharged by `close`.
- The atomic update is the in-place service form and is now stated for any owned
  place, with the slot case as one instance; Rule 6's assignment clause forces
  it for a linear element ("Assigning over any owned place releases the old
  value if it is affine and is rejected if it is linear"), and the update's `f`
  consumes the old value by value instead.
- Rule 4 is what makes this composition possible: the locator returns an index,
  the caller supplies it as an argument, and Rule 9's "an index enters an effect
  only through an argument, evaluated once at the call" keeps the rows narrow.
- Rule 11's own `while r.len > 0` example supplies `take_back`'s precondition
  from the loop condition.
- Both round-one tensions are gone: `truncate` is library code (Rule 6's library
  list), and `close(move c.f)` is decided by the consuming-move sentence, so the
  derivation no longer has to avoid them.

Rewrite and cost: none needed. The recycle is one discriminant test and one
`close`, exactly `std::optional<File>::reset` or Rust's `Option::take`. The
ordered removal is one memmove of the tail, exactly `std::vector::erase`. The
drain is the destructor loop C++ and Rust emit.

Rules consistent: yes.   Task achievable: yes, at zero cost.
Changed by: `remove_at`'s row and contract, and the generalization of the atomic
update to any owned place.
Verdict: accepted-fine

---

### 4-9: Rule 4 and effect rows: constructing an element into the append slot

Question: can a callee build a large element directly in the block, given that
the caller forms every reference and no row names an index expression?

Round one: `undecided-rule-gap` (verifier agreed). Rule 9's example formed a
reference to the slot at the length while Rule 6 and Rule 7 required
`k < r.len`; nothing decided whether the append slot was referenceable.

Strongest program: an element whose bulk is inline, constructed by a separately
compiled callee.

```text
struct Entry { key: u64, payload: Array<u8, 96> }
fn build_into(slot: &Entry, src: &Src)  writes(slot), reads(src)

build_into(&r.next, &s)         // names the append slot, but nothing publishes it
place_back(&r, ???)             // would overwrite r.next from its own argument
```

Trace:

- Rule 6 makes the append slot nameable at last: "a window has four named parts
  that paths and effect rows may name ... `r.next` (the append slot, at index
  `r.len`)", and "This vocabulary is ordinary: the rows below use nothing a user
  function cannot write." So `build_into(&r.next, &s)` has a well-formed row and
  the round-one contradiction with Rule 7 is gone — the append slot is not
  reached as `r[r.len]` at all.
- But the value written there stays invisible. Rule 6: measures are read-only,
  "A program reads them like fields and can never assign them; only the
  operations below change them", and the only listed operations that raise
  `r.len` are `place_back`, `insert_at` and `append`, each of which supplies the
  new element itself — `place_back(&r, x)` writes `r.next` from `x`. No
  operation publishes a slot the program has already filled, so the two-step
  "construct then commit" that C++'s `emplace_back` performs has no spelling
  here. This is a decided rejection, not an ambiguity: no reading of the listed
  operations produces a length-only increment.
- Rule 15 closes the other route: a function "returns owned values only" and
  "Passing a large value in and back out is expressed as a reference parameter
  with a `writes` entry", which is precisely the form that cannot reach the
  unpublished slot.

Rewrite and cost: two forms, both real.

```text
x = make(&s);  place_back(&r, move x)          // (a) one element move
place_back(&r, blank);  build_into(&r.last, &s) // (b) one placeholder store
```

(a) materializes the 104-byte `Entry` in a local and `place_back` moves it into
the slot: one extra `sizeof(Entry)` copy per appended element against C++'s
in-place construction. Rule 6's "no program point lies between" does not apply
here and no rule requires the backend to materialize the argument at its
destination; the Known-costs entry says exactly this ("Construction into the
append slot moves one element where C++ constructs in place; usually elided by
the backend, not guaranteed"). (b) replaces the copy with one store of a valid
placeholder and then fills in place; `&r.last` is in bounds for free, since
`place_back`'s `ensures r.len == entry(r).len + 1` gives `r.len > 0`, so this
form costs one `sizeof(Entry)` initialization instead of one copy. Where the
element's bulk is a `Box` field rather than inline, (b) collapses to a few words
and the cost effectively vanishes; for an inline payload it does not.

Rules consistent: yes.   Task achievable: rewrite, at one element move or one
placeholder store per appended element.
Changed by: the named part `r.next` (which decides the question round one could
not) and `place_back`'s by-value argument (which is why the answer is still no).
Verdict: rejected-real-cost

---

### 5-5: Box inside Box: relinking an owned structure in place

Question: can a structure whose edges are owning `Box` values be rotated in
place, the case where one Box must leave another Box's field?

Round one: `undecided-rule-gap` (verifier agreed and found no reconciling
reading). The fallback was a Rule 16 arena with `u64` links, three bounds
compares per rotation.

Strongest program: the standard right rotation, in place, with no allocation.

```text
struct Node { key: Int, left: Option<Box<Node>>, right: Option<Box<Node>> }

fn rotate_right(slot: &Option<Box<Node>>)  writes(slot)
{
    hole: Option<Box<Node>> = None
    swap(slot, &hole)                                     // hole = Some(root), slot = None
    match &hole { Some(rb) => { swap(slot, &(*rb).left) }  None => {} }
                                                          // slot = Some(l), root.left = None
    match slot  { Some(lb) => {
        match &hole { Some(rb) => { swap(&(*rb).left, &(*lb).right) }  None => {} }
    } None => {} }                                        // root.left = l.right, l.right = None
    match slot  { Some(lb) => { swap(&(*lb).right, &hole) }  None => {} }
                                                          // l.right = Some(root), hole = None
}
```

Trace:

- The naive three-move form is now decided against, and decided cleanly:
  `l = move (*root).left` falls under Rule 6's "A move out of a field or out of
  Box content consumes the whole owner: the owner ceases to exist, its other
  affine parts are released", so it would free the right subtree and leave
  `(*root).right` unnameable; every later statement of the rotation is then
  rejected. Round one could not tell whether this form was admitted; it is
  admitted and useless, which is the same answer for the programmer.
- `swap` is what replaces it: "Two operations apply to any owned place, not only
  to window slots: `swap(p: &T, q: &T)  writes(p), writes(q)  // built in; p and
  q may be the same place", with the example `swap(&a.left, &b.right)`. Each
  swap above pairs two disjoint places (different roots, or one under `hole` and
  one under `slot`), so Rule 10 clause 1 is satisfied without needing the
  same-place allowance.
- Rule 2's payload step is the other half: `rb` names `hole.Some.0` and `lb`
  names `slot.Some.0`, each under the refinement fact its `match` establishes.
  Each arm performs exactly one write through each live payload reference, so
  the second-write question raised in 3-12 is never reached.
- Rule 10 clause 3 kills `rb` at the final `swap(&(*lb).right, &hole)`, since
  `hole` is a proper prefix of `rb`'s path; the program does not use it
  afterwards.
- No hole is left anywhere: `hole` is a local holding a valid `None` between the
  swaps, which is a value, not a compiler-maintained absence, so Rule 12's "No
  place is ever partially moved" holds throughout. Rule 1's relocatability is
  what makes each swap a pair of word copies.
- Nothing is allocated or freed, so the `Result` of Rule 14 never appears in the
  rotation.

Rewrite and cost: none needed; the rotation is the program above. Four swaps of
a niche-optimized `Option<Box<Node>>` are eight word-sized loads and stores
against the C++ pointer rotation's six, plus three discriminant tests with
writer-supplied unreachable arms — two extra memory operations and three
well-predicted branches per rotation. Against safe Rust's `Option::take`-based
rotation, which is the same swap-with-`None` plus the same matches, it is zero.
Round one's arena fallback, with its three bounds compares per rotation and the
loss of the ownership tree, is no longer needed.

Rules consistent: yes.   Task achievable: yes; two extra memory operations and
three predicted branches per rotation against C++, zero against safe Rust.
Changed by: `swap` on any two owned places and Rule 2's payload step; the
consuming-move sentence is what rules out the naive form.
Verdict: accepted-fine

---

### 5-6: Box inside a window: a stable object while the window moves

Question: does a `Box` in a window slot keep a reference into the heap object
alive across appends, the way `vector<unique_ptr<T>>` keeps a `T*` in C++?

Round one: `rejected-real-cost` (verifier agreed). The library `push` declared
`writes(buf)`, so the reference died at every iteration and each use cost a
block-base load, a slot load and a compare.

Strongest program: a data-determined slot's heap object updated inside a loop
that keeps appending, with growth possible.

```text
b: Box<Slots<Box<Node>>>
k = pick(..)                                   // data-determined, k < (*b).len
p = &(*(*b)[k]).value

for i in 0..m {
    if (*b).room == 0 {
        grow(&b, 2 * (*b).cap)?                // writes(*b): p dies here only
        p = &(*(*b)[k]).value                  // re-form; k < (*b).len is carried
    }
    place_back(&*b, Box::new(mk(i))?)          // writes .next and .len: p survives
    *p = *p + 1
}
```

Trace:

- Forming the path needs `k < (*b).len` (Rule 7) and continues through `[k]`,
  `*` and `.value` (Rule 2).
- `place_back` writes `(*b).next` and `(*b).len`. Rule 6: a live `r[i]` "never
  overlaps `r.next` or `r.free`", and `r.len` is not a prefix of `r[k]`, so
  Rule 10 clause 3 does not fire and `p` crosses the append. The bound is
  carried by `ensures r.len == entry(r).len + 1`.
- `grow(&b, cap)  writes(*b)` is a proper prefix of `(*b)[k]`, so `p` dies
  there, and correctly: the block may be reallocated. The re-formation's bound
  is free, because `grow`'s `ensures (*b).len == entry(*b).len` carries
  `k < (*b).len` across the call (Rule 11's entry clause).
- Rule 5's address stability still buys nothing by itself — `c = move b` kills a
  reference even though "the heap object did not move" — which is why the fix is
  the part-precise row, not an address argument.
- The library `push` of Rule 6's list still declares `writes(*b)`, because it may
  call `grow`; hoisting its capacity test into the loop, as above, is what keeps
  the narrow row on the hot path. Rule 6 permits it directly: `room` is a
  readable measure and `place_back` requires only `r.room > 0`.

Rewrite and cost: the hoisted growth test shown above is the rewrite, and it
costs nothing that C++ does not also pay — `std::vector::push_back` performs the
same capacity compare on every call. On the reallocation path the re-formation
is one block-base load plus one slot load, once per doubling, against C++'s zero;
amortized over the elements appended that is well under one instruction per
element, and the `grow` itself has just copied the whole block. Round one's
per-iteration two dependent loads and compare are gone, and the
order-destroying `swap_remove` workaround it proposed is unnecessary (and if a
program does want the Box out, `swap(&(*b)[k], &local)` now does it in order).

Rules consistent: yes.   Task achievable: yes; zero on the hot path, two loads
per reallocation.
Changed by: `place_back`'s part-precise row plus Rule 6's overlap sentence;
`swap` on any two places removes the round-one workaround's side effect.
Verdict: accepted-fine

---

### 5-8: Box of a linear value: unboxing it and surviving allocation failure

Question: can a linear resource live behind a `Box`, given that Rule 8 requires
an explicit consumption on every exit path?

Round one: `undecided-rule-gap` (verifier agreed and called the allocation-failure
half the stronger gap). Two holes: nothing could take the Box apart, and the
`Err` arm of `Box::new` swallowed the payload.

Strongest program: a linear payload boxed, with an early exit and a failing
allocation.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, n: u64 }

fn stage(c: own Conn) -> Result<u64, Err>
{
    b = match Box::new(move c) {
        Ok(bx)           => bx,
        Err((_, back))   => { let Conn { f, .. } = move back; close(move f); return Err(E0) }
    }
    x = (*b).n
    if bad(x) { close(move (*b).f); return Err(E1) }     // consumes *b, frees the cell
    close(move (*b).f)                                    // normal exit
    return Ok(x)
}
```

Trace:

- Rule 5: "A fallible allocation that takes a by-value payload hands it back on
  failure, so a linear payload is never lost", with the signature
  `Result<Box<Conn>, (Oom, Conn)>`. The `Err` arm binds an owned `Conn`, and
  Rule 6's destructuring (`let Conn { f, .. } = move back`) discharges the
  `File` while `..` covers the Copy field. Rule 8's "every exit path" is now
  satisfiable on the allocation-failure edge, which round one showed it was not.
- Rule 6's consuming move is what takes the Box apart: the move out of `(*b).f`
  "consumes the whole owner", so `*b` ceases to exist and, per Rule 5's unbox
  line, the cell is freed. `n` is Copy, so no remaining linear part rejects the
  move; it is read into `x` beforehand because the owner is gone afterwards.
- Both `return` edges consume the `File` explicitly, so nothing depends on the
  compiler releasing anything — which Rule 8 forbids for linear values and
  Rule 5 defers to Rule 8 ("at scope exit the compiler releases its memory
  recursively (Rule 8)").
- Rule 8's own example is now executable as written: "`f = move *b;  close(move
  f)`   // `Box<File>`: unbox, then close."

Rewrite and cost: none needed, and round one's pre-reserved pool workaround —
which forced the resource count to be sized up front — is no longer required.
Against C++ `unique_ptr<Conn>` with a destructor: the same field load, the same
`close`, the same free, plus the `Result` test that C++ replaces with an
exception edge; against Rust `Box<Conn>` with `Drop`, the same plus an explicit
`close` call the programmer writes instead of the compiler. Zero on the success
path, and the failure path runs the destructuring that C++ would run in a catch
block.

Rules consistent: yes.   Task achievable: yes, at zero cost.
Changed by: Rule 5's payload-return sentence and Rule 6's consuming-move
sentence.
Verdict: accepted-fine
