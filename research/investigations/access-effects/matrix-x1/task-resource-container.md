# Engineering task under candidate x1 — dynamic resource container

Derived against the frozen rule set in `CANDIDATE-X1.md` (2026-09-18) and only that
file. Engineering-task framing and the discriminating programs are quoted from
`PROGRAMS.md` where they are touched (row "Growable container of resources", P1,
P4, P7, P8, P9, P18). Nothing under "Not in this candidate", "Deferred", or the
second "Pending owner ruling" item is used; Rule 16 is used only where a section
says so and the dependence is marked at the end.

Task text: *Dynamic resource container: a Vector of non-Copy elements (Box and
linear payloads) with push, growth, replace, remove, and hand-over of an element
to another owner; every failure path (allocation failure) must keep every
resource accounted for.*

Cost convention from the protocol, item 7: runtime performance is the only cost.
Verbosity, extra parameters and duplicated code are not costs. One **move** means
one relocation of `sizeof(T)` bytes; Rule 1's stated consequence — "any value can
be relocated by copying its bytes (memmove, realloc), because nothing inside it
points anywhere" — makes every move a plain byte copy with no fixup, so move
counts are directly comparable against C++ and Rust move constructors/`memcpy`.

---

## 1. The question this task tests

Can a growable container hold values that must not be duplicated and must not be
silently dropped — `Box<Node>` (affine) and `File`/`Conn` (linear) — and support
push, growth, replace, ordered and unordered removal, and transfer of one element
to another owner, such that every allocation-failure edge leaves exactly one
owner for every resource, under Rule 6's closed set of move-out operations and
Rule 8's rule that the compiler never releases a linear value?

---

## 2. Programs

### 2.1 Element types

```text
linear type File;                        // Rule 8: "Linearity is declared on external-resource types"
fn close(f: own File)

struct Node { key: u64, bytes: DynBox<u8> }   // affine: DynBox of Copy elements
struct Conn { f: File, id: u64 }              // linear by containment (Rule 8)

// The four element types this task must carry:
//   Box<Node>   affine, non-Copy, 8 bytes in the slot, one heap object behind it
//   File        linear, non-Copy, opaque width W
//   Conn        linear by containment, 16 bytes in the slot
//   Box<File>   linear behind a Box; Rule 8 says "Box<File> // linear: must be taken
//               apart and its File closed"
```

### 2.2 The container

`Vector<T>` is library code; Rule 6 states it is: "`Vector<T>` is library code: a
DynBox plus a `push` that, when `len_of == cap_of`, allocates a larger DynBox,
moves `[0, len_of)` across, and replaces the old one."

```text
struct Vector<T> { buf: DynBox<T> }          // Rule 1: owned field, no reference inside

fn new_vector<T>(c: u64) -> Result<Vector<T>, Oom>
    contract { ensures when Ok: len_of(result.buf) == 0;
               ensures when Ok: cap_of(result.buf) == c; }
{
    b = DynBox::new<T>(c)?                   // Rule 14: "Allocation returns a Result and never traps"
    Ok(Vector { buf: move b })               // Rule 15: "A function returns owned values only"
}

fn count<T>(v: &Vector<T>) -> u64   reads(v.buf)
    contract { ensures result == len_of(v.buf); }
{ len_of(v.buf) }
```

### 2.3 The strongest program: push of a linear element with a failing growth

This is the case where no static analysis can help. The element is linear, so the
compiler may not release it (Rule 8); the growth allocation is data-determined and
may fail; and on the failure edge the element is in flight — owned by neither the
caller nor the container.

```text
enum Pushed<T> { Ok, Full(own T) }           // Rule 1: an enum payload that is an owned value
                                             // Rule 8: linear T makes Pushed<T> linear (see gap G3)

fn push<T>(v: &Vector<T>, x: own T) -> Pushed<T>    writes(v.buf)
    contract {
        ensures when Ok:   len_of(v.buf) == len_of(deref(entry(v)).buf) + 1;
        ensures when Full: len_of(v.buf) == len_of(deref(entry(v)).buf);
        ensures when Full: cap_of(v.buf) == cap_of(deref(entry(v)).buf);
    }
{
    if len_of(v.buf) == cap_of(v.buf) {
        match DynBox::new<T>(2 * cap_of(v.buf) + 1) {
            Ok(nb)  => { move_all(&v.buf, &nb);          // cannot fail, allocates nothing
                         v.buf = move nb }               // Rule 6's "replaces the old one"
            Err(e)  => { return Full(move x) }           // x is still an owned local here
        }
    }
    push_nogrow(&v.buf, move x)
    Ok
}
```

The order-preserving relocation, written with only the operations Rule 6 lists:

```text
fn move_all<T>(old: &DynBox<T>, nb: &DynBox<T>)   writes(old), writes(nb)
    contract {
        requires cap_of(nb) >= len_of(old);
        requires len_of(nb) == 0;
        ensures  len_of(old) == 0;
        ensures  len_of(nb) == len_of(deref(entry(old)));       // needed by push's join, see 3.3
    }
{
    n = len_of(old)
    i = 0
    while i + i + 1 < n {
        invariant a: len_of(old) == n - i;
        invariant b: len_of(nb) == i;
        t = swap_remove(old, i)              // yields e_i; refills slot i from the last slot
        push_nogrow(nb, move t)
        i = i + 1
    }
    while len_of(old) > 0 {
        invariant c: len_of(nb) + len_of(old) == n;
        t = pop(old)                         // the displaced tail comes off in ascending order
        push_nogrow(nb, move t)
    }
}

// n = 6, elements e0..e5:
//   sr(0) -> e0, old = [e5,e1,e2,e3,e4]
//   sr(1) -> e1, old = [e5,e4,e2,e3]
//   sr(2) -> e2, old = [e5,e4,e3]      (2i+1 = 5 < 6 was the last admitted step)
//   pop   -> e3, e4, e5
//   nb = [e0,e1,e2,e3,e4,e5]           order preserved
```

