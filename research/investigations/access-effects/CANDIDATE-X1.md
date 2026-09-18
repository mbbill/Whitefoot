# Candidate x1: frozen rule set (2026-09-18)

Status: every rule below was confirmed by the owner in the 2026-09-17/18 sessions. Items under "Pending owner ruling" and "Deferred" are not rules. Nothing else is part of the candidate; in particular the mechanisms listed under "Not in this candidate" must not be assumed by anyone deriving under it.

Evaluation criteria, in the owner's words: safety through types and proofs at the current WF level (no runtime traps, no unsafe), runtime performance for single-thread and parallel computation, and enough expressiveness to build compilers, browsers, and kernels. Code is written by AI, so an ugly but equal-performance form is not a defect. A rejection counts as expressiveness loss only when no equal-performance rewrite exists.

Notation: design pseudocode, not current WF syntax except where stated. `&path` forms a reference; `move x` consumes an owned value; `own T` is a by-value parameter; an effect row follows the signature; `contract { requires ...; ensures [when Variant:] ...; }` is the existing WF contract block; measures `len_of`, `cap_of` are the existing WF measures; `entry(p)` denotes the pre-state of parameter `p` as in the existing spec; `Int` and `u64` are Copy.

## Rule 1. Values have value semantics; no aggregate ever contains a reference

Every struct, enum, tuple, array, slice-like value, Box, DynBox, and generic instantiation holds only owned values. This is recursive and closed under wrapping: there is no type parameter, wrapper, or variant payload through which a reference can be stored.

```text
struct Node { value: Int, left: Option<Box<Node>> }     // owned children: allowed
struct Bad  { r: &Int }                                  // rejected
enum   Bad2 { A(&Int), B }                               // rejected
Option<&T>                                               // rejected, whatever T is
```

Consequence: any value can be relocated by copying its bytes (memmove, realloc), because nothing inside it points anywhere.

## Rule 2. A reference is a local name for a path

A path starts at a local variable or a parameter and continues through fields, `*` (Box content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7). A reference variable names a path; it is not storage of its own.

```text
p = &v.buf[i]          // p names the path v.buf[i]
q = p                  // q names the same path; both may be live
r = &p.x               // r names v.buf[i].x; a reference extends a path, it does not point at p
&p                     // rejected: a reference variable is not storage
```

A reference may be rebound. At a control-flow join, a reference variable's target is the set of paths it may name; every check on it must hold for every member of the set.

```text
w = if cond { &v1 } else { &v2 }     // w names one of {v1, v2}
p = &w.buf[i]                        // p names one of {v1.buf[i], v2.buf[i]}
```

There is no `uniq` or `mut` marker on references. Whether a callee may write through a reference parameter is stated by its effect row (Rule 9).

## Rule 3. Reference validity is a fact

"p is valid" is a fact like any other. It is established when p is formed and invalidated when any proper prefix of p's path is written, moved out of, replaced, or freed, by a statement or by a call. Writing the storage at p's path or below it (a content write) does not invalidate p. Using an invalid reference is rejected.

```text
p = &v.buf[i]
v.buf[j] = 5           // content write on a sibling or the same slot: p stays valid
grow(&v)               // declares writes(v.buf): v.buf is a proper prefix of v.buf[i]: p invalid
use(p)                 // rejected

p = &*b                // b: Box<Node>; p names the heap Node
c = move b             // b is a proper prefix of *b: p invalid, even though the Node did not move
use(p)                 // rejected; form a new reference from c
```

Validity is re-established only by forming the reference again. A move never re-roots an existing reference: after `w = move v`, references formed from `v` are invalid and are not reinterpreted as references into `w`.

## Rule 4. References are never stored, returned, or captured into anything that escapes

A reference may be bound to a local, passed as a call argument, and used within the function that formed it or received it. It cannot be assigned into any aggregate (Rule 1), cannot be returned, and cannot be captured by a function value that is stored or returned. A function that "finds" something returns an index or other owned data; the caller forms the reference.

```text
fn find(v: &DynBox<Entry>, key: Int) -> Option<u64>      // returns an index, never &Entry
    reads(v)
    contract { ensures when Some: result < len_of(v); }

match find(&v, k) { Some(i) => { p = &v[i]; ... }  None => ... }   // caller re-derives, one address computation
```

## Rule 5. Box<T> is an owning pointer on the single global heap

`Box<T>` owns exactly one heap object of fixed size. There is one heap; Boxes carry no store or region brand and can be moved, stored in aggregates, and returned freely. `*b` is a path (Rule 2). A Box is affine: at scope exit the compiler releases its memory recursively (Rule 8); there are no destructors.

