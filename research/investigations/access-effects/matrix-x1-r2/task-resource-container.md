# Engineering task under candidate x1, revision 4 — dynamic resource container

Derived against the frozen rule set in `CANDIDATE-X1.md` (revision 4, 2026-09-18)
and only that file. This is the round-two update of
`matrix-x1/task-resource-container.md`, which was derived against revision 2.
Round one's structure is kept where it still holds; every place the revised rules
change the derivation is redone and marked. Section 9 lists exactly what changed.

Task text: *Dynamic resource container: a Vector of non-Copy elements (Box and
linear payloads) with push, growth, replace, ordered remove, in-place sort, and
hand-over of an element to another owner; every failure path (allocation failure)
must keep every resource accounted for.*

Cost convention, protocol item 7: runtime performance is the only cost.
Verbosity, extra parameters and duplicated code are not costs. One **move** means
one relocation of `sizeof(T)` bytes; Rule 1's stated consequence — "any value can
be relocated by copying its bytes (memmove, realloc), because nothing inside it
points anywhere" — makes every move a plain byte copy with no fixup, so move
counts compare directly against C++ move constructors and Rust `memcpy`.

Notation follows the rule file: measures and window parts are members
(`(*v.buf).len`, `(*v.buf).room`, `r.next`, `r.last`, `r.filled`, `r.free`),
there is no `len_of` and no `par` statement, and two adjacent statements are
annotated with whether they may overlap.

---

## 1. The question this task tests

Can a growable container hold values that must not be duplicated and must not be
silently dropped — `Box<Node>` (affine) and `File` / `Conn` / `Box<File>`
(linear) — and support push, growth, replace, ordered and unordered removal,
insertion, in-place sort, and transfer of one element to another owner, such that
every allocation-failure edge leaves exactly one owner for every resource, under
revision 4's window operations (`place_back`, `take_back`, `insert_at`,
`remove_at`, `append`, `grow`, `set`, `replace`, `swap`, the atomic update) and
Rule 8's rule that the compiler never releases a linear value?

Round one answered "yes, with cost", where the cost was dominated by two
rewrites that revision 4 deletes outright: order-preserving growth at 2.5 moves
per element, and ordered removal at 3 moves per shifted element. The question
this round actually tests is what is left once those are built in.

---

## 2. Programs

### 2.1 Element types

```text
linear type File;   fn close(f: own File)

struct Node { key: u64, bytes: Box<Array<u8>> }   // affine: a Box of Copy elements
struct Conn { f: File, id: u64 }                  // linear by containment (Rule 8)

// The four element types this task must carry:
//   Box<Node>   affine, non-Copy, one pointer in the slot, one heap object behind it
//   File        linear, non-Copy, opaque width W
//   Conn        linear by containment, 16 bytes in the slot
//   Box<File>   a Box whose content is linear, hence linear (Rule 8)
```

### 2.2 The container

Rule 6 names it under "Library code, all zero-cost compositions of the above":
"`Vector<T>` (a `Box<Slots<T>>` plus a `push` that calls `grow` when `room` is
zero)".

```text
struct Vector<T> { buf: Box<Slots<T>> }        // Rule 1: owned field, no reference inside
                                               // Rule 5: a runtime-capacity Slots lives only in a Box

fn new_vector<T>(c: u64) -> Result<Vector<T>, Oom>
    contract { ensures when Ok: (*result.buf).len == 0, (*result.buf).cap == c; }
{
    b = Box::new(Slots::new<T>(c))?            // Rule 6's own constructor line; see gap G-B
    Ok(Vector { buf: move b })                 // Rule 15: a function returns owned values only
}

fn count<T>(v: &Vector<T>) -> u64   reads(*v.buf)
    contract { ensures result == (*v.buf).len; }
{ (*v.buf).len }                               // Rule 6: a measure is read like a field
```

Two pushes with different rows. This split is the single most consequential new
thing in revision 4 for this task, and it is not an optimization: the two
functions have different interfaces across separate compilation.

```text
// (a) the part-precise push: cannot grow, so it does not write the block
fn push_here<T>(v: &Vector<T>, x: own T)   writes((*v.buf).next), writes((*v.buf).len)
    contract { requires (*v.buf).room > 0;
               ensures (*v.buf).len == entry(*v.buf).len + 1; }
{ place_back(&*v.buf, move x) }

// (b) the general push: may grow, so it writes the whole block
fn push<T>(v: &Vector<T>, x: own T) -> Pushed<T>   writes(v.buf)
    contract {
        ensures when Ok:   (*v.buf).len == entry(*v.buf).len + 1;
        ensures when Full: (*v.buf).len == entry(*v.buf).len,
                           (*v.buf).cap == entry(*v.buf).cap;
    }
```

### 2.3 The strongest program: push of a linear element with a failing growth

This is the case where static analysis provably cannot help: the element is
linear, so the compiler may not release it (Rule 8); the growth is
data-determined and may fail; on the failure edge the element is in flight, owned
by neither the caller nor the container; and a reference into the container is
live across the call.

```text
enum Pushed<T> { Ok, Full(own T) }     // Rule 1: an owned payload
                                       // Rule 8: "propagates through every aggregate of Rule 1:
                                       // a struct, enum, tuple, Array, Slots, Ring, or Box" —
                                       // so Pushed<Conn> is linear and cannot be dropped silently

fn reserve<T>(v: &Vector<T>, extra: u64) -> Result<Unit, Oom>   writes(v.buf)
    contract {
        ensures when Ok:  (*v.buf).room >= extra, (*v.buf).len == entry(*v.buf).len;
        ensures when Err: (*v.buf).len == entry(*v.buf).len, (*v.buf).cap == entry(*v.buf).cap;
    }
{
    if (*v.buf).room >= extra { return Ok }
    need = (*v.buf).len + extra
    want = 2 * (*v.buf).cap + 1
    if want < need { want = need }
    grow(&v.buf, want)?                // one reallocation, may extend in place; see gap G-A
    Ok
}

fn push<T>(v: &Vector<T>, x: own T) -> Pushed<T>   writes(v.buf)
{
    if (*v.buf).room == 0 {
        match reserve(&v, 1) {
            Ok     => { }
            Err(_) => { return Full(move x) }      // x is still an owned local here
        }
    }
    place_back(&*v.buf, move x)                    // requires room > 0: discharged at the join, §3.3
    Ok
}
```

