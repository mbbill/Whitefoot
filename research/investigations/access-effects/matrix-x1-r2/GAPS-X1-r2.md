# Gap and cost report for candidate x1, revision 4 — targeted second round

Date: 2026-09-18. Judged against the frozen rule text in `../CANDIDATE-X1.md` (revision 4),
by re-reading that file for every item below rather than trusting the cell that reported it.

**How to read this report.** The matrix is a grid of the candidate's sixteen rules against
themselves. A name like "cell 4-9" means the cell where Rule 4 meets Rule 9; a name like
"the tree-rewrite task" means one of the four engineering tasks re-derived this round. All
code is written in revision 4's own notation: measures and window parts are members of the
storage (`r.len`, `r.next`), never `len_of(r)`; there is no `par` statement, so two things
that may run at the same time are written as two adjacent statements with an explicit
sentence saying whether they may overlap. Quotations in bold-marked blockquotes are the
exact sentences of revision 4.

Round one derived all 136 cells against revision 2. Round two re-derived only the 66 cells
whose outcome revision 4 could change, plus four of the eight tasks. Everything in this
report is about those 66 cells and four tasks, except Part 1, which settles the round-one
record in full.

---

## Part 0 — The short version

Revision 4 closes every memory-safety hole round one found, and closes them by the sentences
round one asked for, not by construction or by accident. It also closes the resource-obligation
holes, the three defective examples, and five of the seven expressiveness decisions round one
put to the owner. Fourteen of the nineteen round-one undecided cells in this subset become
decided; the undecided count over the re-derived cells falls from 19 to 2.

What it does **not** close is a new and narrower family of questions created by the very
mechanism that closed the old ones. Window parts (`r.next`, `r.last`, `r.filled`, `r.free`)
are introduced as names that "paths and effect rows may name", and three separate questions
now turn on how far that goes:

1. Is a window part a *place* — may a program form a reference to `r.next`, read it, write it?
   Under the permissive reading the candidate admits an uninitialized read and a free of
   uninitialized storage. This is the one memory-safety hole in revision 4.
2. Does *writing* a window part invalidate a reference into a slot? `r.filled` overlaps a live
   `r[i]` but is not a proper prefix of it, and Rule 3 invalidates only on a proper prefix. Two
   cells of this round reach opposite conclusions, and for `remove_at` the permissive answer
   leaves a valid reference silently naming a different element.
3. Does a *range* reference die when the window shrinks past it? The sentence that kills a
   reference at a `take_back` is written for `&r[i]` only. Under the letter of Rule 3 a range
   reference survives the drain and reads a released element.

Each is one sentence away from closed, and all three sentences belong in Rule 6 beside the
window-parts paragraph.

On cost, the picture improved sharply and shifted. Round one's largest cost cluster — no
exchange operation, so in-place sort, partition and ordered removal were unavailable or 3x —
is gone: `swap` with its aliasing exemption, `insert_at`, `remove_at`, `append` and `grow`
delete it. Round one's second cluster — every window operation declared the whole container,
so every interior reference died at every call — is gone: window parts make an appending
callee precise across separate compilation, which is the headline requirement of the
mutable-graph task and now works. In their place, one genuinely new cost appears: Rule 2's
static path shape refuses `while (n) { n = &(*n.left) }`, so every by-reference descent of an
owned linked structure must recurse or move to pool indices. That lands on the hottest loop of
the tree-rewrite task and is the round's strongest counterexample.

---

## Part 1 — Every round-one hole, gap and cost: does revision 4 close it?

Round one's record is in `../REVIEW-X1-round1.md` (sections A to E) and
`matrix-x1/GAPS-X1.md` (thirteen real omissions, thirteen cost clusters, nine unsafe claims).
The review's lettering is used here; the gap-report numbers are given in parentheses where
they differ.

### A. The three memory-safety holes: all three closed