When the container is a bag (order is not observable — a connection table, a free
list, a set of open handles) the schedule collapses to the cheaper reversing form:

```text
fn move_all_bag<T>(old: &DynBox<T>, nb: &DynBox<T>)   writes(old), writes(nb)
    contract { requires cap_of(nb) >= len_of(old); ensures len_of(old) == 0; }
{
    while len_of(old) > 0 { t = pop(old); push_nogrow(nb, move t) }   // reverses the order
}
```

### 2.4 Caller-side accounting of the failure edge

```text
// c: own Conn arrives from somewhere; the container is a Vector<Conn>
match push(&table, move c) {
    Ok         => { }                         // the container owns it now
    Full(back) => { close_conn(move back) }   // the caller must discharge it: Rule 8 requires an
                                              // explicit consumer on this exit path
}
```

For the affine element type the failure edge needs no writer action, because the
compiler releases affine values:

```text
match push(&nodes, move b) {                  // b: own Box<Node>
    Ok         => { }
    Full(back) => { }                         // Rule 8: "at scope exit the compiler releases their
                                              // memory recursively (Box, DynBox)". Accounted, no code.
}
```

### 2.5 Replace

```text
// (a) affine element, old value discarded
v.buf[k] = move fresh_box                     // Rule 6: "Assigning buf[k] = x where the old value is
                                              // affine releases the old value"; requires k < len_of

// (b) linear element, old value consumed in place — the atomic update
v.buf[k] = |old: own File| { close(move old); move fresh }
                                              // Rule 6: "the old value goes into f by value, f's
                                              // result is committed, no program point lies between"

// (c) linear element, old value must survive and reach the caller — not expressible as an
//     atomic update (see gap G1); the window operations are the only route:
old = swap_remove(&v.buf, k)                  // yields e_k, refills slot k from the last slot
push_nogrow(&v.buf, move fresh)               // requires len_of < cap_of: holds, len just dropped
// container order is now [.., e_{n-1} @ k, .., fresh @ n-1]

// (d) as (c) but order-preserving: one extra swap-with-the-last composition
t = swap_remove(&v.buf, k)                    // t = e_k, slot k holds e_{n-1}, len = n-1
push_nogrow(&v.buf, move fresh)               // fresh at n-1, len = n
u = swap_remove(&v.buf, k)                    // u = e_{n-1}, slot k holds fresh, len = n-1
push_nogrow(&v.buf, move u)                   // e_{n-1} back at n-1, len = n
// net: slot k = fresh, every other slot unchanged, t = the old element, 6 moves
```

### 2.6 Remove

```text
// (a) back:      t = pop(&v.buf)                     requires len_of > 0
// (b) unordered: t = swap_remove(&v.buf, k)          requires k < len_of
// (c) ordered:   t = remove_ordered(&v.buf, k)       below

fn remove_ordered<T>(buf: &DynBox<T>, k: u64) -> own T   writes(buf)
    contract { requires k < len_of(buf);
               ensures len_of(buf) == len_of(deref(entry(buf))) - 1; }
{
    n = len_of(buf)
    t = swap_remove(buf, k)          // t = e_k; slot k now holds e_{n-1}, len = n-1
    // slots [k, n-2) hold e_{k+1}..e_{n-2} and slot k holds e_{n-1}: the suffix must be
    // rotated left by one.  A cycle of length L = n-1-k is L-1 transpositions that all
    // share one position, and the only transposition Rule 6 admits is swap-with-the-last:
    j = k
    while j + 1 < n - 1 {
        invariant d: len_of(buf) == n - 1;
        u = swap_remove(buf, j)      // 2 moves
        push_nogrow(buf, move u)     // 1 move: this composition is exactly swap(j, len-1)
        j = j + 1
    }
    move t
}
```

### 2.7 Hand-over to another owner

```text
// (a) to a function (a worker, a sink) — parity with Rust
e = swap_remove(&v.buf, k)
consume(move e)

// (b) to another container, failure-safe by pre-reservation: the transfer cannot fail
reserve(&dst, 1)?                                  // one fallible step, before anything moves
e = swap_remove(&src.buf, k)
push_nogrow(&dst.buf, move e)                      // requires len_of < cap_of: from reserve

// (c) to another container without pre-reservation: the element comes back on failure
e = swap_remove(&src.buf, k)                       // src: len n -> n-1, so cap > len
match push(&dst, move e) {
    Ok         => { }
    Full(back) => { push_nogrow(&src.buf, move back) }   // always fits: src lost a slot above
}

fn reserve<T>(v: &Vector<T>, extra: u64) -> Result<Unit, Oom>   writes(v.buf)
    contract { ensures when Ok: cap_of(v.buf) >= len_of(v.buf) + extra;
               ensures when Ok: len_of(v.buf) == len_of(deref(entry(v)).buf); }
{
    if cap_of(v.buf) - len_of(v.buf) >= extra { return Ok }
    nb = DynBox::new<T>(len_of(v.buf) + extra)?    // on Err: nothing has moved, v is untouched
    move_all(&v.buf, &nb)
    v.buf = move nb
    Ok
}
```

