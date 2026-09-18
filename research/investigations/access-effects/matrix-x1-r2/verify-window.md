# Verifier's re-trace of `cells-window-a.md` and `cells-window-b.md` (revision 4)

Every one of the 30 cells was re-traced against `CANDIDATE-X1.md` revision 4 and
against nothing else. Five verdicts change. Two gaps the cells raise are
dismissed as covered by a quoted sentence; one gap the cells did not raise is
confirmed and one gap a cell resolved for itself is confirmed. One accepted
program under one reading of Rule 6's window-parts paragraph frees uninitialized
storage; it is written out in full at the end.

## Per-cell entries

### cells-window-a.md

- **1-3: agree.** `add_node`'s row is `writes((*g.nodes).next), writes((*g.nodes).len)`;
  Rule 10 clause 3 asks for "a proper prefix among the call's write paths" and
  Rule 6 answers "a live `r[i]` (which has `i < r.len`) never overlaps `r.next`
  or `r.free`". The bound survives by Rule 11's `entry` carry. The move half is
  Rule 3's own printed example. `rejected-zero-cost` is the conservative label
  for a strongest program one of whose statements is rejected; I keep it.
- **1-8: agree.** The drain loop is discharged by Rule 6, "No operation releases
  a linear element ... the program must take every element out and consume it",
  with `take_back`'s `requires r.len > 0` supplied by Rule 11's own
  `while r.len > 0` example. The emptied-block question that cell 6-8 raises is
  real but is *not* a gap (see "Dismissed gaps" D2), so 1-8 reaching
  `accepted-fine` is correct and 6-8, not 1-8, is the cell that has to move.
- **1-12: agree.** Rule 6's "a remaining linear part rejects the move (take it in
  the same destructuring)" decides the `move c.in_f` branch by name, and
  `let Conn { f, g, .. } = move c` is the printed admitted form. Note in passing
  that 1-12 labels a probe-rejection-plus-zero-cost-rewrite `accepted-fine`
  while 1-3 and 3-8 label the same shape `rejected-zero-cost`; both readings of
  protocol step 6 are defensible and I do not correct either, but the batch
  should pick one.
- **2-6: agree** on the verdict, **with one trace claim made gap-dependent.**
  `take_back` under `i + 1 < dst.len`: Rule 6, r[i] "overlaps `r.last` unless
  `i != r.len - 1` is proved" — proved, so the reference survives, and Rule 11's
  `entry` carry re-proves `i < dst.len` with no compare. The `append`
  destination side is decided by "never overlaps `r.next` or `r.free`". The
  `q = &src[j]` rejection is right, but the cell's stated reason
  ("`src.filled` overlaps every live `src[j]`") leans on the unstated
  part-as-prefix question (gap G1); the *decided* reason is Rule 6's
  `ensures src.len == 0` plus "A reference `p = &r[i]` into a window ... stays
  valid while that fact holds", which fails outright at `src.len == 0`. The
  "one scaled add per re-formed reference after `insert_at`/`remove_at`" cost is
  charged only under one reading of G1.
- **2-8: agree.** `close(move (*b).f)` is Rule 6's consuming move with `*b` as
  the owner and `n` Copy, so no remaining linear part rejects it; Rule 5's
  unbox line supplies the free. `p` dies by Rule 3, "a proper prefix of p's path
  is ... moved out of", and the read is correctly hoisted above the move. The
  zero-cost claim is right: no `sizeof(Conn)` copy appears because the field is
  taken directly out of Box content.
- **2-9: agree.** `&r[r.len]` is refused by Rule 7's `requires i < r.len`, and
  the set-valued half is Rule 2's "every check on it must hold for every member
  of the set" over Rule 10 clause 1. The cell's aside that "a user function that
  writes `r.next` cannot make the slot live" presumes such a function is a
  program at all; that presumption is gap G2.
