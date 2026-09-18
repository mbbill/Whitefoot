# Verification of `matrix-x1/rows-05-12.md` against CANDIDATE-X1.md

Independent re-trace of every cell in row 5 (Box) and row 12 (joins) against the
frozen rule file, and only that file. Three verdicts change; two cells keep their
label but lose their stated ground. The recurring defect in this file is the
path-overlap judgment: the deriver invents an index-distinctness obligation where
Rule 10's distinct-field ground removes it (5-10), and calls distinct fields
"different roots" where the roots are in fact the same (12-13). A second recurring
defect is attributing a fact that survives a *call* to Rule 12's intersection,
when Rule 11 says a call kills it unless an `ensures` restores it (12-14, 12-16,
and 5-6's cross-reference).

## Per-cell findings

**5-5: agree.** `undecided-rule-gap` is right and both quoted gaps are real. Rule
6's closing sentence is universally phrased ("no partial move out of any place",
"the only ways") and Rule 12 repeats it ("No place is ever partially moved"),
while Rule 8's worked line `fn use_it(c: own Conn) { ...; close(move c.f) }` does
exactly the forbidden thing on a field of a local. I checked for a reconciling
reading and found none: if the prose wins, a linear field can never be closed and
Rule 8's own requirement is unsatisfiable; if the example wins, Rule 6's
enumeration is not closed. The second gap (no variant-payload selector in Rule 2's
path grammar, while Rule 1 blesses `Option<Box<Node>>`) is independent and also
real. The arena rewrite's cost is correctly priced: Rule 3's "Writing the storage
at p's path or below it (a content write) does not invalidate p" makes three field
writes legal, and Rule 11's ban on quantified facts leaves the three bounds
compares standing.

**5-6: agree.** `rejected-real-cost` is right. Rule 3's prefix clause invalidates
`p` because `buf` is a proper prefix of `*(buf[k]).value`, and Rule 5's own
`c = move b // p invalid (Rule 3); the heap object did not move` confirms that
address stability buys nothing. The cost is real against the C++ idiom it names:
`vector<unique_ptr<T>>` growth does not move the pointee, so correct C++ keeps the
`T*` in a register while the rewrite reloads the block base and the slot. Two
defects that do not move the verdict. (a) The restoring `push_nogrow(&buf, move b)`
needs `len_of(buf) < cap_of(buf)` (Rule 6's `requires`), which the loop of pushes
destroys; the honest restore is the library `push` with its `Result`. (b) The
parenthetical "often free - see 12-14, where Rule 12's 'facts about indices below
n survive' keeps `k < len_of` across the push" is wrong as stated: `push` declares
`writes(buf)`, so Rule 11 kills `k < len_of(buf)` at the call "unless the callee's
`ensures` re-establishes it". The fact is recoverable, but only by giving `push`
an `ensures len_of(buf) >= len_of(deref(entry(buf)))`, not by Rule 12.

**5-7: correction: accepted-fine -> rejected-zero-cost.** The rules do reject a
line of the strongest program, and this file's own convention (5-12, 5-14, 5-15 all
report `rejected-*` while recording "Task achievable: yes") is that a rejected line
sets the label. `h = Holder { data: move b }` moves out of `b`, and Rule 11 is
explicit: "A fact that mentions a path is invalidated when that path is written (by
statement or call) unless the callee's `ensures` re-establishes it" — `len_of(*b)
== n` mentions `*b`, nothing re-establishes it, so `(*(h.data))[mid]` is rejected
under Rule 7. The cell's own better rewrite ("perform the transfer before
establishing the fact") costs nothing at all, so the label is `rejected-zero-cost`,
not a per-transfer compare. Everything else in the cell is correct: Rule 7's
"`Box<array<T, N>>` is indexed through the path `(*b)[i]` and is not a separate
case" adds no obligation, and the `par` over `&(*b)[0..mid]` / `&(*b)[mid..n]` is
Rule 13's "disjoint ranges: accepted" line. Program defect that does not move the
verdict: `if mid <= n` never establishes `mid < n`, so the final index is
unprovable even without the move.

**5-8: agree.** `undecided-rule-gap` is right and the second gap is the stronger
one. Rule 8 asserts `Box<File> // linear: must be taken apart and its File closed`
while Rule 6 and Rule 12 forbid the only operation that could take it apart; and
independently, Rule 5's `Result<Box<Node>, Oom>` with Rule 14's "Allocation returns
a `Result` and never traps" gives `Box::new(move c)` an `Err` edge on which the
consumed linear payload appears in no result, against Rule 8's "Linear values must
be consumed by an explicit operation on every exit path". I checked the escape
routes: `move b` on the whole Box is no discharge, because Rule 5 defers release to
"the compiler releases its memory recursively (Rule 8)" and Rule 8 says the
compiler never releases linear values. The pooled rewrite is sound and free —
`push_nogrow` has no `Result`, `pop` is a sanctioned move out of storage, the drain
loop is Rule 11's own `while len_of(buf) > 0 { pop(&buf) }` shape, and the closing
`truncate` is a no-op — and the pre-sizing constraint is correctly reported as a
constraint rather than a per-operation cost.