The fallback growth, used where `grow`'s failure arm is unavailable (gap G-A) and
also as `shrink_to_fit`. It is written entirely in operations whose contracts the
rule file states, and it is failure-safe by construction because the single
fallible step happens before anything moves:

```text
fn regrow<T>(v: &Vector<T>, want: u64) -> Result<Unit, Oom>   writes(v.buf)
    contract { requires want >= (*v.buf).len;
               ensures when Ok:  (*v.buf).cap == want, (*v.buf).len == entry(*v.buf).len;
               ensures when Err: (*v.buf).len == entry(*v.buf).len, (*v.buf).cap == entry(*v.buf).cap; }
{
    nb = Box::new(Slots::new<T>(want))?      // the only fallible step; nothing has moved yet
    append(&*nb, &*v.buf)                    // ONE memcpy; ensures (*v.buf).len == 0
    swap(&v.buf, &nb)                        // v.buf is the new block, nb the old empty one
    Ok                                       // nb leaves scope: an empty block, freed (§3.7)
}
```

Note what `swap` is doing here and why the obvious line is not written. For a
linear element type, `v.buf = move nb` is **rejected**: Rule 6, "Assigning over
any owned place releases the old value if it is affine and is rejected if it is
linear", and `Box<Slots<Conn>>` is linear by containment whatever its current
length. `swap` releases nothing, so it is the admitted form, and it costs the
same two pointer-sized stores.

### 2.4 Caller-side accounting of the failure edge

```text
// c: own Conn arrives from somewhere; the container is a Vector<Conn>
match push(&table, move c) {
    Ok         => { }                              // the container owns it now
    Full(back) => { let Conn { f, .. } = move back // Rule 6: a move out of a field consumes the owner
                    close(move f) }                // Rule 8: an explicit consumer on this exit path
}

// affine element: the failure edge needs no writer action
match push(&nodes, move b) {                       // b: own Box<Node>
    Ok         => { }
    Full(back) => { }                              // Rule 8: the compiler releases affine values
}

// a linear payload that never reached a container, on a failed allocation
match Box::new(Conn { f: move file, id: 0 }) {
    Ok(bc)         => { holder = move bc }
    Err((_, back)) => { let Conn { f, .. } = move back;  close(move f) }   // Rule 5's payload return
}
```

### 2.5 Replace

All three shapes are built in as of revision 4; none needs a rewrite.

```text
// (a) affine element, old value discarded
set(&(*v.buf)[k], move fresh_box)      // Rule 6: "old value: affine released, linear rejected"
                                       // requires k < (*v.buf).len (Rule 7)

// (b) linear element, old value consumed in place — the atomic update, with an extra argument
fn swap_in(old: own File, fresh: own File) -> File { close(move old); move fresh }
(*v.buf)[k] = swap_in((*v.buf)[k], move fresh)     // Rule 6: "place = f(place, args...)"

// (c) linear element, old value must reach the caller
old = replace(&(*v.buf)[k], move fresh)            // Rule 6: "the old value is returned"
// ... old is an owned local; close(move old) or hand it on

// (d) exchange with a place that is not a window slot
swap(&(*v.buf)[k], &holder.current)                // Rule 6: swap applies to any two owned places
```

### 2.6 Remove and insert

```text
// (a) back:       t = take_back(&*v.buf)                   requires (*v.buf).len > 0
// (b) unordered:  swap(&(*v.buf)[k], &(*v.buf).last)       no k != last-1 branch needed
//                 t = take_back(&*v.buf)                   this pair is Rule 6's `swap_remove`
// (c) ordered:    t = remove_at(&*v.buf, k)                one memmove, requires k < (*v.buf).len
// (d) insert:     insert_at(&*v.buf, k, move x)            one memmove, requires k <= len, room > 0
// (e) bulk:       append(&*dst.buf, &*src.buf)             one memcpy, requires dst.room >= src.len
```

Round one's `remove_ordered` — a loop of swap-with-the-last compositions costing
three moves per shifted element — is deleted. `remove_at` is the operation.

### 2.7 In-place sort