### 2.8 Discharging the container itself

```text
// affine elements: nothing to write; Rule 6's scope exit releases slots [0, len_of) recursively
// linear elements: the program must empty it
fn drain_close(v: &Vector<File>)   writes(v.buf)
    contract { ensures len_of(v.buf) == 0; }
{
    while len_of(v.buf) > 0 { f = pop(&v.buf); close(move f) }
}
```

---

## 3. Rule-by-rule trace at the interesting points

### 3.1 The container type is well formed

- Rule 1: "Every struct, enum, tuple, array, slice-like value, Box, DynBox, and
  generic instantiation holds only owned values." `Vector<T> { buf: DynBox<T> }`
  and `Pushed<T> { Ok, Full(own T) }` hold owned values only. Accepted.
- Rule 8: "Linearity is declared on external-resource types and propagates through
  aggregates: a struct, array, Box, or DynBox containing a linear part is linear."
  `Vector<File>` is linear by its `DynBox<File>` field. Rule 6 repeats it: "A
  DynBox whose element type is linear is itself linear (Rule 8)".
- `Pushed<File>` is an **enum** containing a linear part. The propagation list
  names "a struct, array, Box, or DynBox" and does not name enum or tuple; Rule 1's
  aggregate list does. See gap **G3** — the whole failure-accounting story depends
  on `Pushed<File>` being linear.

### 3.2 `push`'s body against its own row

