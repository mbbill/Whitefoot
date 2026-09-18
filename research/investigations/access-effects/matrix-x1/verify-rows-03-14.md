# Verification of `matrix-x1/rows-03-14.md` against the frozen candidate x1

Verifier pass: every cell re-traced literally against `CANDIDATE-X1.md`. Nothing outside
that file is used; nothing from "Not in this candidate" is imported. Runtime performance is
the only cost counted; verbosity, extra parameters and duplicated code are not costs.

## Per-cell result

**3-3: agree.** `undecided-rule-gap` stands. Rule 2's sentence "At a control-flow join, a
reference variable's target is the set of paths it may name; every check on it must hold for
every member of the set" is the only text covering a loop-carried reference re-formed from
itself, and it neither bounds the set nor gives a summary form. Rule 3's content clause
("Writing the storage at p's path or below it (a content write) does not invalidate p") is
correctly used to keep `p` alive within an iteration.
*Program note (strengthening, not a correction):* the strongest program leans on a variant
payload and therefore inherits 3-12's second gap. That crutch is avoidable, and the gap is
worse without it:
```text
struct Tree { n: Int, kids: DynBox<Tree> }     // Rule 1: owned throughout; finite, since DynBox is a handle
p = &t
loop { if len_of((*p).kids) == 0 { break }; p = &(*p).kids[0] }   // Rule 2 grammar only: field, then [i]
```
Every component here is in Rule 2's grammar, the chain is constructible (a zero-length
`kids` terminates it), and `p`'s target set at the header is still unbounded. The gap is
therefore independent of 3-12 and the cell's own rewrite (index-carried traversal) and cost
(at most one predicted compare per hop) are unaffected.

**3-4: agree.** `rejected-zero-cost` stands. Rule 4 rejects the returned reference verbatim;
Rule 11's "A fact that mentions a path is invalidated when that path is written ... unless
the callee's `ensures` re-establishes it" plus `push_nogrow`'s `ensures` is the correct
route for `i < len_of(v)`; Rule 10's "Function-typed parameters carry a full signature with
its own row and contract" is the correct route for the inward form. Two minor trace holes,
neither verdict-changing: the ordinary program never discharges `push_nogrow`'s
`requires len_of(buf) < cap_of(buf)`, and the "one indirect call per operation" residual is
real under the cell's own separate-compilation framing — it is zero only if the callee is
monomorphized per callback, which the rules permit but nowhere promise. Recorded as a cost
below rather than as a verdict change.

**3-5: agree.** `rejected-zero-cost` stands. Rule 3's own example decides the first program
("`c = move b` ... p invalid, even though the Node did not move"); Rule 10.2 plus Rule 9's
"`writes` covers writing, replacing, moving out of, and freeing the storage at the path"
correctly makes the consumed `own` argument invalidate the interior reference in the arena
variant; the re-formation reloads a value already in a register, so zero. The dependency
note on 3-12 is correctly placed (the three programs use whole-Box fields and whole slots).

**3-6: agree.** `accepted-fine` stands. The two halves of Rule 3 are applied exactly as
written: `writes(buf)` on every window operation (Rule 6) is a proper prefix of every element
path and kills `last`; sibling and same-slot content writes do not. The atomic-update
analysis is literally correct — `&buf[k]` is the written path, not a proper prefix, so it
survives, while `&buf[k].field` has `buf[k]` as a proper prefix and dies. Rule 12's
intersection at the `if` join is correctly applied. Two trace holes, neither
verdict-changing: `push_nogrow`'s capacity precondition is again not discharged by the stated
invariant, and the per-iteration re-formation needs `len_of(runs)` — a header word that a
C++ cached-pointer loop never loads. The cell's "block base provably unchanged" argument
covers the base but not the length; keeping `len` in a register across the loop is an
ordinary optimisation, not a rule guarantee. Cost recorded below.

