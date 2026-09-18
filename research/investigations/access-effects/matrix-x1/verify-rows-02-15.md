# Verification of `rows-02-15.md` against frozen CANDIDATE-X1

Every cell in `matrix-x1/rows-02-15.md` (2-2 … 2-16, 15-15, 15-16) re-traced
literally against `CANDIDATE-X1.md` (2026-09-18). Three verdicts change; several
traces and cost lines are corrected without changing the verdict.

---

## Per-cell results

**2-2: correction: undecided-rule-gap (ground: unbounded loop-carried target set) -> undecided-rule-gap (ground: Rule 2's path grammar has no variant-payload element; the unbounded-set ground is dismissed).**

The claimed gap does not exist. Rule 2 states a *semantic* criterion, not a
representation requirement: "At a control-flow join, a reference variable's
target is the set of paths it may name; every check on it must hold for every
member of the set." Nothing in the rule file demands that the set be finite or
finitely represented, and every member of `head.(next.*)^k` satisfies Rule 3
(no proper prefix written) and Rule 4 (no escape), so the criterion is met.
The cell's "literal finite-set reading" imports a requirement the text does not
contain.

Independently, the set cannot be unbounded under the frozen grammar at all.
Rule 2: "A path starts at a local variable or a parameter and continues through
fields, `*` (Box content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule
7)." A loop-carried self-rebinding can only grow a path without bound if the
type is recursive, and a recursive value type in this candidate is recursive
only through an enum (`Option<Box<Node>>`, Rule 1's own example) — `struct Node
{ child: Box<Node> }` has no finite value. Since the grammar's four elements
contain no variant projection, path depth is bounded by type depth and every
loop-carried target set is finite.

The real gap is that the same closed grammar makes the cell's program
unwritable: `&*((*p).next)` has to descend into `Some`'s payload, and no listed
path element does that. This is not a private defect of this cell's example —
Rule 1 blesses `Option<Box<Node>>` and Rule 12 prescribes `slots:
DynBox<Option<Entry>>` for "non-Copy payloads", which cannot be read or written
in place if no path reaches inside a variant. Either "fields" silently covers
variant payloads or Rule 1's and Rule 12's own structures are unusable by
reference. The text does not say, so the cell stays `undecided-rule-gap` on this
ground.

**2-3: agree** (`rejected-zero-cost`). Rule 3's "invalidated when any proper
prefix of p's path is written … by a statement or by a call" kills the member
`v1.buf[i]`; Rule 2's every-member clause rejects `use(p)` on both runs; the
branch-duplication rewrite is free (protocol item 7) and correct C++ must also
re-derive after a reallocating grow. One qualification the cell omits: in the
`then` arm the re-formation also needs `i < len_of(v1.buf)` again, because Rule
11 says "A fact that mentions a path is invalidated when that path is written …
unless the callee's `ensures` re-establishes it". That is recoverable at zero
runtime cost only because `grow` is the writer's own function and can declare
`ensures len_of(buf) == len_of(deref(entry(buf)))`; without such an `ensures`
the rewrite pays one compare.

**2-4: correction: undecided-rule-gap -> rejected-zero-cost, because Rule 9's
closing sentence decides the case the cell calls open.**

Rule 9: "A function body is checked against its own row: every statement's
effect and every callee's substituted row must be covered by the declared row."
Rule 9's first sentences fix what a row may contain: "Effects are declared only
on reference parameters" and "An effect row lists `reads(path)` and
`writes(path)` where each path starts at a reference parameter." The closure
body's call `helper(part, input)` substitutes `reads(input)`, `input` is not a
parameter of `h`, so no declarable row covers it and the body check fails. The
closure is rejected; it is not accepted-with-invisible-effects. The cell's first
horn ("captured accesses are invisible to the Rule 10 and Rule 13 comparisons")
requires ignoring the body-check sentence, so it is not a permitted reading.
Rule 4's qualifier "that is stored or returned" is left as near-dead text, but
dead text is not an undecided program. The rewrite the cell already gives —
pass the capture as an ordinary reference parameter — is free under protocol
item 7 and generates identical code, so `rejected-zero-cost`.

**2-5: agree** (`rejected-zero-cost`). Rule 3's own worked example (`c = move b`
… "p invalid, even though the Node did not move") plus "A move never re-roots an
existing reference" plus Rule 2's every-member clause gives the rejection; Rule
5's "the heap object did not move" makes the re-formation free. The Rule 1 note
about back edges is correctly attributed to Rule 1 rather than counted here.

**2-6: agree** (`rejected-real-cost`), with the cost line sharpened. Rule 9's
"Signatures never contain index expressions; an index enters an effect only
through an argument" forces `writes(buf)`, and Rule 3's proper-prefix clause
then invalidates every reference into the block; the ordinary program survives
under "Writing the storage at p's path or below it (a content write) does not
invalidate p". The genuine residual runtime cost is **one bounds compare** per
re-formed access, not a compare plus a base reload: proofs "are erased before
lowering", `pop` does not relocate the block, and the address recomputation is
therefore CSE-able by the backend. The compare is unavoidable, because `i < n`
and `len_of(buf) == n - 1` do not give `i < len_of(buf)` for a data-determined
`i`, so the verdict stands.

**2-7: agree** (`undecided-rule-gap`), and this is the sharpest gap in the
sheet. Rule 2 writes the path symbolically ("`p = &v.buf[i]` // p names the path
`v.buf[i]`") while Rule 3's invalidation list is closed over prefixes of the
path and never mentions the index variable. The two readings differ
*observably and unsafely*, not just in style — see the accepted-unsafe section.

**2-8: agree** (`undecided-rule-gap`), but the cell's claim that the non-Box
case is "decided" is wrong and makes the contradiction wider. Rule 6 is
exhaustive: "There is no `take` operation and no partial move out of any place:
the only ways to move a value out of storage are consuming a whole local (`move
x`), the window operations, and the atomic update", and Rule 12 repeats it: "No
place is ever partially moved". The cell's ordinary program `close(move c.f)`
after `c = swap_remove(&pool, k)` is a partial move out of the local place `c`,
so it falls under the same prohibition — and so does Rule 8's own example `fn
use_it(c: own Conn) { ...; close(move c.f) }`. So the rule file contains the
same conflict twice (Box content and struct field), not once. A reading that
reconciles them exists — "consuming a whole local" could mean consuming it by
destructuring, leaving no hole — but the file states no destructuring or unbox
operation, so it stays undecided.

**2-9: agree** (`undecided-rule-gap`). Rule 7's "Reading `buf[k]` or forming
`&buf[k]` requires the fact `k < len_of(buf)`" is unconditional and is
contradicted by Rule 9's own example `put_at(&buf[len_of(buf)], 3)`. Two further
sentences side with Rule 7 and the cell does not cite them: Rule 6's "Slots
`[0, len_of)` hold values; slots `[len_of, cap_of)` hold nothing" and Rule 12's
"every path is either wholly present or the program cannot name it". The
practical answer is the same under either reading (`push_nogrow`, zero cost), so
only the rule text is at issue.

**2-10: agree** (`rejected-real-cost`). Rule 10 clause 1's "must be proved
disjoint (different roots, or indices or ranges proved distinct)" is unavailable
for `perm[i]` vs `perm[j]`, and the injectivity ground is excluded by Rule 11's
"There are no quantified facts over array elements". The rewrite's one compare
and branch per call pair is a real cost against C++. The double-buffer unroll is
correctly priced at zero.

**2-11: correction: undecided-rule-gap -> rejected-zero-cost, because Rule 12
clause 1 as written admits only the cell's reading A.**

Rule 12: "At a join, facts are intersected (a fact survives only if it holds on
every incoming edge); a reference's target set is the union of its targets (Rule
2)." Facts are properties of paths — Rule 11 speaks of "A fact that mentions a
path" — and no sentence in the file attaches a fact to a particular member of a
reference's target set. On the edge that ran `truncate(&v2, 8)` the fact
`len_of(v1) == 8` is not established, so it does not hold on every incoming edge
and does not survive; Rule 2's "every check on it must hold for every member of
the set" then fails on member `v2` for `k < len_of(v2)`. The program is
rejected by the text as written. The cell's reading B (facts indexed by the edge
that delivered a member) is an invented mechanism. The cell's own first rewrite
— duplicate the loop into both arms — works and is free under protocol item 7,
so `rejected-zero-cost`; the hoisted-runtime-test alternative is not needed and
its header load and compare should not be charged.

**2-12: agree** (`rejected-zero-cost`) for the reference question. Rule 3's "'p
is valid' is a fact like any other" with Rule 12's "a fact survives only if it
holds on every incoming edge" rejects `use(p)`, and Rule 12's "every path is
either wholly present or the program cannot name it" is correctly read as the
reason a reference can never observe a hole. One flag: both the strongest
program and the rewrite contain a whole-local move on one edge only, and the
file never says whether that is admitted — Rule 8 has the compiler release
affine values "at scope exit … recursively" and states no per-path release
discipline and no drop flag. The cell defers this to an 8-12 cell, which is
reasonable, but the zero-cost rewrite depends on it, so it is listed as a
confirmed gap below.

**2-13: agree** (`rejected-real-cost`). Rule 13 explicitly reuses Rule 10's
judgment, the scatter's data-determined indices are not "proved distinct", and
the injectivity of `owner` is a Rule 11-excluded quantified fact. The gather
inversion, the pairwise tests and the per-worker buffers are correctly priced,
and the gather's cost (one inverse array plus one O(n) pass) is a genuine
runtime cost. Minor over-reading to note: Rule 13's "The existing counted-loop
forms (per-element maps, adjacent ranges passed to a helper, admitted
reductions) are expressed with range references" is a description of how the
existing families are written, not an obviously closed admission list; the
scatter loop is rejected by the pairwise judgment regardless, so nothing turns
on it.

**2-14: agree** (`accepted-fine`). Rule 3's invalidation list is closed over
writes, moves, replacements and frees of a *named* prefix, and Rule 14's
"Allocation and release carry no effect entry and never make two parallel arms
conflict" settles the par arm. The cell's dismissal of the relocating-allocator
worry is correct and should not be a gap: Rule 2 makes a reference "a local name
for a path", not an address, so non-relocation is a lowering obligation on the
trusted base, not a rule contradiction. Two notes: the last two lines of the
"strongest program" are a Rule 9/Rule 3 rejection (`grow` declares
`writes(a.buf)`), not an allocation effect, so the cell's program is only
`accepted-fine` for the allocation question it asks; and the cold-path
re-formation may also need one compare to re-prove `i < len_of(a.buf)` under
Rule 11, which the cell mentions but does not price — it is on the OOM path, so
it does not change the verdict.

**2-15: agree** (`accepted-fine`), with one cost correction. The trace is right:
a row with a single `writes` entry has no pairwise partner under Rule 10 clause
1, and every member of the substituted set is a live writable path, so the
set-valued destination raises no obligation. The zero-copy claim is properly
grounded in the no-zero-copy-ABI assumption for aggregate round trips. The
understated cost is the type-changing case: the cell says the value "must become
a whole local, moved out at the top and the new value written back, costing one
struct move in and one out", but moving a field out of a place is exactly what
Rule 6 forbids ("no partial move out of any place") and Rule 12 repeats ("No
place is ever partially moved"). The real price is consuming and rebuilding the
*whole enclosing aggregate*, i.e. a copy proportional to the container, not to
the field — unless the place is a DynBox slot, where Rule 6's atomic update
applies and the transform must preserve the type.

**2-16: agree** (`rejected-real-cost`). Rule 3 against `alloc`'s `writes(a.buf)`
invalidates every reference into the pool at every allocation; the sibling-slot
content write correctly leaves `r` valid; the per-hop bounds compare is forced
by Rule 11's exclusion of "every stored handle is in bounds". The stale-handle
program is genuinely accepted and genuinely weaker than P2 asks for; it is
listed under accepted-unsafe below as an identity failure, not a memory failure,
since `truncate` released the old occupant and the new occupant is a fully
initialized value of the same type ("no program point can observe a slot inside
the window as empty"). The cell's dependence on provisional Rule 16 is correctly
declared.

**15-15: agree** (`rejected-zero-cost`). Rule 15's "A function returns owned
values only" with Rule 4's "cannot be returned" removes the C++ `Big&` shape;
the container-plus-index rewrite is Rule 4's own prescription and costs one
shift/add the C++ callee performed anyway. One rewrite stronger than the four
listed should be recorded, because it removes the cell's only residual doubt: if
the two candidates cannot share a container, duplicate the branch at the *use*
site (`if c { use(&a) } else { use(&b) }`), which Rule 4 permits since a
reference "may be bound to a local, passed as a call argument"; behind an
abstraction boundary the callee returns the owned selector (a tag or index) and
the caller branches. Duplicated code is free under protocol item 7, so the
callback inversion and its indirect call are never forced, and `rejected-zero-cost`
is right for the right reason.

**15-16: agree** (`rejected-real-cost`), with one trace repair. The interface,
the capacity re-test, the Rule 3 re-formation per allocation and the per-hop
bounds compare are all correctly derived, and the P2/P3 identity shortfall is
correctly separated from memory safety. The repair is in the fourth bullet: the
recipe "bind `n = len_of(a.buf)` first, then use `alloc`'s `ensures len_of(a.buf)
== result + 1`" does not recover `l < len_of(a.buf)` in the program as written,
because a second recursive `build` call sits between the binding and the
allocation and `build`'s only `ensures` is `when Ok: result < len_of(a.buf)` —
nothing states that `build` never shrinks the window, so `n <= len_of(a.buf)` is
unavailable after it. The fix is free at runtime: give `build` the extra
monotonicity `ensures len_of(a.buf) >= len_of(deref(entry(a.buf)))`, which is an
"affine comparison over measures" and so admitted by Rule 11. Note also that the
per-hop cost may be one compare *plus one header load* rather than one compare,
depending on the unresolved Rule 11 granularity question listed below.

---

## Confirmed gaps

1. **No path element reaches into a variant payload (2-2).** Rule 2: "A path
   starts at a local variable or a parameter and continues through fields, `*`
   (Box content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7)." Rule 1
   nevertheless blesses `struct Node { value: Int, left: Option<Box<Node>> }`
   and Rule 12 prescribes `slots: DynBox<Option<Entry>>` for "non-Copy
   payloads". Either "fields" covers variant payloads or no reference can reach
   a payload, which makes recursive structures untraversable and Rule 12's
   Option-based table unreadable for non-Copy elements.
2. **Snapshot vs symbolic index in a formed path (2-7).** Rule 2: "`p =
   &v.buf[i]`  // p names the path `v.buf[i]`", with Rule 3's closed list
   "invalidated when any proper prefix of p's path is written, moved out of,
   replaced, or freed". Mutating `i` is neither a prefix event nor a
   re-formation, and the two readings put a store in different slots — one of
   them out of bounds.
3. **No operation takes a linear value out of a Box or a field (2-8).** Rule 6:
   "There is no `take` operation and no partial move out of any place: the only
   ways to move a value out of storage are consuming a whole local (`move x`),
   the window operations, and the atomic update." Rule 8 asserts the opposite
   twice: "`Box<File>`  // linear: must be taken apart and its File closed" and
   `fn use_it(c: own Conn) { ...; close(move c.f) }`.
4. **`&buf[len_of(buf)]` (2-9).** Rule 7: "Reading `buf[k]` or forming `&buf[k]`
   requires the fact `k < len_of(buf)` (Rule 7)", against Rule 9's example
   `put_at(&buf[len_of(buf)], 3)   // the argument fixes the slot at the call`.
   Rule 6's "slots `[len_of, cap_of)` hold nothing" and Rule 12's "every path is
   either wholly present or the program cannot name it" side with Rule 7.
5. **Conditional whole-local move and scope-exit release (2-12, and every
   duplicate-the-branch rewrite that moves on one edge only).** Rule 8: "Affine
   values are consumed at most once; at scope exit the compiler releases their
   memory recursively (Box, DynBox) and runs no user code." The file states no
   per-path release discipline and no drop flag, and says nothing about a value
   moved on one incoming edge and live on another; P8 additionally requires "no
   runtime drop flag".
6. **Granularity of fact invalidation (15-16, 2-16).** Rule 11: "A fact that
   mentions a path is invalidated when that path is written (by statement or
   call) unless the callee's `ensures` re-establishes it." Rule 3 distinguishes
   a content write from a prefix write for *references*, but Rule 11 makes no
   such distinction for *facts*, so it is unstated whether `writes(a.buf[h])`
   invalidates `len_of(a.buf) == n`. Under the invalidating reading, every
   traversal hop that writes a node also reloads the header.

## Dismissed gaps

- **2-2, unbounded loop-carried target set.** Covered by Rule 2: "every check on
  it must hold for every member of the set" is a semantic criterion with no
  finiteness or representability requirement, and by the same rule's four-element
  path grammar, which admits no recursive descent at all without a variant
  projection, so the set is bounded by type depth in any writable program.
- **2-4, a captured reference has no declarable effect.** Covered by Rule 9: "A
  function body is checked against its own row: every statement's effect and
  every callee's substituted row must be covered by the declared row", together
  with "each path starts at a reference parameter". The uncoverable access makes
  the closure fail its own body check; the program is rejected, not undecided.
- **2-11, whether a fact at a join belongs to the edge's member.** Covered by
  Rule 12: "At a join, facts are intersected (a fact survives only if it holds
  on every incoming edge)", with Rule 11's "A fact that mentions a path" fixing
  facts to paths rather than to members of a target set.
- **2-14, relocating allocator (already dismissed by the cell; confirmed).**
  Covered by Rule 2's "A reference is a local name for a path" — validity is
  path-based, so non-relocation is an obligation on the trusted base, not a rule
  contradiction.

## Confirmed real costs

- **2-6**: one bounds compare per access re-formed after any window operation
  (`pop`, `push_nogrow`, `swap_remove`, `truncate`), because Rule 9 forbids a
  finer row than `writes(buf)` and `i < n` with `len_of == n - 1` does not give
  `i < len_of`. The address recomputation is *not* a cost: proofs are erased
  before lowering and the block does not move.
- **2-10**: one compare and one branch per call-site pair whose indices come
  from data; C++ makes the call with neither.
- **2-13**: one n-element inverse-permutation array plus one O(n) pass per
  permutation change to turn a scatter into a gather; the alternatives are k(k-1)/2
  compares with a fully sequential fallback arm, or one extra buffer per worker
  plus an O(n) merge.
- **2-16 / 15-16**: one bounds compare per traversal hop, because "every stored
  handle is in bounds" is a Rule 11-excluded quantified fact; plus one header
  load per hop if gap 6 resolves the invalidating way; plus one generation word
  and one compare per dereference wherever reuse must be detected (which a
  generational C++ or Rust arena also pays).
- **2-15 (corrected upward)**: a type-changing transform of a value held in a
  struct field costs consuming and rebuilding the whole enclosing aggregate, not
  one field move, because Rule 6 and Rule 12 forbid the partial move; only a
  DynBox slot with a type-preserving transform is free.
- **2-14**: one compare on the allocation-failure path to re-prove the bound.

## Accepted-unsafe programs constructible in these cells

1. **2-7, out-of-bounds write under the symbolic reading.** Every step is
   admitted by a quoted rule; only the unresolved index semantics separates the
   program from a memory error.

   ```text
   n = len_of(buf)
   i = n - 1
   p = &buf[i]          // Rule 7 satisfied: i < len_of(buf)
   i = i + 1            // Rule 3's invalidation list does not mention i
   put_at(p, 7)         // fn put_at(slot: &Int, x: own Int)  writes(slot)
   ```
   Under the symbolic reading `p` names `buf[i]` with the current `i`, so the
   store lands at `buf[n]`, outside the window Rule 6 defines, with no rule
   re-checking the bound after formation.

2. **2-16, stale handle names a new occupant (accepted, memory-safe,
   identity-unsafe).** The sequence `id1 = alloc(...); truncate(&a.buf, 0); id3
   = alloc(...); q = &a.buf[id1]` is accepted by Rule 7 (the bound holds again)
   and blessed by Rule 16: "A stale index that is still in bounds names the
   current occupant of that slot: a logic error, not a memory error." No freed
   or relocated storage is read, but P2's "every old pointer into b2 must be
   refused afterwards" and P3's requirement that "a relation to a deleted node
   must not silently become a relation to a replacement node" are both lost
   unless the program adds a generation word.

3. **2-4, captured-reference race — only under the reading this verification
   rejects.** If Rule 4's qualifier were read as permitting a non-escaping
   closure to capture a reference whose accesses no row names, then

   ```text
   h = |x: &Buf| merge_into(cap, x)      // cap: &Accum, captured, invisible to h's row
   par { h(&a); h(&b) }                   // Rule 13 sees only writes(a), writes(b)
   ```
   passes Rule 13's disjointness check while both arms write `cap`. Rule 9's
   body-check sentence forecloses this reading, which is why cell 2-4 is
   corrected to `rejected-zero-cost`; it is recorded here because the unsafe
   outcome is what makes that sentence load-bearing.
