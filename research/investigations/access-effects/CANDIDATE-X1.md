# Candidate x1: frozen rule set (revision 5, 2026-09-18)

Status: every rule below was confirmed by the owner in the 2026-09-17/18 sessions. Revision 5 applies the wording fixes from the targeted second round (`REVIEW-X1-round2.md`): window parts are row vocabulary only, range references die with their bound, the state in which adjacent statements are compared, `grow` returns a Result, runtime-capacity constructors are payload-free primitives, the atomic update's restriction is on writes to prefixes only, `writes` covers a path and everything below it. Revision 4 adopted member spelling for measures and named window parts; revision 3 folded in the round-one rulings. The list under "Proposed additions" holds the items awaiting the owner. The mechanisms under "Not in this candidate" must not be assumed by anyone deriving under it.

Evaluation criteria, in the owner's words: safety through types and proofs at the current WF level (no runtime traps, no unsafe), runtime performance for single-thread and parallel computation, and enough expressiveness to build compilers, browsers, and kernels. Code is written by AI, so an ugly but equal-performance form is not a defect. A rejection counts as expressiveness loss only when no equal-performance rewrite exists.

Notation: design pseudocode, not current WF syntax except where stated. `&path` forms a reference; `move x` consumes an owned value; `own T` is a by-value parameter; an effect row follows the signature; `contract { requires ...; ensures [when Variant:] ...; }` is the existing WF contract block; measures are read-only pseudo-fields of a storage (`r.len`, `r.cap`, `r.room`, `r.head`) that only built-in operations change, replacing the existing `len_of` family; `entry(p)` denotes the pre-state of parameter `p` as in the existing specification, so `entry(r).len` is the length at call entry; `Int` and `u64` are Copy. Indexing through a Box is written with the existing explicit dereference: `deref(b)[i]`, `deref(b).len`; there is no `*` operator. Two adjacent statements "may overlap" means the implementation is permitted to execute them with overlapping execution, as the existing PAR-1 and PAR-2 rules permit; the program's meaning is always its sequential meaning.

## Rule 1. Values have value semantics; no aggregate ever contains a reference

Every struct, enum, tuple, `Array`, `Slots`, `Ring`, `Box`, and generic instantiation holds only owned values. This is recursive and closed under wrapping: there is no type parameter, wrapper, or variant payload through which a reference can be stored.

```text
struct Node { value: Int, left: Option<Box<Node>> }     // owned children: allowed
struct Bad  { r: &Int }                                  // rejected
enum   Bad2 { A(&Int), B }                               // rejected
Option<&T>                                               // rejected, whatever T is
```

Consequence: any value can be relocated by copying its bytes (memmove, realloc), because nothing inside it points anywhere.

## Rule 2. A reference is a local name for a path

A path starts at a local variable or a parameter and continues through fields, `deref` (Box content), `[i]` (index, Rule 7), `[lo..hi]` (range, Rule 7), or the payload of an enum variant. A payload step is available only under the refinement fact that the enum currently holds that variant, which a `match` or `if let` on the enum establishes in the selected arm and which any write to the enum invalidates. A reference variable names a path; it is not storage of its own. An index expression inside a path is evaluated when the reference is formed; the path records that value, and later assignments to the variables the expression used do not change it.

```text
p = &deref(v)[i]           // p names slot i of the window inside v, with i's value at this point
i = i + 1              // p still names the old slot
q = p                  // q names the same path; both may be live
r = &p.x               // r names deref(v)[i].x; a reference extends a path, it does not point at p
&p                     // rejected: a reference variable is not storage

match &n.left {
    Some(child) => {   // child names the payload path n.left.Some.0 under the fact "n.left is Some"
        n.left = None  // the fact is gone: child is invalid from here
    }
    None => {}
}
```

A reference may be rebound. At a control-flow join, a reference variable's target is the set of paths it may name; every check on it must hold for every member of the set. A path has a static shape: a loop-carried rebinding may change only the index values inside the path, never extend the path through itself.

```text
w = if cond { &v1 } else { &v2 }     // w names one of {v1, v2}
p = &deref(w)[i]                         // p names one of {deref(v1)[i], deref(v2)[i]}
loop { p = &deref(p.kids)[0] }           // rejected: after k iterations the path is k steps long, so the join has no finite
                                     // target set. A path may carry a runtime index (deref(pool)[i], one bound fact), never a
                                     // runtime depth. Walk owned links by recursion, or keep the nodes in a pool and iterate an index
```