**A1 — prefix overlap judged without an index-distinctness proof** (gap report's Gap 3).
Round one's program freed a heap node through `replace_at(&g, i, nb)` while a reference formed
from `g[j]` stayed valid, because Rule 3 spoke of a "proper prefix" without saying that two
indexed paths overlap unless proved distinct. **Closed**, by a sentence added to Rule 3:

> Whether one path is a prefix of another, and whether two paths overlap, is judged
> conservatively: two indexed positions on the same storage are taken to overlap unless their
> indices or ranges are proved distinct, exactly as in Rule 10.

Rule 3 also now prints the program itself, with the right answer and the escape:

```text
q = &(*(*g)[j]).value  // g: Box<Slots<Box<Node>>>
replace_at(&g, i, nb)  // writes (*g)[i]: overlaps (*g)[j] unless i != j is proved: q invalid
                       // with the fact i != j, q survives
```

**A2 — the index in a formed path is a snapshot** (Gap 4). **Closed**, by Rule 2:

> An index expression inside a path is evaluated when the reference is formed; the path records
> that value, and later assignments to the variables the expression used do not change it.

Cell 2-7 traces this verbatim and moves from undecided to accepted-fine.

**A3 — a compiler-inserted scope-exit release was not an invalidating event** (Gap 2), round
one's only unconditional memory-safety hole. **Closed**, by Rule 3's expanded list:

> ... by a statement, by a call, or by the compiler-derived release at scope exit; the scope of
> the local variable at which p's path starts ends

and by the worked example, which is round one's own program with the verdict reversed:

```text
p: &Int
{ b = Box::new(...)?; p = &(*b).value }   // b's scope ends here and b is released
use(p)                 // rejected: p's root has gone out of scope
```

### B. The two resource-obligation holes: both closed

**B1 — `truncate` silently discarded linear elements** (Gap 5). **Closed** twice over. Rule 6
demotes `truncate` to library code ("`swap_remove` ..., `truncate`, `clear`, ... all zero-cost
compositions of the above"), so it is a loop of `take_back` that hands each element to the
caller; and Rule 6 states the general prohibition:

> No operation releases a linear element: a storage whose element type is linear is itself
> linear (Rule 8) and the program must take every element out and consume it.

**B2 — a linear payload lost on the `Err` arm of a fallible allocation** (Gap 6). **Closed**,
by Rule 5:

> A fallible allocation that takes a by-value payload hands it back on failure, so a linear
> payload is never lost.

with the signature shown: `b = Box::new(Node { ... })?  // Result<Box<Node>, (Oom, Node)>: on
Err the Node comes back`. Cell 8-14 moves from accepted-unsafe to accepted-fine and the
round-one workaround (an empty one-element window per linear cell, sixteen bytes of block
header each) is retired.

### C. The five internal inconsistencies: all five closed

**C1 — Rule 8's `close(move c.f)` against Rule 6's ban on partial moves** (Gap 1, round one's
headline: eight of nineteen undecided cells). **Closed** by ranking the two, in Rule 6:

> A move out of a field or out of Box content consumes the whole owner: the owner ceases to
> exist, its other affine parts are released, and a remaining linear part rejects the move
> (take it in the same destructuring).

and by the destructuring form round one asked for: `let Conn { f, g, .. } = move c`. Cells
1-12, 2-8, 3-8, 4-8, 5-8, 8-8 and 8-10 all resolve on this sentence.

**C2 — Rule 9's `put_at(&buf[len_of(buf)], 3)`, a reference to a slot outside the window.**
**Closed**: the example is replaced by `place_back(&r, 3)  // appending: writes(r.next),
writes(r.len)`. Cells 2-9 and 9-12 no longer face a normative example contradicting the rule it
illustrates.

**C3 — Rule 8's containment list omitted enums and tuples.** **Closed**: "a struct, enum,
tuple, Array, Slots, Ring, or Box containing a linear part is linear".

**C4 — whether a pre-call fact survives as a fact about `entry(p)`** (Gap 12, used by four
cells and written nowhere). **Closed**, by Rule 11:

> Across a call, a fact known before the call about a measure of an argument survives as a fact
> about `entry(p)` of that argument, which is how an `ensures` of the shape
> `r.len == entry(r).len + 1` connects to what the caller knew.

**C5 — assignment over an owned place that is not a window slot** (Gap 11). **Closed**, by Rule
6: "Assigning over any owned place releases the old value if it is affine and is rejected if it
is linear."

### D. The seven expressiveness decisions: five closed outright, one closed at a new cost, one partly

**D1 — a reference rebound through its own path in a loop** (Gap 7). **Closed by rejection**,
in Rule 2:

> A path has a static shape: a loop-carried rebinding may change only the index values inside
> the path, never extend the path through itself.

with round one's own program printed as the rejected example
(`loop { p = &(*p.kids)[0] }  // rejected: the path would grow without bound`). The rejection is
clean and terminating; it is also the round's new cost (Part 3, cost group 2).

**D2 — the reach of the atomic in-place update** (Gap 8, three sub-questions). **Closed on all
three.** Rule 6: `place = f(place, args...)` — any owned place, further arguments admitted, and
"f is total and returns the place's type ... failure is an enum in the place", which answers the
fallibility question by forbidding `?` inside `f`. A fourth clause is new: "f's row must not
overlap any prefix of place", and that clause is a real cost of its own (Part 3, cost group 5).

**D3 — the type of a range reference** (Gap 9). **Closed**, by Rule 7: "A range reference
`&x[lo..hi]` has the parameter type `&[T]`: a reference kind with measure `len`, formed only
from an indexable or from another range reference, never a stored value." Re-ranging (`sub =
&part[a..b]`) is printed, which is what recursive divide-and-conquer needed.

**D4 — what holds after a `par` block and what an early exit in an arm means** (Gap 10).
**Closed by deletion.** There is no `par` statement; Rule 13 grants overlap permission over two
adjacent ordinary statements, and

> Because the meaning is sequential, results, errors, early exits, and linear obligations are
> exactly those of the sequential program; nothing new is defined for the overlapped case.

answers all three of round one's sub-questions at once. Cell 10-14 moves from undecided to
accepted-fine on this sentence alone.

**D5 — variant payload paths** (gap report §2.2, priced there as cost RC11). **Closed**, by
Rule 2: a path continues through "the payload of an enum variant", under a refinement fact a
`match` establishes. The tree-rewrite task's literal shape — two inline `Option<Box<Node>>`
children — becomes writable, and round one's forced child-window fallback (one extra dependent
load and one allocation per node) is retired.

**D6 — bulk window operations.** **Partly closed.** `swap` on any two places with an explicit
aliasing exemption, `insert_at`, `remove_at`, `append` and `grow` are all adopted, which is more
than round one asked for and deletes round one's largest cost cluster. Two residues: there is no
operation that moves *part* of one window into another, so an order-preserving split is a
per-element loop (Part 3, cost group 7); and `grow` is printed with no result type while Rule 14
says allocation returns a `Result` (Part 2, group 6).

**D7 — an `appends` effect kind so an interior reference survives a user-level append across
separate compilation.** **Closed**, and better than proposed: window parts let a user function
declare the same row as the built-in, and Rule 6 prints the case:

```text
fn add_node(g: &Graph, n: own Node) -> u64   writes((*g.nodes).next), writes((*g.nodes).len)
p = &(*g.nodes)[i]
id = add_node(&g, node)                       // p survives
```

### E. The seven surviving costs, and the rest of the round-one cost list

**E1 — one compare and one predicted branch per data-determined index.** **Open, unchanged, and
now permanent.** Rule 11 still admits no quantified facts over elements, and the escape round one
flagged is explicitly out of reach: the bitmask fact is listed under "Deferred to a future
concurrency and layout round". This is the dominant cost of the mutable-graph task in both rounds.

**E2 — parallel scatter to data-determined destinations.** **Open, unchanged.** Rule 11 plus the
exclusion of channels and atomics.

**E3 — find-then-mutate re-descends.** **Reduced.** The generalized atomic update fuses find and
mutate into one descent (`node.left = insert(node.left, k)` is Rule 6's own line), so the common
case is free. The split form still re-descends, and one unwritten premise remains: whether a
function-typed parameter is instantiated per call site (Part 2, group 10).

**E4 — construction into the append slot copies one element.** **Open, unchanged.** Revision 4
makes the slot *nameable* (`r.next`), which removes round one's contradiction, but no operation
publishes an already-filled slot, so the copy stands. Recorded in the rule file's own Known costs.

**E5 — growth copies the whole window.** **Closed** by `grow(&b, cap) // may reallocate in
place`, conditional on group 6 below.

**E6 — pool allocation serializes; generation words for identity.** **Partly.** Allocation may now
overlap a read or a slot write of the same pool, and a held reference survives it, which is new.
Allocation against allocation is unchanged (both write `r.next` and `r.len`), and the generation
word is unchanged.

**E7 — a user-level allocator cannot revoke references into a block it freed.** **Open by design,
and now explicit** rather than inferred. Rule 16: "The same holds for a reference into a
user-level pool or allocator: storage that still exists stays readable, and what the program
considers 'freed' is the program's own bookkeeping."

**Gap 13 — integer overflow.** **Open**, and still arguably outside x1, which is a delta over the
ownership and access rules (Part 2, group 15).

**The other round-one cost clusters.** Round one's cluster "every window operation declares the
whole container, killing every interior reference" is **closed** by window parts. "Box-linked
structures must carry a child window" is **closed** by payload paths. "A linear value cannot
cross a fallible allocation in a Box" is **closed** by the hand-back. "No exchange, no partial
move" is **largely closed** by `swap` and the bulk operations, with the residue in cost group 6.

---

## Part 2 — Confirmed round-two gaps, grouped by the missing or ambiguous rule

Twenty groups. For each: the question a reader cannot answer, the program that turns on it, an
honest re-reading of revision 4, and the smallest fix. Four groups turn out to be covered on
re-reading and are marked as such; they are kept here so the same questions are not re-opened.

---

### Group 1. Is a window part a place, or only a name inside an effect row?

**Cells:** 9-12, 4-9, 2-9. **Task:** generic algorithms (its G6). **Covered: NO.**
**This is the one memory-safety hole in revision 4.**

Rule 6 introduces the parts in permissive language:

> Besides its slots, a window has four named parts that **paths and effect rows may name**, all
> interpreted at call entry: `r.next` (the append slot, at index `r.len`) ... `r.free` (all slots
> from `r.len` up).

and closes the paragraph with

> This vocabulary is ordinary: the rows below use nothing a user function cannot write.

while the same paragraph's opening says of the underlying storage

> slots `[0, len)` hold values, `[len, cap)` hold nothing.

No sentence says the parts are names for rows and the overlap judgment only. If "paths may name
`r.next`" means what it says, then `&r.next` is a path a program may form, and both halves of the
following are well-formed:

```text
struct Entry { key: u64, body: Box<Slots<u8>> }       // affine, not Copy

fn peek(slot: &Entry) -> u64          reads(slot)     { (*slot).key }
fn stash(slot: &Entry, e: own Entry)  writes(slot)    { (*slot) = move e }

b: Box<Slots<Entry>>                                  // (*b).len == 0, (*b).cap == 8

k = peek(&(*b).next)         // Rule 6 says a path may name r.next; Rule 6 says that slot
                             // "holds nothing". Rule 10 clause 1 sees one read and has
                             // nothing to prove disjoint. An uninitialized read.

stash(&(*b).next, move e)    // the callee cannot tell its slot is outside the window, and
                             // Rule 6 says "Assigning over any owned place releases the old
                             // value if it is affine" -> it frees the Box it reads out of
                             // uninitialized storage. Memory corruption.

place_back(&*b, move e2)     // writes r.next from its own argument: whatever stash left
                             // there is overwritten and nothing is released.
```

The read half needs only a `reads` row, so a rule forbidding *writes* to the append slot would
not close it. Rule 6's linear clause ("rejected if it is linear") already blocks the linear case,
so the hole is confined to Copy and affine element types — where it is worse than a leak.

The two cells that met this disagree about whether it is even permitted. Cell 9-12 rules `r.next`
unassignable, but by *inference*: slots outside the window hold nothing, and a writable append
slot would be the `take`/`put` hole that the exclusion list forbids. Cell 4-9, in the same batch,
writes `build_into(&r.next, &s)` as a well-formed call. An inference from an exclusion list is not
a rule, and the protocol tells a cell that meets an ambiguity to record it, which is why 9-12 is
this round's second undecided cell.

**Smallest fix:** one sentence in Rule 6 — the four parts are names for effect rows and the
overlap judgment only; a program may not form a reference to a part, and a path in a *program*
still starts at a local or parameter and continues only as Rule 2's grammar allows.

**Severity: unsafe if misread.** **Cost of the safe reading: zero** (every program in this round
that names a part does so inside a row).

---

### Group 2. Does writing a window part invalidate a reference into a slot?

**Cells:** 2-6, 6-10, 4-6, 4-16, 3-9. **Tasks:** resource container (its second text note),
mutable graph. **Covered: NO**, and two cells of this round contradict each other.

Rule 6 gives the overlap relation:

> a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`, overlaps `r.last`
> unless `i != r.len - 1` is proved, and always overlaps `r.filled`.

Rule 3 gives the invalidation relation, and it is a different relation:

> ... invalidated by any of the following: **a proper prefix** of p's path is written, moved out
> of, replaced, or freed ... Writing the storage at p's path or below it (a content write) does
> not invalidate p.

`r.filled` is "all slots below `r.len`" — a position at the same depth as `r[i]`, not shorter. So
it *overlaps* `r[i]` and is not a *proper prefix* of it, and the text never says which relation
governs invalidation. Rule 10 clause 3 repeats "proper prefix" and inherits the same silence.

```text
b: Box<Slots<Entry>>                 // (*b).len == 10, established statically
p = &(*b)[7]                         // formed under 7 < (*b).len

y = remove_at(&*b, 2)                // writes((*b).filled), writes((*b).len); one memmove
consume(move y)                      // ensures (*b).len == entry(*b).len - 1, so 7 < (*b).len

use(p)                               // Strict reading: rejected, re-form from the index.
                                     // Literal reading: accepted — and p now names the
                                     // element that was at index 8 before the memmove.
```

Neither reading is memory-unsafe: a memmove neither frees a slot nor empties one, and an
operation that shrinks the window past `p` kills it anyway through the formation fact. The
literal reading is an *identity* error of exactly the kind Rule 16 blesses for stale indices —
but Rule 16 speaks of indices, and here a *reference* silently changes which element it names.

Three cells of this round took three positions. Cell 2-6: "`insert_at` and `remove_at` both write
`r.filled` and therefore kill every element reference." Cell 6-10: "`r[i]` is not a proper prefix
of `r[k]` ... So a reader holding `&r[k]` across a sort keeps a valid reference to a slot whose
contents changed — which is what C++ gives and what Rust forbids." Cell 4-6 asserted that any
live reference dies at `remove_at`, and its verifier reversed the sub-claim on the ground quoted
above. Note that 6-10's case is a `swap` loop, where survival is both safe and wanted, while
2-6's case is a memmove, where survival renumbers. A single answer must cover both, which is why
the fix has to name the renumbering operations rather than the part.

**Smallest fix:** one sentence in Rule 6 — every operation that renumbers slots (`insert_at`,
`remove_at`, source-side `append`, `place_front`, `take_front`) invalidates every reference into
that storage; a write to `r.filled` that does not renumber (a `swap`, a `set`, a sort) does not.

**Severity: unsafe if misread** (identity, not memory). **Cost of the strict reading:** one scaled
add to re-form the reference after an edit, no load and no branch, since the bound follows
affinely from `i < k` and the `ensures`.

---

### Group 3. Does a range reference die when the window shrinks past it?

**Cell:** 3-7 (this round's first undecided cell). **Covered: NO.** The two readings differ by a
use-after-free.

Rule 6's validity sentence is written for an element reference and for no other kind:

> A reference `p = &r[i]` into a window is formed under the fact `i < r.len` and stays valid
> while that fact holds: `place_back`'s `ensures` carries it across the call; `take_back`'s does
> not, so p dies at a `take_back`.

A range reference is formed differently (Rule 7: "requires `lo <= hi <= r.len`; `part.len == hi -
lo`") and its measure is fixed at formation (Rule 2: the path "records that value"). `take_back`
writes `r.last`, an index position at the same depth as the range step, so it is not a proper
prefix of `&r[lo..hi]`, and Rule 3's other sentence applies literally.

```text
struct Entry { t: Box<Tag> }                  // affine: each element owns a heap object
b: Box<Slots<Entry>>                          // (*b).len == 10

part = &(*b)[0..2]                            // Rule 7: requires 0 <= 2 <= (*b).len; part.len == 2

while (*b).len > 0 {
    e = take_back(&*b)                        // writes((*b).last), writes((*b).len)
    consume(move e)                           // releases the Box<Tag> inside e
}

x = &part[1]                                  // Rule 7: requires 1 < part.len, and part.len == 2
read(x)                                       // slot 1 has left the window and its Tag is freed
```

Under Rule 6's sentence read generally, `part` dies at the first `take_back` that overlaps it and
the rewrite (re-form after the drain) costs nothing. Under Rule 3's content-write sentence read
literally, every line above is admitted and the last one is a use-after-free.

**Smallest fix:** extend Rule 6's sentence — "a reference `&r[lo..hi]` stays valid while
`hi <= r.len` holds" — or, equivalently, state that any operation decreasing `r.len` invalidates
every reference into `r`.

**Severity: unsafe if misread.** **Cost of the safe reading: zero.**

---

### Group 4. In which state are the two statements' paths read when they may overlap?

**Cell:** 1-16 (found by this round's verifier, not by the deriver). **Covered: NO.**

Rule 6 says window parts are "all interpreted at call entry", and the overlap sentence speaks of
"a live `r[i]` (which has `i < r.len`)". Both terms are read in *one* state. Two adjacent
statements have two different entry states, and nothing fixes which one the judgment uses.

```text
n0 = (*g.nodes).len
id = alloc(&g, node)         // writes((*g.nodes).next), writes((*g.nodes).len)
                             // appends at index entry(*g.nodes).len, which is n0
set_data(&g, n0, 7)          // writes((*g.nodes)[n0]); requires n0 < (*g.nodes).len
                             // — which holds in the state AFTER the allocation

// May these two statements overlap?
```

Read in the post-allocation state, `n0` is a live slot, and Rule 6's "a live `r[i]` never overlaps
`r.next`" grants the permission — licensing two concurrent writes to one slot, one of them the
move of the appended element into it. Read in the pre-allocation state, `n0` is not live, nothing
grants the permission, and Rule 10 clause 1's "indices or ranges proved distinct" withholds it,
which is the correct answer. Cell 1-16 granted it; its verifier withdrew the sub-claim. The cell's
verdict does not depend on it — the rewrite is disjoint ranges of a pre-filled block — but a
careful reader can derive the race from the text.

**Smallest fix:** one sentence in Rule 13 — the write and read paths of two adjacent statements
are judged in the state before the first, and a fact established only by the first statement does
not enter the judgment.

**Severity: unsafe if misread.** **Cost of the correct reading: zero** here; in general it forbids
an allocate-then-fill pair from overlapping, which is the right answer.

---

### Group 5. Stack depth, in a language that promises no runtime trap

**Task:** tree rewrite (its fourth gap). **Cells:** 1-4, 2-2, 3-3, 4-12 by implication.
**Covered: NO.** No sentence anywhere in the file mentions stack depth.

Two unbounded recursions over data-shaped depth are now mandatory rather than optional. Rule 8
makes the compiler's release recursive:

> at scope exit the compiler releases their memory recursively (Box, Slots, Ring, Array) and runs
> no user code

and Rule 2's static path shape (see Part 1, D1) refuses the iterative alternative for traversal,
so the *program's own* descent must recurse too.

```text
struct Node { v: u64, left: Option<Box<Node>>, right: Option<Box<Node>> }

fn depth_sum(n: &Node) -> u64   reads(n) {          // the admitted traversal form
    match &n.left { Some(c) => depth_sum(c), None => 0 }
}
// A list-shaped tree of 10 million nodes recurses 10 million frames.
// Dropping the same tree recurses the same depth inside the compiler's release.
// The candidate says "no runtime traps"; nothing says what this does.
```

Both are fixable inside a program — the tree-rewrite task writes an allocation-free
swap-chained flattening at four swaps per node for release, and a depth budget at one compare per
level for descent — but only if the writer knows it must. The evaluation criteria name "no
runtime traps" as a safety property, and a stack overflow is a memory error.

**Smallest fix:** one sentence stating the stack discipline: either a bound the compiler proves, a
statement that the trusted base supplies an unbounded stack, or an explicit admission that this is
deferred with the release depth as the harder half.

**Severity: unsafe if misread.** **Cost of the self-imposed budget:** one compare and one predicted
branch per descent level; neither C++ nor Rust pays it, both relying on the guard page.

---

### Group 6. Does `grow` return a `Result`, and what holds after it fails?

**Cells:** 6-6, 6-14, 3-4, 7-16, 15-16. **Task:** resource container (its G-A, which sits directly
on that task's stated obligation). **Covered: NO.**

The operation table prints an unconditional contract:

> `grow(&b, cap)              writes(*b)                                            // Box<Slots<T>> only; may reallocate in place`
> `contract { requires cap >= (*b).cap; ensures (*b).cap == cap, (*b).len == entry(*b).len; }`

while Rule 14 says:

> Allocation returns a `Result` and never traps (Rule 5 for the payload).

An operation that "may reallocate" allocates, so it can fail, and the table's `ensures` should be
the success arm. The sheets disagree on the spelling in consequence: cell 3-4 writes
`grow(&v, 2 * (*v).cap)?` and cells 7-16 and 15-16 write `grow(&g.buf, bigger)` with no `?`.

```text
fn push(v: &Vector<T>, x: own T) -> Result<Unit, (Oom, T)>
    writes(*v.buf)
{
    if (*v.buf).room == 0 {
        grow(&v.buf, 2 * (*v.buf).cap)?     // is this a Result? what holds on the Err arm?
                                            // if `cap` and `len` are not re-established,
                                            // the caller must reload (*v.buf).len: one load
    }
    place_back(&*v.buf, move x)             // requires room > 0
    Ok(Unit)
}
```

This is the rule file's own `Vector` — "a `Box<Slots<T>>` plus a `push` that calls `grow` when
`room` is zero" — so the failure path of the candidate's own container is underivable as written.
No cost rides on the success path. The cost rides on whether the failure arm is *usable*: if it is
not, growth must be written as `Box::new(Slots::new(2 * cap))?` followed by `append` and `swap`,
which is one full memcpy per doubling where `realloc` or `mremap` can extend a large mapping in
place and copy nothing — roughly two gigabytes of traffic for a one-gigabyte container.

**Smallest fix:** print `grow(&b, cap) -> Result<Unit, Oom>` with `ensures when Err: (*b).cap ==
entry(*b).cap, (*b).len == entry(*b).len`.

**Severity: cost.**

---

### Group 7. What is the `Err` payload of a runtime-capacity window constructor?

**Task:** resource container (its G-B). **Covered: NO.**

Rule 5 promises the hand-back in general terms ("A fallible allocation that takes a by-value
payload hands it back on failure"), and Rule 6 constructs every container with a call that has no
payload to hand back but whose result type is nonetheless unstatable:

```text
fn new_vector<T>(cap: u64) -> Result<Vector<T>, ???> {
    b = Box::new(Slots::new<T>(cap))?     // Rule 6's own constructor line.
                                          // By Rule 5 the Err arm returns the payload:
                                          // Err((Oom, Slots<T>)) — but Rule 5 also says a
                                          // runtime-capacity shape "may appear only as the
                                          // content of a Box: never inline in another value
                                          // and never as a local variable", and an enum
                                          // payload is inline in a value.
    Ok(Vector { buf: b })
}
```

Benign — a fresh empty window owns nothing, so there is nothing to lose — but the constructor
every container in the file uses cannot state its own failure type.

**Smallest fix:** one clause — a fallible allocation whose payload is a freshly constructed
runtime-capacity shape fails with plain `Oom`, because the payload owns nothing.

**Severity: wording.**

---

### Group 8. There is no bulk move of *part* of a window

**Cell:** 6-6. **Covered: NO** — this is a missing operation, not an ambiguous sentence.

`append` is all-or-nothing:

> `append(&dst, &src)         writes(dst.free), writes(dst.len), writes(src.filled), writes(src.len)   // one memcpy`
> `contract { requires dst.room >= src.len; ensures dst.len == entry(dst).len + entry(src).len, src.len == 0; }`

and a range reference has no writable length ("a reference kind with measure `len`"), so no
operation moves a suffix of one window onto another.

```text
// B-tree node split, or Vec::split_off: move slots [mid, len) of src onto dst, in order.
// Wanted: append_range(&dst, &src[mid..src.len])  — no such operation.
// Available:

k = src.len - mid
for t in 0..k {
    invariant h: src.len == entry(src).len - t;
    e = remove_at(&src, mid)        // one memmove of the shrinking tail per element
    place_back(&dst, move e)        // one element move per element
}
```

Two element moves and two measure updates per element, in a loop that does not vectorize, against
one `memcpy` in C++ and Rust — roughly twice the traffic on the split path. Everything else about
bulk movement is now free: whole-window growth is `grow`, whole-window transfer is `append`.

**Smallest fix:** one operation, `append_range(&dst, &src, lo)`, writing `dst.free`, `dst.len`,
`src.filled` and `src.len`, with `ensures src.len == lo`.

**Severity: cost.**

---

### Group 9. May three or more statements that pairwise qualify overlap all at once?

**Cells:** 14-16, 13-16, 1-16. **Covered: NO.**

Rule 13 grants the permission pairwise over adjacent statements:

> Two adjacent statements of one block may overlap when the first's write paths are disjoint from
> the second's read and write paths and vice versa

```text
fill(&(*a.buf)[0..q])            // three heterogeneous statements, pairwise disjoint
scan(&(*b.buf)[0..q], &out1)     // s1 vs s2 disjoint, s2 vs s3 disjoint, s1 vs s3 disjoint
tally(&(*c.buf)[0..q], &out2)    // may all three run at once? The text says only "two".
```

No cell was blocked: the arithmetic cases in 13-16 and 14-16 escape through Rule 13's counted-loop
forms ("The existing counted-loop forms ... keep their permissions"), which are k-way by
construction. A k-way fan-out of *different* statements has no stated licence.

**Smallest fix:** one clause — a maximal run of adjacent statements that are pairwise disjoint may
overlap as a group.

**Severity: expressiveness.**

---

### Group 10. Is a function-typed parameter instantiated per call site?

**Cells:** 3-4, 5-15, 1-4. **Carried from round one's E3. Covered: NO.**

Rule 10's whole statement is:

> Function-typed parameters carry a full signature with its own row and contract, and a call
> through one uses that row.

Nothing says whether a call site that passes a function *literal* produces a direct call.

```text
fn find_and_use<F>(t: &Tree, k: u64, f: F)   reads(t), writes(f.ctx)
    where F: fn(node: &Node, ctx: &Acc) writes(ctx)
{ ... one descent; at the found node, f(node, &acc) ... }

find_and_use(&tree, key, |n, c| { (*c).total = (*c).total + (*n).v })
// If this is monomorphized, the callback is inlined and the fused find-then-mutate is free.
// If it is an indirect call, every hit pays one indirect call that the C++
// pointer-returning form does not.
```

Three cells price the inward find-then-mutate form at zero by asserting monomorphization. If that
is not promised, cells 3-4 and 5-15 become rejected-real-cost. Their verifier left the verdicts
standing because round one's reviewer accepted the same reading and the notation section defers to
"the existing WF" forms — which is a reasonable inference and not a sentence.

**Smallest fix:** one clause in Rule 10 — a call through a function-typed parameter whose argument
is a literal at the call site is a direct call.

**Severity: cost** (two verdicts turn on it).

---

### Group 11. Must a supplied function argument match the parameter's declared signature exactly?

**Task:** generic algorithms (its G5, new this round). **Covered: NO.**

The same sentence as group 10 is the only one that touches function-typed parameters, and the
generic-algorithms assignment asks precisely "what must a caller supply".

```text
fn map_into<T, U, F>(src: &[T], dst: &Slots<U, N>, f: F)
    reads(src), writes(dst.free), writes(dst.len)
    where F: fn(x: &T, ctx: &Ctx) -> own U   reads(x), reads(ctx)

// May a caller supply a function with a SMALLER row (reads(x) only, no ctx),
// a WEAKER requires, or a STRONGER ensures than the parameter declares?
map_into(&part, &out, |x, _c| double(x))       // reads(x) only: accepted, or rejected?
```

Zero cost on every reading: under the strictest one, every callback is written to match character
for character, taking an unused context parameter where one is declared, which is free.

**Smallest fix:** one clause — a supplied function's row must be covered by the declared row, its
`requires` implied by the declared `requires`, and its `ensures` must imply the declared `ensures`.

**Severity: expressiveness.**

---

### Group 12. Does a declared write path cover writes strictly below it?

**Task:** mutable graph (its third gap). **Covered: NO explicitly**, though Rule 3's phrasing
points the right way.

Rule 9 says what `writes` covers at a path:

> `writes` covers writing, replacing, moving out of, and freeing the storage at the path.

It never says that a write to a path *extending* a declared write path is covered by it. Rule 3
uses the wider phrase for a different purpose ("Writing the storage at p's path or below it"), and
Rule 10's rejected example `bad(&vv, &vv[0])` assumes the downward relation for the overlap test.

```text
fn repair_twin(g: &Graph, peer: u64)   writes((*g.nodes).filled)
    contract { requires peer < (*g.nodes).len; }
{
    // writes a slot of a Slots that lives inside a Box inside the slot (*g.nodes)[peer]:
    remove_at(&*(*g.nodes)[peer].uses, k)     // path: (*g.nodes)[peer].uses ... — strictly
                                              // below the declared (*g.nodes).filled
}
```

Zero cost either way: the strict reading is satisfied by promoting the peer index to an argument
and declaring the second path as well, which substitutes to the same storage at the call.

**Smallest fix:** one clause in Rule 9 — a declared write path covers every write at that path or
below it.

**Severity: wording.**

---

### Group 13. What may a contract say: refinement facts, and indexed paths?

**Task:** tree rewrite (its second and third gaps). **Covered: NO** on either half.

*First half — a refinement fact as a precondition.* Rule 2 makes a payload step available "only
under the refinement fact that the enum currently holds that variant, which a `match` or `if let`
on the enum establishes in the selected arm". Rule 11 lists refinement facts as facts, but exhibits
variant routing only on a *result* (`ensures when Variant:`).

```text
fn link(parent: &Node, kid: own Box<Node>)   writes(parent.left.Some.0)
    contract { requires parent.left is Some; }     // may a requires state a refinement fact?
```

Not load-bearing for any cell: every callee in this round takes `&Node` and does its own `match`,
one discriminant test both baselines also execute. It does leave a precondition unstatable, which
is a logic obligation rather than a safety one.

*Second half — whether the ban on index expressions reaches the contract block.* Rule 9 says:

> Signatures never contain index expressions; an index enters an effect only through an argument,
> evaluated once at the call.

The effect row is settled by the second clause. Whether an `ensures` may mention an indexed path
is not:

```text
fn resolve(p: &Pool, h: Handle) -> Result<u64, Stale>   reads((*p.slots)[h.idx])
    contract { requires h.idx < (*p.slots).len;
               ensures when Ok: (*p.slots)[h.idx].gen == h.gen; }   // legal?
```

Under the strict reading the guarded pool form reloads the generation at the caller: one load, one
compare, one predicted branch per call.

**Smallest fix:** two clauses — a `requires` may state a refinement fact about a parameter, and the
ban on index expressions applies to the effect row only, a contract being free to mention an
indexed path whose index is a parameter.

**Severity: cost** (conditional, on the second half only).

---

### Group 14. The fact vocabulary: what an explicit `use` step may cite, and whether a non-affine assignment yields affine facts

**Task:** mutable graph (its first gap), generic algorithms (its G2, carried from round one).
**Covered: NO** on both, and neither is load-bearing this round.

Rule 11's list ends with a form it never defines:

> Facts are the existing WF forms: affine comparisons over measures and integer values, refinement
> facts from a dominating branch or a `match` arm, loop-header invariants ..., **explicit `use`
> steps inside an `invariant`**, and callee contracts.

```text
// (a) what may a `use` step cite?
invariant h: i < (*p.slots).len;
    use mask_bound;                 // no grammar says what a use step names or what it may derive

// (b) does a non-affine assignment establish affine facts about its result?
mid = lo + (hi - lo) / 2            // are lo <= mid and mid < hi available afterwards?
```

Round one called (a) the mutable-graph task's highest-value question because it decided whether a
masked index on a power-of-two pool could discharge its bound. Revision 4 makes the answer moot
without closing the gap, by listing the bitmask fact under "Deferred to a future concurrency and
layout round" — so the escape is closed by decision, and the per-probe compare in cost group 1 is
permanent rather than conditional. (b) costs nothing: the division is displaced into a helper whose
`ensures` are affine, so the binary-search loop pays nothing.

**Smallest fix:** a grammar for `use` steps, in the existing specification rather than in x1.

**Severity: expressiveness.**

---

### Group 15. Integer overflow

**Task:** mutable graph (its second gap). **Carried from round one's Gap 13. Covered: NO.**

The candidate's only statement about integers is the notation line "`Int` and `u64` are Copy".
Nothing says whether an increment carries a proof obligation, a written outcome, or a wrapping
form — and the generation counter that Rule 16 itself prescribes is an increment:

```text
fn recycle(p: &Pool, i: u64)   writes((*p.slots)[i].gen)
    contract { requires i < (*p.slots).len; }
{
    (*p.slots)[i].gen = (*p.slots)[i].gen + 1      // proof? outcome arm? wrapping? unstated
}
```

This belongs to the kernel specification rather than to x1, which is a delta over the ownership and
access rules. It is recorded because two tasks needed it and could not find it. At most one compare
and one predicted branch per deletion either way.

**Severity: wording** (a scope note, not an x1 defect).

---

### Group 16. The atomic update's "f's row must not overlap any prefix of place" — covered, restrictively

**Cells:** 6-10. **Tasks:** tree rewrite (its load-bearing gap), generic algorithms (its section 7
counterexample), mutable graph (its second counterexample). **Covered: YES**, and the covered
reading is the expensive one.

Rule 6:

> `node.left = insert(node.left, k)   // f is total and returns the place's type; f's row must not`
> `c.f = reopen(c.f)                  // overlap any prefix of place; failure is an enum in the place`

The prefixes of `n.left` are `n` and the root local; the prefixes of `r[k]` are `r` and its root.
Rule 3 fixes how overlap is judged ("judged conservatively"), and a path is trivially overlapped by
its own prefix, so `reads(n.right)` overlaps the prefix `n`, and `reads(r[m])` overlaps the prefix
`r`. Read literally, `f` may touch nothing under the place's root — not a sibling field, not
another slot of the same window, not the container the place lives in.

```text
n.left = fold(n.left, &n.right)     // the canonical binary constant-fold:
                                    // fold's row reads(n.right) overlaps the prefix n.
                                    // Rejected under the literal reading.

// admitted rewrite, +2 stores and a transient hole in the tree:
t = None
swap(&n.right, &t)                  // take the sibling out
n.left = fold(n.left, &t)
swap(&n.right, &t)                  // put it back
```

Three tasks flagged this as their load-bearing ambiguity and one cell recorded it as a gap. I do
not: the sentence decides the case, and what the purposive reading wants (`k != m` would be enough
for two slots; disjoint fields would be enough for two children) would be a *new* rule. It is
therefore a decided cost, priced in Part 3, cost group 5 — and it is the one place where revision
4's most general new operation is unusable exactly where it would help most, since any `f` that
reads the surrounding structure overlaps the root.

**Severity: cost**, not a gap.

---

### Group 17. A write strictly below a variant payload path — covered

**Cells:** 3-12, 9-9. **Covered: YES.**

Rule 2 says a refinement fact is one "which any write to the enum invalidates", which read broadly
would kill a payload reference at the first write *through* that reference. Rule 3 decides it:

> Writing the storage at p's path or below it (a content write) does not invalidate p.

```text
match &tbl.slot {
    Some(e) => {                         // e names tbl.slot.Some.0 under the refinement fact
        (*e).count = (*e).count + 1      // below e's path: a content write; e stays valid
        (*e).stamp = now                 // second write: still valid, no re-match
    }
    None => {}
}
tbl.slot = None                          // a write of the enum place: the fact dies, e invalid
```

The broad reading would make Rule 3's content-write sentence false for a write through the payload
reference itself, so only the narrow reading leaves Rule 3 self-consistent. Cost under the narrow
reading: zero; under the broad one it would have been one discriminant load, compare and predicted
branch per additional write. Cell 3-12 moves from undecided to accepted-fine on this.

---

### Group 18. Whether emptying a linear window discharges the container's own obligation — covered

**Cell:** 6-8. **Task:** resource container (its first text note). **Covered: YES.**

Rule 6 states the obligation and, in the same breath, how it is met:

> No operation releases a linear element: a storage whose element type is linear is itself linear
> (Rule 8) and **the program must take every element out and consume it**.

Rule 8's own example says the same of the type: "`Slots<File, 4>   // linear: every element must be
taken out and closed", and Rule 6's release sentence covers the emptied block: "at scope exit the
compiler releases the slots inside the window recursively and frees the block".

```text
b: Box<Slots<File>>                    // linear by containment (Rule 8)
while (*b).len > 0 {
    f = take_back(&*b)                 // writes((*b).last), writes((*b).len)
    close(move f)                      // the explicit consumption Rule 8 demands
}
// scope exit: the window holds no element; the compiler frees the block. Accepted.
```

The obligation attaches to the elements; the emptied block is memory. The competing reading would
make `Box<Slots<File>>` a type no program can finish, and cells 1-8 and 4-8 of this round both read
it the accepted way. Residual wording weakness, worth one clause: Rule 8's "the compiler never
releases them" still literally names the *container*, whose type is linear even when it is empty.

---

### Group 19. A function value that captures a reference and is only passed as an argument — covered

**Cells:** 2-4, 10-13. **Task:** generic algorithms (its G1, carried from round one). **Covered:
YES, by making the permission vacuous.**

Rule 4 permits the narrow case by its wording ("cannot be captured by a function value **that is
stored or returned**"), and Rule 9 then removes every effectful instance of it:

> An effect row lists `reads(path)` and `writes(path)` where each path starts at a reference
> parameter ... A function body is checked against its own row: every statement's effect and every
> callee's substituted row must be covered by the declared row.

```text
acc: &Acc
h = |x: &Buf| { merge_into(acc, x) }    // h's body performs writes(acc); acc is not a parameter
                                        // of h, so no row of h can name it: the body fails its
                                        // own check. Rejected before Rule 13 is consulted.

fn h2(acc: &Acc, x: &Buf)  writes(acc), reads(x)     // the rewrite: same machine word,
h2(&acc, &a); h2(&acc, &b)                           // same register. May not overlap
                                                     // (both write acc) — correctly.
```

The rules are consistent; the permission is simply empty for anything that reads or writes through
the capture. With no `par` statement, the stakes are higher than in round one, since the declared
row is now the *only* thing that decides whether two adjacent calls may overlap — which is what
makes the vacuity a feature rather than a defect.

---

### Group 20. Two editorial defects

**Cells:** 1-16, 6-6. **Covered: NO** (they are drafting slips, decided elsewhere).

Rule 16's `alloc` still carries a cross-reference to a section that no longer exists:

> `fn alloc<T>(a: &Arena<T>, x: own T) -> u64   // row: the append slot of *a.buf and (*a.buf).len; see the open proposal`

while "Proposed additions awaiting owner ruling" now reads "None. The window-parts vocabulary was
adopted into Rule 6 in revision 4." The row is fully determined without the reference — Rule 6's
`place_back(&r, x)  writes(r.next), writes(r.len)` plus "A user function that only appends declares
the same row as `place_back`" — so no derivation was blocked. The pointer should name Rule 6.

Second: Rule 6's library list calls `Deque<T>` a zero-cost composition — "`Deque<T>` (`Box<Ring<T>>`
plus growth)" — while `grow` is annotated "`Box<Slots<T>>` only". A ring therefore grows by a fresh
allocation plus `append`, one full copy per doubling (Rust's `VecDeque` parity; `std::deque` moves
nothing and pays an indirection per access instead). Either `grow` extends to rings or the list
should say so.

**Severity: wording.**

---

## Part 3 — Confirmed real costs, grouped by cause

"Real" means no rewrite inside the rule set removes it. Runtime performance is the only cost
counted; verbosity, extra parameters and duplicated code are not costs. Status is against round
one.

### Cost 1. One compare and one predicted branch per data-determined index — UNCHANGED, now permanent

**Rule responsible:** Rule 11 — "There are no quantified facts over array elements ('for all i ...')
and no per-slot occupancy facts" — with Rule 7's proof obligation.
**Cells:** 2-2, 2-16, 3-3, 4-12, 5-16, 7-16, 8-16, 10-16, 12-16, 15-16.
**Tasks:** mutable graph (its dominant cost), generic algorithms, tree rewrite.

```text
// every hop of an IR edge list, every free-list pop, every hash probe:
h = (*g.edges)[e].target                 // an index read out of the data
if h < (*g.nodes).len {                  // Rule 7: the fact is unavailable, so the program tests
    n = &(*g.nodes)[h]
    ...
} else {
    // an arm no rule can delete: there is no trap and no `unreachable`
}
```

Round one left the power-of-two bitmask escape open; revision 4 closes it by decision, listing "A
bitmask fact (`x & (c - 1) < c` for a power-of-two `c`)" under Deferred. Equal to safe Rust; real
against C++, where it also blocks software-pipelining two hops.

### Cost 2. Iterative by-reference descent of an owned linked structure is refused — NEW

**Rule responsible:** Rule 2 — "A path has a static shape: a loop-carried rebinding may change only
the index values inside the path, never extend the path through itself."
**Cells:** 1-4, 2-2, 3-3, 4-12. **Task:** tree rewrite (its strongest counterexample this round).

```text
// wanted, and refused:
p = &root
while (*p).left is Some {
    p = &(*p).left.Some.0          // rejected: the path grows through itself
}

// admitted form 1 — recursion: +1 call and +1 return per hop, O(depth) stack where the
// loop used O(1), no tail-call form stated anywhere in the file:
fn descend(n: &Node) -> u64   reads(n) {
    match &n.left { Some(c) => descend(c), None => (*n).v }
}

// admitted form 2 — index links in one window: +1 compare and +1 predicted branch per hop
// (cost 1 above), and it gives up the ownership identity guarantee (Rule 16).
```

This is the round's one genuinely new cost, and it lands on the hottest loop of a tree-rewriting
compiler pass. The no-trap promise compounds it: once descent must recurse, a careful program adds
a depth budget, one further compare and predicted branch per level that neither baseline pays (see
Part 2, group 5). The rule file records the refusal under Known costs ("recursion or indices cost
the same loads") but prices only the loads, not the frames or the budget.

### Cost 3. Find-then-mutate, split across a call boundary — REDUCED

**Rule responsible:** Rule 4 — "cannot be returned".
**Cells:** 1-4, 3-4, 5-15. **Task:** tree rewrite.

```text
// fused: free, and this is the common case revision 4 newly admits
node.left = insert(node.left, k)                 // Rule 6's own line

// split, because the finder is separately compiled: Rule 4 forbids returning the reference
match find(&tree, k) { Some(h) => { mutate(&tree, h) } None => {} }
// +d dependent loads, +d discriminant tests, +d calls and returns per find-mutate pair
```

Also: a durable *cursor* into a Box-linked structure does not exist at all. The replay form costs a
second full chain of dependent, frequently missing loads; the only durable cursor is a pool handle
at one compare per use plus a generation word where identity matters.

### Cost 4. The atomic update commits one store per level on the unwind — NEW

**Rule responsible:** Rule 6 — "the old value enters f by value, f's result is committed".
**Cells:** 4-5, 5-15. **Task:** tree rewrite.

```text
fn insert(t: own Option<Box<Node>>, k: u64) -> own Option<Box<Node>> { ... }
node.left = insert(node.left, k)     // one pointer-word store at every level on the way back up
                                     // ~20 stores per insert into a balanced million-node tree,
                                     // to lines already hot
```

Against C++'s pointer-to-pointer descent (`Node** p = &root; while (*p) p = &(*p)->left;`), which
stores once — and that descent is itself refused by cost 2. Zero against idiomatic safe Rust, which
also rebuilds the spine. Note the direction of travel: this cost *replaces* a larger round-one cost,
since round one had no admitted in-place update with arguments at all.

### Cost 5. The atomic update's `f` may not read anything under the place's root — NEW

**Rule responsible:** Rule 6 — "f's row must not overlap any prefix of place" (Part 2, group 16).
**Cells:** 6-10. **Tasks:** tree rewrite, generic algorithms (its section 7 counterexample), mutable
graph (its second counterexample).

```text
r[k] = merge(r[k], &r[m])            // rejected: reads(r[m]) overlaps the prefix r
n.left = fold(n.left, &n.right)      // rejected: reads(n.right) overlaps the prefix n
lat[i] = meet(lat[i], &graph)        // rejected: any f that reads the pool overlaps its root

// rewrite for two slots: +1 element move and +1 measure update per merge
x = swap_remove(&r, m); r[k] = merge(r[k], move x)
// rewrite for two fields: +2 stores and a transient hole (see group 16)
// rewrite for a lattice over a graph: two columns ping-ponged between rounds,
//   +1 column of memory, no extra pass, zero time cost
```

The sharpest finding of the mutable-graph task: revision 4's most general new operation cannot be
used on a pooled graph at all. Safe Rust pays neither the two-field nor the two-slot form (disjoint
field borrows, `get_many_mut`).

### Cost 6. No hole, no moved-from state: relocating a non-Copy element — REDUCED (this was round one's largest cluster)

**Rule responsible:** Rule 6 — "A move out of a window slot or an array element is rejected" — with
the exclusion of `take`/`put` holes and of partial moves.
**Tasks:** generic algorithms (insertion-sort inner shift, stable retain, unstable retain, the
in-place fold), resource container (the short-run finish of an in-place sort).

```text
// insertion sort's inner shift over T = Box<Record>: no temporary hole exists,
// so the run is shifted by adjacent swaps
while j > lo && less(&part[j], &part[j - 1]) {
    swap(&part[j], &part[j - 1])     // 2 loads + 2 stores per displaced element
    j = j - 1                        // against a moved-out temporary's 1 load + 1 store
}
// 3L element copies against L + 2; scalar and serialized where pdqsort's memmove vectorizes.
// Predicted 10-20% slower sort for 16-byte elements.
```

What changed: round one had **no** two-slot exchange for any non-Copy element type, which removed
in-place sort, in-place partition and stable compaction for every affine and linear type and was
the whole generic-algorithms counterexample. Revision 4's built-in `swap` — "p and q may be the same
place" and "no `i != j` branch needed in a partition loop" — makes partition, quicksort and
`swap_remove` parity with `std::swap`. What survives is only the *chain* case, where the traveling
value should stay in a register across consecutive exchanges, and the in-place fold, which cost 5
forbids.

### Cost 7. An order-preserving partial move of a window — NEW (residue of the same cause)

**Rule responsible:** Rule 6's operation set (Part 2, group 8).
**Cell:** 6-6. Two element moves per element against one `memcpy`; roughly 2x on a B-tree node split
or a `split_off`. Whole-window growth and whole-window transfer are now free, which is the larger
half of round one's cost and is closed.

### Cost 8. Construction into the append slot moves one element — UNCHANGED

**Rule responsible:** Rule 6 — every operation that raises `r.len` supplies the element itself.
**Cell:** 4-9. **Task:** resource container.

```text
x = make(&s); place_back(&r, move x)          // one sizeof(Entry) move per append
place_back(&r, blank); build_into(&r.last, &s) // or one placeholder store per append
// against C++ emplace_back, which constructs at the destination.
```

Revision 4 removes round one's *contradiction* (the append slot is now nameable as `r.next`) without
removing the cost, since no operation publishes a slot the program already filled. Real for an
inline payload; a few words when the element's bulk is behind a `Box`. Already in the rule file's
Known costs.

### Cost 9. Scatter to data-determined destinations cannot overlap — UNCHANGED

**Rule responsible:** Rule 11 (injectivity is a quantified fact) with Rule 13, and the exclusion of
channels and atomics.
**Cells:** 10-16, 13-16. **Tasks:** generic algorithms, mutable graph.

```text
scatter(&in[0..mid], &out, &perm[0..mid])
scatter(&in[mid..n], &out, &perm[mid..n])     // may NOT overlap: both carry writes(out) whole
// privatize: P * (*nodes).len * 8 bytes plus a merge pass
// transpose:  one pass plus a permanent |E| * 8 bytes
```

Parity with safe Rust and with rayon; a real loss against C++ with one atomic add per edge, and
neither rewrite tunes away.

### Cost 10. Two allocations from one window may not overlap — REDUCED

**Rule responsible:** Rule 6's `place_back` row with Rule 10 clause 1.
**Cells:** 1-16, 5-16, 13-16, 15-16. **Task:** mutable graph.

```text
id1 = alloc(&g, n1)          // writes((*g.nodes).next), writes((*g.nodes).len)
id2 = alloc(&g, n2)          // identical paths: no distinctness proof exists. May not overlap.

p = &(*g.nodes)[i]
id = alloc(&g, node)         // NEW in revision 4: p survives, and this may overlap a write
(*p).a = id                  // of any live slot. Round one refused both.
```

The rewrite — one pre-filled `Box<Array<Node>>` and per-worker disjoint ranges — costs nothing per
hop, because handles stay one level. Round one's per-worker pools with two-level handles, and their
extra dependent load on every hop for the life of the structure, are withdrawn.

### Cost 11. A data-chosen slot rewrite declares `r.filled`, serializing the pool — REDUCED, still dominant for rewriting

**Rule responsible:** Rule 6's part vocabulary with Rule 9's "Signatures never contain index
expressions".
**Task:** mutable graph (its summary judgment). **Cells:** 10-16, 13-16.

```text
fn delete_edge(g: &Graph, e: u64)   writes((*g.nodes).filled), writes((*g.edges).filled)
// the target slot is read out of the graph, so it cannot be an argument, so the row is
// .filled-wide on one root. No other statement may overlap it, and (under the strict
// reading of Part 2 group 2) every interior reference dies at it.
```

Revision 4 improved the *creation* axis — an appending user function is precise, and a reference
survives it — and left the *rewriting* axis. For an IR pass, rewriting is the hot one.

### Cost 12. Generation words wherever identity must survive slot reuse — UNCHANGED

**Rule responsible:** Rule 16 ("a logic error, not a memory error") with Rules 1 and 4, which make an
index the only durable name.
**Cells:** 2-16, 3-16, 4-16, 5-16, 7-16, 10-16, 15-16, 16-16.

Per dereference: one generation load (usually the same cache line), one compare, one predicted
branch, plus a stale arm at the caller. Per handle: four to eight bytes, so an edge grows from 8 to
12 or 16 and halves the edges per cache line in a pointer-chasing traversal. With a pool *reset* in
the picture (cell 16-16), two detectors are needed — generation and epoch — so two loads, two
compares and one to two branches. Equal to a Rust slotmap; the whole cost against a C++ raw pointer,
which pays with undefined behaviour instead.

### Cost 13. Growth through the fallback, if `grow`'s failure arm is unusable — NEW, conditional

**Rule responsible:** the gap in Part 2, group 6. One `memcpy` of the whole window per doubling where
`realloc`/`mremap` extends a mapping in place: about two gigabytes of traffic for a one-gigabyte
container. Also: `grow` requires `cap >= (*b).cap`, so *shrink to fit* is only expressible as a new
block plus `append` plus `swap` — one full copy where `realloc` usually shrinks in place at zero.
Disappears entirely if `grow` is fallible with an `Err` arm that leaves the window intact.

### Cost 14. No channels, no atomics — UNCHANGED

**Rule responsible:** the exclusion list; the channel primitive is under Deferred.
**Cells:** 13-16, 14-16. **Task:** mutable graph (its parallel worklist).

A sparse fixpoint costs O(nodes) per round plus a barrier instead of O(dirty) — the only asymptotic
loss in either round. A free-list byte arena cannot have its reused blocks filled concurrently
(the disjointness invariant is quantified over the free list's contents), so per-worker sub-arenas
raise peak memory by roughly k times the per-worker imbalance and forfeit cross-worker reuse.

### Cost 15. The guarded partition scan — UNCHANGED

**Rule responsible:** Rule 11 (the sentinel premise is a quantified fact over elements).
**Task:** generic algorithms. One compare and one predicted branch per scanned element, about n per
partition level, against `libstdc++`'s unguarded partition; parity with safe Rust.

### Costs closed by revision 4

For the record, five of round one's clusters are gone, not reduced:

- **Every window operation killed every interior reference.** Closed by window parts. The
  mutable-graph task's headline requirement — a node reference held live across a separately
  compiled append — now works, and a reload per node creation disappears.
- **Order-preserving growth at about 2.5 element moves each, ordered removal at 3 moves per shifted
  element, interior exchange at 9 moves.** Closed by `grow`, `remove_at`, `insert_at` and `swap`.
  The resource-container task's round-one counterexample (a predicted 4-8x gap on an ordered
  connection table) is retired.
- **Box-linked structures forced to carry a child window** (one extra dependent load and one
  allocation per node). Closed by variant payload paths.
- **A linear value could not cross a fallible allocation in a `Box`** (sixteen bytes of block header
  per linear cell). Closed by the hand-back.
- **`place_back` could not be declared precisely across separate compilation.** Closed; and an
  append may now overlap a kernel over the settled prefix, which safe Rust refuses and C++ admits
  only as undefined behaviour.

---

## Part 4 — Every accepted-unsafe claim, re-traced against revision 4

Nine claims, five of which the cells of this round label `accepted-unsafe`.

### Survives the text — memory errors

**U1. Reading and writing the append slot** (cells 9-12, 4-9, 2-9; Part 2, group 1). The program is
printed there. Rule 6 makes `r.next` a path that "paths and effect rows may name" and says the same
slots "hold nothing"; a callee given `&r.next` cannot tell, and Rule 6's assignment clause then
releases an "old value" read out of uninitialized storage. **Survives**, and it is the only memory
error in revision 4. Confined to Copy and affine element types, since the linear case is separately
rejected.

**U2. A range reference used after the window shrank past it** (cell 3-7; Part 2, group 3).
**Survives conditionally** — only under the reading in which Rule 3's content-write clause governs a
range reference. The cell correctly refused to resolve it and marked itself undecided rather than
claiming the acceptance.

### Survives the text — identity errors

**U3. A reference renumbered by `remove_at` or `insert_at`** (cell 4-6, found by this round's
verifier; Part 2, group 2). The program is printed there. `(*b).filled` overlaps `(*b)[7]` but is not
a proper prefix of it, so Rule 3's invalidation list does not fire, while the `ensures` keeps the
formation fact standing. Memory-safe — a memmove neither frees nor empties a slot — and identity-wrong.
**Survives**, and it reverses cell 4-6's own headline contrast, which claimed the reference form is
protected where the index form is not.

**U4. A stale in-bounds handle names the slot's new occupant** (cells 4-16, 2-16, 16-16).
**Survives, by decision.** Rule 16 is confirmed and states it in terms: "A stale index that is still
in bounds names the current occupant of that slot: a logic error, not a memory error. Programs that
need to detect it keep a generation number as data." Revision 4 adds a second route into it, since
`insert_at` and `remove_at` on a pool renumber every handle in the structure at once.

**U5. A reference into a user-level pool survives the program's own free** (cell 3-16). **Survives,
by decision, and now explicitly.** Rule 16's new final sentence extends the disposition from indices
to references: "storage that still exists stays readable, and what the program considers 'freed' is
the program's own bookkeeping." What Rule 3 still protects is exactly storage lifetime: only `grow`,
scope exit, or a whole-owner move refuse the reference.

**U6. Slot reuse composed with a pool reset** (cell 16-16). **Survives**, with load-dependent
detection: the bounds test fires nondeterministically with input size, so two detectors are needed
rather than one. Revision 4 closes the silent-discard half of round one's version of this cell, since
`truncate` is now a `take_back` loop and Rule 6 refuses to release a linear element.

### Does not survive revision 4

**U7. Overlap permission licensing two writes to one slot** (cell 1-16, withdrawn by this round's
verifier). The deriver granted `alloc(&g, node)` and `set_data(&g, n0, 7)` permission to overlap by
reading Rule 6's "a live `r[i]` never overlaps `r.next`" across a state change. The correct trace
withholds it: Rule 10 clause 1 requires "indices or ranges proved distinct", and `n0` is not proved
distinct from the append index. **Does not survive** — but the text gap that let a careful reader
derive it is real and is Part 2, group 4.

**U8. A linear payload lost on the `Err` arm of a fallible allocation** (round one's cell 8-14).
**Closed** by Rule 5's hand-back; the cell is accepted-fine this round. Separately, the cell's printed
program is itself *rejected* by Rule 8 on its `Ok` arm, where a second `own File` parameter is never
consumed — a program defect, not a rule defect, repairable at zero cost by carrying it out in the `Ok`
type.

**U9. `truncate` discarding linear elements unclosed** (round one's cells 6-8, 8-16, 16-16).
**Closed** by Rule 6's "No operation releases a linear element" plus `truncate`'s demotion to library
code.

---

## Part 5 — Verdict totals over the 66 re-derived cells

Using the verifiers' corrected verdicts (five corrections in the window batch, four in the rules
batch).

| Verdict | Round one (same 66 cells) | Round two | Change |
|---|---|---|---|
| `accepted-fine` | 15 | **30** | +15 |
| `rejected-zero-cost` | 8 | **13** | +5 |
| `rejected-real-cost` | 19 | **16** | -3 |
| `undecided-rule-gap` | 19 | **2** | -17 |
| `accepted-unsafe` | 5 | **5** | 0 |

Per batch, round two: window rows 1-5 (16 cells) — 11 fine, 3 zero-cost, 2 real-cost, 0 undecided,
0 unsafe. Window rows 6-12 (14 cells) — 8, 2, 3, 1, 0. Validity (18 cells) — 6, 5, 5, 1, 1.
Overlap and allocation (18 cells) — 5, 3, 6, 0, 4.

Reading the table. Seventeen of the nineteen round-one undecided cells became decided, and almost all
of them decided *favourably*: accepted-fine doubled. The two that remain are the two
window-part questions of Part 2, groups 1 and 3 — cell 9-12 (is `r.next` a place?) and cell 3-7 (does
a range reference die at a shrink?) — and both are one sentence from closed.

The unsafe count is flat at five, but its composition changed completely. Round one's two
*resource* unsafeties (the lost linear payload, the discarded linear elements) are closed. What
replaces them is cell 2-16, relabelled by its verifier for consistency with the three companion cells
on the identical stale-handle hole: all five of round two's unsafe acceptances are now the *same*
deliberate disposition, Rule 16's ruling that a stale in-bounds handle is a logic error. That is a
much healthier five than round one's five.

Round-two verdicts by cell, for reference:

- **accepted-fine (30):** 1-8, 1-12, 2-6, 2-7, 2-8, 2-14, 3-6, 3-9, 3-12, 3-13, 4-8, 4-14, 5-5, 5-6,
  5-8, 6-8, 6-9, 6-10, 6-12, 6-14, 6-15, 8-8, 8-9, 8-14, 9-9, 9-16, 10-11, 10-14, 12-13, 13-14
- **rejected-zero-cost (13):** 1-3, 1-10, 1-16, 2-4, 2-9, 3-4, 3-8, 5-15, 7-7, 8-10, 10-13, 10-15,
  15-15
- **rejected-real-cost (16):** 1-4, 2-2, 3-3, 3-10, 4-5, 4-9, 4-12, 5-16, 6-6, 7-16, 8-16, 10-16,
  12-16, 13-16, 14-16, 15-16
- **undecided-rule-gap (2):** 3-7, 9-12
- **accepted-unsafe (5):** 2-16, 3-16, 4-6, 4-16, 16-16

**Tasks.** All four re-derived tasks are achievable with cost, and all four are cheaper than under
revision 2. The resource container is at parity except for an in-place sort's short-run shift and the
reallocation cases; the tree rewrite is zero cost against a shape-for-shape safe-Rust baseline
everywhere except the refused iterative cursor, and beats Rust by one store per in-place child
rewrite; generic algorithms lost its entire round-one counterexample to the built-in `swap` and its
remaining costs are the guarded partition scan and the in-place fold; the mutable graph got its
headline requirement working and kept its two dominant costs, the per-hop bounds compare and the
`.filled`-wide rewriting row.

---

## Part 6 — Fixes, in order of value

1. **Say that a window part is a name for rows and overlap, not a place** (Part 2, group 1). One
   sentence in Rule 6. Closes revision 4's only memory-safety hole and this round's second undecided
   cell. Zero runtime cost.
2. **Say which operations invalidate a reference into a window** (group 2). One sentence naming the
   renumbering operations. Resolves a direct contradiction between two cells of this round and closes
   a silent identity error. Cost of the strict reading: one scaled add per re-formation.
3. **Extend the validity sentence to range references** (group 3). One clause in Rule 6. Closes a
   use-after-free and this round's first undecided cell. Zero cost.
4. **Fix the state in which two adjacent statements' paths are judged** (group 4). One sentence in
   Rule 13. Closes a derivable write/write race. Zero cost.
5. **Give `grow` a result type and a failure arm** (group 6). It is the container's own growth path,
   and the fallback costs one full copy per doubling.
6. **Say something about stack depth** (group 5), now that Rule 2 forces traversal into recursion and
   Rule 8 makes release recursive. Even an explicit deferral is better than silence in a rule set
   whose first promise is no runtime traps.
7. **Say whether a function-typed parameter is instantiated per call site** (group 10). Two verdicts
   turn on it.
8. **Add an `append_range` operation** (group 8), the last piece of the bulk-move family, at 2x on
   every split path until it exists.
9. **Reconsider the atomic update's prefix clause** (group 16). It is decided, and it is what makes
   revision 4's most general new operation unusable on a pooled graph and on a sibling field. A
   narrower clause — `f`'s row must not overlap `place` itself, nor free or relocate its owner — would
   recover the binary constant-fold and the in-place merge at zero cost.
10. **The small ones:** the `Err` type of a window constructor (group 7), three-way overlap (group 9),
    callback signature matching (group 11), downward coverage of a write path (group 12), refinement
    facts in a `requires` and indexed paths in an `ensures` (group 13), the `use`-step grammar (group
    14), integer overflow (group 15, which belongs to the kernel specification), and the two editorial
    defects (group 20).
