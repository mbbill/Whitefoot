# Candidate x1, targeted round two: reviewer's re-check (2026-09-18)

Round two re-derived the 66 cells whose round-one outcome revision 4 could change (four themed agents, two verifiers), updated the four most affected engineering tasks, and had a judge trace every round-one hole against revision 4. Raw files are in `matrix-x1-r2/`; the judge's report is `matrix-x1-r2/GAPS-X1-r2.md`. This file is my own re-trace of what the judge left standing, against revision 4, and the disposition of each item in revision 5.

Totals over the 66 re-derived cells: 30 accepted, 13 rejected with a zero-cost rewrite, 16 rejected with a real cost, 2 undecided, 5 claimed unsafe (all five are the deliberate stale-handle disposition of Rule 16). Round one's three memory-safety holes, both resource-obligation holes, and all five internal inconsistencies are closed by the sentences revision 4 added; the judge quotes each. Round one's two largest cost clusters (whole-container window rows; no exchange or bulk move) are closed by the window parts, `swap`, and the bulk operations.

## A. New holes opened by revision 4's own text, closed in revision 5 by wording

A1. Window parts read as places.

```text
b: Box<Slots<Entry>>            // len 0
k = peek(&(*b).next)            // Rule 6 said parts are names "paths and effect rows may name"
stash(&(*b).next, move e)       // assigning over an "old value" read from uninitialized storage
```

Intended: parts are vocabulary for effect rows and the overlap judgment only; no program forms a reference to a part, reads it, or writes it; the append slot is written only by `place_back` and `insert_at`. Revision 5 says so.

A2. A range reference outliving the window.

```text
part = &(*b)[0..2]              // part.len == 2, fixed at formation
while (*b).len > 0 { e = take_back(&*b); consume(move e) }
x = &part[1]                    // slot 1 has left the window; its Box was consumed
```

Rule 6's validity sentence covered `p = &r[i]` only. Revision 5: a reference into a window, slot or range, stays valid while its bound (`i < r.len`, or `hi <= r.len`) holds; `take_back`, `remove_at`, and `truncate` do not carry it, so the reference dies.

A3. The state in which two adjacent statements' paths are compared.

```text
n0 = (*g.nodes).len
id = alloc(&g, node)            // writes (*g.nodes).next, i.e. the slot at index n0
set_data(&g, n0, 7)             // writes (*g.nodes)[n0]
```

The verifier withdrew the "may overlap" claim because Rule 10 clause 1 demands proved distinctness and `n0` is the append index; but no sentence said in which state the second statement's paths are read. Revision 5: both statements' paths are interpreted in the state before the first statement, the first's `ensures` mapping the second's indices into that state; permission composes pairwise to any number of adjacent statements.

## B. Inconsistencies inside revision 4, fixed in revision 5

B1. `grow` printed without a result type against Rule 14's "allocation returns a Result". Now `grow(&b, cap) -> Result<(), Oom>` with `ensures when Err:` capacity and length unchanged.

B2. `Box::new(Slots::new<T>(cap))?` cannot hand a `Slots<T>` back on failure, since a runtime-capacity shape cannot exist outside a Box. The runtime-capacity constructors are primitives with no payload: `Box::new_slots<T>(cap)`, `Box::new_ring<T>(cap)`, `Box::new_array_filled(n, v)`.

B3. The atomic update's restriction "f's row must not overlap any prefix of place" read literally forbids `n.left = fold(n.left, &n.right)`, since `n` is a prefix of `n.left` and `n.right` lies under `n`. Intended and now written: f must not write, move out of, or free any prefix of `place`; reading anything, and writing storage disjoint from `place`, is allowed. This also removes the judge's cost group on the atomic update.

B4. Rule 9 said `writes` covers the storage "at the path"; Rule 10's own `bad(&vv, &vv[0])` example needs "at or below". Written.

B5. Rule 8's "the compiler never releases them" read against an emptied `Box<Slots<File>>`: a container emptied of its linear elements is an ordinary affine value and is released normally. Written.

B6. Editorial: Rule 16's row comment still pointed at a proposal; `Deque<T>` growth is a fresh ring plus one copy per doubling because `grow` is defined on `Box<Slots<T>>` only. Both fixed.

## C. Behavior that is a consequence of confirmed rules, recorded rather than changed

C1. `insert_at` and `remove_at` write `r.filled`, which is a content write: a slot reference into the window survives and names its slot, whose occupant may have changed after the shift. This is the same disposition as a stale index (Rule 16) and follows from Rule 3's content-write clause, which already keeps `p = &r[i]` valid across `r[j] = 5` with `i == j`. A writer who means "the element" re-forms the reference after a shift. Revision 5 states it in Rule 6.

C2. Two allocations from one pool never overlap (both write `r.next` and `r.len`); per-worker pre-split ranges are the standard form. Allocation now overlaps reads and slot writes of the same pool, which round one refused.

## D. Needs an owner decision

D1. A partial move of a window: `split_off(&src, k, &dst)` moving `src[k..len)` onto `dst`'s back in one memmove. Without it a B-tree node split or `Vec::split_off` costs two element moves per element in a scalar loop, about twice the C++ split path.

D2. What a supplied function argument may differ in from the parameter's declared signature: exact match, or a smaller row, weaker `requires`, stronger `ensures` (ordinary refinement). Zero runtime cost either way; the strict form makes every callback match character for character.

D3. Whether a contract block may state a refinement fact about a parameter (`requires p is Some`), and whether an `ensures` may mention a single indexed path (`(*p.slots)[h.idx].gen == h.gen`). These are fact-language questions; the second decides whether a guarded pool access pays one load, compare and branch per call. Deferred to the fact-language round unless the owner wants it now.

D4. Two guarantees the no-trap promise needs from the implementation, not from the language: the compiler-derived release of an owned chain (a ten-million-node list of Boxes) must run in bounded stack, and a self-recursive descent that revision 4 now forces (Rule 2's static path shape) should have tail calls eliminated or a stated depth budget. Recorded as implementation requirements.

D5. Whether a function-typed parameter is instantiated per call site so that a literal argument becomes a direct call. Three cells price the fused find-then-mutate at zero on that premise. To check against the existing FN rules before deciding.

## E. Costs that survive, with status against round one

- Unchanged: one compare and one predicted branch per data-determined index; scatter with data-determined destinations cannot overlap; construction into the append slot moves one element; open-addressing tables with non-Copy payloads pay one null check per hit; no channels or atomics; generation words for identity across slot reuse.
- New: iterative by-reference descent of an owned linked structure is refused (Rule 2), so the walk is recursive or index-based; the atomic update on a recursive insert stores one word per level on the unwind where a C++ pointer-to-pointer descent stores once (zero against safe Rust); a partial window move costs a per-element loop until D1 is decided.
- Reduced: find-then-mutate re-descends only in the separately compiled split form; pool allocation now overlaps reads and slot writes.
- Closed: whole-container window rows; no exchange or bulk move; child window instead of inline `Option<Box<Node>>`; block header per linear cell across a fallible allocation; every growth copying the whole window.

## F. Judge findings I dismiss or reclassify

- The atomic-update sibling-read cost (judge's cost group on `f`'s row) is a wording defect, not a decided cost: see B3.
- The "k-way overlap" gap is closed by stating that pairwise permission composes (A3).
- Integer overflow and the `use`-step grammar belong to the existing kernel specification, not to x1.