There is no `uniq` or `mut` marker on references. Whether a callee may write through a reference parameter is stated by its effect row (Rule 9).

## Rule 3. Reference validity is a fact

"p is valid" is a fact like any other. It is established when p is formed and invalidated by any of the following: a proper prefix of p's path is written, moved out of, replaced, or freed, by a statement, by a call, or by the compiler-derived release at scope exit; the scope of the local variable at which p's path starts ends; or a refinement fact that a payload step in p's path depends on is invalidated. Writing the storage at p's path or below it (a content write) does not invalidate p. Using an invalid reference is rejected.

Whether one path is a prefix of another, and whether two paths overlap, is judged conservatively: two indexed positions on the same storage are taken to overlap unless their indices or ranges are proved distinct, exactly as in Rule 10.

```text
p = &deref(v)[i]
deref(v)[j] = 5            // a content write on slot j: p stays valid whether or not i == j
grow(&v, cap)          // declares writesderef(v): *v is a proper prefix of deref(v)[i]: p invalid
use(p)                 // rejected

q = &deref(deref(g)[j]).value  // g: Box<Slots<Box<Node>>>
replace_at(&g, i, nb)  // writes deref(g)[i]: overlaps deref(g)[j] unless i != j is proved: q invalid
                       // with the fact i != j, q survives

p = &deref(b)                // b: Box<Node>; p names the heap Node
c = move b             // b is a proper prefix of deref(b): p invalid, even though the Node did not move
use(p)                 // rejected; form a new reference from c

p: &Int
{ b = Box::new(...)?; p = &deref(b).value }   // b's scope ends here and b is released
use(p)                 // rejected: p's root has gone out of scope
```

Validity is re-established only by forming the reference again. A move never re-roots an existing reference: after `w = move v`, references formed from `v` are invalid and are not reinterpreted as references into `w`.

## Rule 4. References are never stored, returned, or captured into anything that escapes

A reference may be bound to a local, passed as a call argument, and used within the function that formed it or received it. It cannot be assigned into any aggregate (Rule 1), cannot be returned, and cannot be captured by a function value that is stored or returned. A function that "finds" something returns an index or other owned data; the caller forms the reference.

```text
fn find(v: &Slots<Entry, N>, key: Int) -> Option<u64>      // returns an index, never &Entry
    reads(v)
    contract { ensures when Some: result < v.len; }

match find(&v, k) { Some(i) => { p = &v[i]; ... }  None => ... }   // caller re-derives, one address computation
```

## Rule 5. Box<T> is the only heap marker

`Box<T>` owns exactly one heap object. There is one heap; Boxes carry no store or region brand and can be moved, stored in aggregates, and returned freely. `deref(b)` is a path (Rule 2). A Box is affine: at scope exit the compiler releases its memory recursively (Rule 8); there are no destructors. `move deref(b)` consumes the Box, yields its content, and frees the cell (Rule 6).

The content type may be any `T`, including the three runtime-capacity shapes `Array<T>`, `Slots<T>`, and `Ring<T>` (Rule 6), which have no compile-time size and may appear only as the content of a Box: never inline in another value and never as a local variable.

A fallible allocation that takes a by-value payload hands it back on failure, so a linear payload is never lost.

```text
b = Box::new(Node { ... })?          // Result<Box<Node>, (Oom, Node)>: on Err the Node comes back
p = &deref(b).left                       // reference into the heap object
c = move b                           // p invalid (Rule 3); the heap object did not move
holder.child = move c                // a Box in a struct field: ordinary ownership
n = move deref(c)                          // unbox: c is consumed, n is the Node, the cell is freed
buf: Box<Slots<u8>>                  // a window of runtime capacity, on the heap
tags: Box<Array<u8>>                 // a fully initialized block of runtime length
```

## Rule 6. Storage shapes, the window, and how values leave storage

Three shapes, each with a constant-capacity form (may be inline) and a runtime-capacity form (only inside a Box, Rule 5):

```text
Array<T, N>   Box<Array<T>>    every slot always holds a value
Slots<T, N>   Box<Slots<T>>    a window: slots [0, len) hold values, [len, cap) hold nothing
Ring<T, N>    Box<Ring<T>>     a ring window: len slots starting at head, wrapping at cap
```

