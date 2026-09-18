# Verification of `matrix-x1/rows-06-11.md` against CANDIDATE-X1.md

Independent re-trace of every cell in row 6 (DynBox window) and row 11 (facts)
against the frozen rule file, and only that file. Five verdicts change.

## Per-cell findings

**6-6: correction: undecided-rule-gap -> rejected-real-cost.** The claimed
contradiction is not one. Rule 6's enumeration is closed: "There is no `take`
operation and no partial move out of any place: the only ways to move a value out
of storage are consuming a whole local (`move x`), the window operations, and the
atomic update." Reading (a) — a bulk relocation of `[0, len_of)` — is not among
them; "`Vector<T>` is library code: a DynBox plus a `push` that ... moves
`[0, len_of)` across" describes what the library push achieves, not a primitive,
and Rule 1's "Consequence: any value can be relocated by copying its bytes" is a
statement about representation, introduced by the word *Consequence*, not an
admitted operation. A second sentence closes reading (a) independently: Rule 6
says of the new block's boundary that "`len_of` is a runtime number stored in the
block header, readable by the program, and changed only by the built-in
operations below", and no listed operation raises `len_of(nb)` from 0 to n except
n calls to `push_nogrow`. So only reading (b) survives, and the deriver's own
`swap_remove`/`pop` schedule is the answer. I re-ran its trace for n = 6 and both
`requires` obligations (`i < len_of(old)` from `2i + 1 < n`, `len_of(nb) <
cap_of(nb)` from invariant `b` and `cap_of(nb) >= n`) and it is correct. The floor
under these operations is 2n moves (one out, one in, per element) and that floor
is unreachable for order preservation, because `pop` is the only zero-overhead
extraction and it extracts in descending order; 2.5n is the right figure. The cost
is therefore confirmed, not conditional.

**6-7: agree.** `accepted-fine` is right: Rule 6 defers the index obligation to
Rule 7, and Rule 7's own example prices it ("the test establishes the fact; one
compare, no trap"). The carry-through for the second `pop` is sound — Rule 11
invalidates `len_of(stack) >= 2` at the first `pop` and the `ensures
len_of(buf) == len_of(deref(entry(buf))) - 1` re-establishes `>= 1` against the
pre-state term, which is not written. Two program defects that do not move the
verdict: `op = code[pc]` has no discharge for `pc < len_of(code)` (`step` needs a
`requires`, zero cost), and the `len_of(stack) < cap_of(stack)` test is
unnecessary after two pops, since `len_of` provably dropped by two — the cell
overstates its own cost slightly.

**6-8: correction: undecided-rule-gap -> accepted-fine.** Rule 8 decides what the
deriver treated as open: "Linear values must be consumed by an explicit operation
on every exit path; the compiler never releases them." A `truncate` that dropped
`File`s out of the window without consuming them would be the compiler releasing
them, which that sentence forbids; so `truncate` is not the discharge for the
elements, whatever it is for the block. The cell's actual question — how a linear
element reaches its consuming operation — is answered without ambiguity by the
drain loop: `pop` is a window operation yielding `own File` into a whole local
(Rule 6), `close(move t)` is the explicit operation, `len_of == 0` holds on every
exit path. Cost is negligible, as derived. A residual ambiguity about the literal
`truncate` call does survive and is recorded below, but it does not decide this
cell and cannot make the drain loop undecided.

