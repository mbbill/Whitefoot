# Verification of `matrix-x1/rows-08-09.md` against CANDIDATE-X1.md

Independent re-trace of every cell in row 8 (value classes) and row 9 (effect
rows) against the frozen rule file, and only that file. Four verdicts change.

## Per-cell findings

**8-8: agree.** `undecided-rule-gap` is right, but the gap is wider than the cell
states and its decisive sentence is quoted only in truncated form. The cell quotes
Rule 6 as "There is no `take` operation and no partial move out of any place" and
stops at the colon. The rest of that sentence is a closed enumeration: "the only
ways to move a value out of storage are consuming a whole local (`move x`), the
window operations, and the atomic update." `move p.a` is none of the three, and
neither is `move c.f` in Rule 8's own example line, nor `move n.f` in the cell's
*ordinary* program, nor the depth-one destructuring `t = move c.a` the cell
proposes as the general technique. So the cell's claim that "the ordinary program
needs nothing more" and that only the two-linear-part case is open is wrong: every
program in this cell that moves a linear field out of an owned local turns on the
same unranked pair of sentences. What keeps this an ambiguity rather than a
decided rejection (contrast 9-12 below) is that Rule 8's contradicting text is not
a throwaway illustration with a provably false obligation — it is that rule's
statement about its own declared type, with prose asserting a requirement
("consuming the struct as a whole and closing its File is required"). Under the
closed-enumeration reading Rule 8's declared `struct Conn { f: File, n: u64 }` is
a type whose `File` no program can ever close. Two coherent readings, no ranking.
The rewrite and the cost the cell gives (a window of linear parts, one relocation
of `sizeof(T)` plus one header store per element against a C++ in-place
destructor, plus the block header) are correct and confirmed below.