Measures, spelled as read-only pseudo-fields: `r.len`, `r.cap`, `r.room` (equal to `r.cap - r.len`); `r.head` on rings only; for an `Array`, `a.len == a.cap`. A program reads them like fields and can never assign them; only the operations below change them. The window boundary `r.len` is a runtime number stored with the block. No slot ever carries a tag, and no program point can observe a slot inside the window as empty.

Construction:

```text
Array::new(v)                        // T Copy: every slot holds v; or a literal [a, b, c]
Box::new_array_filled(n, v)?         // runtime length, T Copy, every slot holds v; Result<Box<Array<T>>, Oom>, no payload; zero-filled maps to calloc
Slots::new<T, N>()                   // empty window
Box::new_slots<T>(cap)?              // empty window of runtime capacity; Result<Box<Slots<T>>, Oom>, no payload; Box::new_ring likewise
Slots::from_array(move a)            // a full window; Slots::into_array(move r) requires r.len == N
```

Window parts. Besides its slots, a window has four named parts that paths and effect rows may name, all interpreted at call entry: `r.next` (the append slot, at index `r.len`), `r.last` (the last filled slot, at index `r.len - 1`), `r.filled` (all slots below `r.len`), and `r.free` (all slots from `r.len` up). Overlap follows from the definitions: a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`, overlaps `r.last` unless `i != r.len - 1` is proved, and always overlaps `r.filled`. The measure `r.len` is itself a write target. Parts are vocabulary for effect rows and the overlap judgment only: no program forms a reference to a part, reads it, or writes it; the append slot is written only by `place_back` and `insert_at`. This vocabulary is ordinary: the rows below use nothing a user function cannot write.

Window operations, shared by `Slots` and `Ring` (`r` stands for the storage, e.g. `deref(b)`):

```text
place_back(&r, x)          writes(r.next), writes(r.len)
    contract { requires r.room > 0;  ensures r.len == entry(r).len + 1; }
take_back(&r) -> own T     writes(r.last), writes(r.len)
    contract { requires r.len > 0;   ensures r.len == entry(r).len - 1; }
insert_at(&r, k, x)        writes(r.filled), writes(r.next), writes(r.len)      // one memmove
    contract { requires k <= r.len, r.room > 0; ensures r.len == entry(r).len + 1; }
remove_at(&r, k) -> own T  writes(r.filled), writes(r.len)                      // one memmove
    contract { requires k < r.len; ensures r.len == entry(r).len - 1; }
append(&dst, &src)         writes(dst.free), writes(dst.len), writes(src.filled), writes(src.len)   // one memcpy
    contract { requires dst.room >= src.len; ensures dst.len == entry(dst).len + entry(src).len, src.len == 0; }
split_off(&src, k, &dst)   writes(src.filled), writes(src.len), writes(dst.free), writes(dst.len)   // one memmove
    contract { requires k <= src.len, dst.room >= src.len - k; ensures src.len == k, dst.len == entry(dst).len + entry(src).len - k; }
grow(&b, cap) -> Result<(), Oom>   writesderef(b)                            // Box<Slots<T>> only; may reallocate in place
    contract { requires cap >= deref(b).cap;
               ensures when Ok:  deref(b).cap == cap, deref(b).len == entryderef(b).len;
               ensures when Err: deref(b).cap == entryderef(b).cap, deref(b).len == entryderef(b).len; }
set(&r[k], x)              writes(r[k])      // old value: affine released, linear rejected
replace(&r[k], x) -> own T writes(r[k])      // the old value is returned
place_front(&r, x)         writes(r)         // Ring only; every logical index shifts, so all references into r die
take_front(&r) -> own T    writes(r)         // Ring only
```

A user function that only appends declares the same row as `place_back`:

```text
fn add_node(g: &Graph, n: own Node) -> u64   writes(deref(g.nodes).next), writes(deref(g.nodes).len)
p = &deref(g.nodes)[i]
id = add_node(&g, node)                       // p survives: deref(g.nodes)[i] with i < len never overlaps .next
```

A reference into a window is formed under a bound and stays valid while that bound holds: `p = &r[i]` under `i < r.len`, `part = &r[lo..hi]` under `hi <= r.len`. `place_back`'s `ensures` carries the bound across the call; `take_back`'s, `remove_at`'s, and `truncate`'s do not, so such references die there. `insert_at` and `remove_at` write `r.filled`, a content write: a surviving slot reference names its slot, whose occupant may have changed, exactly as a stale index does (Rule 16); a writer who means the element re-forms the reference. Ring positions are logical indices; the wrap is the storage's business.

Two operations apply to any owned place, not only to window slots:

```text
swap(p: &T, q: &T)         writes(p), writes(q)     // built in; p and q may be the same place, then nothing happens
swap(&r[i], &r[j])                                  // no i != j branch needed in a partition loop
swap(&a.left, &b.right)

