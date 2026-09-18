# Verification of `matrix-x1/rows-04-13.md` against CANDIDATE-X1.md

Every cell re-traced literally against the frozen rule file. Two verdict
corrections; the rest agree. Cost entries are runtime only.

## Per-cell results

**4-4: correction: undecided-rule-gap -> rejected-zero-cost.** The capture case
is decided, not ambiguous. Rule 9 closes it twice over: "An effect row lists
`reads(path)` and `writes(path)` where each path starts at a reference parameter"
- `log` is not a parameter of `h`, so no row of `h` can name `reads(log)` or
`writes(log)` - and "A function body is checked against its own row: every
statement's effect and every callee's substituted row must be covered by the
declared row" - `h`'s body calls `push_nogrow(&log, ...)`, whose substituted row
is `writes(log)`, which no declared row of `h` covers. So `h` is rejected
whatever the heading of Rule 4 means, and the heading/clause difference the cell
rests on never has to be resolved. The same argument rejects a read-only
capture, since a read is an effect too. The cell's own rewrite (thread the state
as an explicit reference parameter) is right and its cost analysis is right:
zero runtime cost, the environment pointer becomes an argument register, and
extra parameters are not a cost (protocol 7). Rewrite exists at no runtime cost,
so the verdict is `rejected-zero-cost`.

**4-5: agree (undecided-rule-gap).** The gap is real and correctly quoted: Rule
6's "the only ways to move a value out of storage are consuming a whole local
(`move x`), the window operations, and the atomic update" names the atomic
update as a third general mechanism, while its only definition is the
window-shaped `buf[k] = f(buf[k])` with "requires k < len_of(buf)". Nothing
decides whether a struct field is an eligible place. One sharpening the cell
misses: fallback (b), "rebuilding the path by value", is not obviously
expressible either, because reaching a child requires moving out of `*b`, and
`*b` is a path (Rule 5: "`*b` is a path (Rule 2)"), not "a whole local", so the
same enumeration blocks it. If the atomic update is window-only, the only
surviving form for a mutable heap-linked structure is the Rule 16 pool, i.e.
cost (b) is the pool's one compare per hop, not the O(depth) copies.

