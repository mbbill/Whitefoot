# Candidate x1, matrix round one: reviewer's re-check (2026-09-18)

This file records my own re-trace of every item the gap judge left standing in `matrix-x1/GAPS-X1.md`, against the rule text in `CANDIDATE-X1.md` as frozen for the run and against the container decisions the owner made while the run was in progress (four container names, slot-precise effects for the window operations, `truncate` and `swap_remove` as library code). Nothing here is a rule; the owner rules on the items under "Needs an owner decision".

Totals reported by the judge over 136 cells: 44 accepted, 28 rejected with a zero-cost rewrite, 40 rejected with a real cost, 19 undecided, 5 claimed unsafe acceptances. Of the 23 gap groups, the judge itself dismissed 10 as covered by the text. All 8 engineering tasks are achievable; one at zero cost (array kernel), seven with costs listed below.

## A. Memory-safety holes in the text as frozen: three, all closed by one clause each

A1. Prefix overlap without an index-distinctness proof.

```text
q = &(*g[j]).value           // g: Box<Slots<Box<Node>>>, j from data
replace_at(&g, i, nb)        // writes g[i]; if i == j at run time the old Node is freed
y = (*q).value               // the frozen text does not list g[i] among q's prefixes
```

The frozen Rule 3 and Rule 10 clause 3 speak of a "proper prefix" without saying that two indexed paths overlap unless their indices are proved distinct, which Rule 10 clause 1 already demands for the pairwise check. Fix: one sentence stating that prefix and overlap are judged with the same may-overlap rule everywhere. This is the owner's own earlier ruling (`f(&a[i]); g(&a[j])` needs `i != j`), so it is a text fix, not a new decision.

A2. Index expressions in a formed path are snapshots.

```text
p = &buf[i]; i = 1000000; put_at(p, 7)
```

Rule 2 says "p names the path v.buf[i]" without saying whether `i` is re-read. It is the value at formation; the address is computed then. Text fix.

A3. Scope exit of the root local.

```text
p: &Int
{ b = Box::new(Node { value: 7 })?; p = &(*b).value }   // b released at the closing brace
*out = *p                                                // freed heap storage
```

Rule 3 lists invalidation "by a statement or by a call"; the compiler-derived release at scope exit is neither, and no sentence says a reference dies with its root local. Fix: a reference is invalid once the scope of the local its path starts at has ended. Text fix.

## B. Resource-obligation holes: two, both closed by the already-decided container table plus one signature change

B1. `truncate` on a window of linear elements (`Slots<File>`) silently discarded them. `truncate` is now library code written as a loop of `take_back`, which returns each value to the caller, so a linear element type forces the caller to consume each one. The built-in scope-exit release of a window is refused for linear elements by Rule 8 as written. Closed by construction; the rule file must say that no built-in operation releases a linear element.

B2. A linear payload lost on a failed allocation.

```text
b = Box::new(Conn { f: move f, n })?     // Err arm: the File is inside a value nobody can name
```

Fix: a fallible allocation that takes a by-value payload returns it on failure, `Result<Box<T>, (Oom, T)>`; zero cost on the success path. Needs an owner decision (signature shape), listed below.

## C. Inconsistencies inside the frozen text, fixed in the rewrite

C1. Rule 8's example `close(move c.f)` moves a field out of an owned local, which Rule 6 and Rule 12 forbid. The rewrite adds whole-local destructuring (`let Conn { f, n } = move c`; `unbox(move b)`), which takes a value apart without ever leaving a hole. Needs an owner decision (new form).

C2. Rule 9's example `put_at(&buf[len_of(buf)], 3)` forms a reference to the slot at `len_of`, which is outside the window and cannot be referenced. Construction into the append slot is the built-in `place_back`, whose effect is slot-precise. Text fix.

C3. Rule 8's containment list omitted enums and tuples; Rule 1's list governs. Text fix.

C4. Pre-call facts survive as facts about `entry(p)` after a call whose `ensures` mentions `entry`; the existing specification already works this way (two terms per measure). Text fix.

C5. Assignment over any owned place releases the old affine value and is rejected for a linear one; the frozen text said so only for `buf[k]`. Text fix.

## D. Expressiveness gaps that need a decision