place = f(place, args...)  writes(place)            // atomic in-place update: the old value enters f by value,
                                                    // f's result is committed, no program point lies between
node.left = insert(node.left, k)                    // f is total and returns the place's type; f's row must not write,
c.f = reopen(c.f)                                   // move out of, or free any prefix of place (reading anything, and writing
n.left = fold(n.left, &n.right)                     // disjoint storage, is fine); failure is an enum in the place
```

How a value leaves storage. There is no `take` and no hole. A move out of a field or out of Box content consumes the whole owner: the owner ceases to exist, its other affine parts are released, and a remaining linear part rejects the move (take it in the same destructuring). A move out of a window slot or an array element is rejected; use the operations above.

```text
x = move c.f                        // c is consumed; the other fields of c are released
let Conn { f, g, .. } = move c      // several fields at once; `..` covers the rest
n = move deref(b)                         // unbox
x = move r[k]                       // rejected: use take_back, remove_at, replace, or swap
```

Release: at scope exit the compiler releases the slots inside the window recursively and frees the block; `Array` releases every slot. No operation releases a linear element: a storage whose element type is linear is itself linear (Rule 8) and the program must take every element out and consume it. Assigning over any owned place releases the old value if it is affine and is rejected if it is linear.

Library code, all zero-cost compositions of the above: `swap_remove` (swap with the last slot, then `take_back`), `truncate`, `clear`, `Vector<T>` (a `Box<Slots<T>>` plus a `push` that calls `grow` when `room` is zero), `Arena<T>` (`place_back` returning the index as allocation, `truncate(0)` as reset), `Pool<T>` (a `Vector` plus a free list as data), `Deque<T>` (`Box<Ring<T>>` plus growth by a fresh ring and one copy per doubling, `grow` being defined on `Box<Slots<T>>` only), open-addressing tables (`Box<Array<u8>>` of tags plus `Box<Array<Entry>>` or `Box<Array<Option<Entry>>>` of slots, all fully initialized), `String` (`Box<Slots<u8>>` plus growth).

## Rule 7. What can be indexed, and bounds are proved

Indexable things: `Array`, `Slots`, and `Ring` in either form, a range reference into any of them, and a `const` table. `Box<Array<T, N>>` is indexed through the path `deref(b)[i]` and is not a separate case. Every index must be proved in bounds; when the proof is unavailable the program tests the measure, which is ordinary data.

A range reference `&x[lo..hi]` has the parameter type `&[T]`: a reference kind with measure `len`, formed only from an indexable or from another range reference, never a stored value.

```text
a: Array<Int, 8>;  a[i]                 // requires i < 8
r: Slots<Int, N>;  r[i]                 // requires i < r.len
part = &r[lo..hi]                       // requires lo <= hi <= r.len; part.len == hi - lo
fn kernel(part: &[Int]) writes(part) { for k in 0..part.len { part[k] += 1 } }
sub = &part[a..b]                       // requires a <= b <= part.len
if i < r.len { use(&r[i]) }         // the test establishes the fact; one compare, no trap
```

## Rule 8. Value classes: copy, affine, linear; no destructors

A type is copy, affine, or linear. Copy values may be duplicated. Affine values are consumed at most once; at scope exit the compiler releases their memory recursively (Box, Slots, Ring, Array) and runs no user code. Linear values must be consumed by an explicit operation on every exit path; the compiler never releases a linear value. A window of linear elements is consumed by `free_empty(move r)` (or `Box::free_empty(move b)`), whose contract requires `r.len == 0`; the type stays linear, the proof is about the runtime length. Linearity is declared on external-resource types (the existing `linear` modifier) and propagates through every aggregate of Rule 1: a struct, enum, tuple, Array, Slots, Ring, or Box containing a linear part is linear.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, n: u64 }      // linear by containment
let Conn { f, .. } = move c;  close(move f)     // the only way to finish a Conn
f = move deref(b);  close(move f)          // Box<File>: unbox, then close
Slots<File, 4>                       // linear: every element must be taken out and closed
```

