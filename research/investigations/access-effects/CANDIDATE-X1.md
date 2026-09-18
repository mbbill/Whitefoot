# Candidate x1: frozen rule set (revision 2, 2026-09-18)

Status: every rule below was confirmed by the owner in the 2026-09-17/18 sessions. Revision 2 applies the owner's container decisions (four names, slot-precise window operations) and the text fixes from matrix round one (`REVIEW-X1-round1.md`); it adds no rule the owner has not confirmed. Items under "Proposed additions awaiting owner ruling" are not rules. The mechanisms under "Not in this candidate" must not be assumed by anyone deriving under it.

Evaluation criteria, in the owner's words: safety through types and proofs at the current WF level (no runtime traps, no unsafe), runtime performance for single-thread and parallel computation, and enough expressiveness to build compilers, browsers, and kernels. Code is written by AI, so an ugly but equal-performance form is not a defect. A rejection counts as expressiveness loss only when no equal-performance rewrite exists.

Notation: design pseudocode, not current WF syntax except where stated. `&path` forms a reference; `move x` consumes an owned value; `own T` is a by-value parameter; an effect row follows the signature; `contract { requires ...; ensures [when Variant:] ...; }` is the existing WF contract block; `len_of`, `cap_of`, `room_of`, `head_of` are measures; `entry(p)` denotes the pre-state of parameter `p` as in the existing specification; `Int` and `u64` are Copy. Indexing through a Box is written explicitly: `(*b)[i]`, `len_of(*b)`.

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

A path starts at a local variable or a parameter and continues through fields, `*` (Box content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7). A reference variable names a path; it is not storage of its own. An index expression inside a path is evaluated when the reference is formed; the path records that value, and later assignments to the variables the expression used do not change it.

```text
p = &(*v)[i]           // p names the slot i of the window inside v, with i's value at this point
i = i + 1              // p still names the old slot
q = p                  // q names the same path; both may be live
r = &p.x               // r names (*v)[i].x; a reference extends a path, it does not point at p
&p                     // rejected: a reference variable is not storage
```

A reference may be rebound. At a control-flow join, a reference variable's target is the set of paths it may name; every check on it must hold for every member of the set.

```text
w = if cond { &v1 } else { &v2 }     // w names one of {v1, v2}
p = &(*w)[i]                         // p names one of {(*v1)[i], (*v2)[i]}
```

There is no `uniq` or `mut` marker on references. Whether a callee may write through a reference parameter is stated by its effect row (Rule 9).

## Rule 3. Reference validity is a fact

"p is valid" is a fact like any other. It is established when p is formed and invalidated by any of the following: a proper prefix of p's path is written, moved out of, replaced, or freed, by a statement, by a call, or by the compiler-derived release at scope exit; or the scope of the local variable at which p's path starts ends. Writing the storage at p's path or below it (a content write) does not invalidate p. Using an invalid reference is rejected.

Whether one path is a prefix of another, and whether two paths overlap, is judged conservatively: two indexed positions on the same storage are taken to overlap unless their indices or ranges are proved distinct, exactly as in Rule 10.

```text
p = &(*v)[i]
(*v)[j] = 5            // a content write on slot j: p stays valid whether or not i == j
grow(&v)               // declares writes(*v): *v is a proper prefix of (*v)[i]: p invalid
use(p)                 // rejected

q = &(*(*g)[j]).value  // g: Box<Slots<Box<Node>>>
replace_at(&g, i, nb)  // writes (*g)[i]: overlaps (*g)[j] unless i != j is proved: q invalid
                       // with the fact i != j, q survives

p = &*b                // b: Box<Node>; p names the heap Node
c = move b             // b is a proper prefix of *b: p invalid, even though the Node did not move
use(p)                 // rejected; form a new reference from c

p: &Int
{ b = Box::new(...)?; p = &(*b).value }   // b's scope ends here and b is released
use(p)                 // rejected: p's root has gone out of scope
```

Validity is re-established only by forming the reference again. A move never re-roots an existing reference: after `w = move v`, references formed from `v` are invalid and are not reinterpreted as references into `w`.

## Rule 4. References are never stored, returned, or captured into anything that escapes

A reference may be bound to a local, passed as a call argument, and used within the function that formed it or received it. It cannot be assigned into any aggregate (Rule 1), cannot be returned, and cannot be captured by a function value that is stored or returned. A function that "finds" something returns an index or other owned data; the caller forms the reference.

```text
fn find(v: &Slots<Entry, N>, key: Int) -> Option<u64>      // returns an index, never &Entry
    reads(v)
    contract { ensures when Some: result < len_of(v); }

match find(&v, k) { Some(i) => { p = &v[i]; ... }  None => ... }   // caller re-derives, one address computation
```