**3-7: correction: accepted-fine -> accepted-fine (unchanged verdict; one supporting claim
struck).** The cell's program and its verdict survive: the probe program is accepted with
Rule 7's test, and the divergence analysis is right in both directions — every shrink
declares `writes(buf)` (Rule 6) so validity implies in-bounds, and "Validity is re-established
only by forming the reference again" (Rule 3) is why an in-bounds index is not a reference.
The bitwise-mask conclusion is also right and is not a gap: Rule 11 admits "affine comparisons
over measures and integer values", so the premise `i == h & (n-1)` cannot be recorded as a
fact at all and no `use` step can start from it.
What must be struck is the blanket sentence "Range references and element references never
invalidate each other, because Rule 3's relation is on paths and `buf[lo..hi]` neither extends
nor is extended by `buf[k]`." The cell's own program only exercises disjoint ranges, where
the conclusion holds under any reading. Stated generally it is the unsafe half of the gap
recorded as G3 below, and 3-13 itself flags the same relation as unstated — the two cells
contradict each other.

**3-8: agree.** `undecided-rule-gap` stands, and the contradiction is real. Rule 8 writes
`close(move c.f)` in its own example while Rule 6 states "There is no `take` operation and no
partial move out of any place: the only ways to move a value out of storage are consuming a
whole local (`move x`), the window operations, and the atomic update", and Rule 12 states "No
place is ever partially moved". `c.f` is a field of a local, not a whole local, not a window
operation and not an atomic update. The cell's escalation is also correct: under the strict
reading Rule 8's own `Box<File>` line ("must be taken apart and its File closed") names an
operation the candidate does not define, and a linear value that has entered an aggregate can
never be consumed. Reported without resolution, as the protocol requires.

**3-9: correction: accepted-fine -> undecided-rule-gap**, because the cell's own reasoning
step — "substituted `writes(s.grid[k])`: equals p's path when k == j, otherwise a sibling;
neither is a proper prefix -> p valid" — is applied to a path one level too shallow and to a
Copy element type, which is a convenient program, not the strongest. Rule 3 invalidates only
when "any proper prefix of p's path is written, moved out of, replaced, or freed", and Rule 9
makes `writes` cover freeing. With an affine element the two combine into a use-after-free
that the rule text does not clearly refuse:
```text
fn replace_at(g: &DynBox<Box<Node>>, i: u64, x: own Box<Node>)  writes(g[i])
    // body: g[i] = move x   -> Rule 6: "Assigning buf[k] = x where the old value is affine
    //                          releases the old value" -> the old heap Node is freed

q = &*(g[j])                  // Rule 2 grammar: index, then * (Box content); Rule 7: j < len_of(g)
replace_at(&g, i, nb)         // Rule 10: substituted write path is g[i]
y = (*q).value                // proper prefixes of g[j].* are g and g[j]; g[i] is neither, spelled this way
```
At runtime `i == j` is permitted and unproved, and nothing in Rule 3 or Rule 10.3 asks for a
distinctness proof — Rule 10.1's "indices or ranges proved distinct" governs the pairwise
conflict check only. Rule 3's own example even applies the relation permissively across an
unproved index (`v.buf[j] = 5 // content write on a sibling or the same slot: p stays valid`).
The cell's general claim that argument-index-granular rows "leave bystander references alone
across an opaque call" is therefore unsafe for affine elements, and the text does not settle
which reading is meant. See G3; the constructed program is U1.

**3-10: correction: accepted-fine -> rejected-real-cost**, because the cell's strongest
program is rejected, by the cell's own annotation, and the verdict vocabulary must follow the
program. `merge(&v[i], &v[j])` falls to Rule 10.1 — "Two effects on overlapping paths where at
least one is a write must be proved disjoint ... otherwise the call is rejected" — with `i`
and `j` data-determined, and Rule 11 supplies no fact except from a dominating branch. The
best rewrite that keeps what the task needs is the dominating `if i != j`, which is one
compare and one perfectly predicted branch per call against the C++ form that simply aliases.
That cost is unavoidable here: this is P1, whose stated requirement is "parallel permission
for the two writes ... no runtime check", so the element-reference form must be kept and the
test cannot be dropped. The cell's escape hatch — "pass the index instead of the element
reference and let the callee form the reference" — does avoid the compare, but only by
declaring `writes(v)`, and Rule 13 then refuses to run two such calls in a `par` (the rule's
own `par { push(&v, 1); stats(&v) }` line), so it buys the instruction back with P1's
parallelism. Every rule citation in the cell is otherwise correct, including 10.3's
"A reference that is itself an argument is the thing being accessed, not a bystander".
Two claims need narrowing: `q = &v[m]` surviving is sound only because `q` names the slot
itself; the sentence "That is sound precisely because nothing was relocated" does not extend
to `&*(v[m].child)` or any reference below the slot when `Entry` is affine — that is G3/U1
again.

