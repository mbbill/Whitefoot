# Round two, revision 4: verifier's re-trace of `cells-validity.md` and `cells-overlap-alloc.md`

Scope: all 36 cells of the two sheets, re-traced literally against `../CANDIDATE-X1.md`
revision 4. Nothing below is a rule; where the rule text does not decide, the sentence is
quoted and the item is left for the owner.

Method: for each cell I re-read the strongest program, re-applied Rule 3's may-overlap prefix
judgment and its scope-exit clause, Rule 6's window parts / consuming moves / `swap` / atomic
update, Rule 10's pairwise check and clause 3, Rule 12's no-partial-move clause, and Rule 13's
permission test, then compared the cell's cost account against the protocol's rule that only
runtime performance counts.

---

## 1. Per-cell entries — `cells-validity.md`

**1-4: agree.** `p = &*b` after `match &p.left { Some(b) => ... }` extends `p`'s path by
`.left.Some.0.*` on the back edge, which Rule 2's "A path has a static shape: a loop-carried
rebinding may change only the index values inside the path, never extend the path through
itself" refuses; the recursive form re-roots at each frame's parameter ("A path starts at a
local variable or a parameter") and Rule 10's "Recursion is checked through contracts, never by
unfolding bodies" keeps it finite. The durable-cursor cost (a second full descent, or a handle
plus Rule 7's compare) is real. See note N4 on the monomorphization assumption in the fused
form's cost.