- **2-14: agree.** Rule 3's invalidation list is closed over named prefixes, so
  no allocation touches `p`; Rule 14's own `a = build(&x)?; b = build(&y)?` is
  the adjacent pair; `grow`'s `writes(*g.nodes)` is a proper prefix and kills
  `p` on both arms, which Rule 9's "`writes` covers writing, replacing, moving
  out of, and freeing" extends over the failure arm.
- **3-6: agree.** `&runs.last` is formable because Rule 6 says the four parts are
  "named parts that paths and effect rows may name" and the dominating `runs.len > 0`
  gives the slot; Rule 2's "the path records that value" freezes the index at
  formation; Rule 3's content-write sentence keeps the reference across
  `(*last).n = ...`; Rule 6's "never overlaps `r.next`" keeps it across
  `place_back`. Unlike `r.next`, `r.last` under `r.len > 0` is a live slot, so
  gap G2 does not reach this cell.
- **3-8: agree** on `rejected-zero-cost`. Rule 6's consuming move plus Rule 3's
  proper-prefix clause invalidate `p` and the release of `scratch` frees the
  block it named; the destructuring re-roots at zero cost and Rule 3's "A move
  never re-roots an existing reference" is why `q` must be formed again. Trace
  defect, not verdict-changing: neither program discharges Rule 7's "Every index
  must be proved in bounds" for `(*scratch)[0]`; the obligation is identical
  before and after the rewrite, so the delta stays zero.
- **3-12: correction: `undecided-rule-gap` -> `accepted-fine`,** because Rule 3
  decides the second write: "Writing the storage at p's path or below it (a
  content write) does not invalidate p." `(*e).count = ...` writes a path below
  `e`, so under the broad reading Rule 3 would both invalidate `e` (through the
  refinement-fact item of its own list) and not invalidate it (through this
  sentence). Only the narrow reading leaves Rule 3 self-consistent, so the two
  stores are accepted and the cost is the already-recorded "one null check per
  hit versus hashbrown". The cell's own quoting is honest; the sentence it
  needed is in Rule 3, not Rule 2. (What stays undecided is a *different*
  reference into the same enum across a sibling write, which this cell's program
  never forms.)