```text
b = Box::new(Node { ... })?         // allocation can fail: Result<Box<Node>, Oom>
p = &(*b).left                       // reference into the heap object
c = move b                           // p invalid (Rule 3); the heap object did not move
holder.child = move c                // a Box in a struct field: ordinary ownership
```

## Rule 6. DynBox<T>: heap block with a compiler-maintained window

`DynBox<T>` owns one heap block of `cap_of` slots, fixed at allocation. Slots `[0, len_of)` hold values; slots `[len_of, cap_of)` hold nothing. The boundary `len_of` is a runtime number stored in the block header, readable by the program, and changed only by the built-in operations below. No slot ever carries a tag, and no program point can observe a slot inside the window as empty.

```text
buf = DynBox::new<T>(n)?             // cap_of == n, len_of == 0
buf = DynBox::filled(n, v)?          // T Copy: cap_of == len_of == n, every slot holds v

fn push_nogrow<T>(buf: &DynBox<T>, x: own T)  writes(buf)
    contract { requires len_of(buf) < cap_of(buf); ensures len_of(buf) == len_of(deref(entry(buf))) + 1; }
fn pop<T>(buf: &DynBox<T>) -> own T           writes(buf)
    contract { requires len_of(buf) > 0;          ensures len_of(buf) == len_of(deref(entry(buf))) - 1; }
fn swap_remove<T>(buf: &DynBox<T>, k: u64) -> own T   writes(buf)
    contract { requires k < len_of(buf);          ensures len_of(buf) == len_of(deref(entry(buf))) - 1; }
fn truncate<T>(buf: &DynBox<T>, n: u64)       writes(buf)
    contract { requires n <= len_of(buf);         ensures len_of(buf) == n; }
buf[k] = f(buf[k])                   // atomic in-place update: the old value goes into f by value, f's result is
                                     // committed, no program point lies between; requires k < len_of(buf)
```

Reading `buf[k]` or forming `&buf[k]` requires the fact `k < len_of(buf)` (Rule 7). Assigning `buf[k] = x` where the old value is affine releases the old value; where it is linear the assignment is rejected unless written as the atomic update whose function consumes the old value.

At scope exit the compiler releases slots `[0, len_of)` recursively and frees the block. A DynBox whose element type is linear is itself linear (Rule 8) and must be truncated to zero by the program before it can go out of scope.

`Vector<T>` is library code: a DynBox plus a `push` that, when `len_of == cap_of`, allocates a larger DynBox, moves `[0, len_of)` across, and replaces the old one. There is no `take` operation and no partial move out of any place: the only ways to move a value out of storage are consuming a whole local (`move x`), the window operations, and the atomic update.

## Rule 7. What can be indexed, and bounds are proved

Indexable things: an inline `array<T, N>` (constant length, always fully initialized), a `DynBox<T>` (Rule 6), a range reference into either, and a `const` table. `Box<array<T, N>>` is indexed through the path `(*b)[i]` and is not a separate case. Every index must be proved in bounds; when the proof is unavailable the program tests the measure, which is ordinary data.

```text
a: array<Int, 8>;  a[i]              // requires i < 8
buf: DynBox<Int>;  buf[i]            // requires i < len_of(buf)
part = &buf[lo..hi]                  // requires lo <= hi <= len_of(buf); len_of(part) == hi - lo
part[k]                              // requires k < len_of(part)
if i < len_of(buf) { use(&buf[i]) }  // the test establishes the fact; one compare, no trap
```

## Rule 8. Value classes: copy, affine, linear; no destructors

A type is copy, affine, or linear. Copy values may be duplicated. Affine values are consumed at most once; at scope exit the compiler releases their memory recursively (Box, DynBox) and runs no user code. Linear values must be consumed by an explicit operation on every exit path; the compiler never releases them. Linearity is declared on external-resource types and propagates through aggregates: a struct, array, Box, or DynBox containing a linear part is linear.

```text
linear type File;   fn close(f: own File)
struct Conn { f: File, n: u64 }      // linear by containment
fn use_it(c: own Conn) { ...; close(move c.f) }   // consuming the struct as a whole and closing its File is required
Box<File>                            // linear: must be taken apart and its File closed
```

## Rule 9. Effects are declared only on reference parameters

An effect row lists `reads(path)` and `writes(path)` where each path starts at a reference parameter and may continue through fields, `*`, and whole-index or range positions supplied as arguments. `writes` covers writing, replacing, moving out of, and freeing the storage at the path. A by-value parameter has no effect entry: the call site records the consumption (for `move`) or the read (for a copy) of the argument's place. Signatures never contain index expressions; an index enters an effect only through an argument, evaluated once at the call.

