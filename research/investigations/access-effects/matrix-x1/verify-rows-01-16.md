# Verification of `rows-01-16.md` against CANDIDATE-X1

Independent re-trace of every cell in
`matrix-x1/rows-01-16.md` against the frozen rule set, 2026-09-18. Each cell was
re-derived from the rule text alone; the original derivation was read only
afterwards to compare. Runtime performance is the only cost counted.

## Per-cell result

**1-1: agree** (`accepted-fine`). The closure predicate is structurally
recursive; Rule 1 clause 2 ("there is no type parameter, wrapper, or variant
payload through which a reference can be stored") rejects `Sneak` at any depth,
and `Node`/`Tree` recurse through ownership, not through a reference. Note: the
cell defers `Intrusive`'s per-hop price to 1-5 and 1-7 and calls it "no
per-operation cost"; after the 1-5 correction below, that deferred price is a
real per-hop cost, so 1-1's phrase should read "priced in 1-5/1-7 as real", not
"no per-operation cost".

**1-2: agree** (`rejected-real-cost`), with one under-counted cost. Rule 1
clause 1 rejects `DynBox<&Entry>` and Rule 2's line `&p // rejected: a reference
variable is not storage` rejects the reference-to-reference route, so the
multi-root target set must be rebuilt at the use site: +1 tag compare and +1
badly predicted branch per item. Under-counted: the cell writes that
`it.idx < len_of(v1)` "must be tested or carried by `fixup`'s `requires` from
the producer of the worklist". A `requires` is discharged *at the call site*,
and the call site's only knowledge of `it.idx` is a value loaded from `work`;
Rule 11 says "There are no quantified facts over array elements ('for all i
...')", so no fact can ride with a stored index. Every item therefore also pays
the Rule 7 test — one more compare and branch per item that the C++
`vector<Entry*>` form does not pay. Verdict unchanged.

**1-3: correction: `rejected-zero-cost` -> `rejected-real-cost`.** The cell's
own trace names the cost and then discards it: "The `push_nogrow` case costs one
load of the block pointer plus one `lea` per re-formation ... In a loop that
pushes every iteration this is one extra ALU op per iteration." The protocol is
binary — "A rewrite with no runtime cost is `rejected-zero-cost`; one with
unavoidable cost is `rejected-real-cost`" — and the load is unavoidable: Rule 3
invalidates on "any proper prefix of p's path is written ... by a call", and the
cell itself shows why no finer row escapes it (Rule 9's paths "continue through
fields, `*`, and whole-index or range positions supplied as arguments" — there
is no path naming the block header alone, which is exactly the observation the
cell makes). So a C++ program that hoists `Entry*` across a `push_back` into
reserved capacity keeps the address in a register while the candidate must
re-load the block base at every re-formation, and the loss grows to a full
dependent load whenever the invalidated reference ran through a `Box` stored in
the slot (`p = &(*v[i].child).value`). The `move b` half of the cell is
genuinely zero and should be kept as stated; the verdict must follow the
strongest program, which is the call case.

**1-4: agree** (`rejected-real-cost`). Rule 4's single sentence ("cannot be
assigned into any aggregate (Rule 1), cannot be returned, and cannot be captured
by a function value that is stored or returned") kills all three wanted forms,
and the re-walk of form (b) is a second dependent-load descent — a real cost,
correctly identified as the price of a position that must survive unrelated
work. Two pseudocode slips that do not move the verdict: `n = &root` in both
programs forms a reference to a reference parameter, which Rule 2's own line
rejects (it should be the rebinding `n = root`, permitted by "A reference may be
rebound"). Also worth noting for P4 specifically: a cursor over a *growing
vector* (P4's literal shape) is nearly free here, because an index survives
`Vector::push` where a pointer does not; the cell's Box-linked tree is the
stronger case and is the right one to derive.

**1-5: correction: `rejected-zero-cost` -> `rejected-real-cost`.** The rewrite
the cell prices is the pool-index link, and the cell's own summary line names a
cost ("index links cost one `lea` per hop"). More decisively, the index it
introduces is a *data-loaded* index, so Rule 7 applies to every hop — "Every
index must be proved in bounds; when the proof is unavailable the program tests
the measure" — and Rule 11 supplies no fact for it ("There are no quantified
facts over array elements"). That is precisely the per-hop compare, predicted
branch and unprovable error arm that 1-7 calls "small but unavoidable". Two
cells deriving the same rewrite cannot disagree on whether it has a cost, and
1-4's form (c) prices the same hop as "one compare and one predicted branch per
use". The footprint saving (8 bytes per doubly linked node) is a prediction
about cache behaviour, not a derived zero; under PROGRAMS.md's rule that prose
design costs are "predictions with explicit grounds", it cannot cancel a named
instruction. The rest of the cell is correct: cyclic owning shapes cannot be
written down (affinity, not a rejecting clause), and Rule 5's positive half is
quoted accurately.

**1-6: agree** (`accepted-fine`) on the verdict, with an invented operation in
the program. `move_range(&nb, &v, 0, len_of(v))` is not in the candidate, and it
is not a harmless spelling: it would leave `v` with `len_of` unchanged over
moved-from slots, and the following `v = move nb` would then release those slots
under Rule 6's "At scope exit the compiler releases slots `[0, len_of)`
recursively and frees the block" — a double release. Rule 6 also forecloses the
element-by-element spelling: "There is no `take` operation and no partial move
out of any place: the only ways to move a value out of storage are consuming a
whole local (`move x`), the window operations, and the atomic update", and the
window operations are LIFO (`pop`) or scrambling (`swap_remove`), so neither
preserves order. The derivable spelling is the one the rule sentence already
describes — consume the whole `DynBox` local and return a larger one, with the
byte relocation licensed by Rule 1's consequence clause ("any value can be
relocated by copying its bytes (memmove, realloc)"). That yields exactly the
cost the cell claims (one memmove, no per-element move-construction or
teardown), so the verdict stands and only the program needs rewriting. The SSO
half is right: Rule 1 rejects the self-pointing form, the tagged form costs one
predicted branch per `data()` against libstdc++ and zero against libc++.

**1-7: agree** (`rejected-real-cost`). Rule 7's "when the proof is unavailable
the program tests the measure" plus Rule 11's exclusion of quantified facts give
exactly one compare and one predicted branch per hop and an error arm that
cannot be proved unreachable. The mask observation is also correct against the
text: Rule 11's facts are "affine comparisons over measures and integer values",
and `i & (cap-1)` is not an affine expression. This is the cell that owns the
per-hop index cost for the whole row.

**1-8: agree** (`undecided-rule-gap`), and the gap survives a second attack.
Rule 8's "the compiler never releases them" does rule out the reading in which
`truncate` releases linear elements — but that does not settle the case,
because Rule 6's `truncate` contract (`requires n <= len_of(buf); ensures
len_of(buf) == n`) then admits a call that puts seven `File`s outside the window
where Rule 6 says slots "hold nothing" and no operation can ever name them
again, which contradicts Rule 8's "Linear values must be consumed by an explicit
operation on every exit path". So the two readings are a contradiction, not a
mere silence, and the cell is right to stop. The `pop` loop is the correct
available form under either reading and is genuinely zero cost.

**1-9: agree** (`accepted-fine`). Rule 9 admits the eight member paths
(`fn update(o: &Obj, c: Bool) writes(o.a), writes(o.b) // member paths are
allowed`), Rule 10 clause 1's pairwise comparison is discharged by "different
roots" at the field level, and Rule 1 is genuinely what makes that conclusion
free and separately compilable. I additionally checked whether the element
writes inside the loop destroy the length facts under Rule 11's "A fact that
mentions a path is invalidated when that path is written"; they do not (see
dismissed gaps), so the kernel needs no per-iteration re-test and the cell's
zero-guard claim holds.

**1-10: agree on the verdict** (`rejected-zero-cost`) **and correction to one
sub-claim.** The headline derivation is right: Rule 9 admits `writes(g[a])`
because "an index enters an effect only through an argument", Rule 10 clause 1
then demands three distinctness facts that Rule 11 cannot supply, and the
rewrite into four ordinary statements is free because Rule 10 governs "a call"
and nothing compares two consecutive statements pairwise, while Rule 3 keeps a
reference valid across a write at or below its own path. The sub-claim to
withdraw is this one: "a bystander `p = &g[t]` survives the call for every `t`
... That is more permissive than it first looks". The rule text does not give
that. Clause 3 says the reference dies when its path "has a proper prefix among
the call's write paths" and never says how `g[m]` and `g[t]` are compared when
`m` and `t` are unproved; the only reading consistent with the same rule's
clause 1 ("indices or ranges proved distinct") is the conservative one, under
which the bystander *is* invalidated. Taken literally, the permissive reading
admits a read of freed storage (program given under "Accepted-unsafe program"
below). The insert rewrite does not depend on clause 3, so the cell's verdict
and cost stand; the clause-3 paragraph should be replaced by the gap entry.

**1-11: correction: `accepted-fine` -> `rejected-real-cost`.** The cell's own
strongest program is *rejected*: after `cols = pop(&pending)`, the call
`kernel(&cols, n)` cannot discharge its eight `requires`, and the cell says so.
The rewrite it then gives is a runtime guard costing "8 header loads, 8 compares
and 1 branch ... once per boundary crossing", plus an error arm
(`return Err(Ragged)`) that the program's own invariant forbids — the same
unprovable-arm defect 1-7 counts against itself. That cost is unavoidable under
the frozen text: a container of `Cols` cannot carry the relation, because Rule 11
states "There are no quantified facts over array elements ('for all i ...')", so
no `ensures` on `pop` can re-establish it. A verdict of `accepted-fine` would
mean nothing was rejected and nothing was paid, and both are false for the
strongest program. The cell's substance is otherwise correct and should be
restated as: rejected with a constant, hoistable real cost per opaque boundary,
zero when provenance is threaded through `build`'s `ensures`.

**1-12: agree** (`undecided-rule-gap`), with one missed candidate reading. The
conflict is real and correctly quoted: Rule 8's own `close(move c.f)` example
against Rule 6's "no partial move out of any place". The cell should also record
a third reading it did not consider: whole-local destructuring
(`Pair { in_f, out_f, n } = move c`) consumes the place `c` entirely and binds
its parts as separate locals, which satisfies both Rule 6's "consuming a whole
local (`move x`)" and Rule 12's "No place is ever partially moved" with no hole
anywhere. The rule file never states whether that form exists, so the cell stays
undecided — but if the owner grants it, the cell's fallback (split the aggregate,
or use parallel `DynBox`es) is unnecessary and the cost is zero rather than one
extra index computation per access. The drop-flag-free conditional release is
correctly derived and is a genuine win.

**1-13: agree** (`rejected-real-cost`), with the cost understated. Rule 11's
exclusion of quantified facts makes the injective-permutation scatter
unstatable, hence unprovable, hence rejected; "channels or atomics" are under
"Not in this candidate" so no synchronized form exists. One correction to
rewrite 1: the cell says the bucketed ranges' "disjointness is affine in the
prefix sums (Rule 11) and therefore accepted". Prefix sums are data loaded from
a block, and adjacency over the whole boundary array is again a quantified fact
over elements. What Rule 11 actually supplies is "refinement facts from a
dominating branch" for a *fixed* number of arms — i.e. a statically unrolled
`par` with O(k^2) runtime range comparisons in front of it. A `k`-way parallel
loop over data-determined boundaries is still rejected. So the bucketed rewrite
costs two extra passes, `n` extra `u32`, *and* either a fixed unrolled arm count
or the loss of the loop form. Verdict unchanged, cost larger.

**1-14: agree** (`accepted-fine`). Rule 14's example `par { a = build(&x)?; b =
build(&y)? } // both allocate: accepted` sanctions both the parallel allocation
and the `?` inside an arm; Rule 8's affine scope-exit release discharges P18's
"free nothing twice and leave earlier blocks owned" with no written cleanup and
no drop flag. The one branch per allocation for the `Result` and the absence of
in-place growth are correctly recorded as the real, small costs, and both are
paid by `std::vector`/`Vec` too.

**1-15: agree** (`rejected-zero-cost`). The returned-reference and returned-hole
forms die on Rule 4, Rule 15 and the "Not in this candidate" exclusions of slice
values and `take`/`put` holes; `ensures result < len_of(v)` then arrives with the
index so the caller pays no bound test, and re-forming `&v[i]` is one address
computation from a base that no intervening content write can disturb (Rule 3).
This is the one cell in the row where the index form is genuinely free, because
the fact travels with the value instead of being reconstructed. Nit: the closing
`push_nogrow(&v, x)` needs `len_of(v) < cap_of(v)`, which the fragment never
establishes; P1's `push` is the library `Vector::push`, which has no such
`requires`.

**1-16: agree** (`rejected-real-cost`). `writes(a.buf)` against `writes(a.buf[i])`
is Rule 10's own `bad(&vv, &vv[0])` shape, so allocation cannot overlap any other
pool access, and Rule 3 kills every outstanding reference at each `alloc`.
Rewrite 3's two-level handle costs one extra dependent load per hop, which is
the candidate's own recorded "one extra dependent memory access per hop". One
overstatement: rewrite 2 with per-arm free lists is "still rejected" only for the
general case. If the handles are popped *before* the `par` and a dominating
`if i != j` is written, a fixed-arm-count parallel update is accepted with one
compare; what stays rejected is the loop form and any arm that allocates inside
itself, which is the strongest program and so the verdict holds.

**16-16: agree** (`accepted-unsafe`). Rule 16 states the acceptance outright ("A
stale index that is still in bounds names the current occupant of that slot: a
logic error, not a memory error"), the memory-safety audit against Rules 6, 7, 8
and 12 is correct (no uninitialized read, no hole, no double consumption of a
linear value — `swap_remove` with a stale index removes the *wrong* value, never
the same one twice), and the load-dependent detection window after
`truncate(&a.buf, 0)` is a genuine product of Rule 16 composed with itself. The
generation-plus-epoch rewrite and its per-dereference cost are correctly priced,
including that a per-slot generation bump would make reset O(cap). The label is
right under the fixed vocabulary: accepted by the rules, memory-safe in the
candidate's own sense, identity-unsafe.

## Confirmed gaps

1. **1-8, `truncate` on a linear element type.** Quoted: "A DynBox whose element
   type is linear is itself linear (Rule 8) and must be truncated to zero by the
   program before it can go out of scope." Against Rule 8's "Linear values must
   be consumed by an explicit operation on every exit path; the compiler never
   releases them." Missing: whether `truncate(&bufs, n)` with `n < len_of(bufs)`
   on a linear element type is accepted, rejected, or releases the elements. The
   contract as written admits the call and no clause says what happens to slots
   `[n, len_of)`.

2. **1-12, moving one linear field out of an aggregate.** Quoted: "fn use_it(c:
   own Conn) { ...; close(move c.f) }   // consuming the struct as a whole and
   closing its File is required" against "There is no `take` operation and no
   partial move out of any place: the only ways to move a value out of storage
   are consuming a whole local (`move x`), the window operations, and the atomic
   update." Missing: whether `move c.f` consumes the field or the whole local,
   and whether whole-local destructuring exists. This decides whether a struct
   with two linear fields can be finished at all.

3. **1-10 (also 1-3), how indices are compared in the invalidation test.**
   Quoted: "A live reference outside the call whose path has a proper prefix
   among the call's write paths becomes invalid after the call (Rule 3). A
   reference that is itself an argument is the thing being accessed, not a
   bystander." Missing: whether `g[m]` counts as the prefix `g[t]` when `m` and
   `t` are not proved distinct. Rule 10 clause 1 fixes the comparison in the
   *disjointness* direction ("indices or ranges proved distinct") and Rule 13
   inherits that judgment, but no sentence fixes it in the *invalidation*
   direction. The permissive reading — the one the cell adopted — admits the
   unsafe program below; the conservative reading rejects it. A derivation cell
   cannot choose.

## Dismissed gaps

- **1-9/1-11, do element writes invalidate length facts?** Rule 11's "A fact
  that mentions a path is invalidated when that path is written (by statement or
  call)" could be read to kill `n <= len_of(c.a)` at every `c.a[i] = ...`, which
  would reject the P5 kernel outright. Dismissed: the path written is `c.a[i]`,
  not `c.a`, and Rule 3 fixes the same distinction explicitly for the analogous
  case — "Writing the storage at p's path or below it (a content write) does not
  invalidate p." The benign reading is the plain one; the cells were right not to
  raise it. (Raised by me, not by the deriver.)

- **1-6, the missing bulk-move primitive.** Rule 6's operation list has no
  order-preserving range move, which is why the deriver reached for
  `move_range`. Dismissed as a rule gap because Rule 6 itself grants the
  capability — "`Vector<T>` is library code: a DynBox plus a `push` that, when
  `len_of == cap_of`, allocates a larger DynBox, moves `[0, len_of)` across, and
  replaces the old one" — and a spelling exists inside the vocabulary (consume
  the whole `DynBox` local, return a larger one). Only the program in the cell
  needs fixing, not the rule set.