- **4-5: correction: `accepted-fine` -> `rejected-real-cost`,** for two reasons
  the cell itself supplies but does not carry into the label. (i) The C++ form is
  refused: a pointer-to-pointer descent is a reference rebound through its own
  path, which Rule 2 prints as rejected ("`loop { p = &(*p.kids)[0] }` //
  rejected"), and Rule 4 refuses the returned-reference variant. The admitted
  rewrite is the atomic update, and Rule 6 states that its result "is committed",
  so the unwind stores one `Option<Box<Node>>` per level where the C++ idiom
  stores one pointer once — the cell measures this at "roughly 20 stores per
  insert" and then calls the cell accepted. Under protocol step 4/6 a refused
  program plus a rewrite with unavoidable cost is `rejected-real-cost`. Note
  that the candidate's own Known-costs entry ("recursion or indices cost the
  same loads") prices the loads and is silent on these stores. (ii) The written
  program is ill-formed: it uses `Box::new(...)?` inside `insert`, while Rule 6
  requires "`f` is total and returns the place's type" and "failure is an enum
  in the place" — the cell's trace says this and its program contradicts it.
  Repairing it costs more, not less: either the Oom arm returns the place
  unchanged and the insertion is lost silently, or the node is allocated before
  the update and passed as an extra argument, which allocates once per insert
  attempt including duplicate keys.
- **4-8: agree.** `remove_at` hands the element out as `own T` and Rule 6's "No
  operation releases a linear element" is what keeps the obligation travelling
  with it; the atomic update is stated "to any owned place, not only to window
  slots" and Rule 6's assignment clause ("rejected if it is linear") is what
  forces it over `set`. Trace defect only: the bounds for `(*pool)[i]` and
  `remove_at(&*pool, j)` ride on contracts for `find_idle`/`find_dead` that the
  cell does not write out (Rule 4's own `ensures when Some: result < v.len`
  supplies the shape). Teardown of the emptied block is D2, not a gap.
- **4-9: agree** on `rejected-real-cost`. The decisive sentence is Rule 6's
  measure clause — "A program reads them like fields and can never assign them;
  only the operations below change them" — and every listed operation that
  raises `r.len` supplies the element itself, so no reading produces a
  length-only commit. Rewrite (b) is safe because `&r.last` after `place_back`
  is a live slot. But the cell's first program, `build_into(&r.next, &s)`,
  is written as if it were well-formed, which flatly contradicts 9-12's
  conclusion in the same batch; that is gap G2, and for an affine `Entry` this
  exact program is the unsafe one written out at the end of this file.
- **5-5: agree** on the verdict, **correct the cost arithmetic.** The four swaps
  are decided by Rule 6 ("Two operations apply to any owned place ... `swap(&a.left, &b.right)`")
  with every pair on distinct roots, so Rule 10 clause 1 passes without the
  same-place allowance; the naive `l = move (*root).left` is refused downstream
  because the consuming move takes `root`'s other field with it; no hole exists
  because `hole` holds a valid `None`. The rotation is correct as written (I
  traced all four swaps). The stated cost, "two extra memory operations", is not
  derived: swapping two memory places is two loads and two stores, so the four
  swaps are about four loads and six stores against the C++ rotation's three and
  three. Three of those stores are `None` written into a place overwritten by
  the next swap and are removable by ordinary dead-store elimination, so the
  honest figure is "zero to four extra memory operations depending on the
  backend, plus three well-predicted discriminant branches", not two.
- **5-6: agree.** `place_back`'s part-precise row plus Rule 6's overlap sentence
  keep `p` across every append; `grow`'s `writes(*b)` is a proper prefix and
  kills it exactly where the block may be reallocated; `grow`'s
  `ensures (*b).len == entry(*b).len` re-proves `k < (*b).len` for free. Two
  trace defects, neither verdict-changing: `grow(&b, 2 * (*b).cap)` leaves
  `room == 0` when `cap == 0` (6-6 writes `2 * cap + 1` correctly), and the
  closing remark that `swap(&(*b)[k], &local)` takes the Box out "in order"
  omits that `local` must already hold a valid `Box<Node>` — one allocation
  the remark does not price.
- **5-8: agree.** Rule 5's payload-return sentence makes the `Err` arm
  discharge the `File`, and Rule 6's consuming move plus Rule 5's unbox line
  free the cell on both `return` edges; `x` is read before the owner ceases to
  exist. Rule 8's "every exit path" is satisfied without the compiler releasing
  anything.

### cells-window-b.md

- **6-6: agree** on `rejected-real-cost`. `append`'s `ensures src.len == 0` is
  unconditional and `&[T]` is "a reference kind with measure `len`" with no
  writable `len`, so no admitted operation moves part of a window; the
  `take_back`/`place_back` schedule at two element moves per element against one
  `memcpy` is the residue, and the `Ring`/`take_front` variant removes the
  reverse at the price of `writes(r)`. Two program defects to fix, neither
  changing the cost: `split`'s declared row omits `writes(dst.filled)`, which
  `reverse(&dst)` needs under Rule 9's "every callee's substituted row must be
  covered by the declared row"; and no invariant discharges `place_back`'s
  `requires dst.room > 0` on later iterations (Rule 11 admits one; verbosity is
  not a cost).
- **6-8: correction: `undecided-rule-gap` -> `accepted-fine`,** because Rule 8's
  own example states the whole obligation for a linear-element storage:
  "`Slots<File, 4>`   // linear: every element must be taken out and closed".
  Nothing is required of the emptied storage beyond that, and Rule 6's release
  sentence then applies to memory holding nothing linear: "at scope exit the
  compiler releases the slots inside the window recursively and frees the
  block". Reading (a) — the emptied `Box<Slots<File>>` still needs a consumption
  no operation provides — would make a type the candidate lists as usable
  unusable, and is not what any sentence says. Cells 1-8 and 4-8 read it this
  way; with this correction the batch is consistent. Everything else in 6-8 is
  right: `replace` and the extra-argument atomic update are the two admitted
  in-place transitions for a linear element, and `swap` releases nothing.
- **6-9: agree.** Rule 6's window-parts paragraph prints this exact program with
  "p survives", and `duplicate`'s `reads(r[i])` is legal because Rule 9 admits
  "whole-index or range positions supplied as arguments"; Rule 10 clause 1
  clears `reads(r[i])` against `writes(r.next)` by the overlap sentence and
  against `writes(r.len)` because the boundary is "a runtime number stored with
  the block". The growing helper is correctly still refused.
- **6-10: agree** on the verdict; **one trace claim is wrong-side of a gap and
  one recorded gap is dismissible.** The partition is right: Rule 10 clause 1
  states "The built-in `swap` is the one operation whose two arguments may be
  the same place", the invariants discharge Rule 7, and `writes(r.filled)`
  covers both swaps. But "a reader holding `&r[k]` across a sort keeps a valid
  reference" is exactly the claim cell 2-6 denies for `writes(r.filled)`; that
  is gap G1 and the cell should not assert either side. (Neither side is
  unsafe: a function whose row is `writes(r.filled)` cannot call `grow`, whose
  row is `writes(*b)`, so no relocation can hide behind it, and Rule 12 leaves
  no holes.) The atomic-update argument question the cell records as a gap is
  not one: "f's row must not overlap any prefix of `place`" literally forbids
  `reads(r[m])` beside `place = r[k]`, since `r` is a prefix of `r[k]`; reading
  it purposively would be inventing a rule. It is a decided rejection whose
  rewrite the cell already gives, at one extra element move per merge.
- **6-12: agree.** Rule 12's intersection is applied as its own worked example
  applies it, `remove_at`/`insert_at` leave no hole because each moves the
  boundary in the same operation, and the three arms discharge `spare` under
  Rule 8's "every exit path" with Rule 12's "a local consumed on one incoming
  edge is consumed after the join" removing the drop flag. Program nit: `m` in
  `use(&r[m])` is never bound, so the cell's own `m < r.len` discussion is about
  a variable the program does not introduce.
- **6-14: agree.** "may reallocate in place" is what buys `realloc` parity; Rule
  5's payload return is load-bearing rather than convenient with a linear `File`
  in flight; Rule 14 plus Rule 13 clear the two adjacent `build` calls on
  disjoint roots. The failure-path measure reload and the `Ring`-growth bulk
  copy are correctly named. Reading `grow` as fallible is not a gap (see D4).
- **6-15: agree.** Rule 15 plus the `ensures when Some` fact removes the
  caller's test; `remove_at -> own T` is what lets a linear element reach
  `close`; `detach`'s whole-owner destructuring returns one part without a hole;
  `&r[0..r.len]` carries the measure with the reference under Rule 7.
- **8-8: agree.** The two-`File` destructuring is Rule 6's printed form, the
  propagation of a consuming move from `*b` through the Box to `cur`'s payload
  is two applications of the same sentence, Rule 2's payload clause supplies
  `b`, and Rule 2's static-shape clause is not engaged because `cur` is a value
  local, not a reference. One cost the cell waves through: `t = move *b`
  materializes the whole `Chain` out of the cell before the cell is freed, which
  is the same "moves one element where C++ constructs in place; usually elided
  by the backend, not guaranteed" that the Known-costs list already records for
  `place_back`. It is small for this `Chain` and removable by destructuring
  directly against `*b`, but "elidable" is an assumption, not a rule.
- **8-10: agree.** `put(slot, move b)` is Rule 10's own printed rejection; the
  atomic update repairs it with an empty callee row; `link` routes Oom as the
  place's own enum and Rule 5's payload return makes the `File` nameable on that
  arm. `place_back(&r, r[0])` really does change verdict under revision 4:
  clause 2 contributes a *read* of `r[0]`, and a live `r[0]` "never overlaps
  `r.next`", with `r.len` a distinct part. (`r[0]` needs `0 < r.len`, which the
  snippet leaves implicit.)