## Rule 5. Box<T> is the only heap marker

`Box<T>` owns exactly one heap object. There is one heap; Boxes carry no store or region brand and can be moved, stored in aggregates, and returned freely. `*b` is a path (Rule 2). A Box is affine: at scope exit the compiler releases its memory recursively (Rule 8); there are no destructors.

The content type may be any `T`, including the three runtime-capacity shapes `Array<T>`, `Slots<T>`, and `Ring<T>` (Rule 6), which have no compile-time size and may appear only as the content of a Box: never inline in another value and never as a local variable.

```text
b = Box::new(Node { ... })?         // allocation can fail: Result<Box<Node>, Oom>
p = &(*b).left                       // reference into the heap object
c = move b                           // p invalid (Rule 3); the heap object did not move
holder.child = move c                // a Box in a struct field: ordinary ownership
buf: Box<Slots<u8>>                  // a window of runtime capacity, on the heap
tags: Box<Array<u8>>                 // a fully initialized block of runtime length
```

## Rule 6. Storage shapes: Array, Slots, Ring; the compiler-maintained window

Three shapes, each with a constant-capacity form (may be inline) and a runtime-capacity form (only inside a Box, Rule 5):

```text
Array<T, N>   Box<Array<T>>    every slot always holds a value
Slots<T, N>   Box<Slots<T>>    a window: slots [0, len_of) hold values, [len_of, cap_of) hold nothing
Ring<T, N>    Box<Ring<T>>     a ring window: len_of slots starting at head_of, wrapping at cap_of
```

Measures: `len_of`, `cap_of`, `room_of` (equal to `cap_of - len_of`); `head_of` on rings only; for an `Array`, `len_of == cap_of`. The window boundary is a runtime number stored with the block, readable by the program, and changed only by the built-in operations below. No slot ever carries a tag, and no program point can observe a slot inside the window as empty.

Construction:

```text
Array::new(v)                        // T Copy: every slot holds v; or a literal [a, b, c]
Box::new(Array::filled(n, v))?       // runtime length, T Copy, every slot holds v; zero-filled maps to calloc
Slots::new<T, N>()                   // empty window
Box::new(Slots::new<T>(cap))?        // empty window of runtime capacity; Result
Slots::from_array(move a)            // a full window; Slots::into_array(move r) requires len_of(r) == N
```

Window operations, shared by `Slots` and `Ring`, with slot-precise effects (`r` stands for the storage, e.g. `*b`):

```text
place_back(&r, x)          writes(r[len_of(r)]), writes(len_of(r))
    contract { requires room_of(r) > 0;  ensures len_of(r) == len_of(entry(r)) + 1; }
take_back(&r) -> own T     writes(r[len_of(r) - 1]), writes(len_of(r))
    contract { requires len_of(r) > 0;   ensures len_of(r) == len_of(entry(r)) - 1; }
set(&r[k], x)              writes(r[k])      // old value: affine released, linear rejected
replace(&r[k], x) -> own T writes(r[k])      // the old value is returned
r[k] = f(r[k])             writes(r[k])      // atomic in-place update: the old value enters f by value,
                                             // the result is committed, no program point lies between
                                             // the three require k < len_of(r)
place_front(&r, x)         writes(r)         // Ring only; every logical index shifts, so all references into r die
take_front(&r) -> own T    writes(r)         // Ring only
```

A reference `p = &r[i]` into a window is formed under the fact `i < len_of(r)` and stays valid while that fact holds: `place_back`'s `ensures` carries it across the call; `take_back`'s does not, so p dies at a `take_back`. Ring positions are logical indices; the wrap is the storage's business.

Release: at scope exit the compiler releases the slots inside the window recursively and frees the block; `Array` releases every slot. No built-in operation releases a linear element: a storage whose element type is linear is itself linear (Rule 8) and the program must take every element out with `take_back` and consume it.

Library code, all zero-cost compositions of the above: `swap_remove`, `truncate`, `clear`, `Vector<T>` (a `Box<Slots<T>>` plus a `push` that allocates a larger block, moves the window across, and replaces the Box), `Arena<T>` (`place_back` returning the index as allocation, `truncate(0)` as reset), `Pool<T>` (a `Vector` plus a free list as data), `Deque<T>` (`Box<Ring<T>>` plus growth), open-addressing tables (`Box<Array<u8>>` of tags plus `Box<Array<Entry>>` or `Box<Array<Option<Entry>>>` of slots, all fully initialized), `String` (`Box<Slots<u8>>` plus growth).