The comparator is a function-typed parameter with its own row (Rule 10:
"Function-typed parameters carry a full signature with its own row and
contract"). Sorting runs over a range reference, whose type is `&[T]` (Rule 7).

```text
fn sort<T>(part: &[T], less: fn(a: &T, b: &T) -> Bool  reads(a), reads(b))   writes(part)
    contract { ensures part.len == entry(part).len; }
{
    if part.len < 2 { return }
    p = partition(part, less)                        // ensures p < part.len
    sort(&part[0..p], less)
    sort(&part[p + 1 .. part.len], less)             // the two calls may overlap: disjoint ranges
}

fn partition<T>(part: &[T], less: fn(a: &T, b: &T) -> Bool  reads(a), reads(b)) -> u64   writes(part)
    contract { requires part.len >= 2; ensures result < part.len; }
{
    hi = part.len - 1
    i  = 0
    j  = 0
    while j < hi {
        invariant s1: i <= j;
        invariant s2: j <= hi;
        if less(&part[j], &part[hi]) {               // read/read on possibly the same place: allowed
            swap(&part[i], &part[j])                 // i == j on most iterations: "then nothing happens"
            i = i + 1
        }
        j = j + 1
    }
    swap(&part[i], &part[hi])
    i
}
```

The short-run finish is where the cost of this task now lives. A C++ or Rust
insertion sort lifts one element into a hole and shifts the run with one
`memmove`; there is no hole here (Rule 6: "A move out of a window slot or an
array element is rejected"), so the run is walked with adjacent swaps:

```text
// insertion of the element at i into the sorted prefix
j = i
while j > 0 {
    invariant s3: j <= i;
    if less(&part[j], &part[j - 1]) { swap(&part[j], &part[j - 1]);  j = j - 1 }
    else { break }
}
```

When the container is kept sorted as it is built, the element is in hand before
it enters the window and the cost disappears, because `insert_at` is the shift:

```text
k = lower_bound(&*v.buf, &x, less)                   // reads only; ensures k <= (*v.buf).len
reserve(&v, 1)?
insert_at(&*v.buf, k, move x)                        // one memmove, exactly C++ vector::insert
```

### 2.8 Hand-over to another owner

```text
// (a) to a function
e = remove_at(&*v.buf, k)
consume(move e)

// (b) to another container, pre-reserved: after the reservation nothing can fail
reserve(&dst, 1)?                                    // the one fallible step, before anything moves
e = remove_at(&*src.buf, k)
place_back(&*dst.buf, move e)                        // requires room > 0: from reserve's ensures

// (c) to another container, unreserved: the element comes back and goes home, in order
e = remove_at(&*src.buf, k)                          // src.len == entry - 1, so src.room > 0
match push(&dst, move e) {
    Ok         => { }
    Full(back) => { insert_at(&*src.buf, k, move back) }   // requires k <= src.len, src.room > 0
}
```

### 2.9 Discharging the container itself

```text
// affine elements: nothing to write. Rule 6: "at scope exit the compiler releases
// the slots inside the window recursively and frees the block"

fn drain_close(v: &Vector<Conn>)   writes(*v.buf)
    contract { ensures (*v.buf).len == 0; }
{
    while (*v.buf).len > 0 {
        c = take_back(&*v.buf)                       // own Conn, a whole local
        let Conn { f, .. } = move c                  // Rule 6's destructuring form
        close(move f)
    }
}

fn drain_close_boxed(v: &Vector<Box<File>>)   writes(*v.buf)
    contract { ensures (*v.buf).len == 0; }
{
    while (*v.buf).len > 0 { b = take_back(&*v.buf);  f = move *b;  close(move f) }   // unbox
}
```

---

## 3. Rule-by-rule trace at the interesting points

### 3.1 The container and the failure enum are well formed

- Rule 1: "Every struct, enum, tuple, `Array`, `Slots`, `Ring`, `Box`, and
  generic instantiation holds only owned values." `Vector<T> { buf: Box<Slots<T>> }`
  and `Pushed<T> { Ok, Full(own T) }` qualify.
- Rule 5: the runtime-capacity shapes "may appear only as the content of a Box:
  never inline in another value and never as a local variable." `Vector` stores
  `Box<Slots<T>>`, not `Slots<T>`. Accepted.
- Rule 8: linearity "propagates through every aggregate of Rule 1: a struct,
  enum, tuple, Array, Slots, Ring, or Box containing a linear part is linear."
  `Vector<Conn>` is linear through `Box<Slots<Conn>>`, and — this is the sentence
  round one could not find — `Pushed<Conn>` and `Result<Box<Conn>, (Oom, Conn)>`
  are linear because the list now names enum and tuple. A caller cannot discard
  the failure arm and lose the resource without a diagnostic. Round-one gap G3 is
  closed.

### 3.2 `push_here`'s row, and why it is the interesting one

- Rule 9: "The rows of the built-in operations of Rule 6 use only this vocabulary
  plus the window parts of Rule 6, which any user function may use too." So
  `writes((*v.buf).next), writes((*v.buf).len)` is a legal user row: the path
  starts at the reference parameter `v`, continues through the field `buf`,
  through `*`, and ends at a window part.
- Rule 9's body check: the single callee `place_back(&*v.buf, move x)` substitutes
  to exactly those two paths, and its `requires r.room > 0` is discharged by
  `push_here`'s own `requires`. Covered.
- Rule 10 clause 3, at a call site with a live reference:

```text
p = &(*v.buf)[i]              // formed under i < (*v.buf).len
push_here(&v, move x)         // writes (*v.buf).next, (*v.buf).len
use(p)                        // ACCEPTED
```

  Rule 6: "a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or
  `r.free`", and `r.len` is a measure member, not a prefix of `r[i]`. Rule 6
  states the conclusion outright for the identical user function `add_node`:
  "p survives: `(*g.nodes)[i]` with `i < len` never overlaps `.next`". The bound
  is re-proved for free by Rule 11's entry clause plus
  `ensures r.len == entry(r).len + 1`.
- With the general `push`, whose row is `writes(v.buf)` because `grow` (or the
  `swap` in `regrow`) writes the Box handle, `p` dies: `v.buf` is a proper prefix
  of `(*v.buf)[i]`. The rewrite is Rule 4's index, one scaled add.

This is the shape a front end wants — reserve once, then append while holding a
reference to the element being filled — and revision 4 admits it. Safe Rust
refuses it outright (`&mut Vec` is exclusive); C++ admits it only while capacity
lasts, with no diagnostic when it does not.

### 3.3 The join after the growth branch

`place_back` requires `r.room > 0`. Rule 12: "At a join, facts are intersected (a
fact survives only if it holds on every incoming edge)."

- Else edge: the branch condition `(*v.buf).room == 0` was false, so
  `(*v.buf).room > 0` by Rule 11's "refinement facts from a dominating branch"
  over an affine comparison on a measure.
- Then edge, `Ok` arm: `reserve`'s `ensures when Ok: (*v.buf).room >= extra` with
  `extra == 1` gives the same fact. Rule 11: "Contracts use `requires`, `ensures`,
  and `ensures when Variant:` for result-routed relations."
- Then edge, `Err` arm: returns; contributes no edge.
- Intersection: `(*v.buf).room > 0` on both surviving edges. No runtime test
  beyond the one capacity branch the C++ and Rust forms also emit.

Inside `reserve`, `grow`'s `requires cap >= (*b).cap` is discharged on both
branches of the `want` computation by affine arithmetic over measures: on the
first, `want == 2 * cap + 1 > cap`; on the second, the failed test
`(*v.buf).room >= extra` gives `cap - len < extra`, hence
`want == len + extra > cap`, using Rule 6's definition of `room` as `cap - len`.

### 3.4 The allocation-failure edge keeps every resource accounted for

Every failure point in §2, enumerated:

1. **`grow` inside `reserve` fails.** `x` has not been consumed: the only
   consumer downstream is `place_back`, not reached. `return Full(move x)`
   consumes it exactly once. Whether `reserve`'s `ensures when Err` can be
   stated at all is gap **G-A**: `grow`'s contract in the rule file has no
   failure arm.
2. **`Box::new` inside `regrow` fails.** Nothing has moved; `append` has not run;
   `?` propagates. `v` is untouched, and Rule 14 gives allocation no effect entry,
   so no fact about `v` is invalidated. The `Err` payload of a window constructor
   is gap **G-B**, which carries no resource.
3. **`append` mid-flight.** It cannot fail: it allocates nothing. It is one
   memcpy, and Rule 6's "No slot ever carries a tag, and no program point can
   observe a slot inside the window as empty" plus its `ensures src.len == 0`
   mean there is no intermediate state in which an element has two owners or
   none.
4. **Replacing the old block.** For an affine element type, `v.buf = move nb`
   releases the old, now-empty block. For a linear element type that assignment is
   rejected (Rule 6's assignment clause) and `swap(&v.buf, &nb)` is used instead;
   `swap` releases nothing. Round one's gap G4 is therefore half decided (the
   assignment is decided — it is rejected) and half a text note (N1, §8).
5. **Hand-over without pre-reservation (§2.8c).** `insert_at`'s `requires k <= r.len,
   r.room > 0` follows from `remove_at`'s `ensures r.len == entry(r).len - 1` and
   the pre-fact `k < entry(r).len`. The element is never ownerless and never has
   two owners, and the container's order is restored exactly.
6. **`Box::new` of a linear payload.** Rule 5: "A fallible allocation that takes a
   by-value payload hands it back on failure, so a linear payload is never lost",
   and the example type is `Result<Box<Node>, (Oom, Node)>`. Round-one gap G2 is
   closed, and §2.4's `Err((_, back))` arm is the derivation.

No runtime flag, sentinel, tag or drop flag appears anywhere above, on any edge.

### 3.5 Getting a linear resource out of an element

Round one's gap G5 — how a linear aggregate is taken apart — is closed by Rule 6:
"A move out of a field or out of Box content consumes the whole owner: the owner
ceases to exist, its other affine parts are released, and a remaining linear part
rejects the move (take it in the same destructuring)", with the forms
`x = move c.f`, `let Conn { f, g, .. } = move c`, and `n = move *b`.

- `let Conn { f, .. } = move c`: `c` is a whole local; `id` is Copy and needs no
  consumer; `f` is bound and closed. Accepted.
- `f = move *b` for `b: Box<File>`: unbox, then `close(move f)`. Accepted.
- A `Conn2 { in_f: File, out_f: File }` would reject `move c.in_f` under the
  same sentence's clause "a remaining linear part rejects the move", and is
  written `let Conn2 { in_f, out_f } = move c`.

`Vector<Conn>` and `Vector<Box<File>>` are therefore constructible and
dischargeable, which they were not under revision 2.

### 3.6 References across the mutating operations

```text
p = &(*v.buf)[i]                       // under i < (*v.buf).len
place_back(&*v.buf, move x);   use(p)  // accepted: .next and .len, §3.2
append(&*v.buf, &*other.buf);  use(p)  // accepted: writes .free and .len on this side;
                                       // "never overlaps r.next or r.free"
take_back(&*v.buf);            use(p)  // accepted only with i != entry.len - 1 proved:
                                       // "overlaps r.last unless i != r.len - 1 is proved"
remove_at(&*v.buf, k);         use(p)  // dies: writes r.filled, which "always overlaps" a live r[i]
insert_at(&*v.buf, k, move x); use(p)  // dies, same clause
grow(&v.buf, cap);             use(p)  // dies: writes(*b), a proper prefix
```

The `r.filled` lines are the conservative reading of Rule 3 against Rule 6's
overlap sentence; see text note **N2**. Under it the rewrite is one re-formation
from the retained index, whose bound follows affinely (`i < k` and
`r.len == entry(r).len - 1` give `i < r.len`): one scaled add, no load, no
branch, where C++ keeps a register and is silently wrong if the memmove crossed
the element.

### 3.7 The empty linear block at the end of `regrow`

After `append`, the old window has `len == 0`; after `swap`, it is the local `nb`,
which reaches its scope end. Rule 6, Release: "at scope exit the compiler releases
the slots inside the window recursively and frees the block" — with no slots
inside the window, there is nothing to release — and "No operation releases a
linear element: a storage whose element type is linear is itself linear (Rule 8)
and the program must take every element out and consume it", which `append`
satisfied by moving every element into the new window, where the obligation
continues. The two sentences do not collide on an empty window. The residual
wording risk is note **N1**; no runtime cost rides on either reading.

### 3.8 The atomic update, and what its overlap clause refuses

Rule 6: "`place = f(place, args...)` ... f is total and returns the place's type;
f's row must not overlap any prefix of place".

- §2.5b is admitted: `swap_in`'s only reference parameters are none; its
  arguments are by value, so its row is empty and overlaps nothing.
- Merging element `j` into element `k` is **not** written as an atomic update
  that reads the container, because such an `f` would declare `reads(*v.buf)`,
  which overlaps `*v.buf`, a prefix of `(*v.buf)[k]`. The admitted form takes the
  other element out first:

```text
a = remove_at(&*v.buf, j)                         // one memmove; own Conn in hand
(*v.buf)[k2] = merge((*v.buf)[k2], move a)        // k2 = k adjusted for the removal
```

  Rust writes `let a = v.remove(j); v[k2].merge(a);` — the same two steps and the
  same memmove.
- Mutating two elements at once is not an atomic update at all and is admitted
  directly: `combine(&(*v.buf)[i], &(*v.buf)[j])` with a row
  `writes(a), writes(b)` is accepted by Rule 10 clause 1 given the fact `i != j`
  ("accepted only with the fact `i != j`"). Rust needs `split_at_mut` or
  `get_many_mut` for the same program.

### 3.9 Bounds inside the sort

Rule 7: "Every index must be proved in bounds." In `partition`, `hi = part.len - 1`
needs `part.len >= 2` (the caller's test, carried in `requires`); `j < hi` and the
invariant `i <= j` give `i < part.len` and `j < part.len`. `swap(&part[i], &part[j])`
needs no `i != j` fact at all: Rule 10 clause 1, "The built-in `swap` is the one
operation whose two arguments may be the same place", and Rule 6, "p and q may be
the same place, then nothing happens". `less(&part[j], &part[hi])` is read/read,
which "is allowed" even when `j == hi` is not excluded. So the partition loop
emits no disjointness branch — the same instruction stream as the C++ loop.

The subrange calls are legal by Rule 7's range rule, `&x[lo..hi]` "requires
`lo <= hi <= r.len`": `p < part.len` from `partition`'s `ensures`, so
`0 <= p <= part.len` and `p + 1 <= part.len`.

---

## 4. Overlaps the rules admit and refuse

Revision 4 removed the `par` statement, so each item is a pair of adjacent
statements with the judgment written out. Rule 13: "Two adjacent statements of one
block may overlap when the first's write paths are disjoint from the second's read
and write paths and vice versa."

### 4.1 Admitted

```text
// (a) element-wise work over disjoint halves; the elements are Boxes
kernel(&(*v.buf)[0..mid]);  kernel(&(*v.buf)[mid..n])            // may overlap: disjoint ranges

// (b) the two recursive sort calls
sort(&part[0..p], less);  sort(&part[p + 1 .. part.len], less)   // may overlap: disjoint ranges

// (c) two readers of the whole container
s1 = scan(&*v.buf);  s2 = scan(&*v.buf)                          // may overlap: read/read

// (d) appending while a settled prefix is being processed  — NEW in revision 4
place_back(&*v.buf, move x);  kernel(&(*v.buf)[0..n])            // may overlap, n <= entry len:
                                 // writes are .next and .len; "a live r[i] ... never overlaps
                                 // r.next or r.free", and the range's own len was snapshotted
                                 // at formation (Rule 2)

// (e) two allocations
a = Box::new(Node { .. })?;  b = Box::new(Node { .. })?          // may overlap: Rule 14, allocation
                                                                 // carries no effect entry

// (f) processing a handed-over element beside the container it came from
e = remove_at(&*v.buf, k);
process(&e);  scan(&*v.buf)                                      // may overlap: different roots

// (g) draining two containers of linear resources
drain_close(&va);  drain_close(&vb)                              // may overlap: different roots

// (h) two elements at proved-distinct indices
touch(&(*v.buf)[i]);  touch(&(*v.buf)[j])                        // may overlap given i != j
```

Item (d) is the one the baselines cannot express: Rust refuses a shared borrow
across `push` at compile time, and C++ permits it with iterator invalidation as
undefined behaviour. Here the permission follows from the window-part definitions
rather than from a writer's assertion.

### 4.2 Refused

```text
place_back(&*v.buf, move x);  place_back(&*v.buf, move y)   // both write .next and .len
place_back(&*v.buf, move x);  s = scan(&*v.buf)             // writes .len meets reads *v.buf
remove_at(&*v.buf, i);  remove_at(&*v.buf, j)               // refused even with i != j: both write
                                                            // r.filled and r.len
append(&*a.buf, &*s1.buf);  append(&*a.buf, &*s2.buf)       // both write a.free and a.len
// splitting one relocation across two workers: there is no range-to-range bulk move for
// non-Copy elements (a move out of a slot is rejected), so a parallel memcpy is not expressible
// scatter by a data-determined destination index: two place_back calls on buckets d1, d2 with
// d1 != d2 unprovable — already a recorded known cost of the candidate
```

Rust refuses every line of this list identically (`&mut Vec` is exclusive), so
nothing is lost against it. Against a hand-rolled parallel relocation, the fourth
and fifth items are real; neither `std::vector` nor `RawVec` performs one either.

---

## 5. Cost table against an idiomatic C++ / Rust implementation

Baselines: C++ `std::vector<std::unique_ptr<Node>>` / `std::vector<Conn>` with
move constructors and `std::sort`; Rust `Vec<Box<Node>>` / `Vec<Conn>` and
`slice::sort_unstable`. `n` is the length, `L` a shifted-run length,
`W = sizeof(T)`.

| Operation | Baseline | Candidate x1 r4 form | Extra element moves / copies | Extra loads / branches | Extra allocations | Lost parallelism |
|---|---|---|---|---|---|---|
| push, no growth | 1 move, 1 length store, 1 capacity branch | `push_here` / `place_back` after the same branch | 0 | +1 branch at the caller for the `Pushed` discriminant when the general `push` is used; 0 for `push_here` | 0 | none |
| push with growth, via `grow` | 1 `realloc` (may extend in place) + `memcpy` when it moves | `grow(&v.buf, want)?` | 0 | 0 | 0 | growth is serial in both |
| push with growth, `regrow` fallback (gap G-A) | as above | `Box::new` + `append` + `swap` | 0 on a moving `realloc`; **+n moves** whenever the baseline's `realloc` extends in place | 0 | 0 (one block, as the baseline) | relocation cannot be split in either |
| `reserve` | `try_reserve` | §2.3 | as the growth rows | +1 branch on the `Result` | 0 | none |
| shrink to fit | `realloc` down, usually in place, 0 copies | `regrow` to a smaller `want` | **+n moves** (one memcpy where the baseline may copy nothing) | 0 | 0 | none |
| replace, affine, old discarded | 1 move + recursive free | `set` | 0 | 0 | 0 | none |
| replace, linear, old consumed in place | 2 moves | atomic update (§2.5b) | 0 | 0 | 0 | none |
| replace, old exported to the caller | `mem::replace` = 2 moves | `replace` (§2.5c) | 0 | 0 | 0 | none |
| exchange two places | `mem::swap` = 3 moves | `swap` | 0 | 0 | 0 | none |
| remove at the back | 1 move | `take_back` | 0 | 0 | 0 | none |
| remove unordered | 2 moves | `swap` + `take_back` | +1 move (3 vs 2) unless the swap-with-a-dead-slot is lowered as an exchange | 0 (no `k != len-1` branch) | 0 | none |
| remove ordered | 1 move + vectorized `memmove` of `L*W` | `remove_at` — "one memmove" | 0 | 0 | 0 | none |
| insert ordered | 1 move + `memmove` | `insert_at` — "one memmove" | 0 | 0 | 0 | none |
| bulk transfer between containers | `Vec::append` = 1 `memcpy` | `append` — "one memcpy" | 0 | 0 | 0 | same |
| in-place sort, partition phase | swap-based partition | §2.7 `partition` | 0 | 0 (no `i != j` branch: `swap` may alias) | 0 | none; the two recursive calls may overlap without an assertion |
| in-place sort, short-run finish | hole + 1 `memmove` per run: `L` element copies + 2 | adjacent `swap` chain | **+2L element copies** (3L vs L+2), and the vectorized shift is lost: 2 loads + 2 stores per step, scalar and serialized | 0 | 0 | none |
| keep-sorted insertion | `vector::insert` after `lower_bound` | `lower_bound` + `insert_at` | 0 | 0 | 0 | none |
| hand-over, pre-reserved | 2 moves + 1 `memmove` | §2.8b | 0 | +1 branch on `reserve`'s `Result` | 0 | none |
| hand-over, unreserved with rollback | C++ has no rollback; Rust aborts | §2.8c | 0 on success; +1 `memmove` on the failure edge only | +1 branch on the discriminant | 0 | none |
| reference held across an append | C++ keeps the pointer while capacity lasts (UB otherwise); Rust refuses the program | `push_here`'s part-precise row keeps it | 0 | 0 | 0 | **gained**: §4.1(d) overlaps an append with a read of the prefix |
| reference held across `remove_at` / `insert_at` | C++ keeps a register (wrong if the memmove crossed it) | re-form from the index (N2) | 0 | +1 scaled add per re-formation, no load, no branch | 0 | none |
| reference held across growth | C++ `realloc` invalidates too | re-form from the index | 0 | +1 base load + 1 scaled add on the reallocation path only | 0 | none |
| two elements mutated at once | `split_at_mut` / `get_many_mut` | `combine(&r[i], &r[j])` with `i != j` | 0 | 0 with a static fact; +1 compare when the indices are data-determined | 0 | none |
| drop the container, affine | recursive `Drop` | compiler release at scope exit | 0 | 0 | 0 | none |
| drop the container, linear | `Drop` calls `close` n times | `drain_close` | 0 | +n length loads/stores that `Drop`'s own walk also performs | 0 | none |
| allocation failure | C++ throws; `Vec::push` aborts | `Pushed<T>` / `Result` on every fallible step | 0 | +1 branch per fallible operation | 0 | none |

Two rows run the other way and are worth stating explicitly:

- **Allocation failure.** Against `Vec::try_reserve` or a C++ build with
  exceptions disabled, the branch count is the same, and the candidate
  additionally *proves* that every arm accounts for every resource, with no drop
  flag and no sentinel.
- **Aliasing information.** §4.1(a) and (b) need no `noalias` assertion and no
  `split_at_mut`; disjointness follows from Rule 1 and Rule 5. Same instructions,
  better information.

---

## 6. Strongest counterexample against the candidate for this task

Round one's counterexample — an ordered table of linear resources with
mid-sequence removal, 3× scalar moves per shifted element and no vectorization —
is **gone**: `remove_at` is one memmove and `insert_at` is one memmove. The
strongest construction now available is the pair below.

**(1) A very large container grown by pushes, if `grow`'s failure arm is not
usable.** Take `Vector<Conn>`, `W = 16`, grown to 64 M elements (1 GB) by
`push`. The baselines call `realloc`; for blocks of this size both glibc and the
Rust system allocator go to `mremap`, which extends the mapping and copies **zero
bytes** for most doublings. The `regrow` fallback of §2.3 must allocate a new
block and `append`, copying the full 1 GB at each of the last few doublings —
about 2 GB of memcpy traffic over the run against approximately none. This is not
a rewrite cost that better code removes: `append` requires a destination window
that already exists, and no admitted operation extends a block in place. Only
`grow`, "may reallocate in place", does, and whether `grow` can be used on a path
that must survive allocation failure is exactly gap **G-A**. If G-A resolves as
`grow(&b, cap) -> Result<Unit, Oom>` with an `ensures when Err` that leaves the
window intact, this counterexample disappears entirely and growth is at parity.

**(2) In-place sort of non-Copy elements.** `std::sort` and `sort_unstable` are
pdqsort: a swap-based partition (parity here) plus an insertion-sort finish on
runs of about 20 that lifts one element into a hole and shifts the rest with a
`memmove`. The hole is unrepresentable — Rule 6, "A move out of a window slot or
an array element is rejected", and Rule 12, "No place is ever partially moved" —
so the finish is a chain of adjacent `swap`s: 2 loads and 2 stores per displaced
element against 1 load and 1 store, scalar where the baseline vectorizes, and
each step dependent on the last. For 16-byte elements the finish phase is roughly
a fifth of pdqsort's time, so the honest prediction is a 10-20 % slower sort, not
a 4-8× gap as in round one.

The rewrite that removes even this changes the algorithm rather than the code:
sort an index array of `u64` (Copy, so shifting is ordinary duplication and
vectorizes) and then permute the resources once with `swap`s, at n extra loads,
n extra stores and one extra block. That is the pattern Rule 16 licenses. It wins
when `W` is large and loses when `W` is small, which is the same tradeoff a C++
programmer faces.

**(3) The counterexample the candidate survives, restated because it got
stronger.** There is no arrangement of allocation failures in §2 that loses or
duplicates a resource, and none uses a runtime flag, tag or sentinel: Rule 12's
"No place is ever partially moved", Rule 6's "no program point can observe a slot
inside the window as empty", Rule 5's payload return and Rule 8's propagation of
linearity through enums and tuples together make the state space in which a leak
could hide non-existent. Revision 4 additionally makes the *rollback* of a failed
hand-over order-preserving and constant-cost (§2.8c), which neither baseline
offers at all.

---

## 7. Rules consistent, and task achievable — recorded separately

**Rules consistent for this task: yes, with the two gaps and two text notes in
§8.** Nothing in §2 is admitted by one rule and refused by another. The three
places where round one found rules pulling against each other are all decided
now: a linear payload on a failed allocation (Rule 5's payload sentence), a
linear part inside an aggregate element (Rule 6's consuming-move paragraph), and
linearity through an enum (Rule 8's aggregate list). The one sentence pair that
still has to be read rather than applied — emptying versus freeing a linear
window — has no runtime consequence either way.

**Task achievable: yes, with cost, and the cost is now small and confined.**
Every operation the task names is expressible: push (zero cost), growth (zero
cost through `grow`; one memcpy per doubling and the loss of in-place extension
through the fallback), replace in all three shapes (zero cost), ordered and
unordered removal and insertion (zero cost), bulk transfer (zero cost), in-place
sort (zero cost in the partition, +2 element copies per displaced element in the
short-run finish), hand-over (zero cost, with a constant-cost order-preserving
rollback), and the failure paths (zero cost, with stronger evidence than either
baseline provides). One capability is *gained* over both baselines: a reference
into the container survives an append whose row is part-precise, and an append
may overlap a read of the settled prefix.

**Per-operation verdicts:**

| Operation | Verdict | Round one |
|---|---|---|
| Container and element typing, contracts, `Pushed<T>` linearity | `accepted-fine` | `accepted-fine` / gap G3 |
| push without growth | `accepted-fine` | `accepted-fine` |
| Growth through `grow` | `accepted-fine`, conditional on gap G-A | — (no `grow` existed) |
| Growth through the `regrow` fallback | `rejected-real-cost` (one memcpy per doubling; no in-place extension) | `rejected-real-cost` (2.5n moves) |
| Shrink to fit | `rejected-real-cost` (one memcpy where `realloc` may copy nothing) | not derived |
| Replace, old discarded or consumed in place | `accepted-fine` | `accepted-fine` |
| Replace, old exported to the caller | `accepted-fine` (`replace` is built in) | `undecided-rule-gap` (G1) |
| Remove at the back, unordered, ordered; insert; bulk append | `accepted-fine` | ordered was `rejected-real-cost` |
| In-place sort, partition phase | `accepted-fine` | not derived |
| In-place sort, short-run finish | `rejected-real-cost` (no hole; 3L vs L+2 copies, vectorization lost) | not derived |
| Hand-over, reserved and unreserved with rollback | `accepted-fine` | `accepted-fine` |
| Allocation-failure accounting, all four element types | `accepted-fine` | two sub-cases were gaps (G2, G5) |
| Discharging a linear container, including `Conn` and `Box<File>` | `accepted-fine` | `undecided-rule-gap` (G5) |
| Reference held across a part-precise append | `accepted-fine`, and better than both baselines | `rejected-real-cost` |
| Reference held across `insert_at` / `remove_at` / growth | `rejected-zero-cost` (one scaled add), text note N2 | `rejected-real-cost` |
| Overlapped range work, overlapped recursive sort, append beside a prefix read | `accepted-fine` | partly `accepted-fine` |
| Overlapped push, overlapped growth, scatter, producer/consumer | `rejected-zero-cost` against Rust; the pipeline needs a deferred mechanism | same |
| `grow` on a path that must survive allocation failure | `undecided-rule-gap` (G-A) | — |
| `Err` payload of a runtime-capacity window constructor | `undecided-rule-gap` (G-B) | — |

**Headline verdict: `rejected-real-cost`**, driven now by exactly two sub-cases —
the hole-free short-run shift inside an in-place sort, and the reallocation cases
(growth under the fallback, and shrink) where the baseline's `realloc` can extend
or shrink a mapping in place. Everything else the task asks for is
`accepted-fine`. Round one's headline was the same word for much larger reasons.

---

## 8. Rule gaps and text notes

Per the protocol, none is resolved here.

**G-A — does `grow` return a `Result`, and what does it guarantee on failure?**
Rule 6 states the whole of it as:

> `grow(&b, cap)              writes(*b)                                            // Box<Slots<T>> only; may reallocate in place`
> `    contract { requires cap >= (*b).cap; ensures (*b).cap == cap, (*b).len == entry(*b).len; }`

Rule 14 states:

> "Allocation returns a `Result` and never traps (Rule 5 for the payload)."

`grow` allocates, so the two cannot both be read literally: the signature has no
`Result`, and the `ensures` is unconditional. The candidate's own evaluation
criteria rule out the third possibility ("no runtime traps"), so the only
consistent reading is a fallible `grow` with a failure arm — but the text does not
state the failure arm, and this task's stated obligation is precisely that every
allocation-failure edge accounts for every resource. What is missing is one line:
whether it is `grow(&b, cap) -> Result<Unit, Oom>` with
`ensures when Err: (*b).cap == entry(*b).cap, (*b).len == entry(*b).len`. Nothing
in this task's success path costs anything either way; the failure path is
underivable as written, and §6(1) prices the fallback that avoids `grow`
altogether.

**G-B — what is the `Err` payload of a runtime-capacity window constructor?**
Rule 5 states:

> "A fallible allocation that takes a by-value payload hands it back on failure, so a linear payload is never lost."

Rule 6's constructor line is `Box::new(Slots::new<T>(cap))?`, and Rule 5 also
says the runtime-capacity shapes "may appear only as the content of a Box: never
inline in another value and never as a local variable". So the payload-returning
`Err((Oom, Slots<T>))` cannot be formed for this constructor, and the text does
not say what the `Err` type is instead. The gap is benign for this task — a fresh
empty window owns nothing — but `new_vector` and `regrow` both go through that
line, so the task cannot state its own failure type without an answer. No runtime
cost rides on it.

**N1 (text note, round one's G4, half decided) — emptying versus freeing a linear
window.** Rule 6, Release:

> "No operation releases a linear element: a storage whose element type is linear is itself linear (Rule 8) and the program must take every element out and consume it."

Read strictly as a type property, a `Box<Slots<Conn>>` stays linear when empty and
nothing discharges it. Read with the preceding sentence ("at scope exit the
compiler releases the slots inside the window recursively and frees the block"),
an empty window has no slots to release and the block free is not the release of a
linear element. This derivation adopts the second reading, which is also what
`drain_close` and the `swap` in `regrow` rely on. The *other* half of round one's
G4 is now decided rather than open: `v.buf = move nb` over a linear block is
rejected by Rule 6's "Assigning over any owned place releases the old value if it
is affine and is rejected if it is linear", and `swap` is the admitted form at the
same cost.

**N2 (text note) — is a written window *part* a proper prefix of a slot?** Rule 3
invalidates a reference when "a proper prefix of p's path is written" and does not
invalidate it when the write is "at p's path or below it (a content write)".
Rule 6 gives overlap, not prefixhood: "a live `r[i]` ... always overlaps
`r.filled`". Rule 3's own conservative clause ("two indexed positions on the same
storage are taken to overlap unless their indices or ranges are proved distinct")
and Rule 10 clause 3 ("under the same may-overlap judgment") point at the strict
reading, under which `insert_at`, `remove_at` and a source-side `append` kill every
element reference — which is also the memmove they perform, so it is the honest
one. This derivation adopts it. Under the permissive reading a reference would
survive a memmove and silently name a different element, which would be a
Rule 16-style logic error rather than a memory error, but Rule 16 speaks only of
stale *indices*. The cost of the strict reading is one scaled add per re-formation
(§5); the cost of the permissive one is a silent renaming with no diagnostic.

**Round-one gaps, disposition:**

| Round one | Status under revision 4 |
|---|---|
| G1, atomic update with further arguments | **Closed.** Rule 6: "`place = f(place, args...)`", with the overlap restriction traced in §3.8. The exporting replace no longer needs it: `replace(&r[k], x) -> own T` is built in. |
| G2, `Box::new` with a linear payload | **Closed.** Rule 5's payload-return sentence, `Result<Box<Node>, (Oom, Node)>`. |
| G3, linearity through an enum | **Closed.** Rule 8 now names "a struct, enum, tuple, Array, Slots, Ring, or Box". |
| G4, discharging a linear window at an overwrite | **Half decided, half N1.** The overwrite is rejected; the emptied-block question is a wording residue with no runtime cost. |
| G5, taking a linear aggregate apart | **Closed.** Rule 6's consuming-move paragraph and `let Conn { f, g, .. } = move c`. |

---

## 9. What changed from round one

1. **Growth stopped being the headline cost.** Round one had to relocate the
   window element by element (`swap_remove` + `push_nogrow`, 2.5n moves
   order-preserving, 2n as a bag, no vectorization). Revision 4 supplies `grow`
   ("may reallocate in place") and `append` ("one memcpy"), so growth is at
   parity with `realloc` or costs exactly one memcpy per doubling. Round one's
   `move_all` and `move_all_bag` are deleted from this task.
2. **Ordered removal stopped being a cost.** `remove_at(&r, k) -> own T` is "one
   memmove". Round one's `remove_ordered` rotation, 3 moves per shifted element
   with the vectorized `memmove` lost outright, is deleted, and with it the
   round-one counterexample built on it (the 4-8× gap on an ordered connection
   table). `insert_at` does the same for insertion, which round one could not
   express at all.
3. **Exporting the old value of a replaced slot became free.** `replace(&r[k], x)
   -> own T` is built in, so round one's 3-move (bag) and 6-move (ordered)
   rewrites, and its gap G1 about smuggling the old value out of an atomic
   update, are both gone.
4. **Linear aggregates became constructible.** Round one could not derive
   `Vector<Conn>` or `Vector<Box<File>>` at all (gap G5). Rule 6's consuming move
   out of a field or Box content, with the "remaining linear part rejects the
   move" clause, supplies the operation; §2.9 discharges both containers.
5. **Failure accounting became fully derivable.** Rule 5's payload return closes
   G2, and Rule 8's aggregate list closes G3, so `Pushed<Conn>` and
   `Err((Oom, Conn))` are linear and cannot be dropped silently. §3.4 no longer
   assumes anything.
6. **A reference into the container now survives an append.** This is the largest
   qualitative change. Round one priced "one base load plus one index computation
   per iteration" for every append-and-use loop, because `push_nogrow`'s row was
   `writes(buf)`. With window parts, `push_here` declares
   `writes((*v.buf).next), writes((*v.buf).len)`, and Rule 6 states the
   conclusion. The cost is zero and it beats both baselines.
7. **`par` is gone.** §4 is rewritten as adjacent statements with an explicit
   may-overlap judgment, and it gained item (d) — an append that may overlap a
   read of the settled prefix — which is not expressible in either baseline.
8. **Measures are members.** Every `len_of(x)` / `cap_of(x)` in round one is now
   `x.len` / `x.cap`, and `deref(entry(v)).buf` is now `entry(*v.buf)`. Mechanical,
   but it is what makes the part-precise rows of item 6 readable as ordinary paths.
9. **In-place sort was added to the task and is the new residual cost.** Round
   one did not cover it. The partition is at parity (thanks to `swap`'s aliasing
   permission, which removes the `i != j` branch), and the short-run finish costs
   2 loads and 2 stores per displaced element against 1 and 1, because the
   baselines' hole is unrepresentable.
10. **Two new gaps, both about allocation failure rather than about ownership.**
    G-A (`grow`'s failure arm) and G-B (the `Err` payload of a window
    constructor). They sit exactly on this task's stated obligation, which is why
    they are reported here even though neither costs anything on a success path.