**1-10: agree.** The row is well formed under Rule 9 ("an index enters an effect only through
an argument, evaluated once at the call") and the call dies on Rule 10 clause 1 for want of
`x != m`, `m != z`, `x != z`. The bystander split is traced correctly: `q = &(*g)[t]` is an
index position at the same depth as each write path, so no write path is a *proper* prefix of
it and Rule 3's "Writing the storage at p's path or below it (a content write) does not
invalidate p" keeps it alive; `w = &(*(*g)[t].tag)` is one step deeper, so `(*g)[x]` is a
proper prefix under Rule 3's "two indexed positions on the same storage are taken to overlap
unless their indices or ranges are proved distinct" and `w` dies. Rewrite 1 is right that two
adjacent statements are never compared for *acceptance* — Rule 13 grants a permission only.

**2-2: agree.** Both round-one grounds are closed by quoted sentences, in opposite directions
(payload step admitted, loop-carried extension refused). The index-only rebinding
`loop { i = next_i(i); p = &(*v)[i] }` is correctly admitted, and the finite-target-set
argument (one shape per formation site, each index a recorded value) follows from Rule 2's
formation sentence plus Rule 12's union clause.

**2-7: agree.** Decided by one sentence and its two worked lines. The cell's extra observation
— that two snapshots of one mutated variable are not automatically distinct, and that failing
to prove it is conservative under Rule 3 — is correct and is not a cost under protocol item 7.

**2-16: correction: accepted-fine -> accepted-unsafe**, because the cell's own trace records an
accepted identity hole in its strongest program and the three companion cells price the same
hole as `accepted-unsafe`. The program ends `id3 = alloc(...)  // id3 == id1 numerically` and
`q = &(*a.buf)[id1]  // accepted: names the new occupant`, and the cell writes "Memory-safe,
identity-unsafe, by decision." That is exactly Rule 16's "A stale index that is still in bounds
names the current occupant of that slot: a logic error, not a memory error", which 3-16, 4-16
and 16-16 all verdict `accepted-unsafe`. Nothing in the program is rejected except the
post-`truncate` `use(p)`, which is the *desired* behaviour, and "Task achievable: yes". An
accepted program with a live identity defect is `accepted-unsafe` under the sheet's own
convention. Everything else in the cell is correct: `place_back`'s slot-precise row plus
Rule 6's "a live `r[i]` ... never overlaps `r.next` or `r.free`" keeps `p` alive across
`alloc`, "`requires r.room > 0`" is what makes that sound, and `take_back` kills it at the
reset.

**3-3: agree.** The reviewer's own loop is the rule file's printed rejection. The ordinary
program is traced correctly line by line, including the sharp one: `bump(&(*v)[i])` leaves
`p` and `r` (same depth) alive and kills `q = &p.x` (proper prefix).

**3-4: agree.** `place_back`'s row plus Rule 6's "`place_back`'s `ensures` carries it across
the call" removes round one's re-formation; `grow(&b, cap)  writes(*b)` is a proper prefix and
correctly kills `p` in the other arm. Two notes, neither moving the verdict: the cell writes
`grow(&v, 2 * (*v).cap)?` with a `?` that Rule 6's signature does not carry (gap G4), and the
zero-cost claim for `lookup_update` rests on per-call-site instantiation (note N4).

**3-7: agree, and the gap is confirmed.** I re-traced the drain program independently and reach
the same fork. `take_back`'s write path is `(*b).last`, "the last filled slot, at index
`r.len - 1`", an index position at the *same depth* as the range step of `&(*b)[0..2]`, hence
never a *proper* prefix of it, so Rule 3's invalidation list does not fire; Rule 3's other
sentence, "Writing the storage at p's path or below it (a content write) does not invalidate
p", then reads as keeping `part` alive. Rule 7 accepts `&part[1]` against the recorded
`part.len == hi - lo`, not against `(*b).len`. The one sentence that would kill it, Rule 6's
"A reference `p = &r[i]` into a window is formed under the fact `i < r.len` and stays valid
while that fact holds: ... `take_back`'s does not, so p dies at a take_back", is written for
`&r[i]` and says nothing about `&r[lo..hi]`. Under the permissive reading the program reads a
slot whose `Entry` was consumed and whose `Box<Tag>` was freed. The cell is right to mark it
`undecided-rule-gap` rather than resolve it; the unsafe reading is listed in section 5.

**3-9: agree.** The reviewer's round-one use-after-free is refused by the rule file's own
worked line "`replace_at(&g, i, nb)  // writes (*g)[i]: overlaps (*g)[j] unless i != j is
proved: q invalid`". `p` (same depth) survives and that is sound, not lucky: Rule 6's "no
program point can observe a slot inside the window as empty" and Rule 12's "every path is
either wholly present or the program cannot name it" guarantee the slot still holds a whole
value of the element type. `c` dies by Rule 3's payload-refinement clause read with Rule 2's
"any write to the enum invalidates" — here the write is at `(*s)[i]`, *above* the enum, so no
ambiguity arises in this cell (contrast note N3).

**3-10: agree.** Rule 10 clause 1 rejects the call verbatim; clause 3 splits the bystanders the
same way Rule 3 does; `swap(&b.left, &(*b.left).right)` is correctly rejected because clause
1's exemption is only for "the same place", not for overlap generally. One cost detail is
understated (real cost RC13): rewrite 2's `t = replace(&(*v)[j], empty)` is called "one extra
store", but for the cell's own `struct Entry { child: Box<Sub>, count: u64 }` the placeholder
`empty` must itself own a `Box<Sub>`, so it costs an allocation, not a store. Rewrite 1 is the
selected form, so the verdict is unaffected.

**3-13: agree.** `(*out)[0..h]` is strictly shorter than `(*out)[3].*.value` and overlaps it at
the index step, hence a proper prefix: `q` dies, correctly, since Rule 9's "`writes` covers
writing, replacing, moving out of, and freeing the storage at the path" frees the replaced
Boxes. The append-beside-kernel permission is sound here, unlike the superficially similar pair
in 1-16: `lo = &(*out)[0..mid]` is formed *before* the append with `mid <= (*out).len`, so the
half-open range excludes the append index and the disjointness is arithmetic rather than
asserted.

**4-12: agree.** Same two sentences as 2-2 and 3-3, applied to `cur = &(*b).next`. The three
ordinary forms (branch union, two-root alternation, index-only rebinding) are exactly the
admitted range, and the `bump(w)` target-set charge under Rule 2's "every check on it must hold
for every member of the set" is correctly priced as an overlap loss, not a runtime cost.

**4-14: agree.** Rule 14's exemption, Rule 5's hand-back, Rule 6's `..` destructuring and
Rule 3's scope-exit clause each do what the cell says, and the rule file prints round one's own
program as its example. The `Box::new(v)?` payload copy is correctly named as a standing cost
with `Box::new(Array::filled(n, v))?` as the zero-cost form.

**5-15: agree.** Form (b) is licensed by Rule 6's atomic update, and the cell is right that
revision 4 *forces* it for a Box-linked spine, since "A move out of a field or out of Box
content consumes the whole owner" removes `insert(move node.left, k)`. Its `d` commit stores
are honestly booked. See note N4 on form (a)'s indirect call.

**7-7: agree.** `part = &part[0..mid]` appends a range step per iteration, which is the growth
Rule 2 forbids; the index rewrite emits the same machine code because the rejected form had to
discharge `mid <= part.len` as well. The `&[T]` parameter type closes the reviewer's round-one
binding defect, and `mid = part.len / 2` being outside Rule 11's "affine comparisons over
measures and integer values" is correctly a cost, not a gap.

**8-9: agree.** The whole-owner sentence, the no-release-of-a-linear-element sentence and
Rule 12's "No place is ever partially moved" together refuse `x = move c.f` while `g` is
linear, kill `p = &c.n`, and bound "moving out of" in a row to the window operations and the
transient half of an atomic update. One borderline reading is flagged as note N2.

**9-9: agree.** `swap_two`'s body is accepted by Rule 6's built-in aliasing clause while
`swap_two(&v, p, p)` still dies at Rule 10 clause 1 — the asymmetry is real and correctly
attributed. The observation that no function can carry both rows, so the writer ships two, is
right and costs nothing under protocol item 7. See note N3 on the refinement-fact argument.

**15-15: agree.** Rule 10 clause 2 brings the consumption into the comparison, Rule 3's "A move
never re-roots an existing reference" kills `q`, and the container-plus-index rewrite is
correctly identified as strictly more general than the C++ selection because it also works for
a linear element type.

---

## 2. Per-cell entries — `cells-overlap-alloc.md`

**1-16: correction: "(e) and (f) may overlap" -> "(e) and (f) are not proved disjoint, so the
permission is withheld"** (overall verdict `rejected-zero-cost` stands). Rule 6 says window
parts are "all interpreted at call entry", and the sentence the cell leans on is "a live
`r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`" — both `i < r.len` and the
index of `r.next` are read in *one* state. Statement (e) `alloc` appends at index
`entry(*g.nodes).len`; statement (f) `set_data(&g, other, d)` discharges its
`requires other < (*g.nodes).len` in the state *after* (e), so `other` may equal the slot (e)
just wrote. Rule 13 demands disjointness "using the same path-overlap and
index/range-disjointness judgment as Rule 10", and Rule 10 clause 1 requires "indices or ranges
proved distinct"; `other` is not proved distinct from the append index. The permission is
therefore not established, and granting it is a write/write race on one slot (program U1 in
section 5). The repair is one word: the slot must be proved live in the *pre*-allocation state
(`other < entry(*g.nodes).len`), which does prove distinctness. Note that 3-13, 10-16 and 13-16
get the same shape right for free, because there the second statement's range was formed before
the append and is half-open, so the exclusion is arithmetic. The cell's own rewrite (disjoint
ranges of a pre-filled `Box<Array<Node>>`) does not depend on the withdrawn sub-claim, and
(c)/(d) remains correctly refused, so `rejected-zero-cost` is unaffected.

**2-4: agree.** Rule 4 permits the capture and Rule 9's closing sentence, "A function body is
checked against its own row: every statement's effect and every callee's substituted row must
be covered by the declared row", makes every effectful use of it fail, since Rule 9's grammar
requires "each path starts at a reference parameter". The "dead letter" characterisation is
accurate: every use of a captured reference is a `reads` or a `writes`, and neither can be
covered. Parameterising the capture is zero cost under protocol item 7.

**3-16: agree.** (1), (2) and (3) each trace correctly, and (2) is the important one: the
reused slot `(*a.buf)[id3]` is the same place at the same depth, never a proper prefix, so
Rule 3's content-write sentence keeps `p` alive and Rule 16's "storage that still exists stays
readable, and what the program considers 'freed' is the program's own bookkeeping" endorses it.
Memory safety survives because the element type here is all-Copy; I checked the affine variant
too, and it also holds — a deeper reference `&(*(*a.buf)[h].boxfield)` dies as a proper prefix,
so no freed Box is reachable. `accepted-unsafe` is right.

**4-6: correction: "any live reference into the window dies at the call [`remove_at`]" ->
"a slot-level reference `&r[i]` with `i < r.len - 1` proved survives, and is renumbered exactly
as the index is"** (overall verdict `accepted-unsafe` stands, and is in fact broader than the
cell claims). The cell's ground is Rule 6's "always overlaps `r.filled`", but overlap is not
invalidation: Rule 3 invalidates only when "a proper prefix of p's path is written, moved out
of, replaced, or freed", and `r.filled` ("all slots below `r.len`") is a position at the *same*
depth as `r[i]`, not shorter. The only sentence that blanket-kills a slot reference at a
shrinking operation names one operation — "`take_back`'s does not, so p dies at a take_back" —
and `remove_at`/`insert_at` have no such sentence, while their `ensures r.len == entry(r).len
∓ 1` leaves the formation fact `i < r.len` standing whenever `i` was proved below the new
length. So the cell's headline contrast ("The reference form is protected and the index form is
not") is backwards for the memmove operations: both are silently renumbered (program U2 in
section 5). This strengthens `accepted-unsafe` rather than moving it; memory safety survives,
because a memmove neither frees nor empties a slot.

**4-16: agree.** Rule 1 plus Rule 4 put nothing in the structure under Rule 3's regime; Rule 16
states the outcome outright. The successive-allocation distinctness proof through Rule 11's
`entry` sentence is correct and is genuinely load-bearing elsewhere (13-16, 14-16). The cell's
new observation — that `remove_at` on a pool renumbers every stored handle in the graph, and
Rule 16 does not warn about it — is a correct reading of the two rules together, not an
invention.

**5-16: agree.** (A)/(B) correctly refused on identical write paths; (C) correctly permitted,
and here the boundary is stated properly — `cur < (*a.buf).len` is established when `p` is
formed, i.e. *before* the allocation, so `cur` is proved distinct from the append index. The
cell also states the trap explicitly ("Proving the bound statically buys the overlap; testing
it at run time spends it"), which is exactly the discipline 1-16 failed to apply.

**7-16: agree.** The builder's reference survives an unbounded number of `alloc` calls, and the
cell correctly identifies "`requires (*a.buf).room > 0`" as the reason this is sound rather
than generous. The per-hop compare is the real finding and is untouched by revision 4, since
Rule 11's "There are no quantified facts over array elements" and the deferred bitmask fact are
unchanged.

**8-14: correction: the strongest program as printed is rejected by Rule 8 on its `Ok` arm**
(verdict `accepted-fine` stands, with the program repaired). `install` takes `g: own File` and
its `Ok` arm is `Ok(b)` — the cell's own comment says "g still owed on this path", and Rule 8
is unambiguous: "Linear values must be consumed by an explicit operation on every exit path;
the compiler never releases them." The trace's claim, "On the `Ok` path the `File` is inside
the returned `Box`; on the `Err` path it is inside the returned tuple; on neither does it
vanish", holds for `f` and not for `g`. The repair is a return type that carries `g` out
(`Result<(Box<Conn>, File), (Oom, File, File)>`, `Ok(b) => Ok((b, move g))`) or `close(move g)`
on the `Ok` arm; either is zero cost, so the cell's finding — Rule 5's hand-back closes round
one's sharpest unsafety and removes the one-slot-`Box<Slots<Conn>>` workaround with it — is
intact.

**10-11: agree.** Rule 11's invalidation clause, the range reference's formation-fixed measure,
and Rule 6's window parts do exactly what the cell says. I checked the load-bearing line: `v0
== (*r)[j].tag` survives `place_back` only because `j < r.len` is known before the call, which
it is (the fact was read through `(*r)[j]`), so `r.next`'s index is proved distinct from `j`.
The comparison against a C++ compiler's post-opaque-call reload is fair and follows from
Rule 1's "no aggregate ever contains a reference".

**10-13: agree.** Rule 9's body check forecloses the closure before Rule 13 is consulted, and
the cell's closing observation is the important one and is correct: Rule 1 removes references
from every aggregate, so without captures a statement can touch only what its arguments name,
which is what makes the declared row the whole truth and Rule 13's judgment sound.

**10-14: agree.** Rule 13's "Because the meaning is sequential, results, errors, early exits,
and linear obligations are exactly those of the sequential program" answers all three of round
one's questions at once, and the `?`-while-linear rejection is Rule 8 working, not a gap. The
cell is right to separate the residual allocator-determinism item as an obligation on the
trusted base rather than an open question about the program (gap G3, dismissed as a language
gap).

**10-16: agree.** (i), (ii) and (iii) all trace correctly, and the allocation-beside-map
permission is sound here for the reason 1-16's is not: `map_range(&(*g.buf)[0..mid], ...)`
carries a half-open range formed with `mid <= (*g.buf).len`, so it excludes the append index by
arithmetic. The caveat the cell states — a map written `for k in 0..(*g.buf).len` *reads* the
measure and loses the permission — is correct and is the same boundary 5-16 draws.

**12-13: agree.** Four pairs, all discharged; "distinct fields" rather than "different roots"
is the right citation (`two(&a.x, &a.y)`). The bound on the hoist rewrite is correctly kept:
Rule 4 guarantees a target set can only arrive from a source-level join, so duplication always
enumerates it, and the data-determined case is priced as the scatter in 13-16.

**13-14: agree.** Rule 13's by-value-consumption clause makes the two statements disjoint on
`f1`'s and `f2`'s places; Rule 14 removes the allocator from the comparison; Rule 5's hand-back
and Rule 6's destructuring make all four arms writable; Rule 8 checks each. Unlike 8-14, every
arm here really does consume both `File`s — I checked all four.

**13-16: agree.** The scatter dies three ways over and each is quoted. The build-beside-process
result is sound: `append`'s `writes(dst.free)` covers slots from `dst.len` up, `part =
&(*dst)[0..m]` was formed with `m <= dst.len` before the call, and `append`'s `ensures dst.len
== entry(dst).len + entry(src).len` only grows the window, so the formation fact survives. The
`i2 == i1 + 1` chain through Rule 11's `entry` sentence is correct.

**14-16: agree.** Rule 14's "There are no store or region parameters anywhere" fixes the shape;
the bump prefix is proved disjoint affinely; the reuse case fails on Rule 11's exclusion of
quantified facts, which revision 4 does not touch. The cell is also right that revision 4 makes
the *identity* half worse and says so, and that `a.head` was never a prefix of a block range,
so revision 2's apparent protection was already illusory. The k(k-1)/2 static tests versus
per-worker sub-arenas trade-off is correctly priced in memory and fragmentation, not in source
size.

**15-16: agree.** `build` declaring `place_back`'s row is licensed by Rule 6's "A user function
that only appends declares the same row as `place_back`", so `root` survives the whole
recursion. The monotonicity `ensures` is an affine comparison over measures and Rule 11 admits
it; without it the second recursive call would destroy the first's bound, which is the
reviewer's round-one repair correctly discharged. The per-allocation capacity test is what a
C++ bump allocator does; the per-hop compare is the verdict's ground.

**16-16: agree.** (1), (2) and (3) trace correctly; (2) is the one place the composition
catches itself and it is the window boundary, not identity, exactly as stated. The
detection-fires-on-small-inputs-only observation is a real and well-grounded finding. The
resource half is closed by "No operation releases a linear element" plus `truncate` as a
`take_back` loop. Two notation slips, no consequence: `side[t] = h` and `side[t]` inside
`(*a.buf)[side[t]].data` should be `(*side)[t]`, since the notation section requires
"Indexing through a Box is written explicitly: `(*b)[i]`"; and `t < (*side).len` is not
discharged.

---

## 3. Confirmed gaps

**G1 (3-7, cell's own, confirmed). A range reference survives the window shrinking past it.**
Rule 6: "A reference `p = &r[i]` into a window is formed under the fact `i < r.len` and stays
valid while that fact holds: `place_back`'s `ensures` carries it across the call;
`take_back`'s does not, so p dies at a take_back." Written for `&r[i]` only. For
`part = &r[lo..hi]` the deciding sentence is instead Rule 3's "Writing the storage at p's path
or below it (a content write) does not invalidate p", because `r.last` is an index position at
the same depth as the range step and so is never a *proper* prefix of it. The two readings
differ by a use-after-free on an affine element type. Smallest fix: extend Rule 6's sentence —
"a reference `&r[lo..hi]` stays valid while `hi <= r.len` holds".

**G2 (new, found in 1-16). Rule 6's window-part overlap sentence does not fix the state in
which it is read.** Rule 6: "Window parts ... all interpreted at call entry", and "a live
`r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`". Between two adjacent
statements the two calls have different entry states, so a slot index proved live only in the
*post*-append state can be precisely the slot the append wrote. The text does not say that the
liveness fact and the part must be interpreted in the same state, and Rule 13's test — "using
the same path-overlap and index/range-disjointness judgment as Rule 10" — does not repair it,
because a reader who has already applied Rule 6's sentence never reaches Rule 10 clause 1's
"indices or ranges proved distinct". Smallest fix: one clause requiring the paths of two
adjacent statements to be judged in the state before the first.

**G3 (new, found in 4-6 / 4-16). No sentence says what `remove_at`, `insert_at` or `append` do
to a live reference into the window.** Rule 6 names exactly one shrinking operation in its
validity sentence — "`take_back`'s does not, so p dies at a take_back" — and `remove_at`'s
`writes(r.filled)` is an overlap, not a proper prefix, of `r[i]`. A memmove relocates every
element above the edit point while leaving `&r[i]` valid by the letter of Rule 3. Memory-safe,
identity-wrong, and it contradicts the `accepted-unsafe` contrast 4-6 draws between references
and indices. Smallest fix: state that every operation that renumbers slots invalidates every
reference into `r`, or make the memmove operations' effect on references explicit beside
`take_back`'s.

**G4 (new, found in 3-4 / 7-16 / 15-16). `grow` has no failure behaviour.** Rule 6 gives
"`grow(&b, cap)  writes(*b)  // Box<Slots<T>> only; may reallocate in place`" with a contract
and no result type, while Rule 14 says "Allocation returns a `Result` and never traps." An
operation that "may reallocate" must be able to fail. The sheets themselves disagree: 3-4
writes `grow(&v, 2 * (*v).cap)?` and 7-16 and 15-16 write `grow(&g.buf, bigger)` with no `?`.
No verdict turns on it, but the signature is under-determined.

**G5 (new, found in 9-9). Whether a write *below* a variant payload invalidates the refinement
fact.** Rule 2: the payload step is available under a fact "which any write to the enum
invalidates". Rule 11: "A fact that mentions a path is invalidated when that path is written
(by statement or call)". Neither says whether writing `n.child.Some.0.*` — strictly below the
enum — counts. Cell 9-9 resolves it (narrowly, and safely: a write inside `Some`'s payload
cannot change the discriminant) by arguing from Rule 3's content-write sentence and Rule 2's
example `n.left = None`, but under the protocol's "If the text is ambiguous, quote the sentence
and mark the cell `undecided-rule-gap`" this was a resolution, not a citation. No verdict turns
on it: in 9-9 the fact is not needed after the recursive call, and in 3-9 the write is *above*
the enum, so the ambiguity is not reached.

**G6 (new, found in 3-4 / 5-15 / 1-4). Nothing says a function-typed parameter is instantiated
per call site.** Rule 10 says only "Function-typed parameters carry a full signature with its
own row and contract, and a call through one uses that row." Three cells price the inward
find-then-mutate form at zero by asserting that "where `f` is a literal at the call site the
instantiation is monomorphic and the call is direct". If it is not promised, the best rewrite
for find-then-mutate on a Box-linked tree costs one indirect call per operation against the C++
form that returns a pointer, and 3-4 and 5-15 would become `rejected-real-cost`. I do not
change the verdicts, because round one's reviewer accepted the same reading (E3) and the
candidate's notation section defers to "the existing WF" forms throughout; but the candidate
text alone does not decide it.

**G7 (sheet summary item 2, confirmed). No licence for k-way overlap of heterogeneous
statements.** Rule 13 grants the permission to "Two adjacent statements of one block". Three or
more different statements that pairwise qualify have no stated permission to overlap all at
once. The three-fill programs in 13-16 and 14-16 and the three-way build in 1-16 want exactly
this; the sheets route around it through Rule 13's counted-loop forms ("per-element maps,
adjacent ranges passed to a helper, admitted reductions"), which do carry a runtime worker
count, so no cell is blocked — but a k-way fan-out of *different* statements is not licensed.

---

## 4. Dismissed gaps

**D1 (sheet summary item 1): Rule 16's stale cross-reference, "see the open proposal".**
Dismissed as a derivation gap. The row is fully determined without it by Rule 6: "`place_back(&r,
x)  writes(r.next), writes(r.len)`" together with "A user function that only appends declares
the same row as `place_back`" and its worked `add_node` example. Editorial only.

**D2 (sheet summary item 3): which of two overlapping allocating statements gets the last
block.** Dismissed as a *language* gap. Rule 13 decides the program's meaning outright:
"Because the meaning is sequential, results, errors, early exits, and linear obligations are
exactly those of the sequential program; nothing new is defined for the overlapped case." What
remains is an obligation on the trusted base's use of the permission, which 10-14 and 13-14
both state correctly as such. Rule 14's "Addresses are not observable, so allocator concurrency
does not affect program determinism" is not needed to carry it.

**D3 (2-4's "dead letter"): that Rule 4's permitted capture has no instance.** Not a gap.
Rule 4's sentence is a restriction ("cannot be captured by a function value that is stored or
returned"), and Rule 9's grammar ("each path starts at a reference parameter") plus its body
check jointly decide every use. The rules are consistent; the permission is simply vacuous for
effectful uses, which the cell states accurately.

**D4 (round one's `truncate` discards linear elements).** Dismissed: covered by Rule 6's "No
operation releases a linear element: a storage whose element type is linear is itself linear
(Rule 8) and the program must take every element out and consume it", plus `truncate`'s listing
under "Library code, all zero-cost compositions of the above". 8-9, 14-16 and 16-16 each cite
it correctly.

**D5 (8-9's atomic update over a linear place).** Dismissed, though it is the closest call in
the sheets. Rule 6 says "Assigning over any owned place releases the old value if it is affine
and is rejected if it is linear", and `(*pool)[k] = drop_file((*pool)[k])` is written as an
assignment. But the atomic update's own definition disposes of the old value differently —
"the old value enters f by value, f's result is committed, no program point lies between" — so
nothing is released and the linear-rejection clause has no subject. The cell's reading is the
only one under which Rule 6's own example family (`c.f = reopen(c.f)`) works at all.

---

## 5. Accepted-unsafe programs constructible in these cells

**U1 (from 1-16's (e)/(f), under the cell's reading — a write/write race).**

```text
n0 = (*g.nodes).len                  // read of the measure, its own statement
id = alloc(&g, node)                 // writes((*g.nodes).next), writes((*g.nodes).len)
set_data(&g, n0, 7)                  // writes((*g.nodes)[n0]); requires n0 < (*g.nodes).len — holds
```

`alloc` writes the slot at index `n0`; `set_data` writes the slot at index `n0`. The cell grants
these two adjacent statements overlap permission by Rule 6's "a live `r[i]` ... never overlaps
`r.next`", read across the state change. Two concurrent writes to one slot, one of them the
move of the appended element into it. The correct trace withholds the permission under Rule 10
clause 1's "indices or ranges proved distinct"; see gap G2.

**U2 (from 4-6 — a silently renumbered reference).**

```text
b: Box<Slots<Entry>>                 // (*b).len == 10, established
p = &(*b)[7]                         // formed under 7 < (*b).len
y = remove_at(&*b, 2)                // writes((*b).filled), writes((*b).len); one memmove
consume(move y)                      // ensures (*b).len == 9, so 7 < (*b).len still holds
use(p)                               // accepted: p now names the element that was at index 8
```

`(*b).filled` overlaps `(*b)[7]` but is not a *proper* prefix of it, so Rule 3's invalidation
list does not fire and Rule 3's content-write sentence keeps `p` valid; Rule 6's only
shrinking-invalidation sentence names `take_back`. Memory-safe (the memmove neither frees nor
empties a slot), identity-wrong, and it contradicts 4-6's claim that the reference form is
protected where the index form is not. See gap G3.

**U3 (from 3-7, under the permissive reading only — a use-after-free).** The cell's own drain
program: `part = &(*b)[0..2]` held while `while (*b).len > 0 { e = take_back(&*b); consume(move
e) }` empties the window, then `x = &part[1]; read(x)`. Accepted under Rule 3's content-write
sentence plus Rule 7's bound against the formation-fixed `part.len == hi - lo`; the `Entry`'s
`Box<Tag>` has already been released by `consume`. I list it as conditional rather than as a
finding against the cell, which correctly refused to resolve the ambiguity. See gap G1.

**Checked and found safe, for the record.** A reference into a pool slot held across
`alloc_reuse` on an *affine* element type (3-16, 16-16): the slot-level reference survives as a
content write, but any reference one step deeper (`&(*(*a.buf)[h].boxfield)`) is killed as a
proper prefix under Rule 3's conservative judgment, so the released Box is unreachable. The
append-beside-kernel and append-beside-map permissions in 3-13, 10-16 and 13-16: the consumer's
range is half-open and formed before the append with `m <= r.len`, so it excludes the append
index by arithmetic. `swap` does not become a back door into nested places (3-10): Rule 10
clause 1 exempts only "the same place".

---

## 6. Confirmed real costs

Costs below are runtime only; verbosity, extra parameters and duplicated source are excluded
per protocol item 7, and every cell that touches them says so correctly.

- **RC1.** One compare and one predicted branch per data-determined index — handle hop,
  free-list pop, hash probe. Rule 11's "There are no quantified facts over array elements" and
  the deferred bitmask fact. Equal to safe Rust, real against C++. (2-2, 3-3, 4-12, 5-16, 7-16,
  10-16, 15-16.)
- **RC2.** Iterative pointer chasing over an owned Box-linked list is refused by Rule 2's
  static path shape. Recursion costs one call and return per hop with no tail-call form and no
  stack-depth statement anywhere in the file; index links cost RC1 per hop. (1-4, 2-2, 3-3,
  4-12.)
- **RC3.** A durable cursor into a Box tree does not exist: the replay form costs a second full
  descent of `d` dependent, frequently missing loads. (1-4.)
- **RC4.** Identity across slot reuse costs a generation word: one load (usually the node's own
  cache line), one compare, one predicted branch per dereference, and a handle growing from 8
  to 12 or 16 bytes, which halves edges per line in a pointer-chasing traversal. With a reset in
  the picture, two loads, two compares and one to two branches, plus a 12-byte handle. (3-16,
  4-16, 5-16, 7-16, 10-16, 15-16, 16-16.)
- **RC5.** Two writes to data-determined slots in one call need a dominating `i != j` test: one
  compare and one predicted branch, plus a duplicated else arm. Zero against Rust's
  `get_many_mut`, real against C++, and free where the test is already in the algorithm
  (union-find, partition). Revision 4 removes it for the `swap` family only. (3-10, 9-9, 10-16.)
- **RC6.** Parallel scatter to data-determined destinations is refused, with no atomics
  available. Privatize costs `P * len * 8` bytes plus a merge pass; transpose costs `|E| * 8`
  bytes held permanently plus one transposition pass. Both real, neither tunable away. (13-16.)
- **RC7.** Two allocations from one window may never overlap: both write `r.next` and `r.len`,
  identical paths. Recovered only by per-worker pools or a pre-filled block with per-worker
  ranges. (1-16, 5-16, 13-16, 15-16.)
- **RC8.** A free-list byte arena cannot have its reused blocks filled concurrently: the
  invariant is quantified over the free list's contents. Per-worker sub-arenas raise peak memory
  by roughly `k` times the per-worker imbalance and forfeit cross-worker reuse — real
  fragmentation. (14-16.)
- **RC9.** A dense-range map over a pool visits free slots: work proportional to `r.cap` rather
  than to the live count, plus one liveness load, compare and branch per slot. (10-16.)
- **RC10.** The atomic-update spine rebuild writes the link at every level on the way back up —
  `d` stores where an in-place C++ insert writes one — and forecloses a tail call. (5-15.)
- **RC11.** `remove_at`/`insert_at` renumber a whole suffix in one memmove; detecting it costs
  an epoch word, one load, one compare and one branch per use, plus a stale arm the writer must
  invent. (4-6.)
- **RC12.** A statement that reads a measure at run time to bound itself loses overlap
  permission against any statement writing that measure: `reads(r.len)` meets `writes(r.len)`.
  Proving the bound statically buys the overlap; testing it spends it. (5-16, 10-16, 13-16.)
- **RC13 (missed by 3-10).** Rewrite 2's placeholder store costs an allocation, not "one extra
  store", whenever the element type owns heap storage — the cell's own
  `struct Entry { child: Box<Sub>, count: u64 }`. A `replace` needs a whole valid `Entry`, and
  constructing one means a `Box::new`. Not the selected rewrite, so the verdict is unaffected.
- **RC14.** `Box::new(v)?` materializes a large inline `T` in a local and moves it into the
  heap: one copy of `sizeof(T)` that C++'s placement-new avoids and Rust also pays. The
  zero-cost form is `Box::new(Array::filled(n, v))?`, which maps to `calloc`. (4-14, and the
  file's own recorded cost about the append slot.)

---

## 7. Notes that are not findings

- **N1.** Neither sheet imports anything from "Not in this candidate"; the only occurrences of
  `par` and `len_of` are quotations of round one, and the only `&mut` is a Rust comparison.
- **N2.** 8-9's reading of "moving out of" in an effect row — the window operations and the
  transient half of an atomic update, never a hole in a place the caller still owns — is
  correct and is forced by Rule 12's "No place is ever partially moved: every path is either
  wholly present or the program cannot name it".
- **N3.** See gap G5: 9-9's refinement-fact argument is sound and safe but resolves an
  ambiguity rather than citing a deciding sentence.
- **N4.** See gap G6: three cells price a function-typed parameter's call at zero by assuming
  per-call-site instantiation.
- **N5.** Minor spelling slips, no consequence: 16-16 writes `side[t]` for `(*side)[t]` twice,
  against the notation section's "Indexing through a Box is written explicitly: `(*b)[i]`";
  10-11 writes `use(&other[m - 1])` without discharging `m > 0`; 16-16 does not discharge
  `t < (*side).len`.