```text
fn bump(p: &Int)  writes(p)
fn stats(v: &DynBox<Int>) -> Int  reads(v)
fn update(o: &Obj, c: Bool)  writes(o.a), writes(o.b)      // member paths are allowed
fn put_at(slot: &Int, x: own Int)  writes(slot)              // x needs no entry

bump(&a.x)                     // call effect: writes(a.x)
put_at(&buf[len_of(buf)], 3)   // the argument fixes the slot at the call; the row itself names no index
```

A function body is checked against its own row: every statement's effect and every callee's substituted row must be covered by the declared row.

## Rule 10. The call-site rule

At a call, substitute the actual argument paths into the callee's row. Then:

1. Compare the substituted effects pairwise. Two effects on overlapping paths where at least one is a write must be proved disjoint (different roots, or indices or ranges proved distinct); otherwise the call is rejected. Read/read overlap is allowed.
2. A by-value argument contributes a consumption (`move`) or a read (copy) of its place to this comparison.
3. A live reference outside the call whose path has a proper prefix among the call's write paths becomes invalid after the call (Rule 3). A reference that is itself an argument is the thing being accessed, not a bystander.

```text
fn two(p: &Int, q: &Int)  writes(p), reads(q)
two(&a, &a)                                  // rejected: writes(a) and reads(a) overlap
two(&a.x, &a.y)                              // accepted: distinct fields
two(&v[i], &v[j])                            // accepted only with the fact i != j

fn bad(outer: &DynBox<DynBox<Int>>, inner: &DynBox<Int>)  writes(outer), writes(inner)
bad(&vv, &vv[0])                             // rejected: writes(vv) and writes(vv[0]) overlap

fn put<T>(slot: &T, value: own T)  writes(slot)
slot = &(*b).next
put(slot, move b)                            // rejected: move b writes the prefix b of slot's path (Rule 3)
```

Function-typed parameters carry a full signature with its own row and contract, and a call through one uses that row. Recursion is checked through contracts, never by unfolding bodies.

## Rule 11. Facts, contracts, and invalidation

Facts are the existing WF forms: affine comparisons over measures and integer values, refinement facts from a dominating branch, loop-header invariants `invariant name: affine_expr compare_op affine_expr`, explicit `use` steps inside an `invariant`, and callee contracts. A fact that mentions a path is invalidated when that path is written (by statement or call) unless the callee's `ensures` re-establishes it. Contracts use `requires`, `ensures`, and `ensures when Variant:` for result-routed relations; the pre-state of a parameter is written `entry(p)`.

```text
n = len_of(buf)
if m <= n {
    for i in 0..m {
        invariant h: len_of(buf) == n - i;   // carried by the writer, checked at every iteration
        pop(&buf)                            // requires len_of(buf) > 0: from i < m and m <= n
    }
}
while len_of(buf) > 0 { pop(&buf) }          // no knowledge: the loop condition is the test
```

There are no quantified facts over array elements ("for all i ...") and no per-slot occupancy facts; occupancy that is determined by data is stored as data (Rule 6, Rule 12).

## Rule 12. Control-flow joins and the absence of holes

At a join, facts are intersected (a fact survives only if it holds on every incoming edge); a reference's target set is the union of its targets (Rule 2). No place is ever partially moved: every path is either wholly present or the program cannot name it. Structures whose occupancy is decided by data keep that occupancy as data.

```text
if cond { push_nogrow(&buf, x) }            // after the join: len_of(buf) >= n, where n was len_of before
                                            // facts about indices below n survive

tags:  DynBox<u8>                           // open-addressing table: occupancy is data
slots: DynBox<Entry>                        // fully initialized (zero-filled); a "logically empty" slot holds a valid value
slots: DynBox<Option<Entry>>                // non-Copy payloads: Option, using a null niche where the type has one
```

## Rule 13. Parallel blocks

`par { A; B }` is accepted when A's write paths are disjoint from B's read and write paths and vice versa, using the same path-overlap and index/range-disjointness judgment as Rule 10. Read/read overlap is allowed. Allocation and release are not effects (Rule 14). The existing counted-loop forms (per-element maps, adjacent ranges passed to a helper, admitted reductions) are expressed with range references.

```text
par { s1 = stats(&v); s2 = stats(&v) }                       // read/read: accepted
par { bump(&v[i]); bump(&v[j]) }                              // needs i != j
par { kernel(&v[0..mid], &out[0..mid]); kernel(&v[mid..n], &out[mid..n]) }   // disjoint ranges: accepted
par { push(&v, 1); stats(&v) }                                // rejected: writes(v) overlaps reads(v)
```