**8-9: agree.** `accepted-fine` is right and every step checks. `writes(pool[k])`
is well formed — Rule 9 admits "whole-index or range positions supplied as
arguments" and forbids only an index *expression* in the signature; the body's
single effect is the declared path, satisfying "every statement's effect and every
callee's substituted row must be covered by the declared row"; and Rule 6 forces
the atomic-update shape ("where it is linear the assignment is rejected unless
written as the atomic update whose function consumes the old value"). The cell's
dismissal of the `fn steal(p: &File) writes(p)` reading is correct and is in fact
doubly grounded: besides Rule 12's "every path is either wholly present or the
program cannot name it", Rule 6's closed enumeration excludes a callee move-out of
a caller's place outright. Worth recording that this is the same enumeration whose
literal force leaves 8-8 open — it does real normative work here, which is why
8-8's gap cannot be waved away. One over-reading that does not move the verdict:
the file gives `swap_remove` only a length `ensures` and never says it refills
slot `k` from the last slot; the cell's conclusion (a pool cannot use it) follows
anyway, because the contract says nothing about where any element ends up.

**8-10: correction: rejected-zero-cost -> undecided-rule-gap.** The three clause-2
rejections are traced correctly, and two of them (`push_nogrow(&buf, buf[0])`,
`transfer(&p, &p)`) do have the zero-cost rewrites claimed. The strongest one does
not: the rewrite the cell offers is itself rejected. It writes
`head.next = Some(move Box::new(n)?)` where `Node` is linear by containment, so
`head.next : Option<Box<Node>>` is a linear place, and Rule 8 says "Linear values
must be consumed by an explicit operation on every exit path; the compiler never
releases them" — an assignment that discards the old occupant is exactly the
compiler releasing it, and Rule 6 states the same prohibition for the one place it
spells out ("where it is linear the assignment is rejected unless written as the
atomic update whose function consumes the old value"). The cell's own 8-16 makes
the point against it: "`Option<Conn>` stays linear whether or not its occupancy is
`None`: Rule 8 classifies types, not values." For an affine `Node` the same line
is worse than rejected — it silently releases the whole tail of the list. The only
repair that keeps a `Box` chain is to move the old link out first (`t = move
head.next`), which is precisely the partial-move question 8-8 leaves unranked. If
that resolves permissively the splice is zero cost; if it resolves against partial
moves, the task needs a representation change to nodes in a window with `u64`
links, and that is not free: a growing node array pays Rule 6's whole-window
relocation on each growth (a `Box` chain never relocates), and the alternative
`DynBox<Box<Node>>` pays one extra dependent load per hop — the candidate's own
recorded cost, "Structures needing a header-plus-tail layout pay one extra
dependent memory access per hop." Zero cost is therefore not established, and
which cost applies is decided by the 8-8 gap.

**8-11: agree.** `accepted-fine` is right. Rule 11's fact list genuinely contains
no ownership form, Rule 11's own next sentence forecloses the workaround ("There
are no quantified facts over array elements ... occupancy that is determined by
data is stored as data"), and Rule 12 names the `Option` representation. The
`match h { ... }` consumes `h` wholly, so nothing here depends on 8-8's gap. The
invalidation of `len_of(buf) == n` across `holder.data = move buf` is right for
the reason the cell gives and also because the fact's path is simply no longer
nameable. One contract defect, zero cost to fix: `ensures len_of(h.data) ==
len_of(b)` names a by-value parameter that the call consumed; Rule 11 provides
`entry(p)` for the pre-state and the clause should use it.

**8-12: correction: accepted-fine -> rejected-zero-cost.** The trace is right and
the cell states its own conclusion plainly — after the join "neither path is
wholly present on every incoming edge and neither can be named. There is nothing
for the program to call `survivor`" — so the strongest program is *rejected*.
Protocol item 4 then fixes the label: "For every rejection, give the best rewrite
under these rules and its runtime cost ... A rewrite with no runtime cost is
`rejected-zero-cost`." The arm-local rewrite is exactly that: the branch on `c`
already existed, each arm runs the same `use` and the same two `close` calls, and
protocol item 7 excludes the duplication. The cell even records "Task achievable:
rewrite", which is the rewrite line, not an acceptance. The same file applies
`rejected-zero-cost` to this shape in 8-10, 8-13 and 9-10; 8-12 is the outlier.
Everything else in the cell stands, including the second-strongest loop program
(one nit: `h` is initialised to `Some` and never set to `None`, so the `None` arm
of the final match is dead — harmless) and the drain-loop cost, which is the 8-8
window cost, not a cost of the join treatment.

**8-13: agree.** `rejected-zero-cost` is right — two arms each carrying
`writes(buf)` cannot meet Rule 10 clause 1's "different roots, or indices or
ranges proved distinct", and the cell's note that the rejection is substantive
(two concurrent `pop`s would race the header and could hand one linear value to
both arms, against Rule 8's "consumed at most once") is correct. But the cost
paragraph rests on a false claim: "Moving a linear value out of a slot needs a
window operation ... so no range-based parallel drain exists." The cell quotes the
enumeration in full and then drops its third member. The atomic update *is* a way
to move a value out of storage, it works on an index into a range reference (Rule
7: "a range reference into either" is indexable, `part[k]` requires `k <
len_of(part)`), and Rule 9 admits a range path "supplied as arguments". So a
single existing block of `Option<Conn>` is drained in parallel:

```text
fn drain_range(v: &DynBox<Option<Conn>>, lo: u64, hi: u64)   writes(v[lo..hi])
    contract { requires lo <= hi; requires hi <= len_of(v); }
{ for k in lo..hi { v[k] = drop_conn(v[k]) } }          // Rule 6 atomic update

par { drain_range(&buf, 0, mid);  drain_range(&buf, mid, n) }   // ranges proved distinct
```

which is Rule 13's own accepted kernel shape, and the real work (`finish`) runs in
parallel with no redistribution at all; only the empty shells remain for a cheap
sequential window teardown. The consequence is that the cell's headline cost —
"the x1 program must decide the number of blocks when the pool is built ... cannot
be rebalanced without moving elements between blocks at one relocation of
`sizeof(Conn)` each" — is not forced, and "Draining a single existing block in
parallel, without such a rebuild, is not expressible at all" is wrong. The verdict
is unaffected and in fact strengthened.

**8-14: agree.** `accepted-unsafe` is right and this is the sharpest finding in
the file. Rule 5 fixes the failure payload as `Result<Box<Node>, Oom>`, so the
argument is not handed back; Rule 9 records the consumption at the call, so Rule
8's "consumed by an explicit operation on every exit path" is formally satisfied;
and the `File` is gone with nothing having closed it. The rules admit a program
that defeats the guarantee linearity exists for, on an arm that Rule 14 gives
every allocation. One defect in the *ordinary* program that the cell missed, and
it is a rejection, not a cost: `cell = DynBox::new<Conn>(1)?` returns early on
`Err` while `f` is still live and unconsumed, so Rule 8's every-exit-path
obligation fails and the function as written does not compile. The repair is to
match instead of `?` and `close(move f)` on the failure arm; it costs one call on
a path that must do that work anyway, so the cell's 16-bytes-per-cell cost is
unchanged.

**8-15: agree.** `accepted-fine` is right, and the argument that a stage which may
consume its resource cannot be written in-out is correct on three independent
grounds the cell names (Rule 8's never-release, Rule 6's linear-assignment
sentence, Rule 12's no-hole sentence). One assumption has to be withdrawn: the
cell leans on "Rule 14's 'Addresses are not observable' leaves the ABI free to
pass and return a large owned value through a hidden pointer, so a compiler may
lower the whole chain to zero copies", and PROGRAMS.md forbids exactly that
premise — "Large-aggregate ownership round trips have no assumed zero-copy ABI."
The cost is therefore the figure the cell gives conditionally and must give
outright: two relocations of `sizeof(Conn)` per stage against a C++ form that
mutates through `Conn&` and returns an enum. That is recorded as a real cost
below; it does not change the verdict, since nothing is rejected and the
vocabulary has no accepted-with-cost label (compare 6-15 and 9-11).

**8-16: correction: accepted-fine -> rejected-real-cost.** The pool pattern is
right in outline and the `truncate`-over-linear reading the cell adopts is
actually forced, not chosen (see dismissed gaps). Two other things are wrong, and
the first is a rejection. `acquire` declares `ensures result < len_of(p.slots)`
and its body obtains `k = pop(&p.free)`; nothing bounds a value read out of the
free list. Rule 11 forecloses the fact that would: "There are no quantified facts
over array elements ('for all i ...') and no per-slot occupancy facts" — which is
exactly a statement about every element of `p.free`. So neither the `ensures` nor
the `p.slots[k] = ...` update's own `k < len_of(p.slots)` obligation is
discharged, and the cell's assertion that "`k < len_of(p.slots)` [is an] affine
comparison over measures" confuses the *form* of the fact with its *ground*. The
program as written is rejected. The repair is a test and an outcome arm at every
`acquire` — `if k < len_of(p.slots) { ... } else { ... }`, making `acquire` return
`Option<u64>` — which costs one compare, one branch and a dead arm per acquisition
against a C++ free-list pool that has none, and no rewrite inside the rules
removes it (scanning for a free slot is O(n); a masked index needs a fact form
Rule 11 does not list; an arena that never reuses answers a different question
than this cell's). That is the definition of `rejected-real-cost`. Second, the
cell's claim that the precise row "is what lets two releases on distinct handles
run in one `par` block (Rule 13)" is false: `release`'s row also carries
`writes(p.free)`, so two releases substitute `writes(p.free)` against
`writes(p.free)` — one root, nothing to prove distinct — and Rule 13 rejects the
pair however distinct `k1` and `k2` are. Releases serialise on the free list.
Minor, and accepted: `fill(p.slots[k], move c)` is a two-argument atomic update
where Rule 6 writes `f(buf[k])`; the governing sentence asks only for "the atomic
update whose function consumes the old value", which it does.

**9-9: agree.** `accepted-fine` is right: the path grammar admits both rows, the
form restriction is about index *expressions* in a signature and not about
indices, the body-covering paragraph is applied correctly, and the conclusion that
precision is a per-function choice bought at no runtime cost follows. Two defects
that do not move it. First, `swap_two`'s body is rejected as written: `v[i] =
const_of(v[i], v[j])` is one statement whose write path is `v[i]` and whose
by-value copy argument reads `v[j]`, and Rule 10 clause 2 puts that read into the
comparison while clause 1 demands "indices or ranges proved distinct" — which is
unavailable inside the body, whose contract says only `i < len_of(v)` and `j <
len_of(v)`. The fix is one contract line, `requires i != j`, at zero runtime cost,
and it reaches the same conclusion about `swap_two(&v, p, p)`. Second, the
recursion example is not expressible: `match n.child { Some(b) => walk(&(*b), d +
1) ... }` inspects an `Option<Box<Node>>` reached through a reference parameter,
and Rule 2's path list is closed — "continues through fields, `*` (Box content),
`[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7)" — with no variant-payload
component and no by-reference match form anywhere in the file. That is recorded as
a confirmed gap below, because it also governs Rule 12's prescribed
`DynBox<Option<Entry>>` representation.

**9-10: agree.** `rejected-zero-cost` is right. Rule 2's "every check on it must
hold for every member of the set" plus clause 1 rejects `merge(w, &v1)`; branch
duplication gives each arm a singleton target set and materialises nothing,
because "A reference variable names a path; it is not storage of its own"; the
nested-call case is clause 2 read together with clause 1, and sequencing removes
the comparison because Rule 10 compares the effects of one call. I looked for a
join the rewrite cannot reach — a loop-carried double buffer, where `cur` and
`nxt` each name `{a, b}` at the loop header and `step(cur, nxt)` is rejected — and
unrolling the loop by two (`step(&a,&b); step(&b,&a)`) recovers it with the same
instruction count, so the zero-cost claim survives the stronger program too. The
C++ analogue in the cost paragraph has its pop and push in the wrong order; that
is prose, not the derivation.

**9-11: agree.** `accepted-fine` is right, and the load-bearing step is sound on
the rule text alone: Rule 11 invalidates a fact "when that path is written", the
fact's path is `v`, and a call whose substituted row is `writes(v[k])` writes the
path `v[k]`. Rule 9 confirms the written path is the argument's, and Rule 6
independently keeps `len_of` intact ("changed only by the built-in operations
below", which an element write is not). Importing Rule 10 clause 1's overlap
judgment into Rule 11 would be inventing a rule, since clause 1 is scoped to "the
substituted effects" of one call. The coarse-row cost the cell prices — one
dependent header load per iteration that cannot be hoisted across an opaque call —
is real but, as the cell says, entirely removed by writing the `ensures`.

**9-12: correction: undecided-rule-gap -> rejected-zero-cost.** There is no
ambiguity to be undecided about. Rule 6 states a normative obligation in its own
voice: "Reading `buf[k]` or forming `&buf[k]` requires the fact `k < len_of(buf)`
(Rule 7)", and Rule 7 repeats it. For the example's index the obligation is
`len_of(buf) < len_of(buf)`, which is false — not unclear, false. A comment on an
illustrative call cannot discharge an obligation that the rule it illustrates
states as unsatisfiable, so the line is a defect in the example and
`&buf[len_of(buf)]` is rejected. Nothing else is in tension: Rule 9's actual
clause, that a row names no index and an index "enters an effect only through an
argument, evaluated once at the call", is fully illustrated by any in-bounds slot.
The cell's own third reading is not needed either. The cell then supplies the
zero-cost rewrite itself — `push_nogrow(&buf, 3)`, "identical object code to what
the `put_at` form would have produced plus the increment that form omits" — so the
verdict is the zero-cost rejection. "Rules consistent: no" stands as an
observation about the file. This matches the disposition already recorded for the
same line in `verify-rows-06-11.md` under cell 6-9.

**9-13: agree.** `rejected-real-cost` is right and is the one place in these two
rows where the rules cost the task something no rewriting recovers. Rule 13
borrows Rule 10's judgment; the distinctness wanted is `perm[i] != perm[j]` over
all pairs; Rule 11 excludes it in terms ("There are no quantified facts over array
elements"); widening the row to `writes(out)` rejects the pair outright; atomics
are under "Not in this candidate". The cell is also right about what is *not*
lost — the same-index map and the gather both parallelise — which is the correct
narrow statement of the loss. The partition rewrite's cost (one extra full pass,
an `n`-element scratch block, roughly threefold memory traffic on a one-load
one-store kernel) is confirmed.

**9-14: agree.** `accepted-fine` is right. The separation is the right one: Rule
14's exemption covers the allocator's own state ("Allocation and release carry no
effect entry and never make two parallel arms conflict"), Rule 9 governs the named
path, and the enumeration of the three places a release can occur is complete
under these rules. `par { drop_block(&h.a); read_block(&h.a) }` is decided by Rule
9 plus Rule 13, exactly as the file's own `par { push(&v, 1); stats(&v) }` line.
The closing observation — that the exemption is sound only because Rule 4 keeps a
reference from escaping the function that formed it, so no arm can name storage
another arm releases — is correct and worth keeping. Two nits: the rule file says
nothing about control flow leaving a `par` arm, which `push(&v1, a)?` does; and
after a `?` there is no join, so the Ok relation simply holds rather than being
intersected with the Err one.

**9-15: agree.** `accepted-fine` is right. The index crosses the boundary as owned
data (Rule 15, Rule 4), Rule 9 consumes it as an argument-supplied index, the two
row entries are trivially disjoint by "different roots", and `probe`'s `ensures`
discharges `occupy`'s `requires` as an affine comparison over measures, so no
compare reaches the object code. The cost comparison is right, including that
Rust's `entry` is unavailable here because Rule 4 forbids the borrow it holds. One
undischarged obligation, zero cost to fix: `probe` ensures only `result <
len_of(t.tags)` while `occupy` and `revise` require `k < len_of(t.slots)`, and no
fact relates the two lengths; `probe` reads `t.slots` and can simply also ensure
`result < len_of(t.slots)`.

**9-16: agree.** `accepted-fine` is right, and the key step is correct: `alloc`'s
`ensures result == len_of(deref(entry(a.buf)))` together with `ensures len_of(a.buf)
== result + 1` makes successive handles provably distinct by affine comparison,
which is what Rule 10 clause 1 and Rule 13 ask for, and the same `ensures` is what
keeps `i1 < len_of(a.buf)` alive across later allocations under Rule 11's "unless
the callee's `ensures` re-establishes it". The rejection of `par { fill; alloc }`
is correct and correctly attributed to Rule 9's "Signatures never contain index
expressions" rather than to any conservatism, and Rule 3's prefix invalidation
correctly supersedes Rule 16's looser "valid until `a.buf` is replaced". One
scope note that does not change the verdict: the cell renders P2 over a uniform
`Arena<Block>`, while P2 asks for blocks of 64, 64, 64 and 32 bytes carved from
one reservation. A `DynBox<T>` has uniform typed slots and the candidate offers no
way to give bytes a type, so the variable-size half of the allocator task is not
covered by this cell's program; it is a different question from the one the cell
asks, but the cell should not be read as having derived P2 in full.

## Confirmed gaps

1. **Moving a linear or affine part out of an owned aggregate** (decides 8-8,
   decides the cost in 8-10). Rule 6: "There is no `take` operation and no partial
   move out of any place: the only ways to move a value out of storage are
   consuming a whole local (`move x`), the window operations, and the atomic
   update." Rule 12: "No place is ever partially moved: every path is either
   wholly present or the program cannot name it." Against Rule 8's own line for
   its own declared type: "`fn use_it(c: own Conn) { ...; close(move c.f) }` //
   consuming the struct as a whole and closing its File is required." Under the
   first two, `move c.f` is not one of the three admitted move-out forms and
   `struct Conn { f: File, n: u64 }` is a type whose `File` can never be closed;
   under the third, one field may be moved out of an owned local. The file states
   both and ranks neither. This is not the 9-12 situation: neither reading states
   a provably false obligation.

2. **Inspecting an enum that is reached through a reference.** Rule 2: "A path
   starts at a local variable or a parameter and continues through fields, `*`
   (Box content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7)." The list
   is closed and has no variant-payload component, and no rule anywhere states a
   by-reference match form; every `match` in the file is on an owned value. Yet
   Rule 12 prescribes `slots: DynBox<Option<Entry>>` for "non-Copy payloads" and
   8-16 assumes "a consumer that needs it branches on the `Option`". If the path
   list governs, a non-Copy `Option` in a slot or a field can never be read or
   branched on in place — only consumed and re-committed by an atomic update, so
   every read of such a slot becomes a full read-modify-write and a
   `Option<Box<Node>>` tree cannot be traversed at all. If a by-reference match is
   implied, no sentence says what it may bind. Load-bearing for 9-9's recursion
   example and for the representation Rule 12 recommends.

## Dismissed gaps

1. **9-12's `put_at(&buf[len_of(buf)], 3)`.** Covered by Rule 6's "Reading
   `buf[k]` or forming `&buf[k]` requires the fact `k < len_of(buf)` (Rule 7)".
   The obligation here is `len_of(buf) < len_of(buf)`, which is false, so the call
   is rejected and the append is `push_nogrow`. An example comment does not rank
   against the normative sentence of the rule it illustrates.

2. **8-9's "moving out of" in a `writes` entry.** The cell resolved it correctly
   and it is covered twice over: Rule 12's "every path is either wholly present or
   the program cannot name it" bars a callee from leaving a hole in a place the
   caller still names, and Rule 6's closed enumeration does not include a callee
   move-out of a reference parameter's target. No ambiguity survives.

3. **`truncate` over linear elements (8-16's aside).** Covered by Rule 8's "the
   compiler never releases them". `truncate` is a built-in; dropping linear
   elements out of the window with no consumer is the compiler releasing them, so
   the cell's reading is forced rather than chosen. Rule 6's "must be truncated to
   zero by the program before it can go out of scope" is then loose wording for
   "must reach `len_of == 0`", which `pop` achieves. The residual literal reading
   is recorded as an accepted-unsafe construction below rather than as a gap.

## Confirmed real costs

- **Linear teardown through a window (8-8, 8-12, 8-13).** One relocation of
  `sizeof(T)` out of the slot plus one header store per element, against a C++
  destructor loop that reads each element in place; plus the block's 16-byte
  header and one dependent load to the payload. For a tree of `n` 24-byte nodes,
  `24n` bytes of extra copy traffic per teardown.
- **A linear value crossing a fallible allocation (8-14).** The allocate-then-
  populate rewrite costs 16 bytes of `DynBox` header per cell that a `Box` would
  hold headerless, and one allocation per cell either way; `16n` bytes of extra
  footprint for `n` linear nodes. Avoidable only by specifying `Box::new`'s `Err`
  arm to return the argument.
- **Pool acquisition through a free list (8-16).** One compare, one branch and an
  outcome arm per `acquire`, because Rule 11's "There are no quantified facts over
  array elements" leaves a value read out of the free list unbounded. Plus:
  releases on distinct handles cannot run in one `par` block, since both write
  `p.free`.
- **Resource pipeline hand-off (8-15).** Two relocations of `sizeof(Conn)` per
  stage against a C++ stage that mutates through `Conn&` and returns an enum,
  since PROGRAMS.md rules out assuming a zero-copy ABI for large owned round
  trips. The `Result` discriminant itself is one register and free.
- **Parallel data-indexed scatter (9-13).** Either the whole parallel speedup, or
  a bucketing pre-pass costing one extra full pass over `n`, an `n`-element
  scratch allocation and roughly threefold memory traffic on a one-load one-store
  kernel. No rewrite inside the candidate recovers it.
- **Arena allocation against filling (9-16).** Allocation serialises against every
  fill in the same arena, so an algorithm that interleaves allocation with
  data-dependent work must restructure into rounds at one extra pass over the work
  list per round; and a traversal that allocates reloads the base each hop instead
  of keeping an address in a register.
- **Growing a linked structure (8-10), conditional on confirmed gap 1.** If
  partial moves are rejected, either Rule 6's whole-window relocation on each
  growth of a node array — which a `Box` chain never pays — or one extra dependent
  load per hop for `DynBox<Box<Node>>`.
- **Reading a non-Copy `Option` slot (confirmed gap 2), if the closed path list
  governs.** Every read becomes an atomic update, i.e. a store of the whole
  `Option<Entry>` back into the slot, where C++ reads through a pointer.

## Accepted-unsafe programs constructible in these cells

1. **8-14's own program, and it is correctly labelled.**

   ```text
   fn install(f: own File, n: u64) -> Result<Box<Conn>, Oom>
   { b = Box::new(Conn { f: move f, n: n })?;  Ok(b) }
   ```

   Rule 5 fixes the failure payload as `Oom`, Rule 9 records the `move` as the
   explicit consuming operation that satisfies Rule 8, and on the `Err` arm the
   `File` is gone unclosed with no program point able to name it. A resource-
   obligation leak on an arm Rule 14 gives every allocation, not memory
   corruption — but it is the failure linearity exists to prevent.

2. **The literal reading of Rule 6's truncate sentence (dismissed gap 3).**

   ```text
   p: Pool                             // slots: DynBox<Option<Conn>>, Conn linear
   ...                                 // n live Conns acquired
   truncate(&p.slots, 0)               // "must be truncated to zero by the program"
   ```

   Every `Conn` leaves the window without reaching `close`, and the block is freed
   at scope exit. Rule 8's "the compiler never releases them" is the sentence that
   rejects it, which is why the gap is dismissed for 8-16's verdict and recorded
   only here. Same construction as the one recorded in `verify-rows-06-11.md`.

3. **Nothing in these cells reads freed or relocated storage.** Every route I
   tried is closed by Rule 3 together with Rule 10 clause 3: `pop`, `swap_remove`,
   `truncate`, `push_nogrow`, `alloc` and a `Vector` growth all declare a write of
   the buffer path, which is a proper prefix of any reference formed into it, so
   "A live reference outside the call whose path has a proper prefix among the
   call's write paths becomes invalid after the call". 8-13's `&pools[w]` is
   exempt only as the accessed argument, not as a surviving bystander, and 9-16
   rejects `use(p)` after an intervening `alloc` on exactly this ground.