## Confirmed real costs

| Cell | Cost, against the C++/Rust form |
|---|---|
| 1-2 | +1 tag compare and +1 mispredictable branch per worklist item (multi-root), **plus** +1 bound compare and branch per item, since Rule 11 lets no fact travel with a stored index. |
| 1-3 (corrected) | One non-hoistable reload of the block base per re-formation after any call declaring a write on the container; a full dependent load when the reference ran through a `Box` held in the slot. Zero for the `move` case. |
| 1-4 | A second full dependent-load descent for the re-walk form; one bound compare per use for the pool-handle form. |
| 1-5 (corrected) | Per hop of an index link: one bound compare and one predicted branch (Rule 7 with no Rule 11 fact), plus an unprovable error arm; address arithmetic usually folds into the addressing mode. Footprint saving is a prediction, not an offset. |
| 1-6 | One predicted branch per small-string `data()` versus libstdc++'s self-pointing form; zero versus libc++, and a memmove win on every growth. |
| 1-7 | One compare and one predicted branch per hop on any data-determined index, an unprovable error arm, and lost vectorization of gather loops. |
| 1-11 (corrected) | 8 header loads, 8 compares and 1 branch per opaque boundary crossing, plus an unprovable `Err(Ragged)` arm; zero when the facts are threaded from `build`'s `ensures`. |
| 1-13 | Two extra full passes over `n` and `n` extra `u32` for the bucketed scatter, **plus** either a statically unrolled arm count with O(k^2) runtime range tests or the loss of the `k`-way loop form. |
| 1-14 | One predicted branch per allocation for the `Result`; no in-place growth (equal to `std::vector`/`Vec`, a loss only against a reserve-then-commit arena). |
| 1-16 | One extra dependent load per hop for the two-level handle, or a serialized construction phase. |
| 16-16 | Two loads, two compares and one to two branches per dereference for generation plus epoch, and 12-byte handles instead of 8. |