- **8-16: agree.** Rule 11's "There are no quantified facts over array elements
  ... and no per-slot occupancy facts" is what refuses the untested free-list
  pop, and the bitmask fact that would remove the compare is in "Deferred". The
  serialisation of two releases is Rule 13 applied to one root, and the
  slot-precise `release` row is legal because `k` is an argument.
- **9-12: correction: `accepted-fine` -> `undecided-rule-gap`,** because the
  cell resolves an ambiguity the protocol tells it to record. Its own closing
  line admits this: "Rules consistent: yes, once the exclusion list is read as
  normative over the inference that 'paths may name `r.next`' makes it
  assignable." Revision 4 says "a window has four named parts that paths and
  effect rows may name" and "This vocabulary is ordinary: the rows below use
  nothing a user function cannot write", and says of the same slots that they
  "hold nothing"; no sentence says `r.next` and `r.free` are not places a
  program may read or write. The cell's argument from the "Not in this
  candidate" list ("`take`/`put` holes") is a good argument and may well be what
  the owner intends, but it is an inference, and the reading it displaces is
  memory-unsafe rather than merely permissive — see the accepted-unsafe program
  below, which needs only a `reads` row. This is gap G2.
- **9-16: agree.** The part-precise `alloc` row clears Rule 10 clause 1 against
  `writes((*a.buf)[i1])` and Rule 13 then permits the fill and the allocation to
  overlap; `i1 != i2` follows affinely from the two `ensures` clauses; the
  bystander survives because neither `.next` nor `.len` is a prefix of a slot,
  and the growing-arena boundary is stated rather than assumed. The stale
  `Rule 16` cross-reference is correctly filed as a text defect.