## Rule 9. Effects are declared only on reference parameters

An effect row lists `reads(path)` and `writes(path)` where each path starts at a reference parameter and may continue through fields, `deref`, payload steps, and whole-index or range positions supplied as arguments. `writes` covers writing, replacing, moving out of, and freeing the storage at the path and everything below it. A by-value parameter has no effect entry: the call site records the consumption (for `move`) or the read (for a copy) of the argument's place. Signatures never contain index expressions; an index enters an effect only through an argument, evaluated once at the call. The rows of the built-in operations of Rule 6 use only this vocabulary plus the window parts of Rule 6, which any user function may use too.

```text
fn bump(p: &Int)  writes(p)
fn stats(v: &Slots<Int, N>) -> Int  reads(v)
fn update(o: &Obj, c: Bool)  writes(o.a), writes(o.b)      // member paths are allowed
fn put_at(slot: &Int, x: own Int)  writes(slot)              // x needs no entry

bump(&a.x)                     // call effect: writes(a.x)
put_at(&r[k], 3)               // the argument fixes the slot at the call; requires k < r.len
place_back(&r, 3)              // appending: writes(r.next), writes(r.len)
```

A function body is checked against its own row: every statement's effect and every callee's substituted row must be covered by the declared row.

## Rule 10. The call-site rule

At a call, substitute the actual argument paths into the callee's row. Then:

1. Compare the substituted effects pairwise. Two effects on overlapping paths where at least one is a write must be proved disjoint (different roots, or indices or ranges proved distinct); otherwise the call is rejected. Read/read overlap is allowed. The built-in `swap` is the one operation whose two arguments may be the same place.
2. A by-value argument contributes a consumption (`move`) or a read (copy) of its place to this comparison.
3. A live reference outside the call whose path has a proper prefix among the call's write paths, under the same may-overlap judgment, becomes invalid after the call (Rule 3). A reference that is itself an argument is the thing being accessed, not a bystander.

```text
fn two(p: &Int, q: &Int)  writes(p), reads(q)
two(&a, &a)                                  // rejected: writes(a) and reads(a) overlap
two(&a.x, &a.y)                              // accepted: distinct fields
two(&r[i], &r[j])                            // accepted only with the fact i != j

fn bad(outer: &Slots<Slots<Int, M>, N>, inner: &Slots<Int, M>)  writes(outer), writes(inner)
bad(&vv, &vv[0])                             // rejected: writes(vv) and writes(vv[0]) overlap

fn put<T>(slot: &T, value: own T)  writes(slot)
slot = &deref(b).next
put(slot, move b)                            // rejected: move b writes the prefix b of slot's path (Rule 3)
```

Function-typed parameters are generic parameters (FN-2): they carry a full signature with its own row and contract, a call through one uses that row, and each instance is a direct call. A supplied function may refine the formal signature: its row may be a subset (a row states only what the body does, so a read-only function cannot declare a write), its `requires` may be weaker, its `ensures` stronger; parameter and result types agree exactly. Recursion is checked through contracts, never by unfolding bodies.

## Rule 11. Facts, contracts, and invalidation

Facts are the existing WF forms: affine comparisons over measures and integer values, refinement facts from a dominating branch or a `match` arm, loop-header invariants `invariant name: affine_expr compare_op affine_expr`, explicit `use` steps inside an `invariant`, and callee contracts. A fact that mentions a path is invalidated when that path is written (by statement or call) unless the callee's `ensures` re-establishes it. Contracts use `requires`, `ensures`, and `ensures when Variant:` for result-routed relations; the pre-state of a parameter is written `entry(p)`. Across a call, a fact known before the call about a measure of an argument survives as a fact about `entry(p)` of that argument, which is how an `ensures` of the shape `r.len == entry(r).len + 1` connects to what the caller knew.

```text
n = r.len
if m <= n {
    for i in 0..m {
        invariant h: r.len == n - i;     // carried by the writer, checked at every iteration
        take_back(&r)                        // requires r.len > 0: from i < m and m <= n
    }
}
while r.len > 0 { take_back(&r) }        // no knowledge: the loop condition is the test
```