**4-6: agree (accepted-unsafe).** Re-traced: `find`'s `ensures when Some:
result < len_of(v)` plus `n == len_of(v)` yields `i < n` over two integer
locals, which mentions no path and so survives Rule 11's "A fact that mentions a
path is invalidated when that path is written"; `pop`'s and `push_nogrow`'s
`ensures` re-derive `len_of(v) == n` affinely, discharging Rule 7 with no
compare. `push_nogrow`'s own `requires len_of(buf) < cap_of(buf)` also
discharges, since `cap_of` is "fixed at allocation" and `len_of(v) == n - 1`.
The program is memory-safe throughout (Rule 12: a slot in the window holds a
valid value), so `accepted-unsafe` - a silent identity error - is the right
label. One precision note: the substituted element is observed only when
`i == n - 1` (`pop` takes the last slot and `push_nogrow` refills it); the cell
states it unconditionally. The phenomenon and the verdict stand.

**4-7: agree (accepted-fine).** Rule 15 and Rule 1 admit the owned `Pair`; both
`ensures` clauses are "affine comparisons over measures and integer values"
(Rule 11); Rule 7's "requires lo <= hi <= len_of(buf)" is exactly what they
supply, and `len_of(part) == hi - lo` comes free. Range-of-a-range (`&part[0..m]`)
is within Rule 2's path grammar and Rule 7's "a range reference into either".
The nested inner bound is not a gap: Rule 7 covers it in terms - "when the proof
is unavailable the program tests the measure, which is ordinary data" - which is
what the cell does. Omission, not verdict-changing: the sketch never establishes
`m <= len_of(out)` for `&out[0..m]`; for a non-Copy element type establishing
that window is the cost recorded under 13-15 below.

**4-8: agree (accepted-fine).** The atomic update is the only in-place service
form for a linear element, exactly as Rule 6 says ("where it is linear the
assignment is rejected unless written as the atomic update whose function
consumes the old value"), and `pop` returns `own T` so `move s` is the
"consuming a whole local" case. The drain loop supplies `len_of(pool) > 0` the
way Rule 11's own example does. The two tensions the cell quotes are genuine and
are listed as gaps below; the derivation does stay clear of both, and adding
`truncate(&pool, 0)` after the drain satisfies Rule 6's "must be truncated to
zero" at zero cost, since `len_of` is already 0.

**4-9: agree (undecided-rule-gap).** The contradiction is exact: Rule 9's
example forms `&buf[len_of(buf)]` while Rule 6 says "Reading `buf[k]` or forming
`&buf[k]` requires the fact `k < len_of(buf)` (Rule 7)". Declaring the example
an erratum would be resolving the ambiguity, which protocol 3 forbids, so
`undecided-rule-gap` is correct rather than a rejection. The fallback
`build_into(&buf[len_of(buf) - 1], &s)` is in bounds because `push_nogrow`'s
`ensures len_of(buf) == len_of(deref(entry(buf))) + 1` gives `len_of(buf) >= 1`.
Cost statement checked and real.

**4-10: agree (rejected-real-cost).** Rule 10.1 is the deciding rule and is
applied correctly: both substituted effects are writes rooted at `v`, and
`j = perm[i]` has no affine relation to `i` because Rule 11 offers no fact about
container contents ("There are no quantified facts over array elements"). The
guard `if i != j` is Rule 11's "refinement facts from a dominating branch". One
baseline nit that does not move the verdict: safe Rust's `slice::swap(i, j)`
needs no `i != j` guard, so the guard is a cost against Rust as well as C++,
not "zero against safe Rust". Cost against C++ (one compare, one predictable
branch per swap) is real, and the out-of-place alternative is correctly priced
as strictly worse.

**4-11: agree (rejected-real-cost).** Rule 11's "There are no quantified facts
over array elements ('for all i ...')" is quoted correctly and is the rule that
decides it; Rule 7's test license is the only remaining form. The hoisting
argument (`len_of(nodes)` loop-invariant because `nodes` is only read) is sound,
and the residual cost - one compare and one branch per edge, plus the loss of a
straight-line gather - is real against C++ and zero against safe Rust.

**4-12: agree (undecided-rule-gap), with a second gap in the same program.**
The back-edge gap is genuine: Rule 2 states "At a control-flow join, a reference
variable's target is the set of paths it may name; every check on it must hold
for every member of the set" and Rule 12 states the union; on a loop back edge
that set is `head`, `head.next`, ... with no finite representation, widening,
depth bound or rejection anywhere in the file. A second, earlier gap the cell
walks past: the walk needs to name the payload of an `Option<Box<Node>>` through
a reference, and Rule 2's grammar is "A path starts at a local variable or a
parameter and continues through fields, `*` (Box content), `[i]` (index, Rule
7), or `[lo..hi]` (range, Rule 7)" - a variant payload projection is not listed,
and whether "fields" covers it is unstated. Written as a call,
`content(cur) -> &Node` is flatly rejected by Rule 4's "cannot be returned", so
the program does not even reach the join question by that route. Verdict
unchanged; both gaps recorded.

**4-13: agree (rejected-real-cost).** Rule 13's disjointness test rejects two
arms that both declare `writes(q)`, and the fused form
`par { i = find(&v, a); bump(&v[j]) }` is rejected because `v` is a proper prefix
of `v[j]` and one effect is a write, which Rule 10.1's "different roots, or
indices or ranges proved distinct" cannot discharge for a prefix pair. "Not in
this candidate" does exclude "channels or atomics", and Rule 14's exemption
covers allocation only, so no route exists. The lost dynamic load balancing is a
real, workload-unbounded cost and is correctly tied to the already-recorded
lock-free-ring cost.

**4-14: agree (accepted-fine).** Rule 14's "Allocation and release carry no
effect entry and never make two parallel arms conflict" leaves only
`reads(x)` vs `reads(y)`, which Rule 13 accepts; Rule 4 restricts references and
not ownership, so `-> own Box<Node>` is the sanctioned escape. The `?`-in-an-arm
question is properly booked to 13-14 rather than answered here. The named
`sizeof(T)` copy for `Box::new` of a large inline `T` is a real cost and the
`DynBox` alternative is correctly priced.

**4-15: agree (rejected-zero-cost).** Rule 4's "cannot be returned" rejects
`pick`; the caller-side select is one `cmov`, the same instruction C++ emits, and
proofs are erased before lowering, so nothing runtime is lost. The precision
loss on a two-element target set is a checking loss, not a runtime cost, and the
duplicate-call recovery is correctly priced at zero because Rule 12 intersects
per edge.

**4-16: agree (accepted-unsafe).** Rule 16 states the outcome in terms ("A stale
index that is still in bounds names the current occupant of that slot: a logic
error, not a memory error"), Rule 3 never engages because Rule 1 and Rule 4 keep
references out of the structure, and the program is memory-safe by Rule 7 plus
Rule 12. The `alloc` distinctness proof via `entry` is re-checked and correct.
The generation-word cost (one load, one compare, links 8 -> 16 bytes, halved
edges per cache line) is real and is the dominant term for pointer chasing.

**13-13: agree (accepted-fine).** The row `writes(v[lo..hi])` is admitted by Rule
9 because `lo` and `hi` are arguments; the substituted arms `writes(v[lo..p])`
and `writes(v[p+1..hi])` are disjoint from `partition`'s `ensures lo <= result`
and `ensures result < hi` plus `p < p + 1`, all affine; Rule 10's "Recursion is
checked through contracts, never by unfolding bodies" makes depth free. One
correction to the cell's reasoning, not its verdict: the "step the file leaves
implicit" is not implicit. Rule 9 already says "every statement's effect and
every callee's substituted row must be covered by the declared row", so each
call inside a nested `par` is checked against the enclosing row directly; no
union rule has to be assumed. The tiling observation (a product of two program
variables is outside Rule 11's affine forms) is correct and its fallback cost is
real.

**13-14: agree (undecided-rule-gap).** Nothing in the file defines control flow
leaving a `par` arm, while the file itself writes `par { a = build(&x)?; b =
build(&y)? }`. The cell's sharper point is also correct: Rule 14's "Addresses are
not observable, so allocator concurrency does not affect program determinism"
argues about addresses and does not cover which arm receives `Err` under a
budget, which is observable. The arm-local `Result` rewrite is accepted by Rule
13 (two distinct write locals) and costs one post-join branch: zero.

**13-15: correction: accepted-fine -> rejected-real-cost.** The rules-consistency
analysis is right - adjacent output ranges `writes(out[0..c1])` and
`writes(out[c1..c1+c2])` are discharged affinely by Rule 10.1, and Rule 2's "A
path starts at a local variable or a parameter" is indeed what makes two result
locals distinct roots. What the cell misses is how `out` comes to have a window
at all in the strongest case. `out = DynBox::filled(c1 + c2, blank)?` is
annotated in Rule 6 as "T Copy: cap_of == len_of == n, every slot holds v", so it
is unavailable for a non-Copy `Entry`; the alternative `DynBox::new<T>(n)?`
yields "cap_of == n, len_of == 0", and Rule 7 requires `lo <= hi <= len_of(buf)`
to form `&out[olo..ohi]`, so the ranges cannot be formed. Rule 6's only
`len_of`-increasing operation is `push_nogrow`, one element per call, and it
declares `writes(buf)`, so the pre-fill cannot be split across `par` arms (Rule
13, identical write paths). The rewrite is therefore a sequential pre-fill loop
`while len_of(out) < c1 + c2 { push_nogrow(&out, make_blank()) }` before the
parallel scatter; its runtime cost against C++ (reserve raw capacity, construct
each output element once, in parallel) is `c1 + c2` extra element constructions
and stores on the critical path, entirely sequential, plus one release per slot
when the scatter overwrites an affine blank ("Assigning `buf[k] = x` where the
old value is affine releases the old value"). For a Copy `Entry` the cell's
zero-cost claim stands, but protocol 2 requires the strongest program first, so
the cell verdict is `rejected-real-cost`.

**13-16: agree (rejected-real-cost).** The scatter rejection is correct and the
three reasons given are the right ones (Rule 13 identical write roots; Rule 9's
"an index enters an effect only through an argument, evaluated once at the call"
cannot narrow a data-read index; Rule 11 has no facts about container contents).
The P2 acceptance proof through `alloc`'s `ensures` and `entry` is re-checked and
correct, including that the three references are formed at the calls, after the
last `alloc`, so Rule 10.3's post-call invalidation never fires. Both rewrite
costs (privatization memory plus merge pass; transposition pass plus permanent
`|E| * 8` bytes) are real.

## Confirmed gaps

- 4-5, eligible places of the atomic update. Rule 6: "There is no `take`
  operation and no partial move out of any place: the only ways to move a value
  out of storage are consuming a whole local (`move x`), the window operations,
  and the atomic update." The atomic update is defined only as
  `buf[k] = f(buf[k])` with "requires k < len_of(buf)". Missing: whether a struct
  field or a `*b` place is an eligible left-hand side. This also decides whether
  any heap-linked structure is mutable outside the Rule 16 pool.
- 4-9, the append-slot reference. Rule 9's example "put_at(&buf[len_of(buf)], 3)
  // the argument fixes the slot at the call; the row itself names no index"
  against Rule 6's "Reading `buf[k]` or forming `&buf[k]` requires the fact
  `k < len_of(buf)` (Rule 7)." Missing: whether a reference to the append slot
  exists, and if so what moves `len_of`.
- 4-12, a reference rebound on a back edge. Rule 2: "At a control-flow join, a
  reference variable's target is the set of paths it may name; every check on it
  must hold for every member of the set." Missing: a finite representation,
  widening, depth bound or rejection for a loop back edge.
- 4-12, naming a variant payload through a reference. Rule 2: "A path starts at
  a local variable or a parameter and continues through fields, `*` (Box
  content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7)." Missing:
  whether a variant payload is reachable as a path at all, given that Rule 4
  forbids a helper returning the reference and Rule 12's own examples store
  `Option<Entry>` in windows.
- 4-8, partial move out of a local. Rule 8's example "fn use_it(c: own Conn)
  { ...; close(move c.f) }" against Rule 6's "no partial move out of any place"
  and Rule 12's "No place is ever partially moved: every path is either wholly
  present or the program cannot name it." Missing: which of the two is normative.
- 4-8, retiring a linear DynBox. Rule 6: "A DynBox whose element type is linear
  is itself linear (Rule 8) and must be truncated to zero by the program before
  it can go out of scope", while `truncate`'s contract carries no linear-element
  `requires` and Rule 8 says of linear values "the compiler never releases them".
  Missing: what `truncate` does to linear elements it drops.
- 13-14, control flow leaving a `par` arm. The file writes
  `par { a = build(&x)?; b = build(&y)? }` and states only "Addresses are not
  observable, so allocator concurrency does not affect program determinism."
  Missing: whether the sibling completes, which `Err` is returned, and what
  discharges a linear value held by an abandoned arm.
- New, found during verification (4-14 / 4-5 territory): scope-exit release
  versus reference validity. Rule 3: validity is "invalidated when any proper
  prefix of p's path is written, moved out of, replaced, or freed, by a statement
  or by a call", while Rule 8 says "at scope exit the compiler releases their
  memory recursively (Box, DynBox) and runs no user code" - a release that is
  neither a statement nor a call. Missing: whether a compiler-inserted release
  invalidates references formed below it. See the constructed program below.

## Dismissed gaps

- 4-4, "a function value that is neither stored nor returned is not covered."
  Covered by Rule 9 twice: "each path starts at a reference parameter" (no row of
  the closure can name the captured `log`) and "A function body is checked
  against its own row: every statement's effect and every callee's substituted
  row must be covered by the declared row" (the closure's `writes(log)` is
  uncovered). The capture is rejected without reading the Rule 4 heading.
- 4-7, "the nested inner bound `result.j < len_of(vv[result.i])` may be outside
  Rule 11's grammar." Covered by Rule 7: "Every index must be proved in bounds;
  when the proof is unavailable the program tests the measure, which is ordinary
  data", with the cost fixed at "one compare, no trap".
- 13-13, "the effect of a `par` block nested in an arm is left implicit."
  Covered by Rule 9: "every statement's effect and every callee's substituted row
  must be covered by the declared row" - each call inside the nested block is
  compared to the enclosing row directly, so no union rule is needed.

## Confirmed real costs

- 4-6 (safe rewrite): one epoch load plus compare and branch per index use, and
  a written stale arm.
- 4-9: one `sizeof(Entry)` copy per element, or one placeholder initialization
  per element, against C++ `emplace_back`.
- 4-10: one compare and one branch per swap against C++'s unguarded
  `std::swap(v[i], v[j])` (and against Rust's `slice::swap`, correcting the
  cell's "zero against safe Rust").
- 4-11: one compare and one never-taken branch per edge, plus the loss of a
  straight-line gather, against C++ or `get_unchecked`.
- 4-12: roughly one instruction per hop - a call and return per link for the
  recursive form, or one bounds compare per link for the pool form - against a
  C++ pointer walk.
- 4-13: the split is fixed before the region, so an irregular frontier keeps its
  imbalance and the slowest arm sets wall time; re-partitioning per round costs
  one extra `|F|` read/write pass that a stealing scheduler does not perform.
- 4-14: one `sizeof(T)` copy for `Box::new` of a large inline `T`.
- 4-16: one generation load plus compare per hop on top of the bounds compare,
  and links grow 8 -> 16 bytes, halving edges per cache line.
- 13-13 (tiling fallback only): one extra dependent load per tile, one
  allocation per tile, and the loss of one contiguous buffer.
- 13-15 (new, see the correction): for a non-Copy element type, a sequential
  pre-fill of `c1 + c2` elements on the critical path before the parallel
  scatter, plus one affine release per slot when the scatter overwrites the
  blanks. C++ reserves capacity and constructs each output element once, in
  parallel.
- 13-16: privatization costs `P * len_of(nodes) * 8` bytes and a merge pass;
  transposition costs one pass and a permanent `|E| * 8` bytes; neither
  disappears with tuning.

## Accepted-unsafe programs in these cells

1. 4-6 as written: a returned index outlives the element it named, and the
   `pop`/`push_nogrow` pair restores `len_of(v) == n` so `i < len_of(v)` is
   re-proved statically and `charge(&v[i], amount)` runs with no compare on a
   slot the search never saw (observable when `i == n - 1`). Memory-safe by Rule
   12; a silent identity error.
2. 4-16 as written: under Rule 16 a stale handle whose slot has been reused
   names the new occupant, accepted with no diagnostic, against the engineering
   task's "a relation to a deleted node must not silently become a relation to a
   replacement node".
3. Constructed here, and worse than either because it is a genuine memory error
   rather than an identity error, if the Rule 3 gap above resolves the permissive
   way:

```text
fn f(out: &Int)  writes(out)
{
    p = &*dummy                       // some valid path, so p is bound
    {
        b = Box::new(Node { value: 7, ... })?   // affine local of the inner block
        p = &(*b).value                          // Rule 2: a path through `*`
    }                                            // Rule 8: the compiler releases b here,
                                                 // "runs no user code" - no statement, no call
    *out = *p                                    // Rule 3 invalidates only on a write, move,
                                                 // replace or free "by a statement or by a call"
}
```
   Rule 3's trigger list is qualified by "by a statement or by a call", and a
   scope-exit release is neither; Rule 3's own examples are all explicit
   statements or calls (`v.buf[j] = 5`, `grow(&v)`, `c = move b`). Read
   permissively, `*p` reads freed heap storage. This is the one place in these
   cells where the rule text, taken literally, does not clearly reject a
   use-after-free; it should be closed by making compiler-inserted release an
   invalidating event in Rule 3.
