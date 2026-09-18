# Verification of `matrix-x1/rows-07-10.md` against CANDIDATE-X1.md

Independent re-trace of all seventeen cells in row 7 (what may be indexed, proved
bounds, range references) and row 10 (the call-site rule) against the frozen rule
file, and only that file. Three verdicts change. One rule gap the deriver never
looked at is larger than any gap it reported, and it is the one place in these
cells where a memory-unsafe program is accepted.

## Per-cell findings

**7-7: agree.** `accepted-fine` survives the re-trace. Rule 7's enumeration does
cover both levels ("Indexable things: an inline `array<T, N>` ..., a `DynBox<T>`
(Rule 6), a range reference into either, and a `const` table"), and Rule 1 admits
`DynBox<DynBox<f64>>` because a `DynBox` holds owned values. The load-bearing step
— that `m = len_of(vv[i])` survives `vv[i][j] = kern(vv[i][j])` — is correct as
argued: Rule 11 invalidates a fact "when that path is written", the path written
is `vv[i][j]`, and Rule 3's content-write clause ("Writing the storage at p's path
or below it (a content write) does not invalidate p") is the rule file's own
statement that a write below a path is not a write of that path. Two defects that
do not move the verdict. (i) `mid = n / 2` is used as if it yielded `0 <= mid <=
n` for free, but Rule 11's vocabulary is "affine comparisons over measures and
integer values", and this same cell file argues in 7-11 that masking is outside
that vocabulary and in 7-11's last paragraph that even `min` is unsettled; integer
division by a literal is in exactly the same position. The fallback is one compare
per recursion level against a log-depth call, so the cell's "no test" claim is
slightly optimistic, not wrong in verdict. (ii) The cell passes `&row[0..mid]` to
a parameter typed `&DynBox<f64>` without asking whether a range reference may bind
there; see the range-typing gap below, which is decisive for other programs but
not for this one, because `row_work`'s body performs no window operation.

**7-8: correction: undecided-rule-gap -> accepted-fine.** The reported gap is not
one, and neither horn holds. First horn (enum escapes linearity): Rule 8's general
clause is "Linearity is declared on external-resource types and propagates through
aggregates", and the list after the colon specifies that claim rather than
narrowing the word *aggregates*, whose membership Rule 1 fixes as "Every struct,
enum, tuple, array, slice-like value, Box, DynBox, and generic instantiation".
Two further sentences make the exhaustive-list reading untenable: Rule 6 provides
explicit machinery for linear elements — "where it is linear the assignment is
rejected unless written as the atomic update whose function consumes the old
value" — and Rule 12 recommends the very wrapper at issue, "slots:
DynBox<Option<Entry>> // non-Copy payloads: Option". A reading on which
`Option<File>` is affine would make Rule 6's linear-element clause dead text and
let any linear obligation be discarded by one wrapper, so it is not the reading
the document supports. Second horn (`None` is an undischargeable linear value):
the cell asserts "No rule supplies an operation that consumes a linear-typed value
whose linear part is absent", but Rule 8 does not demand a built-in operation, it
demands that "Linear values must be consumed by an explicit operation on every
exit path", and the cell's own `close_it` is such an operation — a function taking
`own Option<File>` and matching it, exactly the shape of Rule 8's own example
`fn use_it(c: own Conn) { ...; close(move c.f) }`. Scope exit is discharged the
same way as 6-8's drain: `while len_of(pool) > 0 { discard(pop(&pool)) }`, where
`pop` is a window operation yielding `own T` into a whole local (Rule 6's closed
list, "consuming a whole local (`move x`), the window operations, and the atomic
update") and `discard` consumes both variants. So the strongest program works as
written and keeps slot identity, which also retracts the cell's claim that "there
is no rewrite that keeps slot identity". Cost against C++: the `Option`
discriminant where `File` has no null niche, which is the already-recorded "one
null check per hit" cost, and against `std::optional<File>` it is zero; the
`k < len_of(pool)` test is 7-11's cost, not this cell's. One residual, not
decisive: Rule 6's sentence that a linear-element `DynBox` "must be truncated to
zero by the program" still does not say what `truncate` does to the slots it
drops, and Rule 8's "the compiler never releases them" means it cannot release
them — the drain loop reaches `len_of == 0` without that sentence.

**7-9: agree.** `accepted-fine` is right and the reasoning is sound: whatever the
signature form, the substituted path at the call is `writes(out[i])`, Rule 13
admits the two arms on `i != j`, and the address arithmetic is the same as C++'s.
The reported ambiguity about writing `writes(out[i])` in a signature is genuine
(confirmed below) and, as the cell says, changes no acceptance and no cost. One
overstatement to record: "slot and range arguments express every effect shape the
row cannot name" is false, and this file refutes it two cells later — a callee
whose destination indices come from data cannot be given a range that provably
contains them without a per-element test, which is 7-13's rejected histogram. The
cell's own scatter paragraph half-concedes this; its "costs one add per call"
figure is only right when the targets are provably inside the handed-out range.

**7-10: agree, cost corrected upward.** `rejected-real-cost` is right. Rule 10
clause 1 is applied verbatim and `i != j` is a property of the permutation data,
which Rule 11 cannot state. The Copy escape is sound — `t = v[i]; v[i] = v[j];
v[j] = t` is three statements, no call, so no pairwise comparison — and the
affine case is correctly closed by Rule 6's "no partial move out of any place".
The cost is understated by one compare. The rewrite
`if i < j { swap_slots(&v[i], &v[j]) } else if j < i { ... }` does not discharge
Rule 7's obligation for `&v[j]`: `i < j` plus `i < len_of(v)` does not give
`j < len_of(v)`, and the cell's own trace says `j` "must be tested, since `perm[i]`
is data". So the accepted form is one bound test plus one ordering compare per
swap, plus a writer-supplied arm for an out-of-range `j` — two compares and two
branches, not one. In the `j < i` arm the bound comes free from `j < i <
len_of(v)`. Verdict unaffected.

**7-11: agree.** `rejected-real-cost` is exactly right and this is the cleanest
cell in the file. Rule 7 demands the proof, Rule 11 lists the admissible fact
forms and then closes the door explicitly ("There are no quantified facts over
array elements"), and the observation that a validating pre-pass cannot help —
"the conclusion it would have to carry is precisely the forbidden quantified
fact" — is correct and important. The probe-loop boundary is traced correctly:
the mask gives nothing, the affine increment plus wrap carries `i < m` across the
join by Rule 12's intersection. Two program-level notes that do not move the
verdict: the probe indexes `slots[i]` while the bound is `m = len_of(tags)`, so
the writer additionally owes `len_of(slots) == len_of(tags)`, which Rule 11 admits
as an affine comparison over measures but which any write to either container
invalidates; and the `min` remark is a genuine vocabulary gap, recorded below.

**7-12: agree, one reported gap dismissed.** `rejected-zero-cost` stands: Rule 12's
"a fact survives only if it holds on every incoming edge" kills the one-sided
bound, and the arm-duplication rewrite costs nothing at runtime. But the cell's
"unwritten case" is written. Rule 2 says "At a control-flow join, a reference
variable's target is the set of paths it may name; every check on it must hold for
every member of the set." The bound needed for `pool[k]` is a check on `pool`, so
it must hold for both `small` and `large`; a single runtime load of
`len_of(pool)` establishes it for neither, and the program is rejected. The
deriver's "operational" reading — that the test discharges the obligation for the
target actually named — is the invention, not the rule. Dismissing the gap
strengthens the cell rather than weakening it: rejection is what the rewrite
already assumes. Separately, the cost comparison is scoped to the join: the
`k < len_of(...)` test remains a real cost against C++, booked in 7-11.

**7-13: agree.** `rejected-real-cost` is right. Both arms' write path is `hist[b]`
with `b` produced by `bucket(keys[t])` inside the arm, Rule 10 clause 1 requires
"indices or ranges proved distinct", and Rule 11 supplies nothing about data —
this is a true race, correctly refused. The observation that no atomic escape
exists ("channels or atomics" are under "Not in this candidate") is correct and
is what makes the cost unavoidable. Privatize-and-merge is the right rewrite and
the `k*B` memory plus merge pass is a real cost that swamps the parallel gain at
large `B`. I checked the alternative the cell does not mention — partitioning the
histogram instead of the keys, each arm owning `&hist[lo..hi]` — and it is worse:
each arm must still prove every `b` lands inside its own range, which is a
per-element test plus a discard, and each arm reads all the keys, so it multiplies
read traffic by `k`. The rewrite chosen is the best one.

**7-14: agree.** `rejected-real-cost` is right and the derivation is tight. Rule 6
fixes `len_of == 0` at `DynBox::new`, Rule 7's range form "requires lo <= hi <=
len_of(buf)", so the argument cannot be formed before the body is ever considered;
`push_nogrow` is `writes(buf)` so two pushes in two arms overlap on the header
path and Rule 13 refuses them; and `filled` is restricted to Copy by its own
comment. I looked for a way out for non-Copy elements and there is none under
these rules: the arena of Rule 16 opens its window with `alloc`, whose row is
`writes(a.buf)`, so it is the same sequential bottleneck. The claim that the
initializing pass cannot be elided is also correct and is proved from the rule
file rather than asserted — proving the kernel covers every slot is the quantified
fact Rule 11 refuses. The costs (one extra streaming pass for Copy; a sequential
merge or a permanent `(arm, offset)` indirection for non-Copy) are confirmed.

**7-15: agree, one reported gap dismissed and one cost added.** `rejected-zero-cost`
stands for the shape the cell tests: Rule 15 forces the index protocol, Rule 11
admits "callee contracts" as facts, and `ensures when Some: result < len_of(v)` is
Rule 11's own "result-routed relations" form, so `&v[i]` is formed with no runtime
test. The "one unwritten point" — whether an `ensures` may measure an owned result
— is covered by Rule 11's "affine comparisons over measures and integer values",
which does not restrict a measure's argument; this same file relies on exactly
that latitude in 7-7, where it argues Rule 11 "does not restrict the argument of a
measure to a root" in order to write `len_of(vv[i])`. Gap dismissed. Against that,
the cost line "Zero." is too strong in one respect: Rule 4 itself prices the
protocol as "caller re-derives, one address computation", and that recomputation
folds into an addressing mode only for element sizes 1, 2, 4 and 8; for an
arbitrary struct it is a multiply plus an add per search that the C++
reference-returning form does not perform. It is one instruction outside any loop,
amortized over a whole search, so the verdict does not move, but the cost is not
literally zero.

**7-16: agree.** `rejected-real-cost` is right. The per-hop test follows from Rule
7 plus Rule 11's refusal of "for all i" facts, and the cell correctly notes that
the test cannot be hoisted because the fact would be about an `h` that is
reassigned from memory each iteration. Clause 3's invalidation of `p` across
`alloc` is correctly applied and correctly justified as necessary rather than
conservative, since Rule 6's `Vector` growth "allocates a larger DynBox, moves
`[0, len_of)` across, and replaces the old one". The cell also marks its
dependence on provisional Rule 16, which that rule requires. One step used without
being written down: recovering `parent < len_of(a.buf)` after the call needs the
caller's pre-call fact to survive as a fact about `entry(a.buf)`, and Rule 11 says
only that the fact "is invalidated ... unless the callee's `ensures`
re-establishes it". Rule 6's own `push_nogrow` contract would be useless without
that step, so I treat it as presupposed rather than as a defect; it is recorded
below as a minor gap because 10-11 and 10-16 lean on it too.

**10-10: correction: accepted-unsafe -> rejected-zero-cost.** The cell finds a
use-after-free and misses the rule that rejects the program before the hole opens.
Rule 9's last sentence is "A function body is checked against its own row: every
statement's effect and every callee's substituted row must be covered by the
declared row." The closure `h = || { push(&v, 7) }` is passed to `apply(f: fn())`,
and Rule 10 says "Function-typed parameters carry a full signature with its own
row and contract", so `h` must satisfy the empty row of `fn()`. Its body's
statement has effect `writes(v)`, which the empty row does not cover, so the body
fails its check and the program is rejected. The cell's own premise is what
forces this: it establishes that `writes(v)` is "inexpressible" because Rule 9
requires "each path starts at a reference parameter", and an effect that cannot
be expressed cannot be covered — inexpressible is not exempt. Rule 4's qualifier
"captured by a function value that is stored or returned" governs where a function
value may go; it is not a licence for a body to perform effects its row omits, and
Rule 9 is the clause that reads bodies. The consequence is the rule set's real
position: no function value may capture a reference, which is the cell's own
recommended fix, and by its own costing that fix is free — "the captured pointer
becomes an argument register; extra parameters are explicitly not a cost under
these criteria". Hence a rejection with a zero-cost rewrite. "Rules consistent"
also flips to yes. The residual reading under which the cell would be right is
that closure bodies are outside Rule 9 altogether; the rule file grants no such
exemption, and if closures were outside the rules entirely the cell would be
`undecided-rule-gap`, never `accepted-unsafe`. The cell's second observation, that
a call in argument position (`combine(&v, sum(&v))`) is unwritten, is dismissed
below.

**10-11: agree.** `accepted-fine` is right and the derivation is the strongest in
the file. Rule 11's invalidation clause is applied exactly, the `entry(p)` usage
matches Rule 6's `push_nogrow` contract, and the point that `writes(part)` cannot
reach `other` or `meta.index` because Rule 1 guarantees "no aggregate ever
contains a reference" is correct and is the cell's real result. `mid <=
len_of(part)` and `len_of(part) == n` do make both ranges formable with no test,
and Rule 13's adjacency admits the `par`. Two notes: `&other[m - 1]` additionally
owes `m > 0`, which the cell does not mention; and the caveat it inherits from
10-10 ("here it shows up as a fact the caller wrongly keeps") lapses under that
cell's correction, since a capturing callback is rejected rather than trusted.

**10-12: agree.** `rejected-zero-cost` is right. Rule 2's "every check on it must
hold for every member of the set" forces the pair `dst = a, src = a` into Rule 10
clause 1, which rejects it; no rule offers a correlated target set; and unrolling
by two removes the join so that every call sees single-target references. The cost
claim is sound and if anything conservative — the unrolled form really does delete
the parity branch and the two pointer stores, and duplicated source is excluded
from cost by the criteria. The two neighbouring results are also correct:
conditional consumption is settled by Rule 10 clause 2 plus Rule 12's
intersection with no drop flag (P8's requirement), and a reference invalidated on
one edge dies at the join because Rule 3 makes validity "a fact like any other".
The indexed variant's two arms do carry different proofs of the same call, as
claimed.

**10-13: correction: accepted-unsafe -> rejected-zero-cost.** Same missed clause as
10-10, and here it bites even more cleanly because the capture's effect has an
explicit callee row. `g = |s| { compress_into(s, &scratch) }` is passed where the
declared type is `fn(s: &DynBox<u8>) writes(s)`. Rule 9: "every statement's effect
and every callee's substituted row must be covered by the declared row." The
substituted row of `compress_into(s, &scratch)` is `writes(s), writes(scratch)`,
and `writes(scratch)` is not covered by `writes(s)`, so `g`'s body fails its check
and `run2(g, ...)` never gets as far as Rule 13. The benign closure `h = |s| {
helper(s, &input) }` is rejected by the same clause, since `reads(input)` is not
covered either — which is the right answer, because the cell itself observes that
nothing distinguishes the two to the checker. The rewrite that makes `aux` a
parameter of the function-typed parameter is the form the rules require, and the
cell prices it at zero. Everything else in the cell is correct: Rule 14 exempts
allocation from the arm comparison, read/read arms are admitted explicitly, a
bystander reference is caught by the pairwise test without needing clause 3, and
the closing observation that captures are "the only route by which two `par` arms
can reach a common path, because Rule 1 removes references from every aggregate"
is right — which is exactly why closing the route by Rule 9 restores soundness.

**10-14: agree.** `undecided-rule-gap` is correct and the gap is real, not a
failure to read. Rule 13 defines `par` solely by an acceptance condition and says
nothing about an arm that exits early, while Rule 14's own example puts `?` inside
both arms. The three unstated consequences the cell lists (whether the sibling
completes, which error is produced, whether a completed arm's binding exists on
the propagating path) are each needed to give the program a meaning, and the
second one genuinely collides with Rule 14's determinism sentence, which claims
only that "Addresses are not observable". The linear half is decided correctly
against Rule 8's "on every exit path", and the Result-returning rewrite does avoid
the unstated semantics at no hot-path cost. I could not derive an answer from the
text either.

**10-15: agree.** `rejected-real-cost` is right. Clause 3 is quoted and applied
correctly — `cache` is a proper prefix of `cache[i]`, so a container-wide row kills
a reader on an untouched slot — and the slot-granular rewrite genuinely rescues it,
because with `i != k` proved, `cache[k]` is not a prefix of `cache[i]` and clause 3
does not fire. The two prices named are both real and correctly attributed: the
slot-level callee can no longer resize, and the index comparison is the writer's
to supply. One compare and one branch per fill, not per read, is the right figure.
The use of Rule 15's "Passing a large value in and back out is expressed as a
reference parameter with a `writes` entry" to avoid a copy is a correct reading and
matches PROGRAMS' note that large-aggregate round trips have no assumed zero-copy
ABI.

**10-16: agree.** `rejected-real-cost` is right, and the (i) result is the sharpest
observation in row 10: Rule 10's comparison is per call, so splitting a two-write
edit into two single-effect calls never enters clause 1 at all, and the aliasing
question disappears without a runtime test. That is sound rather than a loophole,
because the two calls run in sequence and neither leaves a live reference across
the other. (ii) is clause 3 applied correctly and for the right reason, and the
contract chain `parent < len_of(entry) == result < len_of(a.buf)` does restore the
bound, modulo the pre-state-fact step recorded below. (iii) fails for the same
reason as 7-10 and the dense-range rewrite is the right recovery; its cost (work
proportional to `cap_of` rather than to the live count, plus generation words where
identity is required) is confirmed. The cell also marks its dependence on
provisional Rule 16, as that rule requires.

## Confirmed gaps

1. **Window operations on a range reference, and what type a range reference has**
   (affects 7-7, 7-13, 7-14, 10-11, 10-12; not reported by the deriver). Rule 7
   says "Indexable things: an inline `array<T, N>` ..., a `DynBox<T>` (Rule 6), a
   range reference into either, and a `const` table", listing a range reference
   beside `DynBox<T>` rather than as one. Rule 6 declares every window operation
   over that other kind — "fn pop<T>(buf: &DynBox<T>) -> own T writes(buf)" — and
   says of the boundary that "`len_of` is a runtime number stored in the block
   header, readable by the program, and changed only by the built-in operations
   below". Rule 13's example `par { kernel(&v[0..mid], &out[0..mid]); ... }` and
   every range-taking helper in this matrix require a range reference to bind to a
   helper parameter, but "slice types as first-class values" is under "Not in this
   candidate", so no parameter type for a range is ever written. The text therefore
   neither permits nor forbids `pop(&v[0..mid])`, and gives no meaning for it if
   permitted. Consequences both ways: forbidden, and no signature can be written
   for `qsort(part)`, `row_work(row)` or P6's helper at all; permitted, and the
   accepted-unsafe program below follows.
2. **`writes(out[i])` in a signature** (7-9). Rule 9's first sentence: "An effect
   row lists `reads(path)` and `writes(path)` where each path starts at a reference
   parameter and may continue through fields, `*`, and whole-index or range
   positions supplied as arguments." Its last: "Signatures never contain index
   expressions; an index enters an effect only through an argument, evaluated once
   at the call", with the example comment "the row itself names no index". The
   first clause is vacuous unless `writes(out[i])` with `i` a parameter is legal;
   the comment reads as if no row may name an index. Not decisive: the
   slot-reference form substitutes to the same path at the call, at the same cost.
3. **Non-affine integer operations in the fact vocabulary** (7-7, 7-11). Rule 11:
   "Facts are the existing WF forms: affine comparisons over measures and integer
   values, refinement facts from a dominating branch, loop-header invariants ...,
   explicit `use` steps inside an `invariant`, and callee contracts." Whether
   `min`, integer division by a literal (`mid = n / 2`) and masking (`i = hash &
   (m-1)`) are modelled well enough to yield facts is unstated; 7-11 assumes they
   are not, 7-7 assumes division is. Each case has a one-compare fallback, so no
   verdict turns on it.
4. **`par` with an early exit** (10-14). Rule 13 defines the construct only as
   "`par { A; B }` is accepted when A's write paths are disjoint from B's read and
   write paths and vice versa", while Rule 14's example is "par { a = build(&x)?;
   b = build(&y)? } // both allocate: accepted". Completion of the sibling, the
   choice of propagated error, and the existence of a completed arm's binding on
   the propagating path are all unstated, and the second one would be an
   observable nondeterminism that Rule 14's determinism sentence does not cover,
   since it speaks only of addresses.
5. **Normalizing nested range paths for the disjointness test** (7-7, as the
   deriver reported). Rule 10 clause 1 asks for "indices or ranges proved
   distinct" and never defines how `vv[i][lo..hi][a..b]` is compared with
   `&vv[i][c..d]`. Non-decisive here, since every comparison in these cells is
   between siblings under one parent.
6. **Pre-call facts in `entry(p)` form** (7-16, 10-11, 10-16; used, not reported).
   Rule 11 says a fact mentioning a written path "is invalidated ... unless the
   callee's `ensures` re-establishes it", and no sentence says the pre-call fact
   survives as a fact about `entry(p)`. Every `ensures` of the shape `len_of(buf)
   == len_of(deref(entry(buf))) + 1` is unusable without that step, so the design
   presupposes it; it is recorded as unwritten, not as a contradiction.

## Dismissed gaps

- **7-8, linearity through an enum.** Covered by Rule 8's general clause
  "propagates through aggregates", whose membership Rule 1 fixes ("Every struct,
  enum, tuple, array, slice-like value, Box, DynBox, and generic instantiation"),
  and confirmed by Rule 6's dedicated linear-element sentence "where it is linear
  the assignment is rejected unless written as the atomic update whose function
  consumes the old value" and Rule 12's recommendation of
  `slots: DynBox<Option<Entry>>`. The second horn is covered by Rule 8's own
  wording, "consumed by an explicit operation on every exit path", which a
  writer-supplied consuming function satisfies.
- **7-12, what a runtime measure test on a multi-target reference proves.** Covered
  by Rule 2: "every check on it must hold for every member of the set." The test
  discharges nothing for either member, so the access is rejected — which is what
  the cell's rewrite already assumes.
- **7-15, whether an `ensures` may measure an owned result.** Covered by Rule 11's
  "affine comparisons over measures and integer values", which places no
  restriction on a measure's argument; 7-7 relies on the same latitude for
  `len_of(vv[i])`.
- **10-10 and 10-13, the capturing-closure hole.** Covered by Rule 9: "A function
  body is checked against its own row: every statement's effect and every callee's
  substituted row must be covered by the declared row." An effect that Rule 9's
  path grammar cannot express cannot be covered, so the body is rejected. This is
  the correction above, not a surviving inconsistency.
- **10-10, a call in argument position.** Rule 10's first sentence is "At a call,
  substitute the actual argument paths into the callee's row." An argument that is
  itself a call contributes no path; it is its own call, compared under Rule 10
  where it occurs, and clause 2's "its place" for the resulting value is a fresh
  temporary. The cell concedes the desugaring is free in any case.
- **10-13, importing by-value consumption into `par`.** Rule 13 says the judgment
  is "the same path-overlap and index/range-disjointness judgment as Rule 10", and
  Rule 10 clause 2 states what a by-value argument "contributes ... to this
  comparison". The import is stated, not inferred.

## Confirmed real costs

- **7-10**, non-Copy swap at data-determined indices: one bound test plus one
  ordering compare and their branches per swap, plus a writer-supplied
  out-of-range arm. The cell's "one compare" understates it; the `j < i` arm gets
  its bound free.
- **7-11**, every data-sourced index: one compare and one data-dependent branch per
  use, or one extra ALU operation and a second load address when it becomes a
  select. Zero against safe Rust.
- **7-13**, parallel scatter: `k` private tables of `B` counters plus a `k*B` merge
  pass; at large `B` the private tables miss cache and the parallel form is lost
  outright. No atomic escape exists in this candidate.
- **7-14**, fresh buffers: for Copy elements one extra streaming initialization
  pass over the output (up to 50 percent more write traffic on a memory-bound
  kernel); for owned elements, parallel construction into one buffer is not
  expressible at all, and the rewrite pays either a sequential merge of `n`
  elements or a permanent extra dependent load per access.
- **7-15**, the index protocol: one address computation per search, free only for
  element sizes 1, 2, 4 and 8. Rule 4 itself names it ("one address computation").
- **7-16**, handle traversal: one compare and one branch per hop, and the branch
  blocks software-pipelining of two hops; stale-handle detection adds a load, a
  compare, a branch and 4 to 8 bytes per node and per handle.
- **10-15**, slot-granular lazy fill: one compare and one branch per fill.
- **10-16**, dense-range parallel node map: one load, compare and branch per slot
  and work proportional to `cap_of` rather than to the live count.

## Accepted-unsafe program constructible in these cells

Under the permissive side of confirmed gap 1 — a range reference binds to a
`&DynBox<T>` parameter, which every range-taking helper in 7-7, 7-13, 7-14 and
10-11 requires — the window operations become callable on a sub-range, and two
`par` arms holding adjacent sub-ranges of one block both write the one header:

```text
v: DynBox<Box<Node>>                         // affine elements

fn drain_one(part: &DynBox<Box<Node>>)  writes(part)
    contract { requires len_of(part) > 0; }
{ t = pop(part); consume(move t) }           // Rule 6: pop's row is writes(buf)

par { drain_one(&v[0..mid]); drain_one(&v[mid..n]) }
// Rule 13: the substituted write paths are v[0..mid] and v[mid..n], adjacent
// ranges, "disjoint ranges: accepted". Nothing else in the rule set is consulted.
```

Every check passes: Rule 7 forms both ranges from `0 <= mid <= n <= len_of(v)`,
Rule 10 clause 1 separates the two write paths by adjacency, and Rule 13 admits
the block on exactly the example it publishes. At runtime both arms decrement the
same `len_of`, which Rule 6 places in one header — "The boundary `len_of` is a
runtime number stored in the block header" — so the two arms can race to the same
slot and return the same `Box<Node>` twice, which is a duplicated affine value and
a double free at scope exit. Even sequentially the program has no meaning, since
no rule says which slot `pop(&v[0..mid])` removes or what it does to `len_of(v)`.

This is the only memory-unsafe acceptance I could construct in these cells. The
two the deriver reported (10-10, 10-13) are rejected by Rule 9's body check, and
the one implied by 7-8's first horn — dropping `Some(f)` because an enum is held
not to propagate linearity — is unavailable under the reading this verification
adopts.