There are no quantified facts over array elements ("for all i ...") and no per-slot occupancy facts; occupancy that is determined by data is stored as data (Rule 6, Rule 12).

## Rule 12. Control-flow joins and the absence of holes

At a join, facts are intersected (a fact survives only if it holds on every incoming edge); a reference's target set is the union of its targets (Rule 2); a local consumed on one incoming edge is consumed after the join. No place is ever partially moved: every path is either wholly present or the program cannot name it (Rule 6). Structures whose occupancy is decided by data keep that occupancy as data.

```text
if cond { place_back(&r, x) }               // after the join: r.len >= n, where n was r.len before
                                            // facts about indices below n survive
if cond { x = move c.f }                    // c is consumed after the join on both edges

tags:  Box<Array<u8>>                       // open-addressing table: occupancy is data
slots: Box<Array<Entry>>                    // fully initialized; a "logically empty" slot holds a valid value
slots: Box<Array<Option<Entry>>>            // non-Copy payloads: Option, using a null niche where the type has one
```

## Rule 13. Overlapped execution

The program's meaning is its sequential meaning. Two adjacent statements of one block may overlap when the first's write paths are disjoint from the second's read and write paths and vice versa, using the same path-overlap and index/range-disjointness judgment as Rule 10; read/read overlap is allowed; a by-value consumption counts as a write of the argument's place. Allocation and release are not effects (Rule 14). The existing counted-loop forms (per-element maps, adjacent ranges passed to a helper, admitted reductions) keep their permissions, with the ranges written as range references. The paths of both statements are interpreted in the state before the first statement; the first statement's `ensures` maps the second's indices into that state, so an index that is live only after an append is not distinct from the append slot. Permission composes: any run of adjacent statements that pairwise may overlap may all overlap. Because the meaning is sequential, results, errors, early exits, and linear obligations are exactly those of the sequential program; nothing new is defined for the overlapped case.

```text
s1 = stats(&v); s2 = stats(&v)                                  // read/read: may overlap
bump(&r[i]); bump(&r[j])                                        // may overlap given i != j
kernel(&r[0..mid], &out[0..mid]); kernel(&r[mid..n], &out[mid..n])   // disjoint ranges: may overlap
push(&v, 1); stats(&v)                                          // may not overlap: writesderef(v) meets readsderef(v)
```

## Rule 14. One global heap; allocation can fail; the allocator has no effect

There is one heap, provided by the trusted base, internally synchronized. Allocation and release carry no effect entry and never prevent two statements from overlapping. Allocation returns a `Result` and never traps (Rule 5 for the payload). Addresses are not observable, so allocator concurrency does not affect program determinism. There are no store or region parameters anywhere.

```text
a = build(&x)?; b = build(&y)?             // both allocate: may overlap
Box::new(v)?                                // Result on every allocation
```

## Rule 15. Return values are owned

A function returns owned values only. Positions found by a search are returned as indices with a bounds relation in `ensures` (Rule 4). Passing a large value in and back out is expressed as a reference parameter with a `writes` entry, not as move-and-return.

## Rule 16. Pools and arenas are usage, not types

Confirmed by the owner on 2026-09-18, knowingly reversing the design tree's earlier rejection of the arena-index-pool pattern. A pool is a `Slots` (or `Box<Slots<T>>`) plus indices used as handles; an arena is the same storage used with `place_back` as allocation and `truncate(0)` as reset. A stale index that is still in bounds names the current occupant of that slot: a logic error, not a memory error. Programs that need to detect it keep a generation number as data. The same holds for a reference into a user-level pool or allocator: storage that still exists stays readable, and what the program considers "freed" is the program's own bookkeeping.

```text
struct Arena<T> { buf: Box<Slots<T>> }
fn alloc<T>(a: &Arena<T>, x: own T) -> u64   // row: the append slot of *a.buf and deref(a.buf).len; see the open proposal
    contract { requires deref(a.buf).room > 0; ensures result == entryderef(a.buf).len; ensures deref(a.buf).len == result + 1; }
id = alloc(&a, node)
p  = &deref(a.buf)[id]                         // valid while id < deref(a.buf).len
truncate(&a.buf, 0)                        // reset: elements released; later reads of slot id need id < len again
```