## Accepted-unsafe program constructible in these cells

Under the literal reading of Rule 10 clause 3 that cell 1-10 adopts ("a bystander
`p = &g[t]` survives the call for every `t`"), this program is accepted and reads
freed storage:

```text
struct Node { child: Box<Data>, next: u32 }
g: DynBox<Node>

fn replace_child(g: &DynBox<Node>, m: u64, x: own Box<Data>)  writes(g[m])
    contract { requires m < len_of(g); }
{ g[m].child = move x }              // the old Box is released on assignment (Rule 8 affine)

t = read_index(&meta)                // data
m = read_index(&meta)                // data; nothing proves m != t
if t < len_of(g) && m < len_of(g) {
    p = &(*g[t].child).value         // proper prefixes: g, g[t], g[t].child, *g[t].child
    replace_child(&g, m, nb)         // substituted write path: g[m]
    use(p)                           // Rule 10.3: g[m] is not syntactically among p's
                                     // prefixes, so p is claimed to survive
}
```

When `m == t` at run time the call frees the heap object that `p` names and
`use(p)` reads it. Rule 3's "invalidated when any proper prefix of p's path is
written ... by a call" leaves the same question open, so neither rule text
rejects the program under the permissive reading. The conservative reading —
paths overlap unless their indices are *proved* distinct, exactly as Rule 10
clause 1 requires for its own comparison — invalidates `p` and rejects `use(p)`,
at the cost of re-forming `p` after the call (the 1-3 cost). This is the gap that
must be closed before any cell relies on clause 3.

Separately, 16-16's `accepted-unsafe` is confirmed as derived: it is identity
confusion admitted by Rule 16 on purpose, not a memory-safety break, and the only
check that fires (the Rule 7 bound test) stops firing once pass 2 refills the
window.