**5-9: agree on the label, correct the rewrite.** The printed strongest program is
accepted and the two positive findings hold: Rule 10 clause 3 invalidates only a
reference "whose path has a proper prefix among the call's write paths", and
`(*root).l` is not a prefix of `(*((*root).r)).value`, so sibling-subtree precision
is free; and `merge(&*b1, &*b2)` is disjoint by "different roots" with no emitted
check. Two defects. (a) Protocol 2 is inverted: the strongest witness for the
disjointness half of the cell's question is the *rejected* nested pair, which the
cell demotes to "the ordinary program". (b) The rewrite given for that pair —
"perform the two writes as two statements rather than one call" — does not work for
a merge that must move data from the child into the parent, because that move is
`l = move (*root).l`, the partial move 5-5 records as unresolved. The rewrite that
does work is a single enclosing reference, `fn merge_with_left(n: &Node) writes(n)`
called as `merge_with_left(&*root)`: Rule 10's pairwise comparison applies "At a
call", and Rule 9 checks a body only for coverage ("every statement's effect and
every callee's substituted row must be covered by the declared row"), so the nested
writes are legal inside one row. Cost zero, and the printed one-call-per-write
alternative is unnecessary. Program defect: `struct Node { value: Int, l: Box<Node>,
r: Box<Node> }` with non-optional children is not constructible (every value would
require an infinite allocation chain), which weakens both row-5 cells that use it.

**5-10: correction: rejected-real-cost via a spurious `i != j` obligation ->
rejected-real-cost only on a strengthened program; the printed program is accepted
at two compares.** The deriver missed Rule 10's distinct-field ground. The
substituted effects are `writes(a.buf[i].next)` and `writes(a.buf[j].prev)`. Clause
1 fires only on "Two effects on overlapping paths", and these two paths diverge at
a field selector, so they do not overlap however `i` and `j` compare — fields of one
object are disjoint storage. The rule file's own line settles it: `two(&a.x, &a.y)
// accepted: distinct fields`, and Rule 9's `fn update(o: &Obj, c: Bool) writes(o.a),
writes(o.b)` shows two writes under one root. So `link(&a.buf[i], &a.buf[j])` needs
only the two Rule 7 bounds facts; the `i != j` test and the `link_self` arm are
invented. The cell's verdict survives only because the task is P3's four-write
splice, where the same field is written twice: `splice(x: &Node, m: &Node, y: &Node)
writes(x.next), writes(m.prev), writes(m.next), writes(y.prev)` really does put
`x.next` against `m.next` and `m.prev` against `y.prev`, and Rule 10's `two(&v[i],
&v[j]) // accepted only with the fact i != j` then demands two distinctness proofs
that Rule 11 cannot supply. Corrected cost: three bounds compares plus two
distinctness compares plus the writer-supplied arms for the splice, and two bounds
compares (no distinctness) for the two-node link the cell printed. The Rule 4 half
of the cell is correct, but "Rule 10's pairwise check is nearly vacuous for Box
structures - everything reachable is trivially disjoint by root" is contradicted by
5-9's own rejected `merge(&*root, &*((*root).l))`.