- **10-15: correction: `rejected-real-cost` -> `rejected-zero-cost`,** because
  the per-fill compare the cell charges is not charged by the rules. Rule 3
  prints this case verbatim: "`p = &(*v)[i]` / `(*v)[j] = 5` // a content write
  on slot j: p stays valid whether or not `i == j`". Rule 10 clause 3 needs "a
  proper prefix among the call's write paths", and `(*cache)[k]` is not a proper
  prefix of `(*cache)[i]` — at worst it is the same path, which is not proper —
  so a slot-precise writer does not invalidate a reader of another slot and
  `i != k` need never be proved. (It is safe as well: Rule 12 leaves no holes,
  so the slot still holds a valid value.) With case (ii) free and case (i)
  already free, the only rejection left is a reader across a writer that may
  grow, whose rewrite is one address recomputation plus a monotonicity
  `ensures` of the shape 12-16 uses — which correct C++ must also do after a
  reallocation. That is a zero-cost rewrite. Distinctness is still needed if the
  program wants the fill and the read to *overlap* under Rule 13, but that is
  not what the cell charged.
- **12-16: agree.** The internal join is intersected correctly, the monotonicity
  `ensures` plus Rule 11's `entry` carry keep the caller's handle in bounds
  across a later `obtain` at zero cost, the extra-argument atomic update is a
  shown form with an empty callee row, and the free-list bound test is the same
  unavoidable compare 8-16 isolates from the same sentence. The relabelling from
  round one's `accepted-fine` is right: the program that needs no test is the
  rejected one.

## Confirmed gaps