There is no `take` operation and no partial move out of any place: the only ways to move a value out of storage are consuming a whole local (`move x`), the window operations, and the atomic update. Assigning over any owned place releases the old value if it is affine and is rejected if it is linear.

## Rule 7. What can be indexed, and bounds are proved

Indexable things: `Array`, `Slots`, and `Ring` in either form, a range reference into any of them, and a `const` table. `Box<Array<T, N>>` is indexed through the path `(*b)[i]` and is not a separate case. Every index must be proved in bounds; when the proof is unavailable the program tests the measure, which is ordinary data.

```text
a: Array<Int, 8>;  a[i]                 // requires i < 8
r: Slots<Int, N>;  r[i]                 // requires i < len_of(r)
part = &r[lo..hi]                       // requires lo <= hi <= len_of(r); len_of(part) == hi - lo
part[k]                                 // requires k < len_of(part)
if i < len_of(r) { use(&r[i]) }         // the test establishes the fact; one compare, no trap
```

## Rule 8. Value classes: copy, affine, linear; no destructors

A type is copy, affine, or linear. Copy values may be duplicated. Affine values are consumed at most once; at scope exit the compiler releases their memory recursively (Box, Slots, Ring, Array) and runs no user code. Linear values must be consumed by an explicit operation on every exit path; the compiler never releases them. Linearity is declared on external-resource types (the existing `linear` modifier) and propagates through every aggregate of Rule 1: a struct, enum, tuple, Array, Slots, Ring, or Box containing a linear part is linear.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, n: u64 }      // linear by containment
Box<File>                            // linear: must be taken apart and its File closed
Slots<File, 4>                       // linear: every element must be taken out and closed
```

## Rule 9. Effects are declared only on reference parameters

An effect row lists `reads(path)` and `writes(path)` where each path starts at a reference parameter and may continue through fields, `*`, and whole-index or range positions supplied as arguments. `writes` covers writing, replacing, moving out of, and freeing the storage at the path. A by-value parameter has no effect entry: the call site records the consumption (for `move`) or the read (for a copy) of the argument's place. Signatures never contain index expressions; an index enters an effect only through an argument, evaluated once at the call. The built-in window operations of Rule 6 are the one exception, their rows being fixed by the language.

```text
fn bump(p: &Int)  writes(p)
fn stats(v: &Slots<Int, N>) -> Int  reads(v)
fn update(o: &Obj, c: Bool)  writes(o.a), writes(o.b)      // member paths are allowed
fn put_at(slot: &Int, x: own Int)  writes(slot)              // x needs no entry

bump(&a.x)                     // call effect: writes(a.x)
put_at(&r[k], 3)               // the argument fixes the slot at the call; requires k < len_of(r)
place_back(&r, 3)              // appending is the built-in; its row names the slot at len_of
```

A function body is checked against its own row: every statement's effect and every callee's substituted row must be covered by the declared row.

## Rule 10. The call-site rule

At a call, substitute the actual argument paths into the callee's row. Then:

1. Compare the substituted effects pairwise. Two effects on overlapping paths where at least one is a write must be proved disjoint (different roots, or indices or ranges proved distinct); otherwise the call is rejected. Read/read overlap is allowed.
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
slot = &(*b).next
put(slot, move b)                            // rejected: move b writes the prefix b of slot's path (Rule 3)
```

Function-typed parameters carry a full signature with its own row and contract, and a call through one uses that row. Recursion is checked through contracts, never by unfolding bodies.

## Rule 11. Facts, contracts, and invalidation

Facts are the existing WF forms: affine comparisons over measures and integer values, refinement facts from a dominating branch, loop-header invariants `invariant name: affine_expr compare_op affine_expr`, explicit `use` steps inside an `invariant`, and callee contracts. A fact that mentions a path is invalidated when that path is written (by statement or call) unless the callee's `ensures` re-establishes it. Contracts use `requires`, `ensures`, and `ensures when Variant:` for result-routed relations; the pre-state of a parameter is written `entry(p)`. Across a call, a fact known before the call about a measure of an argument survives as a fact about `entry(p)` of that argument, which is how an `ensures` of the shape `len_of(r) == len_of(entry(r)) + 1` connects to what the caller knew.

```text
n = len_of(r)
if m <= n {
    for i in 0..m {
        invariant h: len_of(r) == n - i;     // carried by the writer, checked at every iteration
        take_back(&r)                        // requires len_of(r) > 0: from i < m and m <= n
    }
}
while len_of(r) > 0 { take_back(&r) }        // no knowledge: the loop condition is the test
```