**5-11: agree.** `accepted-fine` is right. Rule 10's "Recursion is checked through
contracts, never by unfolding bodies" carries `sum`, the rule file imposes no
termination obligation (noting its absence is not inventing one), and the fact
survival across `f(&other)` is exactly Rule 11's "A fact that mentions a path is
invalidated when that path is written" — `writes(other)` mentions no path under
`*b`. The structural observation is sound: Rule 1's closure ("no type parameter,
wrapper, or variant payload through which a reference can be stored") plus Rule 5's
"owns exactly one heap object" makes an owned Box structure a forest, so no
sharing invariant has to be stated. Program defect that does not move the verdict:
`use((*b)[n - 1])` has no `n > 0` in scope, so the index is unprovable as written
(and on `u64` the subtraction is itself unguarded).

**5-12: agree.** `rejected-zero-cost` is right. The loop is rejected, and Rule 8
gives a cleaner ground than the one cited: "Affine values are consumed at most
once", which the back edge violates directly; Rule 12's "every path is either
wholly present or the program cannot name it" reaches the same place at the header.
The diamond case is correctly free, since the release edge is statically
identified and Rule 5 confirms "there are no destructors". The rewrite is free but
not for the reason given — breaking out of the loop at the first hit is not
available, because `step(i)` must still run for every `i`. The zero-cost form is
loop splitting: run until the first `ready(i)`, consume once, run the remainder
without the test, which is tail duplication and free under the candidate's own cost
rule. Bookkeeping defect: "Depends on" omits Rule 12, the row's own rule, and lists
Rule 11, which the trace never uses.

**5-13: agree.** `accepted-fine` is right. The substituted write paths
`*((*root).l)` and `*((*root).r)` diverge at distinct fields, which is Rule 10's
`two(&a.x, &a.y)` shape, and Rule 14 removes the two allocations from the conflict
judgment with its own `par { a = build(&x)?; b = build(&y)? }` line, so the `?` in a
`par` arm is blessed by the rule file rather than assumed. Nothing is emitted at
runtime for the acceptance, and the recorded stopping point (runtime-selected
subtrees are unformable by Rule 4) is consistent with 5-10 and 5-16. Typo only: the
struct declares `Box<Node2>` for a `Node`.

**5-14: agree.** `rejected-zero-cost` is right. Rule 14's "Addresses are not
observable" removes the identity test, Rule 1 plus Rule 5 removes the two-handles-
to-one-object premise, and the recursive-build program is accepted and free (Rule 8
releases `l` on the statically identified `?` edge). The zero-cost claim depends on
Rule 16, which the cell declares. Program looseness: `b1 = lookup_by_name(...)`
returning an owning handle would move the object out of the registry, which Rule 6
forbids; the point about identity survives because it does not depend on how the
handles were obtained.

**5-15: agree.** `rejected-zero-cost` is right. Rule 4 rejects the returned
reference at the signature, and the callback is licensed by Rule 10's "Function-
typed parameters carry a full signature with its own row and contract, and a call
through one uses that row", with `writes(x)` substituted to `writes((*t).value)`
and covered by `writes(t)` under Rule 9's body check; the recursive arms are covered
the same way, since `*((*t).l)` is a path below `t`. The residual indirect call is
correctly disclosed rather than hidden, and is the only thing standing between this
label and `rejected-real-cost`; the rule file says nothing about specialization, so
the optimistic reading is a compiler assumption, not a rule.

**12-12: agree.** `accepted-fine` is right. The three edges do all support
`hit < n`, Rule 12 intersects to keep it, and the carried `invariant g: len_of(buf)
== n` discharges Rule 7 with no runtime work; nothing in the loop writes `buf`, so
neither Rule 3 nor Rule 11 bites. The handling of the unenumerable target set is
honest and the cell is right that its verdict does not turn on it, because both
checks are uniform over the set. One program defect: `p = &buf[0]` has no `0 < n`
in scope.