- Rule 9: "A function body is checked against its own row: every statement's effect
  and every callee's substituted row must be covered by the declared row." The
  callees are `DynBox::new` (Rule 14: "Allocation and release carry no effect
  entry"), `move_all` substituted to `writes(v.buf), writes(nb)`, the assignment
  `v.buf = move nb` (a write of `v.buf`), and `push_nogrow` substituted to
  `writes(v.buf)`. `nb` is a local, not a path from a reference parameter, so it
  contributes no entry to the row. Declared `writes(v.buf)` covers the rest.
  Rule 9 allows the member path: "fn update(o: &Obj, c: Bool)  writes(o.a), writes(o.b)
  // member paths are allowed."
- Rule 10 clause 1, inside `move_all(&v.buf, &nb)`: the substituted effects are
  `writes(v.buf)` and `writes(nb)`. Different roots, so they "must be proved
  disjoint (different roots, ...)" — satisfied by inspection. Accepted.
- Rule 10 clause 2: `push_nogrow(&v.buf, move x)` contributes the consumption of
  `x` to the pairwise comparison. `x` is a local, disjoint from `v.buf`. Accepted.

### 3.3 The join after the growth branch

`push_nogrow` requires `len_of(buf) < cap_of(buf)`. Rule 12: "At a join, facts are
intersected (a fact survives only if it holds on every incoming edge)."

- Else edge: the branch condition was `len_of(v.buf) == cap_of(v.buf)` and it was
  false, so `len_of(v.buf) < cap_of(v.buf)` by Rule 11's "refinement facts from a
  dominating branch" over an affine comparison.
- Then edge, `Ok(nb)` arm: `cap_of(nb) == 2 * cap_of(entry) + 1` from
  `DynBox::new`, `len_of(nb) == len_of(deref(entry(old)))` from `move_all`'s
  `ensures`, and the old `len_of == cap_of`. So
  `len_of(v.buf) = c < 2c + 1 = cap_of(v.buf)`. All three are affine comparisons
  over measures, which Rule 11 lists first among the fact forms.
- Then edge, `Err` arm: returns, contributing no edge to the join.
- Intersection: `len_of(v.buf) < cap_of(v.buf)` holds on both surviving edges.
  `push_nogrow`'s `requires` is discharged with **no runtime test beyond the one
  branch the C++/Rust form also has**.

Without `move_all`'s second `ensures` the then-edge fact is unavailable and the
program is rejected; the repair is a contract line, zero runtime cost.

### 3.4 The allocation-failure edge keeps every resource accounted for

This is the task's stated obligation. Enumerate every failure point in §2:

1. **`DynBox::new` inside `push` fails.** At that program point `x` is an owned
   local that has not been consumed: the only consumer downstream is
   `push_nogrow`, which is not reached. `v.buf` has not been written — Rule 11: a
   fact "is invalidated when that path is written"; Rule 14 gives allocation no
   effect entry, so the failed call writes nothing and invalidates nothing, and
   `cap_of(v.buf)`/`len_of(v.buf)` survive to justify the `ensures when Full`
   clauses. `return Full(move x)` consumes `x` exactly once on this path, and
   Rule 15 ("A function returns owned values only") admits the owned payload.
   **Every resource has exactly one owner on this edge, and the code that achieves
   it executes only on the edge.**
2. **`DynBox::new` inside `reserve` fails.** Same shape with no element in flight.
   `v` is untouched: `?` propagates `Err(Oom)` and Rule 8's affine release frees
   nothing because nothing was allocated.
3. **`move_all` mid-flight.** It cannot fail: it performs no allocation. Note the
   ordering that makes this true — the whole allocation happens *before* the first
   element moves. Even if the relocation could be interrupted, both `old` and `nb`
   are live DynBoxes and every element sits in exactly one window, because
   `swap_remove`/`pop` and `push_nogrow` are the only movers and each is a single
   extract-then-insert pair; Rule 6's "No slot ever carries a tag, and no program
   point can observe a slot inside the window as empty" means there is no
   intermediate state in which a slot holds a half-moved value.
4. **`v.buf = move nb` with a linear element type.** The overwritten value is the
   old `DynBox<File>`, which `move_all`'s `ensures len_of(old) == 0` proves empty.
   See gap **G4** on whether emptiness discharges a linear DynBox at an overwrite
   as it does at scope exit.
5. **Transfer without pre-reservation (§2.7c).** `push(&dst, move e)` returns
   `Full(back)`; `push_nogrow(&src.buf, move back)` requires
   `len_of(src.buf) < cap_of(src.buf)`, which follows from `swap_remove`'s
   `ensures len_of(buf) == len_of(deref(entry(buf))) - 1` and the pre-state
   `len_of <= cap_of`. The element is never ownerless and never has two owners.
6. **`Box::new` of a linear payload.** Rule 5's only statement of the signature is
   `b = Box::new(Node { ... })?   // allocation can fail: Result<Box<Node>, Oom>`.
   Read literally for `T = File`, the `Err` arm discards the `File` that went in by
   value, which Rule 8 forbids ("the compiler never releases them") and which
   silently loses a resource. See gap **G2**; the derivation below uses the
   payload-returning form `Result<Box<T>, (Oom, own T)>`, which costs nothing on
   the success path.

### 3.5 Replace of a linear element

Rule 6: "Assigning `buf[k] = x` where the old value is affine releases the old
value; where it is linear the assignment is rejected unless written as the atomic
update whose function consumes the old value."

- §2.5a (affine) is the first arm of that sentence. One move, and the released
  `Box<Node>` frees its heap object recursively — the same work as Rust's
  `v[k] = new`. Accepted, zero cost.
- §2.5b (linear, old consumed) is the second arm: `f` consumes `old` via `close`
  and returns `fresh`, which "is committed". Accepted, one move in, one move out
  into `f` — the same two moves as `*slot = new` after `drop(old)` in Rust.
- §2.5c/d (linear, old must reach the caller) is the interesting case. `f` returns
  exactly one value and that value "is committed" to the slot, so `f` cannot also
  export the old element as a result. The only other export route would be an
  effect of `f` — writing the old value into a second container. Rule 9 confines
  effect paths: "An effect row lists `reads(path)` and `writes(path)` where each
  path starts at a reference parameter." A value captured by reference into a
  function value is not a parameter, so the write cannot appear in any row, and
  Rule 9's coverage sentence then rejects `f`'s body. The remaining possibility is
  an extra reference **parameter** on `f` — but Rule 6 writes the update as
  `buf[k] = f(buf[k])` and says nothing about further arguments. See gap **G1**.
  Under the restrictive reading the rewrite is §2.5c (unordered, 3 moves) or
  §2.5d (ordered, 6 moves) against `std::mem::replace`'s 2 moves.

### 3.6 Removal, and why a left shift is not available

An ordered removal wants `buf[j] = buf[j+1]` for the suffix. For a Copy element
that is a duplication, admitted by Rule 8 ("Copy values may be duplicated") and
vectorizable. For `Box<Node>`, `File` or `Conn` it is a move out of the place
`buf[j+1]`, and Rule 6 closes the list: "There is no `take` operation and no
partial move out of any place: the only ways to move a value out of storage are
consuming a whole local (`move x`), the window operations, and the atomic update."
`buf[j+1]` is not a whole local; the window operations are `push_nogrow`, `pop`,
`swap_remove` and `truncate`, none of which reads an interior slot into a value
while leaving the window shape alone except `swap_remove`, which also refills the
slot it empties. So the shift must be built from swap-with-the-last compositions,
as in §2.6c: three moves per position where `memmove` does one.

`swap_remove(buf, j)` followed by `push_nogrow(buf, u)` is exactly `swap(j, len-1)`:
the first yields `e_j` and refills slot `j` from the last slot (2 moves, one length
store); the second writes `e_j` into the new last slot (1 move, one length store).
Rule 11 discharges both `requires` from the invariant `len_of(buf) == n - 1` and
the loop condition `j + 1 < n - 1`.

### 3.7 References into the container across a mutation

This is P1's and P4's question. Rule 3: a reference "is invalidated when any proper
prefix of p's path is written, moved out of, replaced, or freed, by a statement or
by a call."

```text
p = &v.buf[i]
push(&v, move x)          // substituted row: writes(v.buf); v.buf is a proper prefix of v.buf[i]
use(p)                    // rejected (Rule 3, Rule 10 clause 3)
```

Rule 10 clause 3 says the same from the call side: "A live reference outside the
call whose path has a proper prefix among the call's write paths becomes invalid
after the call." This holds **even when the push does not grow**, because
`push_nogrow`'s row is `writes(buf)`, the whole path. Rule 4's own comment prices
the rewrite: keep the index, re-derive the reference, "one address computation".

P4's `Cursor { at: pointer-or-handle to v, i: 0 }` in its pointer form is refused
outright by Rule 1 ("there is no type parameter, wrapper, or variant payload
through which a reference can be stored") and Rule 4 ("It cannot be assigned into
any aggregate (Rule 1), cannot be returned"). The handle form — `Cursor { i: u64 }`
plus a reference to the container supplied at each use — is admitted and is the
form Rule 4 prescribes: "A function that 'finds' something returns an index or
other owned data; the caller forms the reference."

P7's take/put aliasing question does not arise: `take` is listed under "Not in this
candidate" and Rule 12 states "No place is ever partially moved", so the hole P7
requires cannot be created. The candidate's answer to P7 is the atomic update,
which has no observable hole by construction.

### 3.8 Linear elements at scope exit

Rule 6: "A DynBox whose element type is linear is itself linear (Rule 8) and must
be truncated to zero by the program before it can go out of scope." Read together
with `truncate`'s contract (`requires n <= len_of(buf); ensures len_of(buf) == n`)
and Rule 6's scope-exit sentence ("At scope exit the compiler releases slots
`[0, len_of)` recursively"), a `truncate(&buf, 0)` on a `DynBox<File>` would have
the compiler release `File` values. Rule 8 forbids exactly that: "Linear values
must be consumed by an explicit operation on every exit path; the compiler never
releases them." Rule 8's sentence is categorical and decides it: for a linear
element type `truncate` is usable only at `n == len_of(buf)` (a no-op), and the
program empties the container by `pop` plus an explicit consumer, as in §2.8. The
work is the same n consumer calls that Rust's `Drop` glue performs. Not a cost.

### 3.9 Taking a linear aggregate apart

`close_conn` must reach the `File` inside a `Conn`, and `Box<File>` must be
"taken apart" in Rule 8's words. Both need an operation that Rule 6's closed list
does not contain. See gap **G5**; this is the one place where the task's stated
element set ("Box and linear payloads") runs past the rule text. `Vector<File>`
and `Vector<Box<Node>>` need no such operation and are fully derived above.

---

## 4. Parallel opportunities

### 4.1 Admitted by Rule 13

```text
// (a) element-wise work through the Boxes, split by range
par { work(&v.buf[0..mid]); work(&v.buf[mid..n]) }
fn work(part: &DynBox<Box<Node>>)  writes(part)
```
Rule 13: "`par { A; B }` is accepted when A's write paths are disjoint from B's
read and write paths and vice versa, using the same path-overlap and
index/range-disjointness judgment as Rule 10", and Rule 13's own example is the
adjacent-range form. What matters for this task is the strength of the conclusion:
the two arms write *through* `Box` values into two sets of heap objects, and the
disjointness of those heap objects needs no analysis at all, because Rule 5 makes
`Box<T>` own "exactly one heap object" and Rule 1 forbids any reference inside an
aggregate. A C++ `std::vector<std::unique_ptr<Node>>` parallel loop has the same
runtime shape but the compiler has no such fact; the writer asserts it. This is
the capability floor's "Runtime-width adjacent ranges passed to a helper" row of
`PROGRAMS.md`, preserved for non-Copy elements.

```text
// (b) two distinct elements
par { touch(&v.buf[i]); touch(&v.buf[j]) }        // needs the fact i != j (Rule 13's own example)

// (c) read/read over the whole container
par { s1 = scan(&v.buf); s2 = scan(&v.buf) }      // "Read/read overlap is allowed"

// (d) hand-over then parallel work on the element and on the rest
e = swap_remove(&v.buf, k)
par { process(&e); scan(&v.buf) }                 // roots e and v.buf differ: disjoint

// (e) two containers in parallel, both allocating and both releasing
par { fill(&a)?; fill(&b)? }                      // Rule 14: "Allocation and release carry no
                                                  // effect entry and never make two parallel arms
                                                  // conflict"

// (f) draining two linear containers in parallel
par { drain_close(&a); drain_close(&b) }          // distinct roots
```

### 4.2 Refused by Rule 13

```text
par { push(&v, x); push(&v, y) }        // writes(v.buf) vs writes(v.buf): rejected
par { push(&v, x); scan(&v.buf) }       // Rule 13's own rejected example, "writes(v) overlaps reads(v)"
par { swap_remove(&v.buf, i); swap_remove(&v.buf, j) }   // rejected even with i != j: both rows are
                                                         // writes(buf), the whole path, because the
                                                         // window boundary moves
par { move_half(&old, &nb, 0, mid); move_half(&old, &nb, mid, n) }   // growth cannot be split: every
                                                         // window operation takes &DynBox<T> and
                                                         // writes the whole path
```

Rust refuses (a)-(c) of this list identically (`&mut Vec` is exclusive), so nothing
is lost there. The fourth is a real loss only against a hand-written parallel
`memcpy` on a very large reallocation, which neither `std::vector` nor `RawVec`
performs either; what is lost against them is the *vectorized* single-threaded
copy, priced in §5.

A producer/consumer pipeline over this container — one arm pushing, one arm
popping — is not expressible: it needs the channel listed under "Deferred to a
future concurrency and layout round", and `par` with two writers of `v.buf` is
rejected. `PROGRAMS.md`'s "Resource-processing pipeline" row is therefore served
only in its batched fork-join form, consistent with the candidate's already
recorded cost "Lock-free rings are not expressible; batched fork-join with two
buffers is the available form."

### 4.3 Floor check

The `PROGRAMS.md` capability-floor rows that touch a container of resources —
independent declared calls, element-wise same-index read/modify/write, runtime-width
adjacent ranges through a helper, owned composite values and storage, definition
and call-side contracts — are all preserved for non-Copy elements by §4.1(a),(b),(e)
and the contracts throughout §2. The accumulator-recombination row is unaffected
by element class. No admitted overlap is lost relative to an exclusive-reference
baseline.

---

## 5. Cost table against an idiomatic C++ / Rust implementation

Baselines: C++ `std::vector<std::unique_ptr<Node>>` / `std::vector<Conn>` with a
move constructor, and Rust `Vec<Box<Node>>` / `Vec<Conn>`. `n` is the length,
`L = n - 1 - k` the suffix length, `W = sizeof(T)`.

| Operation | Baseline | Candidate x1 form | Extra element moves | Extra loads / branches | Extra allocations | Lost parallelism |
|---|---|---|---|---|---|---|
| `push`, no growth | 1 move, 1 length store, 1 capacity branch | `push_nogrow` after the same branch | 0 | +1 dependent load per `len_of`/`cap_of` read (block header behind the DynBox handle) unless hoisted; +1 branch at the caller for the `Pushed` discriminant | 0 | none |
| `push`, growth, order observable | 1 allocation + `memcpy` of `n*W` | `DynBox::new` + `move_all` (§2.3) | **+1.5n moves** (2.5n total vs 1.0n) and +2n header stores; the `swap_remove` phase is not vectorizable, each refill depends on the previous length | +2n length loads | 0 (same one block) | growth is serial in both |
| `push`, growth, bag semantics | same | `move_all_bag` (§2.3) | +1n moves (2n vs 1n), but the loop is a reverse-order block copy a vectorizer can still widen | +2n length loads, sinkable | 0 | same |
| `push`, growth, large buffer | `realloc` may extend the mapping in place and copy nothing | Rule 6 fixes `cap_of` "at allocation"; the only path "allocates a larger DynBox ... and replaces the old one" | the whole `n*W` copy where the baseline sometimes copies 0 | — | 0 | — |
| `reserve` | `try_reserve` | §2.7 `reserve` | as the growth rows | — | 0 | — |
| replace, affine, old discarded | 1 move + recursive free | `v.buf[k] = move fresh` | 0 | 0 | 0 | none |
| replace, linear, old consumed | 2 moves | atomic update (§2.5b) | 0 | 0 | 0 | none |
| replace, linear, old exported, bag | `mem::replace` = 2 moves | §2.5c | +1 move | +1 length store | 0 | none |
| replace, linear, old exported, ordered | `mem::replace` = 2 moves | §2.5d | +4 moves | +4 length stores | 0 | none |
| remove back | 1 move | `pop` | 0 | 0 | 0 | none |
| remove unordered | 2 moves | `swap_remove` | 0 | 0 | 0 | none |
| remove ordered, rotate rewrite | 1 move + vectorized `memmove` of `L*W` | §2.6 `remove_ordered` | **+2L moves** (3L vs L) and +2L header stores; scalar and serialized on the length, so the vectorized `memmove` is lost outright | +2L length loads | 0 | none |
| remove ordered, slot-map rewrite (Rule 16) | as above | `swap_remove` on the resources + a `DynBox<u64>` order array shifted by a Copy `memmove`, plus a reverse index | 0 on the resources; the shift is a Copy shift, so it matches `memmove` | +1 dependent load per traversal step and loss of sequential access over the resources; +1 store per removal to repair the reverse index | 0 (two extra blocks, allocated once) | none |
| hand-over, pre-reserved | 2 moves | §2.7b | 0 | +1 branch on `reserve`'s `Result` | 0 | none |
| hand-over, unreserved | 2 moves | §2.7c | 0 on the success path; +1 move on the failure edge | +1 branch on the `Pushed` discriminant | 0 | none |
| hold a reference across an append | C++ keeps the pointer valid while `capacity` suffices | Rule 3 kills it unconditionally; re-form from the index | 0 | +1 base load and +1 index computation per append-and-use iteration | 0 | none |
| drop the whole container, affine | recursive `Drop` | compiler release at scope exit | 0 | 0 | 0 | none |
| drop the whole container, linear | `Drop` calls `close` n times | `drain_close`: n `pop` + n `close` | 0 | +n length loads/stores that `Drop` also performs when it walks a length | 0 | none |
| allocation failure | C++ throws, Rust `Vec::push` aborts | `Pushed<T>` / `Result` on every fallible step | 0 | +1 branch per fallible operation | 0 | none |

Two rows deserve emphasis because they run the other way:

- **Allocation failure.** The baseline row is not a cost the candidate pays; it is
  a cost the baseline avoids by not offering the behavior. Against
  `Vec::try_reserve` or a C++ build with exceptions disabled, the branch count is
  the same and the candidate additionally *proves* the accounting, which neither
  baseline does.
- **Parallel element work.** §4.1(a) needs no `noalias` assertion and no
  `split_at_mut`; the alias fact is a consequence of Rule 1 and Rule 5 rather than
  a writer's promise. Same instructions, better information.

---

## 6. Strongest counterexample against the candidate for this task

An ordered, order-observable table of linear resources with mid-sequence removal —
a compiler's live-interval list, a server's priority-ordered connection table, a
kernel's ordered descriptor table. Take `Vector<Conn>` with `W = 16` bytes and
`n = 10000`, a workload of `n` pushes and `n/2` ordered removals near the front.

```text
// the workload, in the candidate's own notation
for step in 0..n { match push(&table, move arrive()) { Ok => {} Full(c) => { close_conn(move c) } } }
while len_of(table.buf) > n / 2 { c = remove_ordered(&table.buf, 0); close_conn(move c) }
```

- **Growth.** 14 doublings; the total relocated volume is 2.5 × 160 KB ≈ 400 KB of
  scalar, length-dependent moves against 160 KB of `memcpy`. The `swap_remove`
  phase cannot be widened because each refill reads the slot the previous length
  store just designated.
- **Ordered removal at the front.** `L ≈ n`, so each removal is ≈ 3n scalar moves
  plus 2n length stores against one vectorized `memmove` of n elements. Over `n/2`
  removals that is ≈ 3.75 × 10^8 element moves against 1.25 × 10^8 — three times the
  count, and each candidate move is a scalar 16-byte copy inside a loop whose trip
  count the optimizer can see but whose body it cannot vectorize, where the
  baseline issues one `rep movsb` or one SIMD copy loop per removal. A 4-8×
  wall-clock gap on this phase is the honest prediction.
- **No rewrite recovers the shape.** The rotation is forced: Rule 6's enumeration
  of move-out operations is closed, `swap_remove` is the only interior extractor
  and it refills the slot it empties, and the atomic update commits its result back
  to the same slot, so no sequence of admitted operations produces a one-pass block
  shift of non-Copy elements.
- **The rewrite that does win changes the data structure**, not the code: resources
  in a bag with `swap_remove`, order in a separate `DynBox<u64>` whose elements are
  Copy and therefore shiftable by an ordinary duplication loop, plus a reverse
  index so a `swap_remove` can repair the order array. That is asymptotically
  *better* than `Vec::remove` (O(1) resource movement instead of O(L)), and it
  costs 16 bytes per element, one dependent load per traversal step, loss of
  sequential access over the resource block, and one extra store per removal. It is
  also precisely the pattern Rule 16 licenses, which is where this task's
  dependence on Rule 16 enters.
- **The second counterexample, smaller but unavoidable**: a parser or interpreter
  that appends to a node vector while holding a reference to the node it is
  filling. C++ keeps the pointer valid while capacity suffices and the writer
  exploits that; Rule 3 invalidates it on every append, including a non-growing
  one, because `push_nogrow`'s row is `writes(buf)`. The rewrite is Rule 4's index,
  costing one base load plus one index computation per iteration. Small, constant,
  and paid in the hottest loop of a front end.

The counterexample the candidate **survives** is worth recording too: the failure
edge. There is no arrangement of allocation failures in §2 that loses or
duplicates a resource, and no runtime flag, sentinel, or tag is used to achieve it
— Rule 12's "No place is ever partially moved" plus Rule 6's "no program point can
observe a slot inside the window as empty" mean the state space in which a leak
could hide does not exist. `PROGRAMS.md` P8's requirement, "no runtime drop flag;
the writer's own branches carry the state", is met literally, and P18's
requirement that every failure arm "must free nothing twice and leave earlier
blocks owned" is discharged by §3.4 without a byte of metadata.

---

## 7. Rules consistent, and task achievable — recorded separately

**Rules consistent for this task: yes, with the five gaps in §8.** Nothing in §2
is admitted by one rule and refused by another once Rule 8's categorical sentence
about linear release is applied to §3.8. Rule 6's closed enumeration, Rule 3's
invalidation, Rule 10's pairwise comparison and Rule 13's disjointness judgment
agree on every program above. The disagreements that remain are gaps in coverage
(an operation Rule 8 assumes and Rule 6 does not list), not contradictions between
two admitted derivations.

**Task achievable: yes, with cost.** Every operation the task names is expressible:
push (zero cost), growth (real cost: 2.5n or 2n moves against one `memcpy`, and no
`realloc` extension), replace (zero cost when the old value is discarded or
consumed in place; +1 to +4 moves when it must be exported), removal (zero cost at
the back and unordered; 3× scalar moves ordered, or a slot-map at 16 bytes and one
dependent load per traversal step), hand-over (zero cost), and the failure paths
(zero cost, and stronger evidence than either baseline provides). The one element
class that is not derivable from the rule text as written is a linear aggregate
that must be taken apart — `Conn`, `Box<File>` — which is gap G5.

**Per-operation verdicts:**

| Operation | Verdict |
|---|---|
| Container and element typing, contracts | `accepted-fine` |
| `push` without growth | `accepted-fine` |
| Growth, bag semantics | `rejected-real-cost` (2n moves, reversal, vs `memcpy`) |
| Growth, order observable | `rejected-real-cost` (2.5n scalar moves, no vectorization, no `realloc`) |
| Replace, old discarded or consumed in place | `accepted-fine` |
| Replace, old exported to the caller | `undecided-rule-gap` (G1); `rejected-real-cost` under the restrictive reading |
| Remove at the back, remove unordered | `accepted-fine` |
| Remove ordered | `rejected-real-cost` |
| Hand-over to another owner | `accepted-fine` |
| Allocation-failure accounting, affine and directly linear elements | `accepted-fine` |
| Allocation-failure accounting, `Box<linear>` payload | `undecided-rule-gap` (G2) |
| Discharging a linear container | `accepted-fine` |
| References held across a mutation | `rejected-zero-cost` for the rejection itself; `rejected-real-cost` for the append-with-cursor loop (one base load plus one index computation per iteration) |
| Parallel element work, adjacent ranges, read/read, two containers | `accepted-fine` |
| Parallel append, parallel growth, producer/consumer | `rejected-zero-cost` against Rust (`&mut` refuses the same programs); the pipeline needs a deferred mechanism |
| Linear aggregate elements (`Conn`, `Box<File>`) | `undecided-rule-gap` (G5) |

**Headline verdict: `rejected-real-cost`.** The container is buildable and its
failure accounting is better evidenced than the baselines', but growth and ordered
mutation of non-Copy elements carry unavoidable runtime cost under Rule 6's closed
operation set, and three sub-cases are rule gaps.

---

## 8. Rule gaps

Each gap quotes the exact sentence and names what the text does not decide. Per the
protocol none of them is resolved here.

**G1 — may the atomic update's function take further arguments, or carry an effect
row?** Rule 6:

> `buf[k] = f(buf[k])                   // atomic in-place update: the old value goes into f by value, f's result is committed, no program point lies between; requires k < len_of(buf)`

Missing: whether `buf[k] = f(buf[k], move fresh)` or
`buf[k] = f(buf[k], &sink)` matches the form, and whether `f` may declare
`writes(sink)` for a reference parameter that is not the updated slot. Rule 9's
"each path starts at a reference parameter" and Rule 10's "Function-typed
parameters carry a full signature with its own row and contract" make the effect
checkable if extra parameters are allowed, and make a captured reference
uncheckable if they are not. The consequence for this task is priced in §5: a
replace that must export the old linear element costs 3 moves (bag) or 6 moves
(ordered) under the restrictive reading against `mem::replace`'s 2.

**G2 — the signature of `Box::new` for a linear payload.** Rule 5:

> `b = Box::new(Node { ... })?         // allocation can fail: Result<Box<Node>, Oom>`

Missing: what happens to the by-value payload on the `Err` arm when the payload is
linear. Read literally the payload is discarded, which contradicts Rule 8's "the
compiler never releases them" and loses a resource on an allocation-failure path —
exactly the property this task must guarantee. The derivation assumes the
payload-returning form `Result<Box<T>, (Oom, own T)>`, which costs nothing on the
success path.

**G3 — does linearity propagate through an enum?** Rule 8:

> "Linearity is declared on external-resource types and propagates through aggregates: a struct, array, Box, or DynBox containing a linear part is linear."

Missing: enum and tuple, both of which Rule 1 lists as aggregates ("Every struct,
enum, tuple, array, slice-like value, Box, DynBox, and generic instantiation holds
only owned values"). The task's whole failure-accounting design returns the
in-flight element in `Pushed<T>`'s `Full` variant and in `Err((Oom, own T))`; if
linearity did not propagate through those, a caller could discard the variant and
the resource would be lost with no diagnostic. The generic sentence ("propagates
through aggregates") and Rule 12's own `DynBox<Option<Entry>>` example both point
at propagation, but the enumerated list does not say it.

**G4 — does emptiness discharge a linear DynBox at an overwrite, or only at scope
exit?** Rule 6:

> "A DynBox whose element type is linear is itself linear (Rule 8) and must be truncated to zero by the program before it can go out of scope."

Missing: the same question for `v.buf = move nb`, which overwrites the old DynBox
rather than letting it go out of scope. Rule 6 licenses the operation in the
abstract — the growth path "allocates a larger DynBox, moves `[0, len_of)` across,
and replaces the old one" — and `move_all`'s `ensures len_of(old) == 0` establishes
emptiness at that point, but the discharge condition is worded only for scope exit.
No runtime cost rides on the answer; if the overwrite is refused, the growth is
written with the old buffer as a local that reaches its scope end, at zero cost.

**G5 — how is a linear aggregate taken apart?** Rule 8 asserts the operation twice:

> `fn use_it(c: own Conn) { ...; close(move c.f) }   // consuming the struct as a whole and closing its File is required`

and

> `Box<File>                            // linear: must be taken apart and its File closed`

Rule 6 closes the list of ways to move a value out of storage:

> "There is no `take` operation and no partial move out of any place: the only ways to move a value out of storage are consuming a whole local (`move x`), the window operations, and the atomic update."

and Rule 12 repeats it: "No place is ever partially moved: every path is either
wholly present or the program cannot name it." `move c.f` is a partial move out of
the place `c.f`, and unboxing `*b` is a move out of the place `*b`; neither is a
whole local, a window operation, or an atomic update. Missing: a whole-value
destructuring form (which would consume the whole local and leave no partially
moved place, and which Rule 8's own comment "consuming the struct as a whole"
gestures at) and an unbox primitive. Without them, a linear-by-containment element
type can never be discharged and `Vector<Conn>` and `Vector<Box<File>>` are not
constructible; with them, the task is derived exactly as §2 shows, at no extra
runtime cost. `Vector<File>` and `Vector<Box<Node>>` do not touch this gap.

---

## 9. Dependence on Rule 16

**Dependent: yes, and confined to identity after removal.**

Nothing in §2, §3 or §4 needs Rule 16 for memory safety or for the failure
accounting. Rule 6 guarantees that every slot inside the window holds a valid
value, Rule 7 requires a bounds fact for every access, and Rule 10 clause 3 kills
every live reference into the container at each mutating call; none of that
consults Rule 16.

Rule 16 enters twice:

1. Rule 4 requires searches to return indices — "A function that 'finds' something
   returns an index or other owned data; the caller forms the reference" — and this
   container's removals (`swap_remove`, `remove_ordered`) renumber elements. An
   index that survives a removal still satisfies `k < len_of(buf)` and names a
   different resource. Rule 16 is the only place in the rule set that states a
   disposition for this: "A stale index that is still in bounds names the current
   occupant of that slot: a logic error, not a memory error. Programs that need to
   detect it keep a generation number as data." Without Rule 16 the candidate has
   no stated position on handle identity in a mutated container, and the task's
   "hand-over of an element to another owner" leaves the other owner holding a
   number whose meaning is undefined by the rules.
2. The slot-map rewrite in §5 and §6 — resources in a bag, order and reverse index
   as separate Copy arrays — is the pattern Rule 16 licenses, and it is the only
   rewrite that restores O(1) ordered removal. Its identity cost (a generation word
   per element, one load and one compare per dereference, plus an outcome arm) is
   the price Rule 16 itself names.

Everything else in this task derives under Rules 1-15 alone.