## Rule 14. One global heap; allocation can fail; the allocator has no effect

There is one heap, provided by the trusted base, internally synchronized. Allocation and release carry no effect entry and never make two parallel arms conflict. Allocation returns a `Result` and never traps. Addresses are not observable, so allocator concurrency does not affect program determinism. There are no store or region parameters anywhere.

```text
par { a = build(&x)?; b = build(&y)? }     // both allocate: accepted
Box::new(v)?  DynBox::new(n)?               // Result on every allocation
```

## Rule 15. Return values are owned

A function returns owned values only. Positions found by a search are returned as indices with a bounds relation in `ensures` (Rule 4). Passing a large value in and back out is expressed as a reference parameter with a `writes` entry, not as move-and-return.

## Rule 16 (provisional, pending owner ruling). Pools and arenas are usage, not types

A pool is a DynBox plus indices used as handles; an arena is a DynBox used with `push_nogrow` as allocation and `truncate(0)` as reset. A stale index that is still in bounds names the current occupant of that slot: a logic error, not a memory error. Programs that need to detect it keep a generation number as data. This reverses the current design tree's rejection of the arena-index-pool pattern; the owner has not yet ruled. Cells and tasks under this candidate use it as stated here and mark their dependence on it.

```text
struct Arena<T> { buf: DynBox<T> }
fn alloc<T>(a: &Arena<T>, x: own T) -> u64   writes(a.buf)
    contract { requires len_of(a.buf) < cap_of(a.buf); ensures result == len_of(deref(entry(a.buf))); ensures len_of(a.buf) == result + 1; }
id = alloc(&a, node)
p  = &a.buf[id]                            // valid until a.buf is replaced; readable while id < len_of(a.buf)
truncate(&a.buf, 0)                        // reset: elements released; later reads of a.buf[id] need id < len_of again
```

## Not in this candidate

`with` blocks or any block-scoped reference form; `&uniq`, `&mut`, or any permission marker on references; lifetimes, regions, `region` statements, region parameters; store brands (`'s`), `Heap<'s>`, `Arena<'s, ...>`, `Box<'s, T>`, `Vector<'s, T>`, providers, `dispose`, capability-based linearity; `allocates` effects; slice types as first-class values; returned references; destructors; runtime traps; `take`/`put` holes; quantified invariants over elements; per-slot occupancy tags maintained by the compiler; channels or atomics; header-plus-tail heap blocks.

## Pending owner ruling

- Rule 16 (pool and arena as usage; stale index is a logic error), which reverses the recorded rejection of the arena-index-pool pattern.
- Whether `FixedVector<T, n>` stays built in as the inline constant-capacity form of the Rule 6 window, or becomes library code over `array<T, N>` plus a count.

## Deferred to a future concurrency and layout round

- A channel primitive in the trusted base (ownership-transfer queue) for producer/consumer pipelines.
- A heap block with a fixed header followed by a runtime-length tail in one allocation.

## Known costs already recorded

- Open-addressing hash tables with non-Copy payloads pay one null check per hit versus hashbrown; measure before deciding.
- Lock-free rings are not expressible; batched fork-join with two buffers is the available form.
- Structures needing a header-plus-tail layout pay one extra dependent memory access per hop.

## Protocol for a derivation cell or task under this candidate

1. State the question the cell tests, in one sentence.
2. Write the strongest program first: the case where static analysis provably cannot help (data-determined indices, lost conditions, mixing, separate compilation, joins), not a convenient variant. Then the ordinary case.
3. Trace acceptance or rejection rule by rule, quoting the rule number and the exact clause used. Do not add rules. If the text is ambiguous, quote the sentence and mark the cell `undecided-rule-gap`.
4. For every rejection, give the best rewrite under these rules and its runtime cost relative to the C++ or Rust form, naming the extra instruction, load, copy, allocation, branch, or lost parallelism. A rewrite with no runtime cost is `rejected-zero-cost`; one with unavoidable cost is `rejected-real-cost`.
5. Record separately whether the rules are consistent and whether the original task is still achievable; a rule set that rejects a task cleanly has not achieved it.
6. Verdict vocabulary: `accepted-fine`, `accepted-unsafe`, `rejected-zero-cost`, `rejected-real-cost`, `undecided-rule-gap`.
7. Runtime performance is the only cost that counts; verbosity, extra parameters, and duplicated code are not costs.