**12-13: agree on the label, correct one citation and one claim.** The four pairs
are enumerated correctly and the `par` is accepted, but `cols.a` versus `cols.b` is
not "different roots" — the root is `cols` for both. The ground is Rule 10's
distinct-field line `two(&a.x, &a.y) // accepted: distinct fields`, the same ground
5-10 failed to apply. The closing claim "This rewrite is always available" is too
strong: Rule 4 stops a target set arriving from a call or a stored value, but 12-12
builds a target set from a *data-determined index* (`p = &buf[hit]`), and no amount
of duplication or peeling enumerates that set. Restrict the claim to source-level
branch joins, which is all the printed program needs. The flagged non-decisive
ambiguity (whether the end of a `par` is a Rule 12 join) is real and correctly
parked.

**12-14: correction: accepted-fine -> rejected-zero-cost.** The strongest program
contains `use(q) // rejected on both edges`, and the cell's own trace confirms the
rejection under Rule 3 and Rule 10 clause 3, with no re-validation available (Rule
3: "Validity is re-established only by forming the reference again"). Under this
file's own convention — 5-12, 5-14 and 5-15 all carry `rejected-*` while reporting
the task achievable — a rejected line in the strongest program sets the label. The
rewrite is genuinely free: correct C++ must also re-form `&v[j]` after a growth,
and the cell says so itself ("unchecked C++ keeps the pointer and is wrong
precisely when the vector grew"), so `rejected-zero-cost`, not a per-reformation
load charged against a buggy baseline. Second correction, to the ground rather than
the outcome: `use(&v[j])` does not survive by Rule 12's intersection. `push`
declares `writes(v)`, so Rule 11 kills `j < len_of(v)` at the call "unless the
callee's `ensures` re-establishes it". Rule 12's worked example works only because
`push_nogrow` carries `ensures len_of(buf) == len_of(deref(entry(buf))) + 1`; the
library `push` must be given `ensures when Ok:` / `ensures when Err:` relations
against `entry(v)` for the same conclusion. Constructible under Rule 11, but it has
to be written.

**12-15: agree.** `accepted-fine` is right. Rule 11's "`ensures when Variant:` for
result-routed relations" delivers the bound on the arm that uses it, which is Rule
4's own printed program, so `&v[i]` costs one address computation and no compare.
The label survives the `use_after(i)` line because no rule of this candidate
rejects it — `i` is a variant binder that is simply not in scope after the match,
which is ordinary scoping the rule file does not legislate; the cell's framing of
it as an intersected-away fact is imprecise but harmless. The `writes`-plus-
`ensures` second shape is correctly grounded in Rule 11's `unless` clause.

**12-16: agree on the label, correct the decisive step.** The claim that
`i < len_of(pool.buf)` survives the second `obtain` by Rule 12's "facts about
indices below n survive" is wrong. That example is a statement-level branch whose
call carries `push_nogrow`'s `ensures`; here `obtain` declares `writes(p.buf)`, so
Rule 11 invalidates the caller's fact at the second call, and `obtain`'s printed
contract (`ensures result < len_of(p.buf)`) says nothing about the pre-state. The
repair is the one the cell applies only to the growing push: add `ensures
len_of(p.buf) >= len_of(deref(entry(p.buf)))`, provable on both internal edges (+1
on the alloc edge, unchanged on the reuse edge), at zero runtime cost. Second
defect, in the cost line: the body calls `alloc(p, x)` without discharging Rule
16's `requires len_of(a.buf) < cap_of(a.buf)`, so the honest count is two compares
per allocation (free-list bound and capacity), not one — though the capacity
compare is one C++'s `push_back` also makes, so the delta against C++ stays at the
free-list compare. Everything else holds: the internal join is intersected
correctly, occupancy-as-data is Rule 6's "no program point can observe a slot
inside the window as empty", and the `truncate` break is Rule 16's own sentence.

## Confirmed gaps

1. **5-5, 5-8 — no operation takes a value out of a field or a Box payload.**
   Rule 6: "There is no `take` operation and no partial move out of any place: the
   only ways to move a value out of storage are consuming a whole local (`move x`),
   the window operations, and the atomic update." Against Rule 8's `fn use_it(c: own
   Conn) { ...; close(move c.f) }` and `Box<File> // linear: must be taken apart and
   its File closed`. Missing: either an admitted field/payload extraction, or a
   statement that Rule 8's example is illustrative and linear values may not be
   placed in aggregates or Boxes at all.
2. **5-5 — no variant-payload selector.** Rule 2: "A path starts at a local
   variable or a parameter and continues through fields, `*` (Box content), `[i]`
   (index, Rule 7), or `[lo..hi]` (range, Rule 7)." Rule 1 blesses `Option<Box<Node>>`
   and Rule 12 blesses `slots: DynBox<Option<Entry>>`, but no path names the payload
   inside `Some`. Missing: a payload selector, or the matching form that binds it.
3. **5-8 — a fallible allocation swallows a linear argument.** Rule 14:
   "Allocation returns a `Result` and never traps", with Rule 5's
   `Result<Box<Node>, Oom>`, against Rule 8's "Linear values must be consumed by an
   explicit operation on every exit path; the compiler never releases them."
   Missing: a value-returning error arm, e.g. `Result<Box<T>, (Oom, own T)>`.
4. **12-12, 12-13 (non-decisive) — no representation for a data-determined target
   set.** Rule 2: "At a control-flow join, a reference variable's target is the set
   of paths it may name; every check on it must hold for every member of the set."
   For `p = &buf[hit]` with `hit` data-determined the set is not enumerable.
   Missing: whether such a set is represented symbolically, and what "every member"
   means when the members differ only by an unknown index.
5. **12-13 (non-decisive) — the end of a `par` is not classified.** Rule 12: "At a
   join, facts are intersected (a fact survives only if it holds on every incoming
   edge)." Rule 13 says nothing about facts after `par { A; B }`, where both arms
   run. Missing: a statement that both arms' `ensures` hold afterwards.
6. **New, found in this pass (5-6's shape, 12-16's shape) — index uncertainty in
   the prefix judgment.** Rule 10 clause 3: "A live reference outside the call whose
   path has a proper prefix among the call's write paths becomes invalid after the
   call (Rule 3)." Clause 1 treats index uncertainty conservatively (`two(&v[i],
   &v[j]) // accepted only with the fact i != j`), but clause 3 and Rule 3 never say
   whether `v.buf[j]` counts as a prefix of `v.buf[i]` when `i != j` is not proved.
   The optimistic reading admits the unsafe program below. Missing: the same
   "proved distinct" requirement in clause 3.