D1. A reference rebound through its own path inside a loop (`p = &p.kids[0]`) has no finite target set. Proposed rule: a path has a static shape; a loop-carried rebinding must keep the shape and may change only index values; otherwise rejected. Equal-cost forms: recursion, or indices into a pool.

D2. The atomic in-place update was defined by one example. Proposed definition: `place = f(place, args...)` for any owned place, `f` total and returning the element type, `f`'s effect row not overlapping any prefix of `place`; failure is expressed by returning an enum into the place, not by `?`.

D3. A range reference has no parameter type. Proposed spelling: `&[T]`, a reference kind with measure `len_of`, never a stored value, formed only by `&x[lo..hi]` or from another range reference.

D4. `par` has an acceptance test but no post-state. Proposed: arms are blocks that cannot exit early (no `return`, `break`, or `?` leaving the arm); each arm's result is an owned value; after the block both arms' `ensures` hold; a linear value moved into an arm must be consumed inside it.

D5. Variant payload paths. The frozen path grammar had no way to name the payload of `Option<Box<Node>>`, so a Box-linked node needed a child window. Proposed: `match &e { A(x) => ... }` binds `x` as a reference to the payload path under the refinement fact `is_A(e)`, invalidated by any write to `e`.

D6. Bulk window operations. Without an exchange operation, in-place sort and partition of owning elements cost 9 moves per swap, ordered removal costs 3 moves per shifted element, and order-preserving growth costs 2.5 moves per element. Proposed built-ins on `Slots` and `Box<Slots<T>>`: `exchange(&r, i, j)`, `insert_at(&r, k, x)`, `remove_at(&r, k) -> T`, `append(&dst, &src)` (moves `src`'s window onto `dst`'s back, `src` becomes empty), and `grow(&b, cap)` on `Box<Slots<T>>` (may reallocate in place; every reference into it dies). All are memmove or realloc at run time, valid because Rule 1 makes every value relocatable.

D7. An `appends(path)` effect kind for user functions that only append to a window, so that references into the window survive such a call across a separate-compilation boundary. Optional; the built-ins already have slot-precise rows.

## E. Real costs that survive, with the rule responsible

E1. One compare and one predicted branch per data-determined index (graph hop, free-list pop, hash probe). Rule 11 has no quantified facts and no bitmask fact. Equal to safe Rust; real against C++. A `x & (c - 1) < c` fact for power-of-two `c` would remove it on hash probes; not proposed now.

E2. Parallel scatter to data-determined destinations is refused (injectivity is a quantified fact). Rewrites: invert-and-gather or bucket-first, each one extra pass. Equal to safe Rust.

E3. Find-then-mutate on a Box-linked tree re-descends once, or uses a function-typed parameter (zero cost when the generic is instantiated per call, which the existing FN rules do), or a pool index.

E4. Construction into the append slot copies one element (`place_back` takes the value by move) where C++ constructs in place; usually elided by the backend, not guaranteed.

E5. Growth copies the whole window unless `grow` (D6) is adopted; zero-filled `Box<Array<T>>` maps to calloc and gets lazy pages.

E6. Pool allocation from one pool serializes against every other access to that pool; per-worker pools are the standard form. Generation words for identity across slot reuse are the program's choice and cost what a Rust slotmap costs. Neither is a language cost.

E7. A user-level allocator over a region cannot revoke references into a block it has freed; references are to storage, and storage that still exists stays readable. Memory-safe, identity-unsafe, consistent with the owner's ruling on stale indices; a freed block's contents are the program's problem.

## F. Judge findings I dismiss

F1. The claimed unsafe `par { drain_one(&v[0..mid]); drain_one(&v[mid..n]) }` needs `pop` on a range reference; `pop` takes the container, not a range, so it cannot be written. Agrees with the judge.

F2. The closure-capture "race" reported by four cells: a function value's body is checked against its own row, and a captured reference cannot appear in that row, so the body is rejected. Agrees with the judge.

F3. Cost cluster "every window operation declares the whole container": true for the frozen text, false for the decided container table, whose window operations have slot-precise effects (`place_back` writes `r[len_of(r)]` and `len_of`; `take_back` moves out `r[len_of(r) - 1]` and writes `len_of`; `set`, `replace`, and the atomic update write `r[k]`). Only the separate-compilation residue in D7 remains.

F4. Stale indices after slot reuse and after pop-then-push are the owner's ruling (a logic error), not holes.