**3-11: correction: accepted-fine -> rejected-zero-cost**, because the rejection is the
cell's own subject. The cell asks whether a contract can revive a reference; the answer is no,
`use(p)` after `refill(&b)` is rejected, and the rewrite (re-form, carrying the bounds fact in
the contract) costs one address computation that the access needs anyway. By the file's own
convention (3-4 and 3-5 are labelled `rejected-zero-cost` on exactly this shape), this cell is
a free rejection, not an acceptance. The reasoning is otherwise right and its treatment of
Rule 3's "'p is valid' is a fact like any other" as loose wording is the correct call, not a
gap — see the dismissed list.

**3-12: correction: undecided-rule-gap -> rejected-real-cost**, because the rules decide
acceptance and only the lowering quality is open. Rule 2's grammar is a closed enumeration —
"A path starts at a local variable or a parameter and continues through fields, `*` (Box
content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7)" — so the payload of a variant
is not nameable and `p = &e.count` is refused, not ambiguous; Rule 6 and Rule 12 equally
refuse binding the payload out ("no partial move out of any place"). What the cell files as
"one reading left open by the text" is not a reading: Rule 6 states the atomic update
`buf[k] = f(buf[k])` outright, with "the old value goes into f by value, f's result is
committed, no program point lies between", and Rule 6 also names it as the required form when
an assignment must consume the old value. `f` receives the whole `Option<Entry>` as an owned
local and may consume it with `move`, which Rule 6 explicitly admits ("consuming a whole local
(`move x`)"), so every read and write of the payload is reachable through it. The rewrite
therefore exists and the cell's own cost estimate is the answer: for a non-Copy payload every
access is a whole-slot value round trip, w bytes in and w bytes out, about eight extra
loads/stores per hit for a 32-byte `Entry`, against C++'s single add through an iterator —
real, on a hash table's hot path, unless the backend lowers the update in place, which no rule
promises. The Copy-payload sidestep (tags plus fully initialised values) and its recorded cost
are correctly derived and remain the cheap path.

**3-13: correction: accepted-fine -> undecided-rule-gap**, because the cell raises the
deciding question, answers it "both readings are sound" and moves on, and that answer is
wrong. The sentence at issue is Rule 3's "invalidated when any proper prefix of p's path is
written, moved out of, replaced, or freed", read against Rule 2's grammar in which
`[lo..hi]` is its own component. The cell's ground for dismissing it — "a range write
relocates nothing (Rule 1) and the candidate keeps no content facts (Rule 11)" — overlooks
Rule 9's "`writes` covers writing, replacing, moving out of, and freeing the storage at the
path". A range write over affine elements frees the heap objects those elements own:
```text
fn refresh(part: &DynBox<Box<Node>>)  writes(part)      // body replaces elements: old Boxes released (Rule 6)
q = &*(buf[3])                                          // path buf[3].*
refresh(&buf[0..hi])                                    // substituted write path buf[0..hi], hi > 3
y = (*q).value                                          // q's proper prefixes are buf and buf[3]; buf[0..hi] is neither
```
Under the path reading `q` survives and reads freed storage; under a storage reading (Rule 3's
next sentence speaks of "the storage at p's path or below it") it dies and the continuation
re-forms at zero cost. The rule text supports both and the difference is memory safety, so the
cell is undecided. The bystander it did choose, `s = &stats.total`, is under a different root
and cannot expose this; the strongest bystander is `&out[k]` or `&*(out[k])` with `k < mid`.
Everything else in the cell is correct and survives: the disjoint-range `par` is the rule
file's own accepted example, `par { push_nogrow(&out, x); kernel(..., lo) }` is rejected for
the stated reason, Rule 14 keeps allocating arms independent, and `use(lo)` itself is
decidable — the write is at `lo`'s own path, hence a content write, hence valid. The
two-buffer rewrite and its O(total) copies are a correctly recorded real cost.

**3-14: agree.** `accepted-fine` stands. Rule 14's "Allocation and release carry no effect
entry" and "Addresses are not observable" are applied correctly; the observation that the
`Vector` grow path invalidates on the `Err` edge too, "conservative, and free", is right and
follows from Rule 3 invalidating "by a statement or by a call" plus Rule 12's intersection.
The no-drop-flag claim is sound although the citation is loose: it follows from Rule 8's
scope-exit release plus branch-local placement, not from Rule 12 itself. The one-branch-per-
allocation cost is correctly identified and correctly scaled.

**3-15: agree.** `accepted-fine` stands; the out-parameter form is Rule 15's own prescription,
`writes(out)` kills the range reference by Rule 3, and re-forming is one scaled add. One
wording correction that does not touch the verdict: the C++ shape `Frame render(const Scene&)`
is not *rejected* by any rule — Rule 15 admits owned returns of any size and merely says the
large case "is expressed as a reference parameter with a `writes` entry"; PROGRAMS.md's "Large-
aggregate ownership round trips have no assumed zero-copy ABI" is a cost statement, not a
prohibition. The cell's observation that the 3-9 lever applies (a finer `writes(out.pix)` row
would preserve more) is correct but inherits G3's uncertainty for affine fields.