**G1 (cells 2-6, 6-10; also touches 6-12, 8-16, 12-16): is a window part a
*proper prefix* of a slot path?** Rule 10 clause 3 reads: "A live reference
outside the call whose path has a proper prefix among the call's write paths,
under the same may-overlap judgment, becomes invalid after the call (Rule 3)."
Rule 6 supplies overlap for the parts — "a live `r[i]` ... always overlaps
`r.filled`" — and Rule 3 says "Whether one path is a prefix of another, and
whether two paths overlap, is judged conservatively", but no sentence says
whether `r.filled` (or `r.last`) counts as a *prefix* of `r[i]` or only as an
overlapping sibling. Consequence: 2-6 concludes that `writes(r.filled)` kills
every element reference and charges one scaled add per re-formation after
`insert_at`/`remove_at`; 6-10 concludes that a reader keeps `&r[k]` across a
`partition` whose row is `writes(r.filled)`. Both are in this batch, and they
contradict each other. Neither reading is unsafe — a row naming `r.filled`
cannot cover `grow`'s `writes(*b)`, so no relocation can hide behind it, and a
reference whose `i < r.len` fact dies (after `take_back`, or after `append`
empties the source) dies anyway under Rule 6's "stays valid while that fact
holds" — so this is a cost question, not a safety question. `r.next` and
`r.last` are separately decided by Rule 6's own two sentences ("p survives" for
the append, "p dies at a `take_back`"); `r.filled` and `r.free` are not.

**G2 (cells 9-12, 4-9, 2-9): may a program form, read, or write `&r.next` and
`&r.free`?** Rule 6: "Besides its slots, a window has four named parts that
paths and effect rows may name, all interpreted at call entry: `r.next` (the
append slot, at index `r.len`) ... `r.free` (all slots from `r.len` up)", and
"This vocabulary is ordinary: the rows below use nothing a user function cannot
write." Against that, of the same slots: "slots `[0, len)` hold values, `[len,
cap)` hold nothing." Nothing states that these two parts are row-only names.
The three cells take three positions: 9-12 rules them unassignable by inference
from the "Not in this candidate" exclusion of "`take`/`put` holes"; 4-9 writes
`build_into(&r.next, &s)` as a well-formed call; 2-9 assumes a user function may
write `r.next` but cannot publish it. Under the permissive reading the candidate
admits an uninitialized read and a free of uninitialized storage (below), so the
answer is load-bearing for the safety claim, not just for expressiveness.

## Dismissed gaps