7. **New, found in this pass (12-16) — content writes versus facts.** Rule 11: "A
   fact that mentions a path is invalidated when that path is written (by statement
   or call)." Rule 3 has an explicit content-write carve-out for references
   ("Writing the storage at p's path or below it ... does not invalidate p"); Rule
   11 has none, so it is not stated whether `p.buf[h] = x` invalidates
   `h < len_of(p.buf)`. Under the strict reading 12-16's declared `ensures` is not
   provable. Missing: the carve-out restated for facts.

## Dismissed gaps

1. **12-16's "the atomic update has no shape that also installs a replacement".**
   Rule 6 constrains the old value and the committed result, not the arity of `f`:
   "`buf[k] = f(buf[k])` // atomic in-place update: the old value goes into f by
   value, f's result is committed, no program point lies between". `buf[k] =
   free_node(buf[k], repl)` fits the stated form, so a free step that consumes the
   old occupant and installs a replacement is expressible.
2. **5-9's implied gap that a nested write pair has no clean rewrite.** Rule 9
   covers it: "A function body is checked against its own row: every statement's
   effect and every callee's substituted row must be covered by the declared row."
   One enclosing reference with `writes(n)` performs both writes inside one row,
   and Rule 10's pairwise comparison applies only "At a call".
3. **5-13's untraced `?` inside a `par` arm.** Rule 14's own example covers it:
   `par { a = build(&x)?; b = build(&y)? }`.

## Confirmed real costs

- **5-6.** Order-preserving access to a `Box` held in a `DynBox` slot pays a block-
  base load plus a slot load on every use, where correct C++ `vector<unique_ptr<T>>`
  keeps one register across the growth. The free alternative (`swap_remove` the Box
  to a local) permutes the container.
- **5-10, corrected.** Two bounds compares for the printed two-node call; for P3's
  four-write splice, three bounds compares plus two index-distinctness compares plus
  the writer-supplied unreachable arms, because `writes(m.next)` meets
  `writes(x.next)` and `writes(m.prev)` meets `writes(y.prev)` on the same field and
  Rule 11 has no fact that discharges them.
- **5-16.** One compare plus one branch per traversal hop; four bytes of generation
  per node with one load and one compare per dereference, mandatory for the
  compiler-graph task by Rule 16's "a logic error, not a memory error"; and the loss
  of parallel allocation into a shared arena, since Rule 16's `alloc` declares
  `writes(a.buf)` while Rule 14 exempts `Box::new`.
- **12-16, corrected.** Two compares per allocation inside `obtain` (the free-list
  bound, because "every free-list handle is in bounds" is the quantified fact Rule
  11 forbids, plus Rule 16's undischarged `requires len_of(a.buf) < cap_of(a.buf)`);
  only the first is a delta against C++.
- **5-15.** One indirect call per found node whenever the callback instantiation is
  not specialized; the rule file says nothing that guarantees specialization.

## Accepted-unsafe program constructible in these cells

Built from 5-6's shape (`DynBox` of `Box`) and gap 6 above. Every line is admitted
by the rule text under the optimistic reading of Rule 10 clause 3, and the last
line reads freed storage.

```text
buf: DynBox<Box<Node>>
fn replace(slot: &Box<Node>, b: own Box<Node>)  writes(slot)     // Rule 9, as put_at