**3-16: agree.** `accepted-unsafe` stands and is the right label. Rule 16 states the
consequence itself — "A stale index that is still in bounds names the current occupant of that
slot: a logic error, not a memory error" — and the cell is correct that memory safety is
retained (in bounds by Rule 7, a well-typed occupant by Rules 6 and 12) while P3's stated
requirement, "removal invalidates every path to m", and the task row's "a relation to a deleted
node must not silently become a relation to a replacement node", both fail silently. The
generation-word cost (edge grows 8 -> 12/16 bytes, one extra load usually in the same cache
line, one predicted compare per edge followed, zero against a Rust generational arena) is
correctly derived, as is the observation that allocation inside a parallel node map is rejected
because both arms would declare `writes(g.buf)`. Minor: `free_node`'s row names `g.free_head`,
a field the declared `struct Arena<T> { buf: DynBox<T> }` does not have.

**14-14: agree.** `accepted-fine` stands. Rule 14 composes with itself as claimed; Rule 8's
"at scope exit the compiler releases their memory recursively (Box, DynBox) and runs no user
code" is the correct ground for the nested `Err` chain leaking nothing and double-freeing
nothing; P19's declared depth bound is discharged on the recursive call and Rule 10's
"Recursion is checked through contracts, never by unfolding bodies" is the right citation. The
per-thread-cache argument is an implementation inference from "Addresses are not observable",
not an invented rule, and is flagged as such. The cell's honesty about its largest dependency
— without Rule 16 surviving the owner ruling, every AST/IR node is an individual `Box::new`
against a C++ front end's three-instruction bump — is the most important line in the cell and
is correctly recorded rather than buried.

**14-15: agree.** `accepted-fine` stands for the program as written. Rule 8's "Linear values
must be consumed by an explicit operation on every exit path; the compiler never releases
them" is discharged per arm, and the program avoids 3-8's contradiction only because `f` is a
whole local, which Rule 6 explicitly admits consuming. One dependency the cell omits and should
carry: the sentence "the obligation travels with the returned value rather than being
discharged by the callee" is only sound if an `own Conn` can ever be taken apart again, which
is exactly the undecided question of 3-8. The resource-pipeline task's "finish or return it on
every typed outcome" is therefore achieved only in its return half under this cell; the finish
half is undecided by 3-8. Not a verdict change, since the cell's own program returns the
`Conn`, but 3-8 belongs in its Depends-on line.

**14-16: agree.** `rejected-real-cost` stands and is well grounded. Rule 11's "There are no
quantified facts over array elements ... and no per-slot occupancy facts" is exactly why the
free-list invariant cannot be stated and Rule 13 must reject the last `par`; the bump-allocated
prefix is accepted through Rule 10.1's "ranges proved distinct" from the affine contracts; the
two rewrites and their costs (k(k-1)/2 static tests, unavailable for a runtime worker count;
per-worker sub-arenas at peak-memory and fragmentation cost) are correctly separated, and the
absence of atomics is correctly cited from "Not in this candidate". Nesting `par` is a
permissible reading, not an invention — Rule 13 judges any A and B, and a nested block's
effects are its arms' by Rule 9's body-checking clause. Minor notation slip: `a.head_entry`
should be the candidate's `entry(...)` form.

## Confirmed gaps