There are no quantified facts over array elements ("for all i ...") and no per-slot occupancy facts; occupancy that is determined by data is stored as data (Rule 6, Rule 12).

## Rule 12. Control-flow joins and the absence of holes

At a join, facts are intersected (a fact survives only if it holds on every incoming edge); a reference's target set is the union of its targets (Rule 2). No place is ever partially moved: every path is either wholly present or the program cannot name it. Structures whose occupancy is decided by data keep that occupancy as data.

```text
if cond { place_back(&r, x) }               // after the join: len_of(r) >= n, where n was len_of before
                                            // facts about indices below n survive

tags:  Box<Array<u8>>                       // open-addressing table: occupancy is data
slots: Box<Array<Entry>>                    // fully initialized; a "logically empty" slot holds a valid value
slots: Box<Array<Option<Entry>>>            // non-Copy payloads: Option, using a null niche where the type has one
```

## Rule 13. Parallel blocks

`par { A; B }` is accepted when A's write paths are disjoint from B's read and write paths and vice versa, using the same path-overlap and index/range-disjointness judgment as Rule 10. Read/read overlap is allowed. Allocation and release are not effects (Rule 14). The existing counted-loop forms (per-element maps, adjacent ranges passed to a helper, admitted reductions) are expressed with range references.

```text
par { s1 = stats(&v); s2 = stats(&v) }                        // read/read: accepted
par { bump(&r[i]); bump(&r[j]) }                               // needs i != j
par { kernel(&r[0..mid], &out[0..mid]); kernel(&r[mid..n], &out[mid..n]) }   // disjoint ranges: accepted
par { push(&v, 1); stats(&v) }                                 // rejected: writes(*v) overlaps reads(*v)
```

## Rule 14. One global heap; allocation can fail; the allocator has no effect

There is one heap, provided by the trusted base, internally synchronized. Allocation and release carry no effect entry and never make two parallel arms conflict. Allocation returns a `Result` and never traps. Addresses are not observable, so allocator concurrency does not affect program determinism. There are no store or region parameters anywhere.

```text
par { a = build(&x)?; b = build(&y)? }     // both allocate: accepted
Box::new(v)?                                // Result on every allocation
```

## Rule 15. Return values are owned

A function returns owned values only. Positions found by a search are returned as indices with a bounds relation in `ensures` (Rule 4). Passing a large value in and back out is expressed as a reference parameter with a `writes` entry, not as move-and-return.

## Rule 16. Pools and arenas are usage, not types

Confirmed by the owner on 2026-09-18, knowingly reversing the design tree's earlier rejection of the arena-index-pool pattern. A pool is a `Slots` (or `Box<Slots<T>>`) plus indices used as handles; an arena is the same storage used with `place_back` as allocation and `truncate(0)` as reset. A stale index that is still in bounds names the current occupant of that slot: a logic error, not a memory error. Programs that need to detect it keep a generation number as data. The same holds for a reference into a user-level pool or allocator: storage that still exists stays readable, and what the program considers "freed" is the program's own bookkeeping.

```text
struct Arena<T> { buf: Box<Slots<T>> }
fn alloc<T>(a: &Arena<T>, x: own T) -> u64   writes((*a.buf)[len_of(*a.buf)]), writes(len_of(*a.buf))
    contract { requires room_of(*a.buf) > 0; ensures result == len_of(entry(*a.buf)); ensures len_of(*a.buf) == result + 1; }
id = alloc(&a, node)
p  = &(*a.buf)[id]                         // valid while id < len_of(*a.buf)
truncate(&a.buf, 0)                        // reset: elements released; later reads of slot id need id < len_of again
```

## Not in this candidate

`with` blocks or any block-scoped reference form; `&uniq`, `&mut`, or any permission marker on references; lifetimes, regions, `region` statements, region parameters; store brands (`'s`), `Heap<'s>`, `Arena<'s, ...>`, `Box<'s, T>`, `Vector<'s, T>`, providers, `dispose`, capability-based linearity; `allocates` effects; slice types as first-class values; returned references; destructors; runtime traps; `take`/`put` holes; quantified invariants over elements; per-slot occupancy tags maintained by the compiler; channels or atomics; header-plus-tail heap blocks. Retired names from earlier drafts: `DynBox`, `FixedVector`, `Run`, `HeapSlots`, `HeapRing`, `DynSlots`, `DynArray`, `DynRing`, `buffer`.

## Proposed additions awaiting owner ruling (from matrix round one; not rules)

Each states what it adds and why it is worth adding. See `REVIEW-X1-round1.md` for the programs that motivated them.