## Not in this candidate

`with` blocks or any block-scoped reference form; `&uniq`, `&mut`, or any permission marker on references; lifetimes, regions, `region` statements, region parameters; store brands (`'s`), `Heap<'s>`, `Arena<'s, ...>`, `Box<'s, T>`, `Vector<'s, T>`, providers, `dispose`, capability-based linearity; `allocates` effects; slice types as first-class values; returned references; destructors; runtime traps; `take`/`put` holes; partial moves that leave an owner alive; quantified invariants over elements; per-slot occupancy tags maintained by the compiler; channels or atomics; header-plus-tail heap blocks; a `par` statement (the matrix files use `par { A; B }` only as a notation for "may A and B overlap"). Retired names from earlier drafts: `DynBox`, `FixedVector`, `Run`, `HeapSlots`, `HeapRing`, `DynSlots`, `DynArray`, `DynRing`, `buffer`, `exchange`.

## Proposed additions awaiting owner ruling

None.

Settled by the existing specification: function-typed parameters are generic parameters instantiated per instance (FN-2), so a function argument is a direct call. Recorded for the implementation plan, not as language rules: the compiler-derived release of an owned chain runs in bounded stack using the freed cells as its worklist; self-recursive descent has tail calls eliminated or a stated depth budget.

## Deferred to a future concurrency and layout round

- A channel primitive in the trusted base (ownership-transfer queue) for producer/consumer pipelines and work stealing.
- A heap block with a fixed header followed by a runtime-length tail in one allocation.
- A bitmask fact (`x & (c - 1) < c` for a power-of-two `c`) to remove the per-probe bounds compare in hash tables.
- Contract facts beyond affine comparisons: a `requires` stating a variant refinement (`p is Some`), and an `ensures` naming a single indexed path (`deref(p.slots)[h.idx].gen == h.gen`); the second decides whether a guarded pool access pays one load, compare, and branch per call.
- A wildcard path form `x.**` ("somewhere under x") for descending an owned linked structure in a loop, at the price of conservative overlap and invalidation under `x`. Revisit only if the pool-plus-index form proves too slow or inexpressible for a real workload.

## Known costs already recorded

- One compare and one predicted branch per data-determined index (graph hop, free-list pop, hash probe); equal to safe Rust, real against C++.
- Scatter to data-determined destinations cannot overlap; invert-and-gather or bucket-first costs one extra pass; equal to safe Rust.
- Find-then-mutate on a Box-linked tree re-descends once unless fused, passed a function-typed parameter, or replaced by a pool index.
- Construction into the append slot moves one element where C++ constructs in place; usually elided by the backend, not guaranteed.
- Open-addressing tables with non-Copy payloads pay one null check per hit versus hashbrown.
- Lock-free rings and work-stealing queues are not expressible; batched fork-join is the available form.
- Header-plus-tail layouts pay one extra dependent memory access per hop.
- Pool allocation from one pool serializes against every other access to that pool; per-worker pools are the standard form. Generation words for identity across slot reuse cost what a Rust slotmap costs.
- A rebinding of a reference that extends its own path in a loop is refused; recursion or indices cost the same loads.

## Protocol for a derivation cell or task under this candidate

1. State the question the cell tests, in one sentence.
2. Write the strongest program first: the case where static analysis provably cannot help (data-determined indices, lost conditions, mixing, separate compilation, joins), not a convenient variant. Then the ordinary case.
3. Trace acceptance or rejection rule by rule, quoting the rule number and the exact clause used. Do not add rules. If the text is ambiguous, quote the sentence and mark the cell `undecided-rule-gap`.
4. For every rejection, give the best rewrite under these rules and its runtime cost relative to the C++ or Rust form, naming the extra instruction, load, copy, allocation, branch, or lost parallelism. A rewrite with no runtime cost is `rejected-zero-cost`; one with unavoidable cost is `rejected-real-cost`.
5. Record separately whether the rules are consistent and whether the original task is still achievable; a rule set that rejects a task cleanly has not achieved it.
6. Verdict vocabulary: `accepted-fine`, `accepted-unsafe`, `rejected-zero-cost`, `rejected-real-cost`, `undecided-rule-gap`.
7. Runtime performance is the only cost that counts; verbosity, extra parameters, and duplicated code are not costs.