**G1 (3-3) — the target set of a loop-carried reference is never bounded.**
Quoted: "At a control-flow join, a reference variable's target is the set of paths it may
name; every check on it must hold for every member of the set." (Rule 2)
Missing: any finiteness requirement, widening, or summary path form. A reference re-formed
from its own path inside a loop (`p = &(*p).kids[0]`) has one member per iteration count, and
no rule says how a check is discharged against an unbounded set.

**G2 (3-8) — a linear field inside an aggregate can never be consumed, or Rule 6 is wrong.**
Quoted: "There is no `take` operation and no partial move out of any place: the only ways to
move a value out of storage are consuming a whole local (`move x`), the window operations, and
the atomic update." (Rule 6) against Rule 8's example line
"`fn use_it(c: own Conn) { ...; close(move c.f) }`" and Rule 8's "`Box<File>` // linear: must
be taken apart and its File closed".
Missing: whether `move c.f` is admitted, and if not, what operation ever reaches a linear part
inside an aggregate. Under the strict reading the resource-pipeline task is inexpressible;
under the permissive reading Rule 12's "every path is either wholly present or the program
cannot name it" needs a stated exception.

**G3a (3-9) — is `g[i]` a prefix of `g[j].*` when `i != j` is not proved?**
Quoted: "It is established when p is formed and invalidated when any proper prefix of p's path
is written, moved out of, replaced, or freed, by a statement or by a call." (Rule 3)
Missing: whether the prefix relation is spelled on path syntax or on the storage the paths
denote. Rule 10.1 supplies a distinctness judgment ("indices or ranges proved distinct") but
only for the pairwise conflict check; Rule 3 and Rule 10.3 never invoke it, and Rule 3's own
example applies the relation across an unproved index without one ("content write on a sibling
or the same slot: p stays valid"). Under the syntactic reading a callee declaring `writes(g[i])`
that releases an affine element leaves a live `&*(g[j])` valid over freed storage (U1).

**G3b (3-13) — does writing a range path invalidate element references inside it?**
Same quoted sentence, plus Rule 3's next sentence, "Writing the storage at p's path or below
it (a content write) does not invalidate p", which switches from path language to storage
language. `buf[lo..hi]` is neither a syntactic prefix of `buf[k]` nor `buf[k]` itself. Missing:
one sentence relating range paths to the element paths they cover. Under the path reading a
`writes(part)` call that replaces affine elements leaves a live `&*(buf[3])` valid over freed
storage (U2). The cell 3-13 dismissal ("Both readings are sound, because a range write
relocates nothing (Rule 1)") is refuted by Rule 9's "`writes` covers writing, replacing,
moving out of, and freeing the storage at the path".

## Dismissed gaps

- **3-12's "a variant payload is not a place".** Covered by rules: Rule 2's path grammar is a
  closed enumeration and simply refuses the reference, while Rule 6 supplies the admitted
  access route — "`buf[k] = f(buf[k])` // atomic in-place update: the old value goes into f by
  value, f's result is committed, no program point lies between" — with `f` free to consume its
  owned local under Rule 6's "consuming a whole local (`move x`)". Acceptance is decided; only
  the lowering cost is open, which is a cost, not a rule gap.
- **3-11's "'p is valid' is a fact like any other".** Covered by Rule 3's "Validity is
  re-established only by forming the reference again", and by Rule 11 listing the admitted fact
  forms, none of which can state `valid(p)`. The cell's own dismissal is confirmed.
- **3-13's note on `use(lo)` itself.** Covered: the arm writes `lo`'s own path, which Rule 3
  names a content write, so `lo` survives. Only element references under the range are open
  (G3b).
- **3-7's bitwise mask.** Not a gap: Rule 11 admits "affine comparisons over measures and
  integer values", so the premise `i == h & (n-1)` cannot be recorded as a fact at all and no
  `use` step has anything to start from. Rule 7's stated escape applies and the test is forced.
- **14-16's nested `par`.** Not a gap: Rule 13 judges arbitrary A and B with Rule 10's overlap
  judgment, and a nested block's effects are its arms' by Rule 9's "every statement's effect
  and every callee's substituted row must be covered by the declared row".

## Confirmed real costs

1. **3-7** — one compare and one perfectly predicted branch per data-determined index (hash
   probe, free-list hop, decoded offset), because no bitwise premise is statable. One more than
   `hashbrown`'s unchecked path or a C++ raw index; equal to safe Rust.
2. **3-10 (corrected)** — one compare and one predicted branch per two-element call with
   data-determined indices, or, if the index-passing form is used to avoid it, the loss of
   Rule 13 parallel permission for those calls (`writes(v)` twice cannot go in a `par`). P1
   asks for both the parallel writes and no runtime check; the candidate gives at most one.
3. **3-12 (corrected)** — for a non-Copy payload, a whole-slot value round trip per access
   (w bytes in, w bytes out; ~8 extra loads/stores for a 32-byte `Entry`) against C++'s single
   add, unless the backend lowers the atomic update in place, which no rule requires. The Copy
   sidestep costs one tag byte per slot plus one load and compare per probe.
4. **3-13** — parallel append into one container is not expressible; per-arm buffers plus a
   merge pass, O(total) copies unless the consumer can read the buffers in sequence.
5. **3-16** — the generation word: an edge grows from 8 to 12 or 16 bytes, one extra load
   (usually the same cache line) and one predicted compare per edge followed. Zero against a
   Rust generational arena; without the word, zero cost and a silent identity defect.
6. **14-14** — one predicted branch per allocation against throwing `new`; amortised one
   byte-copy per allocated byte on arena growth; and, if Rule 16 does not survive the owner
   ruling, a general-allocator call per AST/IR node where a C++ front end bump-allocates in
   three instructions and frees the pool with one store. That last one dominates the cell.
7. **14-16** — per-worker sub-arenas raise peak memory by roughly k times the per-worker
   imbalance and prevent cross-worker reuse of freed blocks without a sequential rebalancing
   pass. P2's free-list-then-parallel-fill shape is not achievable at all.
8. **3-4** — one indirect call per located update under separate compilation, unless the callee
   is monomorphized per callback (permitted, nowhere promised).
9. **3-6** — one `len_of` header load per iteration on top of the re-formed address, unless the
   lowering keeps the length in a register across the loop; a C++ loop caching `Run* last`
   after `reserve` loads neither.

## Accepted-unsafe programs constructible in this file's cells

**U1 (3-9 shape) — element-granular row frees storage under a may-aliasing sibling path.**
```text
fn replace_at(g: &DynBox<Box<Node>>, i: u64, x: own Box<Node>)  writes(g[i])
    // body: g[i] = move x   -- Rule 6 releases the affine old value, freeing its heap Node

q = &*(g[j])                  // Rule 7: j < len_of(g); Rule 2: index then * is a path
replace_at(&g, i, nb)         // Rule 10: substituted write path g[i]; Rule 10.1 has nothing to compare
y = (*q).value                // accepted: g[i] is not spelled among q's proper prefixes {g, g[j]}
```
With `i == j` at runtime — permitted, never proved either way — `y` reads freed storage. The
file's cells 3-9 and 3-10 use exactly this permissive prefix reading to keep bystanders alive.

**U2 (3-13 / 3-7 shape) — range-granular row frees storage under an element path.**
```text
fn refresh(part: &DynBox<Box<Node>>)  writes(part)    // body replaces elements; old Boxes released
q = &*(buf[3])
refresh(&buf[0..hi])                                  // hi > 3; substituted write path buf[0..hi]
y = (*q).value                                        // accepted: buf[0..hi] is not among {buf, buf[3]}
```
Same defect through a range path; this is the relation 3-13 called a non-blocking wording note
and 3-7 asserted as settled in the permissive direction.

**U3 (3-16, already labelled by the deriver) — stale handle names the new occupant.**
Memory-safe and identity-unsafe: `free_node(&g, id1); id3 = alloc_reuse(&g, ...); x = g.buf[id1].op`
reads a live, well-typed, in-bounds `Node` that is a different object, silently. Rule 16 states
this outcome; P3 and the mutable-compiler-graph task forbid it. The cell's `accepted-unsafe`
verdict is correct.

U1 and U2 are the ones that matter: they are memory-unsafe, not merely identity-unsafe, and
they are admitted by the same reading of Rule 3 that four cells in this file rely on. If the
owner intends the storage reading of the prefix relation, one sentence in Rule 3 closes both,
and the only cost is re-forming addresses that the accesses need anyway.