1. Fallible allocation hands the payload back: `Box::new(x) -> Result<Box<T>, (Oom, T)>`. Without it a linear value moved into a failed allocation is lost with no way to close it. Zero cost on the success path.
2. Whole-local destructuring: `let Conn { f, n } = move c` and `unbox(move b)` consume a local and bind its parts as locals, without ever leaving a hole. This is the only way to take a linear aggregate apart (Rule 8's own `Box<File>` example needs it). Zero cost.
3. Static path shape: a loop-carried rebinding of a reference may change only index values, never extend its own path (`p = &p.kids[0]` in a loop is rejected). Without it the target set is unbounded. Equal-cost forms: recursion, or indices into a pool.
4. The atomic update defined in full: `place = f(place, args...)` for any owned place; `f` is total and returns the element type; `f`'s row must not overlap any prefix of `place`; failure is expressed by an enum stored in the place, never by `?`.
5. A parameter type for range references, spelled `&[T]`: a reference kind with measure `len_of`, formed only by `&x[lo..hi]` or from another range reference, never a stored value. Rule 13's own example needs it.
6. `par` post-state: arms cannot exit early (`return`, `break`, `?` may not leave an arm); each arm's result is an owned value; after the block both arms' `ensures` hold; a linear value moved into an arm must be consumed inside it.
7. Variant payload paths: `match &e { A(x) => ... }` binds `x` as a reference to the payload path under the refinement fact `is_A(e)`, invalidated by any write to `e`. Without it a Box-linked tree cannot hold two `Option<Box<Node>>` children inline and pays a child window instead.
8. Bulk window operations: `exchange(&r, i, j)`, `insert_at(&r, k, x)`, `remove_at(&r, k) -> T`, `append(&dst, &src)` (moves `src`'s window onto `dst`'s back, `src` becomes empty), and `grow(&b, cap)` on `Box<Slots<T>>` (may reallocate in place; every reference into it dies). Without them in-place sort and partition of owning elements cost 9 moves per swap, ordered removal 3 moves per shifted element, and order-preserving growth 2.5 moves per element; with them each is one memmove or realloc, valid because Rule 1 makes every value relocatable.
9. An `appends(path)` effect kind for user functions that only append to a window, so references into the window survive such a call across a separate-compilation boundary. Optional; the built-ins already have slot-precise rows.

## Deferred to a future concurrency and layout round

- A channel primitive in the trusted base (ownership-transfer queue) for producer/consumer pipelines and work stealing.
- A heap block with a fixed header followed by a runtime-length tail in one allocation.
- A bitmask fact (`x & (c - 1) < c` for a power-of-two `c`) to remove the per-probe bounds compare in hash tables.

## Known costs already recorded

- One compare and one predicted branch per data-determined index (graph hop, free-list pop, hash probe); equal to safe Rust, real against C++.
- Parallel scatter to data-determined destinations is refused; invert-and-gather or bucket-first costs one extra pass; equal to safe Rust.
- Find-then-mutate on a Box-linked tree re-descends once unless fused, passed a function-typed parameter, or replaced by a pool index.
- Construction into the append slot moves one element where C++ constructs in place; usually elided by the backend, not guaranteed.
- Open-addressing tables with non-Copy payloads pay one null check per hit versus hashbrown.
- Lock-free rings and work-stealing queues are not expressible; batched fork-join is the available form.
- Header-plus-tail layouts pay one extra dependent memory access per hop.
- Pool allocation from one pool serializes against every other access to that pool; per-worker pools are the standard form. Generation words for identity across slot reuse cost what a Rust slotmap costs.

## Protocol for a derivation cell or task under this candidate

1. State the question the cell tests, in one sentence.
2. Write the strongest program first: the case where static analysis provably cannot help (data-determined indices, lost conditions, mixing, separate compilation, joins), not a convenient variant. Then the ordinary case.
3. Trace acceptance or rejection rule by rule, quoting the rule number and the exact clause used. Do not add rules. If the text is ambiguous, quote the sentence and mark the cell `undecided-rule-gap`.
4. For every rejection, give the best rewrite under these rules and its runtime cost relative to the C++ or Rust form, naming the extra instruction, load, copy, allocation, branch, or lost parallelism. A rewrite with no runtime cost is `rejected-zero-cost`; one with unavoidable cost is `rejected-real-cost`.
5. Record separately whether the rules are consistent and whether the original task is still achievable; a rule set that rejects a task cleanly has not achieved it.
6. Verdict vocabulary: `accepted-fine`, `accepted-unsafe`, `rejected-zero-cost`, `rejected-real-cost`, `undecided-rule-gap`.
7. Runtime performance is the only cost that counts; verbosity, extra parameters, and duplicated code are not costs.