**6-9: correction: undecided-rule-gap -> rejected-zero-cost.** The cell's question
is answered entirely by rules the deriver quoted. The container-plus-slot helper
`compact(&t.slots, &t.slots[i])` is rejected by Rule 10 clause 1 — the substituted
paths cannot be "proved disjoint (different roots, or indices or ranges proved
distinct)" when one is a prefix of the other, which is Rule 10's own
`bad(&vv, &vv[0])` line — and the index-parameter rewrite costs one shift-add the
callee performs anyway. The `put_at(&buf[len_of(buf)], 3)` line is likewise
decided, not ambiguous: Rule 6 states flatly "Reading `buf[k]` or forming
`&buf[k]` requires the fact `k < len_of(buf)` (Rule 7)", and `len_of(buf) <
len_of(buf)` is false, so that call is rejected. An illustrative line that
contradicts a normative sentence of the same rule is a defect in the example; it
does not create a case the derivation cannot decide. "Rules consistent: no" stands
as an observation about the file; the verdict is the zero-cost rejection.

**6-10: agree.** `rejected-real-cost` is right and the arithmetic checks out. I
tried to beat the 9-move composition and could not: `swap_remove` always refills
the vacated slot from the tail, so every extraction that is not a `pop` costs two
moves; `buf[i] = x` on an occupied affine slot "releases the old value" (Rule 6)
and would destroy the displaced element; and the atomic update "is committed" back
into the same slot, so it cannot leave a slot free. Rule 10 clause 1 correctly
charges one compare for data-determined `i != j`, per its own `two(&v[i], &v[j])`
line. One unstated premise the cell leans on, which I accept: `len_of(buf) <=
cap_of(buf)` as a standing fact, entailed by Rule 6's "Slots `[0, len_of)` hold
values; slots `[len_of, cap_of)` hold nothing."

**6-11: correction: undecided-rule-gap -> accepted-fine.** This is the most
consequential change. Rule 11 says a fact "is invalidated when that path is
written (by statement or call)". The fact mentions the path `a`; the statement
writes the path `a[i]`; those are different paths, and Rule 9 confirms the
written path is the argument's: "`writes` covers writing, replacing, moving out
of, and freeing the storage at the path." The "overlap reading" imports Rule 10
clause 1's overlap judgment into Rule 11, but Rule 10 clause 1 is explicitly
scoped to a call's own pairwise comparison ("Compare the substituted effects
pairwise") and Rule 11 never refers to it — adopting it is inventing a rule.
Two further sentences settle it in the same direction. Rule 6: `len_of` is
"changed only by the built-in operations below", and an element write is not among
them, so `len_of(a) == n` cannot have become false. Rule 3 decides exactly this
direction for the analogous reference question: "Writing the storage at `p`'s path
or below it (a content write) does not invalidate `p`" — and Rule 3 separately
keeps the opposite direction (a *proper prefix* write does invalidate), which is
precisely the distinction the deriver failed to apply. The invariant survives, the
header is loaded once, and the kernel vectorises: no cost. Cells 6-15, 6-16 and
11-16 should read their header-load dependency as settled narrowly.

**6-12: agree.** `accepted-fine` is right: Rule 12's intersection is semantic (its
own worked example produces `len_of(buf) >= n`, which holds on both edges), Rule 6
forbids a hole on either arm ("no program point can observe a slot inside the
window as empty"), and the one-arm-consumption story is Rule 8 plus Rule 12's
"every path is either wholly present or the program cannot name it". Three program
defects that do not move the verdict, all zero-cost to fix: the row `writes(t.tags),
writes(t.slots)` does not cover `t.count = t.count + 1`, which Rule 9's body check
requires ("every statement's effect ... must be covered by the declared row");
`probe` is given no `ensures h < len_of(t.tags)`, so the indexing is undischarged;
and a zero-filled `slots: DynBox<Entry>` is not available for an affine `Entry`,
since `DynBox::filled(n, v)` is stated for "T Copy" — Rule 12 prescribes
`DynBox<Option<Entry>>` for exactly that case, which the cell's cost paragraph does
price.

**6-13: agree.** `rejected-real-cost` is right. Both arms carry `writes(hist)` on
one root with no index in the row — Rule 9 forbids one ("Signatures never contain
index expressions") — so Rule 13 has nothing to prove distinct. Atomics are under
"Not in this candidate", so privatise-and-merge or a bucketing pre-pass are the
only rewrites and both cost at least one extra full pass. The observation that a
range reference can never move the boundary is correct: every window operation in
Rule 6 takes `&DynBox<T>`. The cell's parallel-append remark should now cite 6-6's
settled 2.5n figure rather than an open reading. Minor: `h1`/`h2` are bound inside
the arms and used after them, and the merge loop's invariant does not carry
`len_of(h2)` or `len_of(hist)`; both are zero-cost repairs.

**6-14: agree.** `rejected-real-cost` is right, and the ground is the absence of
any resize operation, not the fallibility: Rule 6 fixes `cap_of` "at allocation"
and the only growth path "allocates a larger DynBox, moves `[0, len_of)` across,
and replaces the old one". Rust's `RawVec` reaches `realloc`, which for large
mappings often extends in place, so the whole-window copy per doubling is a real
cost against that baseline. The fact story is right too: Rule 14's "Allocation and
release carry no effect entry" plus Rule 11's "when that path is written" means a
failed allocation invalidates nothing. Contract defect, zero-cost: `move_all`'s
stated `ensures len_of(old) == 0` says nothing about `len_of(nb)`, so neither the
invariant `i <= len_of(v.buf)` nor `push_nogrow`'s `requires` is re-provable after
`v.buf = move nb`; it needs `ensures len_of(nb) == len_of(deref(entry(old)))`.

**6-15: agree.** `accepted-fine` is right: nothing is rejected, the bounds relation
crosses the return as a Rule 11 contract fact, and Rule 4's own comment prices the
residue ("the caller re-derives, one address computation"). The one real cost named
— a dependent header load per `len_of` read where Rust's `Vec` has a register field
— is genuine but now hoistable in any loop that does not write the buffer, given
the 6-11 correction.

**6-16: correction: accepted-unsafe -> rejected-real-cost.** The cell's own trace
says "the program is memory-safe by Rule 6 and Rule 7 alone", and Rule 16 states
the disposition in terms: "A stale index that is still in bounds names the current
occupant of that slot: a logic error, not a memory error. Programs that need to
detect it keep a generation number as data." Under the candidate's safety
criterion — "safety through types and proofs at the current WF level (no runtime
traps, no unsafe)" — nothing unsafe is accepted here: no freed or relocated
storage is read, because Rule 6 guarantees an in-window slot holds a valid value
and Rule 10 clause 3 kills every live reference into the pool at each `alloc`.
What is actually true is that P3's stated observable is recovered only by the
generation word the rule itself names, at a real per-hop cost (one extra load and
compare, 4-8 bytes per node, plus an outcome arm). That is the definition of
`rejected-real-cost`. `accepted-unsafe` should be reserved for a program the rules
admit that violates the safety criterion; this is not one.

**11-11: agree.** `rejected-real-cost` is right and Rule 11 is explicit: "There are
no quantified facts over array elements ("for all i ...")". `ok` survives the join
as data but is a fact about a boolean, and no rule converts it. The fused
test-per-element rewrite is the best available; a masked or modular index
(`t = idx[j] % len_of(src)`) would be cheaper and vectorisable but relies on a fact
form Rule 11 does not list, so the conclusion stands. One reasoning defect: the
"ordinary program" carries `invariant i1: r * cols + c <= len_of(m)`, and
`r * cols` with both operands variable is not an `affine_expr`, so that invariant
is not in Rule 11's stated form. The 2D case is still zero-cost, by a different
route the cell does not give — strength-reduce to a row base and let the outer loop
condition `base + cols <= len_of(m)` be the test, after which `base + c <
len_of(m)` is affine in `base` and `c`, one compare per row.

**11-12: agree.** `accepted-fine` is right, and the evidence is decisive: Rule 12's
own example yields `len_of(buf) >= n`, a relation that holds on both incoming edges
though it is written on neither, so "a fact survives only if it holds on every
incoming edge" is semantic entailment and the strongest program's `i <
len_of(buf)` survives. The point that the writer has no proof step at an if-join is
correctly grounded in Rule 11's single location for explicit steps, "inside an
`invariant`". Minor: the then-edge fact `len_of(buf) > 0` is stipulated rather than
established by the fragment; a dominating length test supplies it at zero cost.

**11-13: agree.** `accepted-fine` is right. Rule 13 borrows Rule 10's judgment
verbatim and places no condition on where the fact came from, so a dominating
`if i != j` admits the block exactly as an arithmetic range split does; Rule 13's
"Read/read overlap is allowed" covers the shared `table`. The observation that a
`par` is not a join, so no arm can invalidate the other's facts, follows from the
disjointness that admitted the block in the first place. Duplicating the sequential
arm is correctly not counted as a cost, per protocol item 7.

**11-14: agree.** `accepted-fine` is right, on Rule 14's "Allocation and release
carry no effect entry" read together with Rule 11's "when that path is written":
no write path, no invalidation. The separation of the harmless allocation from the
fact-destroying `v.buf = move nb` is exactly right. Same contract defect as noted
under 6-14 — `move_all` must also ensure the new length — which is a zero-cost
repair, not a verdict change.

**11-15: agree.** `accepted-fine` is right. Rule 11's "Contracts use `requires`,
`ensures`, and `ensures when Variant:`" plus facts as "affine comparisons over
measures and integer values" carries the two-sided `Span` relation across the
return, discharging Rule 7's range obligation `lo <= hi <= len_of(v)` with no
test, and Rule 11's invalidation clause correctly kills it if the caller writes
`v` in between. Naming an integer through a path into the result (`result.lo`) is
a mild extension of Rule 4's bare `result`, but Rule 11 places no restriction on
fact terms beyond their being integer values, so I do not count it as invented.

**11-16: agree.** `rejected-real-cost` is right, and it is the one cell in the file
where Rule 11 refuses the wanted fact by name: "There are no quantified facts over
array elements ("for all i ...") and no per-slot occupancy facts". The single place
the fact language does help is correctly identified — `alloc`'s two `ensures`
combine by affine reasoning to `result < len_of(a.buf)` for a one-hop-fresh handle.
With 6-11 settled narrowly, the cost should be restated: the header load hoists out
of a pure traversal loop, so pure traversal pays a register compare, a shift-add,
an outcome arm and the generation compare, while a mutating traversal reloads the
header at every hop because `alloc`/`remove` write `p.buf`.

## Confirmed gaps

These are genuine ambiguities in the rule text. None of them decides the cell it
appears in, so none of them restores an `undecided-rule-gap` verdict.

1. **Cell 6-8 — `truncate` on a linear-element DynBox.** Rule 6: "A DynBox whose
   element type is linear is itself linear (Rule 8) and must be truncated to zero
   by the program before it can go out of scope." `truncate`'s contract is only
   "`requires n <= len_of(buf); ensures len_of(buf) == n`" and says nothing about
   what becomes of the dropped elements. If "truncated to zero" names the literal
   `truncate(&buf, 0)` call, the sentence prescribes an operation that discards
   linear values; if it means "brought to length zero by the program", the drain
   loop satisfies it and `truncate` with `n < len_of` is simply rejected for linear
   elements by Rule 8. The file does not say which. The cell is still decided,
   because the drain loop is admitted under either reading.

2. **Cells 6-6 and 6-12 — arity of the atomic update.** Rule 6 shows one form:
   "`buf[k] = f(buf[k])` // atomic in-place update: the old value goes into f by
   value, f's result is committed, no program point lies between; requires
   `k < len_of(buf)`". Whether `f` may take further arguments is unstated, and both
   cells rely on it — 6-12's `t.slots[h] = merge(t.slots[h], move e)` and 6-6's
   `buf[k] = merge(buf[k], &buf[m])`. This matters: a hash table cannot reach an
   occupied slot's payload any other way, since `swap_remove` would destroy the
   probe sequence. Nothing in the sentence limits arity, so I read it as permitting
   extra arguments and leave both verdicts standing.

3. **Cell 6-6 — a call result as a by-value argument.** Rule 10 clause 2 covers
   only "A by-value argument contributes a consumption (`move`) or a read (copy) of
   its place to this comparison", and a call result has no place, so whether an
   inner call's effects enter the outer call's pairwise comparison is unstated. The
   deriver flagged this and correctly declined to let it drive the verdict, since
   sequencing into two statements removes the question at zero cost.

## Dismissed gaps

1. **6-6, Vector growth for affine `T`.** Covered by Rule 6 twice over: "the only
   ways to move a value out of storage are consuming a whole local (`move x`), the
   window operations, and the atomic update", and "`len_of` is ... changed only by
   the built-in operations below". No bulk relocation is admitted and nothing else
   can set the new block's length, so reading (a) is not available and the rules
   decide.

2. **6-11, does a sub-path write invalidate a length fact.** Covered by Rule 11's
   "invalidated when *that path* is written", by Rule 6's "changed only by the
   built-in operations below", and by Rule 3's explicit content-write clause,
   "Writing the storage at `p`'s path or below it (a content write) does not
   invalidate `p`". The competing "overlap reading" borrows a judgment Rule 10
   clause 1 confines to a call's own pairwise comparison.

3. **6-9, `put_at(&buf[len_of(buf)], 3)`.** Covered by Rule 6's "Reading `buf[k]`
   or forming `&buf[k]` requires the fact `k < len_of(buf)` (Rule 7)". The
   obligation is `len_of(buf) < len_of(buf)`, which is false; the illustrative line
   is simply wrong and the derivation is not blocked.

4. **6-8, the discharge of a linear element.** Covered by Rule 8's "Linear values
   must be consumed by an explicit operation on every exit path; the compiler never
   releases them", which forbids a built-in from silently dropping them and leaves
   `pop` + `close` as the route. (The narrower question in Confirmed gaps 1
   survives; the cell's question does not.)

## Confirmed real costs

- **6-6 / 6-14, Vector growth of affine elements.** About 2.5n element moves plus
  2n header stores per growth event, against one `memcpy` in `std::vector` or Rust
  `RawVec`; the loop is not a `memcpy` candidate because each `swap_remove`'s
  refill depends on the previous length. Compounded by the absence of any resize
  operation: every doubling copies the whole window, where `realloc` can often
  extend a large mapping and copy nothing.
- **6-10, interior swap of affine elements.** 9 moves and 6 header stores via three
  swap-with-last compositions, against 3 moves for `std::swap` or `slice::swap`;
  roughly 3x the byte traffic in a sort's inner loop. Swap-against-the-end forms
  (heap sift-down, Lomuto's final placement) pay nothing.
- **6-13, parallel scatter into a shared table.** Privatise-and-merge costs k
  private tables of m slots, a zeroing pass and a merge pass of k*m; the bucketing
  alternative costs a full extra pass over the index array. Against a contended
  `atomic<u64>::fetch_add` this loses outright once m is large relative to n, and
  no rewrite inside the candidate recovers it, since atomics are excluded.
- **11-11, gather through a validated index array.** One compare, one branch and an
  outcome arm per element, and the loss of a straight vector gather, against a C++
  form that validates once; roughly 1.5-2x on a load-and-store kernel, and
  irrecoverable because Rule 11 excludes the quantified fact.
- **6-16 / 11-16, pool handle hop.** One shift-add, one bounds compare, one outcome
  arm, and a generation load and compare where identity matters, plus 4-8 bytes per
  node. With 6-11 settled narrowly the bounds compare is against a register in a
  pure traversal, but a mutating traversal reloads the header every hop.
- **6-15, boundary in the block header.** One dependent load per `len_of` read
  where Rust's `Vec` reads a register-resident field; small, and hoistable out of
  any loop that does not write the buffer.

## Accepted-unsafe programs constructible in these cells

- **One, and only under the literal reading of Rule 6's truncate sentence
  (Confirmed gap 1).**

  ```text
  linear type File;   fn close(f: own File)
  files: DynBox<File>                 // linear by containment, Rule 8
  ...                                 // n Files pushed in
  truncate(&files, 0)                 // "must be truncated to zero by the program"
  // scope exit: the block is empty and affine, the compiler frees it
  ```

  Every `File` is discarded without reaching `close`, defeating Rule 8's guarantee
  that "Linear values must be consumed by an explicit operation on every exit
  path". This is a resource-obligation breach, not memory corruption. Rule 8's next
  clause, "the compiler never releases them", is the sentence that should reject
  it, which is why I dismiss the gap as decisive for cell 6-8 while recording it
  here.

- **Nothing in these cells reads freed or relocated storage.** Every route I could
  construct is closed by Rule 3 through Rule 10 clause 3: growth (`v.buf = move
  nb`), `alloc` (`writes(a.buf)`), `truncate`, `pop`, `swap_remove` and
  `push_nogrow` all declare a write of the buffer path, which is a proper prefix of
  any reference formed into it, so "A live reference outside the call whose path
  has a proper prefix among the call's write paths becomes invalid after the call".
  Rule 16's own weaker comment ("valid until `a.buf` is replaced") is superseded by
  that stricter judgment, as cell 6-16 correctly notes.

- **6-16's stale handle is not an instance.** Rule 16 classifies it explicitly:
  "a logic error, not a memory error". It is a cost, not an unsafe acceptance.