**D1 (3-12's payload second-write gap) — covered by Rule 3:** "Writing the
storage at p's path or below it (a content write) does not invalidate p." The
broad reading of Rule 2's "which any write to the enum invalidates" would make
this sentence false for a write through the payload reference itself, so only
the narrow reading leaves Rule 3 consistent. Two field stores through `e`, no
re-match, no extra compare.

**D2 (6-8's emptied linear block) — covered by Rule 8's example:**
"`Slots<File, 4>`   // linear: every element must be taken out and closed",
together with Rule 6's "the program must take every element out and consume it"
and "at scope exit the compiler releases the slots inside the window recursively
and frees the block". The obligation attaches to the elements, the block is
memory, and a window still holding elements fails Rule 8's "on every exit path"
by itself. The competing reading makes `Box<Slots<File>>` a type no program can
finish, which no sentence asks for.

**D3 (6-10's atomic-update argument clause) — covered literally by Rule 6:**
"`f`'s row must not overlap any prefix of `place`." For `r[k] = merge(r[k], &r[m])`
the prefix `r` is overlapped by `reads(r[m])`, so the call is rejected; the
"purposive" reading that only `k != m` matters would be a new rule. Decided, not
undecided — with the cell's own rewrite at one extra element move per merge.

**D4 (6-6 and 6-14 on `grow` printed without a `Result`) — covered by Rule 14:**
"Allocation returns a `Result` and never traps (Rule 5 for the payload)." A
formatting defect in the operation table, not an ambiguity; there is no trap to
read the other way.

## Confirmed real costs

- **4-9**: one `sizeof(Entry)` element move per append, or one placeholder
  initialization if the element is built in place afterwards; no admitted
  operation commits a slot the program has already filled. Matches the recorded
  Known cost.
- **6-6**: an order-preserving *partial* window move (a B-tree node split) costs
  two element moves per element against one `memcpy`, and does not vectorise;
  whole-window growth and whole-window transfer are free.
- **8-16 and 12-16**: one predicted compare and one unreachable arm per free-list
  acquisition, plus a payload-returning signature so the dead arm can discharge
  the argument. Rule 11 has no quantified facts and the bitmask fact is deferred.
- **4-5 (added by this review)**: one pointer-sized store per level on the
  recursion unwind, against one store total for the C++ pointer-to-pointer
  descent that Rule 2's static-shape clause refuses; plus, because "`f` is
  total" forbids `?` inside the update, either one speculative allocation per
  insert attempt (including duplicate keys) or an Oom that the caller cannot
  observe.
- **5-5 (corrected figure)**: four swaps are four loads and six stores against
  the C++ rotation's three and three; three of those stores are dead and
  removable by ordinary dead-store elimination, so the delta is zero to four
  memory operations plus three predicted discriminant branches — not the "two
  extra memory operations" the cell states.
- **6-14**: one measure reload on the allocation-failure path only, and one bulk
  copy per `Ring` growth because `grow` is "`Box<Slots<T>>` only".
- **8-8 (added by this review)**: `t = move *b` moves the whole `Chain` out of
  the heap cell before the cell is freed — the same unguaranteed-elision cost the
  candidate already records for construction into the append slot. Removable by
  destructuring directly against `*b`; small here, real for a large inline
  payload.
- **5-6 (added by this review)**: taking a `Box<Node>` out of a slot in order via
  `swap(&(*b)[k], &local)` needs `local` to already hold a valid `Box<Node>`,
  i.e. one extra allocation; the cell's closing remark prices it at nothing.

## Accepted-unsafe program constructible in these cells

Under the permissive reading of Rule 6's window-parts paragraph — the reading
4-9 writes with and 9-12 argues against, i.e. gap G2 — nothing in the candidate
rejects this, and it reads and then frees uninitialized storage:

```text
struct Entry { key: u64, body: Box<Slots<u8>> }        // affine, not Copy

fn peek(slot: &Entry) -> u64        reads(slot)   { (*slot).key }
fn stash(slot: &Entry, e: own Entry) writes(slot)  { (*slot) = move e }

b: Box<Slots<Entry>>                                   // (*b).len == 0, (*b).cap == 8

k = peek(&(*b).next)              // Rule 6: "paths and effect rows may name" r.next;
                                  // Rule 6: slots "[len, cap) hold nothing"
                                  // -> an uninitialized read, accepted by every rule
                                  //    Rule 10 checks (clause 1: one read, nothing to
                                  //    prove disjoint; clause 3: no live bystander)

stash(&(*b).next, move e)         // the callee cannot tell the slot is outside the window
                                  // Rule 6: "Assigning over any owned place releases the
                                  // old value if it is affine" -> the callee releases the
                                  // Box it reads out of the uninitialized slot: a free of
                                  // whatever bytes are there, i.e. memory corruption

place_back(&*b, move e2)          // writes r.next from its own argument: whatever stash
                                  // stored is overwritten with nothing released
```

The read alone (`peek`) needs only a `reads` row, so even a rule that forbade
writing the append slot would have to forbid reading it too. The linear case is
already blocked — Rule 6's assignment clause is "rejected if it is linear" — so
the hole is confined to Copy and affine element types, where it is worse than a
leak: the callee's release of the old value frees uninitialized storage. Under
9-12's restrictive reading none of this is a program. One sentence saying that
`r.next` and `r.free` are names for effect rows and the overlap judgment only,
and that the sole places a program may name are the slots below `r.len`, closes
it.