i = pick(); j = pick()                      // data-determined, i != j unproved
if i < len_of(buf) { if j < len_of(buf) {
    p = &(*(buf[i])).value                  // Rule 7 discharged; p's prefixes are
                                            // buf, buf[i], *(buf[i])
    replace(&buf[j], Box::new(mk())?)       // call effect: writes(buf[j]).
                                            // Rule 6: "Assigning buf[k] = x where the old
                                            // value is affine releases the old value", and
                                            // Rule 9: "writes covers ... freeing the storage
                                            // at the path" - the old heap Node is freed.
    use(*p)                                 // Rule 10 clause 3 invalidates p only if a write
                                            // path is a *proper prefix* of p's path. buf[j]
                                            // is not syntactically a prefix of buf[i], and no
                                            // clause requires i != j to be proved here.
} }
```

At runtime with `i == j`, `use(*p)` reads the freed `Node`. Rule 10 clause 1 does
not catch it, because `p` is not an argument of the call and clause 1 compares only
the call's own effects. The conservative reading of clause 3 (treat an unproved-
equal index as a prefix, exactly as clause 1 treats it for disjointness) rejects the
program and costs one `i != j` compare with a duplicated arm; the rule file does not
say which reading holds, which is why this is listed as gap 6 rather than as a
defect of the cells.

Two further accepted programs are memory-safe but silently wrong at the task level,
and neither cell flags them as an `accepted-unsafe` variant:

- **5-16 / 12-16 without the generation word.** Rule 16 admits it ("A stale index
  that is still in bounds names the current occupant of that slot"), but PROGRAMS.md
  P3 requires that "a relation to a deleted node must not silently become a relation
  to a replacement node". The rules accept the violating program; only the writer's
  discipline prevents it.
- **5-6's `swap_remove` rewrite.** Accepted and free, but it moves the last element
  into slot `k`, so every other stored handle into `buf` silently changes meaning.
