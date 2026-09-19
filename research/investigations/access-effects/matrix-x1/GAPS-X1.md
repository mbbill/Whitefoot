# Gap and cost report for candidate x1

Date: 2026-09-18. Source: the 8 derivation row files, their 8 verification files, and the
8 engineering task files in this directory, judged against the frozen rule text in
`../CANDIDATE-X1.md`. Every judgment below was made by re-reading the rule file, not by
trusting the cell that reported it.

Last round most reported "gaps" turned out to be artifacts of the prompt rather than holes in
the rule text. This report therefore separates two lists that were previously mixed: **real
omissions** (Part 1), where no sentence in the rule file decides the case, and **decided
questions** (Part 2), where the rule file does decide and the cell that reported a gap was
wrong. Ten reported gap groups fell into Part 2. Thirteen survived into Part 1.

---

## Verdict totals over all 136 cells

Using the verifiers' corrected verdicts.

| Verdict | Cells | Share |
|---|---|---|
| `accepted-fine` | 44 | 32% |
| `rejected-zero-cost` | 28 | 21% |
| `rejected-real-cost` | 40 | 29% |
| `undecided-rule-gap` | 19 | 14% |
| `accepted-unsafe` | 5 | 4% |

Per file: rows 1/16 — 4 fine, 2 zero, 8 real, 2 undecided, 1 unsafe. Rows 2/15 — 2, 6, 5, 4, 0.
Rows 3/14 — 6, 3, 3, 4, 1. Rows 4/13 — 4, 2, 5, 4, 2. Rows 5/12 — 7, 5, 3, 2, 0.
Rows 6/11 — 9, 1, 7, 0, 0. Rows 7/10 — 4, 5, 7, 1, 0. Rows 8/9 — 8, 4, 2, 2, 1.

Half the rule set is clean: rows 6/11 and 5/12 produced no undecided cell at all, and 53% of
all cells are either accepted or rejected at zero runtime cost. The 19 undecided cells are not
spread evenly — **eight of them are the same question** (Gap 1 below), and once the thirteen
gaps in Part 1 are closed, no cell remains undecided.

Task-level results: 8 tasks, all achievable. Two at zero cost (array kernel; generic algorithms
at the boundary mechanism), five with cost, one — the tree rewrite — undecided in the shape the
task literally names because of Gap 1 and Gap 8.

---

# Part 1 — Real omissions

Thirteen groups. For each: what a reader cannot answer from the text, the strongest program
that turns on it, the rule sentences that collide or fall silent, and the smallest fix.

---

## Gap 1. Whether a linear or affine field can be moved out of an owned local

**Cells:** 1-12, 2-8, 3-8, 4-5, 4-8, 5-5, 5-8, 8-8, 8-10.
**Tasks:** resource container (its G5), resource pipeline (its G3), tree rewrite (its G3),
generic algorithms (the element-exchange counterexample).
**Covered by the rule text: NO.** The file states both sides and ranks neither.

Rule 6 closes the list:

> There is no `take` operation and no partial move out of any place: the only ways to move a
> value out of storage are consuming a whole local (`move x`), the window operations, and the
> atomic update.

Rule 12 repeats it:

> No place is ever partially moved: every path is either wholly present or the program cannot
> name it.

Rule 8 does it anyway, in its own example block:

> `fn use_it(c: own Conn) { ...; close(move c.f) }   // consuming the struct as a whole and
> closing its File is required`

and, one line later,

> `Box<File>                            // linear: must be taken apart and its File closed`

`move c.f` is not `move x` on a whole local, is not a window operation, and is not the atomic
update. Either Rule 6's enumeration is not exhaustive, or Rule 8 prescribes an operation the
candidate does not have.

```text
linear type File;  fn close(f: own File)

struct Pair { in_f: File, out_f: File, n: u64 }      // linear by containment (Rule 8)

fn finish(p: own Pair) {
    close(move p.in_f)        // Rule 8's own form. Under Rule 6 and Rule 12: rejected.
    close(move p.out_f)       // If the first line consumed the WHOLE local p, this names a
                              // dead local and is rejected under every reading.
}
```

The three readings are all live in the file and they give three different languages:

- **Strict (Rule 6 and Rule 12 govern).** `move c.f` is rejected. Then no operation anywhere
  reaches a linear part inside an aggregate, so `struct Conn { f: File, n: u64 }`,
  `Box<File>` and `DynBox<Conn>` are types whose resources no program can ever close — and
  Rule 8 declares them legal and obligatory to close. The resource pipeline's typestate stage
  ("a stage decides whether it kept the resource") is not derivable; `Vector<Conn>` and
  `Vector<Box<File>>` do not exist.
- **Field move (Rule 8's line governs).** One field leaves; the rest of the place stays
  nameable. That is exactly the `take`/`put` hole under "Not in this candidate", and Rule 12's
  no-hole sentence becomes false for this case.
- **Whole-local destructuring** (`Pair { in_f, out_f, n } = move p`). This satisfies both Rule
  6 (it consumes a whole local) and Rule 12 (nothing is left partially present), costs nothing,
  and handles `Pair` above. **It is not written anywhere in the rule file.** Cell 1-12's
  verifier flagged that every deriver missed it.

**Why this is the headline gap.** Eight of the nineteen undecided cells are this one question
reached from eight directions, and the answer decides whether a whole class of types is usable.
The strict reading also forces the container and tree costs in Part 3 (RC2).

**Minimal fix:** add whole-local destructuring to Rule 6's enumeration and delete or rewrite
Rule 8's `close(move c.f)` line. One sentence, zero runtime cost, and eight cells resolve.

---

## Gap 2. A compiler-inserted scope-exit release is not an invalidating event

**Cells:** constructed during verification of rows 4/13; the same shape threatens every cell
that forms a reference into a block-local `Box` or `DynBox`.
**Covered by the rule text: NO.**
**This is the only unqualified memory-safety hole in the rule file.**

Rule 3 lists the invalidating events and then qualifies the whole list:

> It is established when p is formed and invalidated when any proper prefix of p's path is
> written, moved out of, replaced, or freed, **by a statement or by a call**.

Rule 8 says the release is neither:

> at scope exit the compiler releases their memory recursively (Box, DynBox) and **runs no user
> code**.

```text
p: &Int
{
    b = Box::new(Node { value: 7, ... })?   // b is a block-local, affine (Rule 5, Rule 8)
    p = &(*b).value                          // Rule 2: p names the path *b . value
}                                            // block exit: the compiler frees b's heap object.
                                             // Not a statement. Not a call. Rule 3's list
                                             // is never triggered, so p is still "valid".
*out = *p                                    // accepted; reads freed heap storage
```

No other sentence closes it. Rule 4 permits the binding ("A reference may be bound to a local
... and used within the function that formed it"). Rule 2 requires only that a path "starts at
a local variable or a parameter" and says nothing about that local still being in scope. Rule
11's fact-invalidation clause speaks about writes, not releases.

**Minimal fix:** in Rule 3, replace "by a statement or by a call" with "by a statement, by a
call, or by the compiler's release at scope exit". One clause; no runtime cost; this is a
drafting slip, not a design question.

---

## Gap 3. Whether the prefix relation is on path syntax or on the storage the paths denote

**Cells:** 1-10, 3-9, 3-13, 5-6, and implicitly every cell that keeps a bystander reference
alive across a call (3-7, 3-10, 2-16, 7-7).
**Covered by the rule text: NO.** The file fixes index comparison in one direction and leaves
it open in the other.

Rule 10 clause 1, the disjointness direction, demands proof:

> Two effects on overlapping paths where at least one is a write must be proved disjoint
> (different roots, or **indices or ranges proved distinct**); otherwise the call is rejected.

Rule 3 and Rule 10 clause 3, the invalidation direction, demand nothing:

> invalidated when any **proper prefix** of p's path is written, moved out of, replaced, or
> freed

> A live reference outside the call whose path has a **proper prefix** among the call's write
> paths becomes invalid after the call.

```text
struct Node { value: Int, ... }
g: DynBox<Box<Node>>                                    // affine elements

fn replace_at(g: &DynBox<Box<Node>>, i: u64, x: own Box<Node>)  writes(g[i])
    contract { requires i < len_of(g); }
{ g[i] = move x }          // Rule 6: assigning over an affine value releases the old one.
                           // Rule 9: "writes covers ... freeing the storage at the path".

i = read_index(&meta)      // data
j = read_index(&meta)      // data; nothing proves i != j
if i < len_of(g) && j < len_of(g) {
    q = &(*(g[j])).value           // proper prefixes of q: { g, g[j], *(g[j]) }
    replace_at(&g, i, nb)          // substituted write path: g[i]. The heap Node that q
                                   // names is freed when i == j at run time.
    y = (*q).value                 // ACCEPTED under the syntactic reading: the text "g[i]"
                                   // is not among the three prefixes spelled above.
}
```

Rule 3's own example does not settle it. The example is

> `v.buf[j] = 5           // content write on a sibling or the same slot: p stays valid`

which is consistent with **both** readings, because Rule 3 separately declares that a write at
`p`'s own path does not invalidate `p`. The example never exercises a write to a *prefix* of
`p` through an unproved index, which is the case above.

The same question reappears one level up for ranges (cell 3-13): `buf[0..hi]` is neither
syntactically a prefix of `buf[3]` nor equal to it, yet in storage terms it lies strictly
above it. Rule 3 switches from path language in its first sentence to storage language in its
second ("Writing the storage at p's path or below it"), which is precisely the distinction at
issue.

**Minimal fix:** state in Rule 3 that two paths are compared by the storage they denote, with
indices and ranges treated as overlapping unless proved distinct — the same test Rule 10 clause
1 already writes. Cost of the conservative reading: the reference is re-formed (RC4), one
address computation. Cost of leaving it open: a use-after-free that every rule quotes as legal.

---

## Gap 4. When the index in a formed path is evaluated

**Cells:** 2-7.
**Covered by the rule text: NO.**

Rule 2 says only:

> `p = &v.buf[i]          // p names the path v.buf[i]`

Rule 3's invalidation list does not mention writing `i`, and nothing re-checks the bound after
formation.

```text
n = len_of(buf)
i = n - 1
p = &buf[i]            // Rule 7 discharged: i < len_of(buf)
i = 1000000            // i is an integer local, not a proper prefix of p's path.
                       // Rule 3 lists no invalidation event here.
put_at(p, 7)           // fn put_at(slot: &Int, x: own Int)  writes(slot)
                       // Snapshot reading: writes buf[n-1]. Symbolic reading: writes buf[1000000].
```

Two sentences lean toward the snapshot reading without stating it: Rule 9's "an index enters
an effect only through an argument, **evaluated once at the call**", and Rule 7's
`len_of(part) == hi - lo`, which fixes a range's measure at formation. Neither is about a
reference formed and held in a local.

**Minimal fix:** one clause in Rule 2 — the index and range expressions in a path are evaluated
when the reference is formed, and the path thereafter names that slot. Zero runtime cost, and
it is almost certainly what was meant.

---

## Gap 5. What `truncate` does to linear elements

**Cells:** 1-8, 4-8, 6-8 (recorded there as an accepted-unsafe construction).
**Tasks:** resource pipeline (its G7), resource container (its G4).
**Covered by the rule text: NO.** Two sentences whose conjunction is unsatisfiable for one call.

Rule 6 prescribes the discharge:

> A DynBox whose element type is linear is itself linear (Rule 8) and **must be truncated to
> zero by the program** before it can go out of scope.

Rule 6 gives `truncate` a contract with no linear-element precondition:

> `fn truncate<T>(buf: &DynBox<T>, n: u64)       writes(buf)`
> `contract { requires n <= len_of(buf);         ensures len_of(buf) == n; }`

Rule 8 forbids the only semantics that would make the prescription work:

> Linear values must be consumed by an explicit operation on every exit path; **the compiler
> never releases them**.

```text
linear type File;  fn close(f: own File)
files: DynBox<File>                 // linear by containment
... ten Files pushed in ...

truncate(&files, 0)                 // requires 0 <= 10: satisfied. Accepted.
                                    // Rule 8 forbids the compiler releasing the ten Files.
                                    // They are now in slots [0, cap_of) outside the window,
                                    // where Rule 6 says slots "hold nothing" and no operation
                                    // can name them again.
// scope exit: Rule 6 releases slots [0, len_of) = nothing, and frees the block.
// Ten Files are gone, unclosed, with no diagnostic anywhere.
```

Rule 8's "the compiler never releases them" correctly rules out one horn. Nothing rules out the
*call*, and Rule 6 positively recommends it. The drain loop (`while len_of(files) > 0 { t =
pop(&files); close(move t) }`) works and costs nothing, so the fix is free.

**Minimal fix:** add `requires` to `truncate` that the discarded slots' element type is not
linear, and rewrite Rule 6's prescription to "must be drained to zero by the program".

---

## Gap 6. What a fallible allocation does with a linear by-value payload

**Cells:** 5-8, 8-14.
**Tasks:** resource container (its G2).
**Covered by the rule text: NO.**

Rule 5 fixes the failure type with no payload return:

> `b = Box::new(Node { ... })?         // allocation can fail: Result<Box<Node>, Oom>`

Rule 14 confirms there is no other exit:

> Allocation returns a `Result` and never traps.

Rule 8 states an obligation the `Err` arm cannot meet:

> Linear values must be consumed by an explicit operation on every exit path.

```text
fn install(f: own File, n: u64) -> Result<Box<Conn>, Oom> {
    b = Box::new(Conn { f: move f, n: n })?    // Rule 9: the call site records the consumption
                                               // of f, so Rule 8 is FORMALLY satisfied.
    Ok(b)
}
// On the Err(Oom) edge the File is inside a Conn that was never constructed, no program point
// can name it, and close was never called. Accepted, with no diagnostic.
```

**Minimal fix:** give the fallible constructors a value-returning error arm,
`Result<Box<T>, (Oom, own T)>`. Zero cost on the success path. Without it, the rewrite is to
allocate an empty `DynBox` first and populate it, which costs 16 bytes of block header per
linear cell that a `Box` holds headerless (RC13).

---

## Gap 7. A reference rebound through its own path in a loop

**Cells:** 3-3, 4-12 (and 2-2 before its ground was corrected).
**Covered by the rule text: NO.** The criterion is stated; no way to discharge it is.

Rule 2:

> At a control-flow join, a reference variable's target is the set of paths it may name; every
> check on it must hold for **every member of the set**.

Rule 12 restates it and adds nothing. Every join example in the file is a two-armed `if`.

```text
struct Tree { n: Int, kids: DynBox<Tree> }     // recursive through DynBox: reachable with
                                               // Rule 2's grammar alone, no variant payload
p = &root
while 0 < len_of(p.kids) {
    p = &p.kids[0]            // at the loop header the target set is
}                             // { root, root.kids[0], root.kids[0].kids[0], ... } — infinite
use(p)
```

One verifier dismissed this on the ground that Rule 2's path grammar cannot descend
recursively without a variant projection, so depth is bounded by type depth. That argument
fails: `DynBox<Tree>` is a recursive type reachable by `[i]` alone, as above. The other two
verifiers confirmed the gap.

The rule file gives no widening, no summary path form, no depth bound, and no rejection clause.
"Every check must hold for every member" has no terminating procedure against an infinite set.

**Minimal fix:** either a summary form (a path with a `*`-closure component) or an explicit
rejection of a reference rebound through its own path. Cost of rejection: the index-carried
rewrite, one bounds compare per hop (RC1), or the recursive form, one call frame per hop.

---

## Gap 8. The reach of the atomic in-place update

**Cells:** 4-5, 5-5, 6-6, 6-12, 12-16, 3-12.
**Tasks:** array kernel (its gap 4), resource container (its G1), tree rewrite (its G2),
generic algorithms (its G4).
**Covered by the rule text: NO** — on all three sub-questions.

The whole definition is one line inside Rule 6:

> `buf[k] = f(buf[k])                   // atomic in-place update: the old value goes into f by
> value, f's result is committed, no program point lies between; requires k < len_of(buf)`

Three things it does not say:

1. **Is a non-index place an eligible left-hand side?** The form is shown only on `buf[k]` and
   carries an index precondition, yet Rule 6's closing sentence lists "the atomic update" among
   the general ways to move a value out of *storage*.
2. **May `f` take further arguments, or carry an effect row?** The sentence constrains the old
   value and the result and is silent on arity.
3. **May `f` be fallible?** Every allocation is fallible (Rule 14), so a rewrite that grows a
   structure must return a `Result`, and the commit semantics of a failed `f` are unstated.

```text
struct Node { op: u8, left: Option<Box<Node>>, right: Option<Box<Node>> }

n.left = rewrite(n.left)                // (1) is a struct field an eligible LHS?
buf[k] = merge(buf[k], move donor)      // (2) may f take a second argument?
buf[k] = try_expand(buf[k])?            // (3) may f allocate, and therefore fail?
```

This is the candidate's only zero-cost in-place transformation primitive. Under the narrowest
reading (index LHS, one argument, total) it serves a shrinking rewrite perfectly and no growing
one, which is the tree-rewrite task's strongest counterexample; and the cross-column affine
move in the array-kernel task and the replace-exporting-the-old-value in the container task
both need question 2.

**Minimal fix:** three clauses. State that the LHS may be any place the program can name, that
`f` may take further arguments and carry an ordinary effect row checked by Rules 9 and 10, and
what a failing `f` commits. The slot is never observable as empty under any of these, because
`f` owns the old value throughout.

---

## Gap 9. The type of a range reference

**Cells:** 7-7 (raised in verification), 6-7.
**Tasks:** array kernel (its gap 1), allocator (its gap 4), generic algorithms (its G3).
**Covered by the rule text: NO** for the typing question; the dangerous horn is closed.

Rule 7 lists a range reference beside `DynBox<T>`, not as one:

> Indexable things: an inline `array<T, N>` (constant length, always fully initialized), a
> `DynBox<T>` (Rule 6), **a range reference into either**, and a `const` table.

"slice types as first-class values" is under **Not in this candidate**. Yet Rule 13's own
accepted example passes ranges across a signature:

> `par { kernel(&v[0..mid], &out[0..mid]); kernel(&v[mid..n], &out[mid..n]) }   // disjoint
> ranges: accepted`

and no parameter type for `kernel` is ever written. Every range-taking helper in this matrix —
`kernel`, the sort's recursive halves, the array-kernel task's eight-column partition helper,
the allocator's block filler — needs a spelling the file does not provide.

```text
fn kernel(part: ???, out: ???)  writes(out), reads(part)      // no type exists for ???
...
par { kernel(&v[0..mid], &out[0..mid]); kernel(&v[mid..n], &out[mid..n]) }   // Rule 13's own line
```

Two consequences turn on it. The array-kernel task's eight-column partition passes either one
`&Cols` plus two counts (3 registers) or eight range references (16 registers) depending on the
answer — an O(1)-per-partition difference, not a real cost. Recursive divide-and-conquer needs
a callee holding a range to form `&part[0..m]`; Rule 7's "into either" does not list a range
into a range, while Rule 2's grammar admits `[lo..hi]` as a general path continuation.

**The dangerous reading is closed.** Cell 7-7 constructed a double-free from two `par` arms
each calling `pop` on an adjacent range of one block, since Rule 6 puts `len_of` in one block
header. That program does **not** survive the text: the window operations are declared
`fn pop<T>(buf: &DynBox<T>)`, and Rule 7 lists a range reference as a thing distinct from a
`DynBox<T>`, so a range reference cannot bind to them.

**Minimal fix:** name the type of a range reference and say whether it re-ranges, and state
explicitly that the window operations do not apply to one.

---

## Gap 10. What holds after a `par` block, and what an early exit inside an arm means

**Cells:** 10-14, 13-14, 12-13 (non-decisive there).
**Tasks:** array kernel (its gap 3), resource pipeline (its G5).
**Covered by the rule text: NO.** Rule 13 is an acceptance test and nothing else.

Rule 13 states only when a block is accepted. Rule 12 defines fact propagation for
*alternatives* —

> At a join, facts are intersected (a fact survives only if it holds on every incoming edge)

— and a `par` is not a join of alternatives, because both arms run. Rule 14 exhibits an early
exit inside an arm without defining it:

> `par { a = build(&x)?; b = build(&y)? }     // both allocate: accepted`

```text
par {
    a = build(&x)?              // if this arm returns Err...
    b = build(&y)?              // ...does this arm complete? Is b bound on the propagating path?
}
// Unstated: (1) which of A's and B's ensures hold after the block;
//           (2) which Err is returned when both arms fail;
//           (3) what discharges a linear value held by an abandoned arm (Rule 8: "every exit path").
```

(2) is an observable nondeterminism under a memory budget: which arm gets `Err` depends on
allocator order. Rule 14's determinism sentence does not cover it — it says only "**Addresses**
are not observable, so allocator concurrency does not affect program determinism".

(3) is the safety-relevant half: a linear resource live in an abandoned arm has no exit path,
which defeats Rule 8's guarantee without any memory error.

Every derivation sidestepped this by returning outcome values from each arm and matching after
the join, which costs one block of sibling work per failing run — practically zero.

**Minimal fix:** state that both arms run to completion, that both arms' `ensures` hold after
the block, and fix an arm order for error selection.

---

## Gap 11. Whether assigning over a struct field releases the old affine value

**Tasks:** multi-index (its gap 1). Not reported by any cell.
**Covered by the rule text: NO.**

Rule 6 states the release semantics for a window slot only:

> Assigning `buf[k] = x` where the old value is affine releases the old value; where it is
> linear the assignment is rejected unless written as the atomic update whose function consumes
> the old value.

```text
struct Obj { payload: Box<Data>, n: u64 }
fn swap_payload(o: &Obj, x: own Box<Data>)  writes(o.payload)
{ o.payload = move x }        // Is the old Box released here? Leaked until scope exit?
                              // Or is the assignment rejected the way a linear slot is?
```

Rule 9 makes `writes(o.payload)` a legal effect and Rule 2 makes the path legal, so the
assignment is certainly *accepted*; only its release semantics is unstated. Rule 8 mentions
release only "at scope exit".

Nothing unsafe follows — an affine value may be dropped by definition — but the answer decides
array-of-structs against struct-of-arrays for any table of objects with owning fields, which is
one extra random cache line per field touched (RC12).

**Minimal fix:** generalize Rule 6's sentence from `buf[k]` to any place.

---

## Gap 12. Whether a pre-call fact survives as a fact about `entry(p)`

**Cells:** 7-16, 10-11, 10-16, 15-16.
**Covered by the rule text: NO** — used by four cells, written nowhere.

Rule 11 says how a fact dies:

> A fact that mentions a path is invalidated when that path is written (by statement or call)
> unless the callee's `ensures` re-establishes it.

Rule 16's `alloc` and Rule 6's window operations all export their result in terms of the
pre-state:

> `ensures len_of(a.buf) == len_of(deref(entry(a.buf))) + 1`

```text
n = len_of(a.buf)               // fact: len_of(a.buf) == n
id = alloc(&a, node)            // writes(a.buf): the fact above dies (Rule 11).
                                // ensures: len_of(a.buf) == len_of(deref(entry(a.buf))) + 1
                                // To use it, "len_of(deref(entry(a.buf))) == n" must be
                                // available — i.e. the dead fact must survive re-labelled
                                // onto the pre-state. No sentence says it does.
use_bound(id)                   // needs id == n
```

Every `ensures` of that shape in the file is unusable without this step, so the design plainly
presupposes it. It is a silence, not a conflict, and the fix has no runtime cost.

**Minimal fix:** one sentence in Rule 11 — a fact about a path holds of `entry(p)` after the
call that wrote it.

---

## Gap 13. Integer overflow

**Tasks:** mutable graph (its G4), multi-index (its gap 3).
**Covered by the rule text: NO**, and arguably out of scope.

The candidate's only statement about integers is Rule 7's notation line: "`Int` and `u64` are
Copy." Nothing says whether an increment carries a proof obligation, a written outcome, or a
wrapping form.

```text
n.gen = n.gen + 1               // generation bump on node deletion. Proof? Outcome arm?
                                // Wrapping? The rule file does not say.
```

This is governed by the existing kernel specification rather than by candidate x1 — x1 is a
delta over the ownership and access rules, not a full language. It is listed only because two
tasks needed it and could not find it, and because a generation counter is the mechanism Rule
16 itself names. At most one compare and branch per deletion either way.

---

# Part 2 — Reported gaps that the rule text already decides

Ten groups. Each was reported as a gap by at least one cell or task; each is answered by a
sentence in the file. These are the class the previous round over-reported, and they are
recorded here so the same questions are not re-opened.

### 2.1 A closure capturing a reference is rejected, not undecided
Reported by cells 2-4, 4-4, 10-10, 10-13 and task generic-algorithms (G1) — four cells, two of
them with claimed data races. All wrong. Rule 9 decides:

> An effect row lists `reads(path)` and `writes(path)` where each path **starts at a reference
> parameter** ... A function body is checked against its own row: **every statement's effect and
> every callee's substituted row must be covered by the declared row**.

```text
h = |x: &Buf| merge_into(cap, x)      // cap: &Accum, captured
par { h(&a); h(&b) }                  // claimed race
```
`h`'s body performs `writes(cap)`. `cap` is not a parameter of `h`, so no row of `h` can name
it, so the effect is uncoverable, so the body fails its own check. Rejected before Rule 13 is
ever consulted. Rule 4's qualifier "captured by a function value **that is stored or returned**"
is a red herring: the narrower case is refused for an independent reason. The rewrite — pass
the capture as an ordinary reference parameter — is the same machine word in the same register.

### 2.2 A variant payload is not nameable by any path
Reported by 3-12, 4-12, 5-5, 8-9, 9-9 and tasks tree-rewrite (G1) and multi-index (gap 2).
Rule 2's grammar is a closed enumeration and simply does not contain the case:

> A path starts at a local variable or a parameter and continues through fields, `*` (Box
> content), `[i]` (index, Rule 7), or `[lo..hi]` (range, Rule 7).

So `&opt.payload` is **refused**, not ambiguous. For a `DynBox` slot the access route is stated:
Rule 6's atomic update takes the whole `Option<Entry>` by value. For a struct field the route
depends on Gap 8 above. The tension with Rule 1's `struct Node { value: Int, left:
Option<Box<Node>> }` and Rule 12's recommended `slots: DynBox<Option<Entry>>` is real, but it
produces a **cost** (RC11), not an undecided acceptance.

### 2.3 Writing `a[i]` does not invalidate a fact about `len_of(a)`
Reported by 6-11, 15-16, 12-16 and tasks allocator (gap 3) and mutable-graph (G2). Three
sentences decide it. Rule 11 names the written path, not an overlapping one:

> A fact that mentions a path is invalidated **when that path is written**.

Rule 6 makes the fact unfalsifiable by an element store anyway:

> The boundary `len_of` is a runtime number stored in the block header ... and **changed only by
> the built-in operations below**.

And Rule 3 draws the same distinction explicitly for references. The "overlap reading" imports
Rule 10 clause 1's judgment, which that clause confines to a call's own pairwise comparison.
This matters: under the wrong reading every element-wise kernel reloads the block header each
iteration. Under the right one the array-kernel task is instruction-for-instruction the C++ loop.
The only cosmetic defect is that Rule 11 lacks Rule 3's explicit carve-out sentence.

### 2.4 An effect row may not name an index
Reported by 7-9 and task mutable-graph (G1). Rule 9 states it twice, once normatively and once
in the example comment:

> **Signatures never contain index expressions**; an index enters an effect only through an
> argument, evaluated once at the call.
> `put_at(&buf[len_of(buf)], 3)   // the argument fixes the slot at the call; the row itself
> names no index`

So `writes(g.nodes[s].succs)` with `s: u64` is rejected. Zero cost under either reading: the
call-site form `&g.nodes[s].succs` substitutes to the same path.

### 2.5 `put_at(&buf[len_of(buf)], 3)` is a defective example, not an open question
Reported by 2-9, 4-9, 6-9, 9-12 — four cells, two of which were corrected by their verifiers
and two of which were not. Rule 6 is normative and unambiguous:

> Reading `buf[k]` or forming `&buf[k]` **requires the fact `k < len_of(buf)`** (Rule 7).

The obligation for that example is `len_of(buf) < len_of(buf)`, which is false, not unclear.
Rule 6's "Slots `[len_of, cap_of)` hold nothing" and Rule 12's no-hole sentence agree. An
example contradicting the rule it illustrates is a defect in the example.

**Consistency note for the owner:** cells 6-9 and 9-12 are recorded `rejected-zero-cost` on this
reasoning while 2-9 and 4-9 are recorded `undecided-rule-gap` on the same question. On the
judgment here, 2-9 and 4-9 should also be `rejected-zero-cost`, which would move the totals to
17 undecided and 30 rejected-zero-cost. The totals in this report use the verifiers' verdicts
as delivered.

*Real consequence:* deleting the example also deletes `emplace_back`. See RC6.

### 2.6 Linearity propagates through enums and tuples
Reported by task resource-container (G3) and task resource-pipeline (G2) as a soundness risk —
a typed outcome enum carrying an `own File` could be discarded. Rule 8's rule is the general
clause; the list after the colon is illustrative:

> Linearity is declared on external-resource types and **propagates through aggregates**: a
> struct, array, Box, or DynBox containing a linear part is linear.

Rule 1 fixes the aggregate set and includes both: "Every struct, **enum, tuple**, array,
slice-like value, Box, DynBox, and generic instantiation". The exhaustive-list reading would
also make Rule 6's linear-element clause and Rule 12's `DynBox<Option<Entry>>` recommendation
dead text. Covered — but the list should be corrected to match Rule 1, because the stake is a
silent resource leak.

### 2.7 Vector growth is not a blessed bulk relocation
Reported by 6-6 (corrected by its verifier) and priced by task resource-container. Rule 6's
enumeration is closed and its length clause independently blocks the shortcut:

> the only ways to move a value out of storage are consuming a whole local (`move x`), the
> window operations, and the atomic update

> `len_of` ... changed only by the built-in operations below

Nothing listed raises the new block's `len_of` from 0 to n except n `push_nogrow` calls. The
prose "`Vector<T>` is library code: a DynBox plus a `push` that ... moves `[0, len_of)` across"
describes what the library does, not a primitive it may call. Decided — and the decision costs
2.5n moves (RC2).

### 2.8 A bitwise mask establishes no bound
Reported by 3-7 (dismissed there), 7-11, and named by task mutable-graph as its highest-value
question. Rule 11 fixes the fact language:

> Facts are the existing WF forms: **affine comparisons** over measures and integer values,
> refinement facts from a dominating branch, loop-header invariants ..., explicit `use` steps
> inside an `invariant`, and callee contracts.

`x & (c - 1) < c` is not an affine comparison, so it cannot be recorded as a fact and no `use`
step has a premise to begin from. Rule 7 states the escape: "when the proof is unavailable the
program tests the measure ... one compare, no trap". Decided — and permanent, so RC1 below is
not conditional. What a `use` step may *derive* (e.g. `lo <= lo + (hi-lo)/2 < hi`) is deferred
to "the existing WF forms" and is a question about the existing specification, not about x1.

### 2.9 Rule 13 imports Rule 10's by-value clause
Reported by 10-13. Rule 13 says so: "using the **same** path-overlap and index/range-disjointness
judgment as Rule 10", and Rule 10 clause 2 is part of that judgment.

### 2.10 A call used as an argument contributes no path
Reported by 6-6 and 10-10. Rule 10 clause 2 speaks of "**its place**"; a call result has no
place, and the value is its own call. Both cells concede that sequencing into two statements is
free.

---

# Part 3 — Real costs, grouped by cause

Thirteen cost clusters, each traced to the rule responsible. "Real" means: no rewrite inside the
rule set removes it. Runtime performance is the only cost counted.

### RC1. One compare and one predicted branch per data-determined index
**Rule responsible:** Rule 11 — "There are no quantified facts over array elements ('for all
i ...') and no per-slot occupancy facts" — with Rule 7's proof obligation.
**Cells:** 1-5, 1-7, 3-7, 4-11, 7-11, 7-16, 11-11, 11-16, 2-16, 5-16, 6-16, 8-16, 12-16, 15-16.
**Tasks:** mutable graph (its headline), multi-index, generic algorithms, allocator, tree rewrite.

Every graph hop, free-list pop, hash probe and decoded offset re-proves its bound because no
fact can travel with a stored index. Program P3 asks for exactly the invariant that is excluded
by name ("next/prev point to live nodes, stated once and reused"). Three escapes were checked
and all fail: hoisting is impossible when the index comes from the data; the power-of-two mask
is not affine (2.8 above); and there is no trap or `unreachable` to fold the else arm into, so
it is live program text. Cost: about two extra micro-ops per hop, loss of a straight vector
gather (roughly 1.5–2x on a load/store gather kernel), and the loss of software pipelining
across two hops. **Zero against safe Rust; real against C++ and unsafe Rust.**

### RC2. No exchange, no partial move: in-place mutation of non-Copy elements
**Rule responsible:** Rule 6's closed move-out enumeration, with Rule 12's no-hole clause.
**Cells:** 6-6, 6-10, 8-8, 5-5.
**Tasks:** generic algorithms (in-place sort, partition and stable retain are **not expressible
at all** for any element type that owns anything), resource container (2.5n growth, 3L ordered
remove), tree rewrite (four window operations instead of one `mem::replace`), mutable graph
(the O(1) use-list splice).

This is the largest cost cluster in the matrix and the one with the cleanest fix.

```text
// wanted: exchange two slots of a DynBox<Box<Node>> in place
t = buf[i]; buf[i] = buf[j]; buf[j] = t    // every line is a partial move out of a place
```
The atomic update cannot serve: `f`'s result is committed straight back into the same slot, so
there is no exit for the old value. Nesting an update inside `f` needs `&buf` alongside
`&buf[k]`, which is Rule 10's own rejected `bad(&vv, &vv[0])`. `swap_remove` relocates the tail
and changes `len_of`.

Prices: interior swap 9 moves and 6 header stores against `std::swap`'s 3 — about 3x the byte
traffic in a sort's inner loop, and no two-load/two-store idiom to recognize. Order-preserving
growth ~2.5n moves and 2n header stores against one `memcpy`, and not vectorizable because each
refill depends on the previous length. Ordered removal at index k: 3L moves against a
vectorized `memmove` of L. The resource-container task's strongest program (10,000 16-byte
linear elements, removal near the front) predicts 4–8x on that phase.

Notably, the refusal protects no stated invariant: Rule 1 already licenses byte relocation,
`len_of` is untouched, and every value keeps exactly one owner throughout. **Adding
`fn exchange<T>(buf: &DynBox<T>, i: u64, j: u64) writes(buf)` to Rule 6's window operations
restores in-place sort, in-place partition and stable retain at zero cost and needs no new fact
form.** Two independent tasks arrived at that same recommendation.

### RC3. Parallel scatter with a data-determined destination
**Rule responsible:** Rule 11 (injectivity is a quantified fact) with Rule 13, and "channels or
atomics" under **Not in this candidate**.
**Cells:** 1-13, 2-13, 6-13, 7-13, 9-13, 13-16, 14-16.
**Tasks:** array kernel (its strongest counterexample), multi-index, generic algorithms,
resource pipeline (fan-in to one sink).

```text
par { scatter(&in[0..mid], &out, &perm[0..mid]); scatter(&in[mid..n], &out, &perm[mid..n]) }
// both arms carry writes(out) on the whole column, because the destination is data and
// Rule 9 can narrow a path only through an argument. Rejected.
```
Rewrites and their prices: invert-and-gather, +8n bytes for the inverse table, one O(n) build
pass (amortizes to zero if the permutation is reused), +1 compare per element; privatize-and-
merge, k private tables plus a k×B merge pass, which loses outright against a contended atomic
once B is large, and cannot express a permutation merge at all; bucket-first, two extra O(n)
passes. **Equal to safe Rust; a real loss against C++ with a programmer assertion or an atomic.**

### RC4. Window operations declare the whole container, killing every interior reference
**Rule responsible:** Rule 9's "Signatures never contain index expressions", which forces
`writes(buf)` on `pop`, `push_nogrow`, `swap_remove` and `truncate`; with Rule 3 and Rule 10
clause 3.
**Cells:** 1-3, 2-6, 3-6, 5-6, 2-16, 9-16, 10-15.
**Tasks:** resource container, mutable graph (any mutator behind a function boundary), tree rewrite.

A reference into a container dies at **every** window operation, including a `push_nogrow` that
provably cannot reallocate, where C++ keeps its pointer valid while capacity suffices. Cost per
re-formation: one block-pointer reload plus one address computation, plus one bounds compare
where the length fact does not survive, and a full dependent load when the reference ran through
a `Box` held in the slot. Zero when the code is inlined into the same function; real at a
separate-compilation boundary.

### RC5. No returned references: find-then-mutate re-descends
**Rule responsible:** Rule 4 — "cannot be returned" — with Rule 3.
**Cells:** 1-4, 3-4, 5-15, 15-15.
**Tasks:** tree rewrite (+d dependent loads, compares, branches and stores per find/mutate pair,
where d is the tree depth — it doubles a lookup whose entire cost is its dependent-load chain).

Three mitigations, each with a residue: fuse search and use into one function (zero, but not
available across separate compilation); pass a callback (one indirect call per hit unless the
instantiation is specialized, which the rules permit and promise nowhere); move the structure
into a Rule 16 pool (one bounds compare per use — RC1).

### RC6. No construction into the append slot
**Rule responsible:** Rule 6's `k < len_of(buf)` formation requirement, which rejects Rule 9's
own `put_at(&buf[len_of(buf)], 3)` example (see 2.5).
**Cells:** 4-9, 6-9, 9-12.
Each element is built in a local and moved in: one `sizeof(T)` copy per push against C++'s
`emplace_back`, which constructs at the destination. The placeholder variant instead pays one
full initialization of the slot.

### RC7. `cap_of` is fixed at allocation; there is no resize and no realloc
**Rule responsible:** Rule 6.
**Cells:** 2-14, 6-14.
**Tasks:** resource container.
Every growth copies the whole window. A buffer grown to 1 GiB moves about 2 GiB that a
realloc-based `Vec` can often avoid entirely by extending the mapping. Compounds with RC2's
2.5x factor for affine elements. `DynBox::filled` also stores the whole reserve eagerly
(Rule 12 admits no uninitialized slot), where C++ and Rust get lazy zero pages from `mmap` —
8 MiB of stores for a 2^20-word arena, dominant for a short-lived one.

### RC8. Pool allocation writes the pool, so it overlaps every other pool access
**Rule responsible:** Rule 16's `fn alloc<T>(a: &Arena<T>, x: own T) -> u64   writes(a.buf)`,
with Rule 13.
**Cells:** 1-16, 9-16, 13-16, 14-16.
**Tasks:** mutable graph, allocator.
Two allocations cannot run in parallel, and no allocation can run beside any fill. Rewrites:
phase separation, which serializes the dominant construction phase; or per-worker arenas, which
force a two-level `(arena_id, slot)` handle costing one extra dependent load on **every**
traversal hop for the life of the structure, and raise peak memory by roughly k times the
per-worker imbalance with no cross-worker reuse of freed blocks.

### RC9. Generation words, wherever identity must survive slot reuse
**Rule responsible:** Rule 16 ("a logic error, not a memory error") with Rules 1 and 4, which
make an index the only durable name.
**Cells:** 2-16, 3-16, 4-16, 5-16, 6-16, 16-16.
**Tasks:** mutable graph, multi-index, allocator, tree rewrite.
Per dereference: one generation load (usually the same cache line), one compare, one predicted
branch, plus an `Option` or error arm at the caller. Per node or handle: 4–8 bytes, so an edge
grows from 8 to 12 or 16 bytes, halving edges per cache line in a pointer-chasing traversal.
**Equal to a Rust generational arena or slotmap; the whole cost against a C++ raw pointer,
which pays with undefined behaviour instead.** Declining the word costs nothing at runtime and
makes the defect silent — which is the accepted-unsafe class in Part 4.

### RC10. No channels and no atomics
**Rule responsible:** "channels or atomics" under **Not in this candidate**; the channel
primitive is under **Deferred**. Not chargeable to any rule.
**Cells:** 4-13, 6-13, 13-16.
**Tasks:** mutable graph (the parallel worklist — batched fork-join costs O(n) per round instead
of O(dirty) plus a barrier, **the matrix's only asymptotic loss**), resource pipeline (stage
overlap over one stateful resource: a barrier per block, and E[max of k] instead of max[E], a
predicted 10–40% throughput loss against a channel pipeline with depth-d queues).
Also: no work stealing, so an irregular frontier keeps the imbalance of its static split and the
slowest arm sets wall time; no lock-free rings (already recorded under **Known costs**).

### RC11. Box-linked structures must carry a `DynBox` child window
**Rule responsible:** Rule 2's closed path grammar (see 2.2).
**Cells:** 3-12, 5-5.
**Tasks:** tree rewrite.
`struct Node { op, lit, kids: DynBox<Box<Node>> }` instead of two inline `Option<Box<Node>>`
fields: +1 dependent load per child hop and +1 allocation plus block-header bytes per node
against a C++ two-pointer node. Zero against a shape-for-shape Rust `Vec<Box<Node>>` node.
Separately, a non-Copy `Option` slot can only be reached by the whole-value atomic update: about
2w bytes of copy per access, roughly eight extra loads and stores for a 32-byte `Entry`, unless
the backend lowers the update in place — which Rule 6 permits and does not require.

### RC12. `len_of` lives in the block header; owning fields push toward struct-of-arrays
**Rule responsible:** Rule 6 for the header; Gap 11 for the layout.
**Cells:** 3-6, 6-15.
**Tasks:** array kernel (+8 dependent loads per partition call), multi-index.
One dependent load per `len_of` read where Rust's `Vec` reads a register-resident field —
hoistable out of any loop that does not write the buffer, given 2.3 above. The layout pressure
costs one extra random cache line per additional object field touched: a hot loop reading four
fields of one object touches 4 lines where a C++ node pool touches 1.

### RC13. A linear value cannot cross a fallible allocation in a `Box`
**Rule responsible:** Gap 6.
**Cells:** 8-14.
The safe rewrite allocates an empty `DynBox` of one and populates it: 16 bytes of block header
per linear cell that a `Box` would hold headerless, i.e. 16n bytes for n linear nodes.

### Recorded gains, for balance
The candidate is not uniformly more expensive. Rule 1 plus Rule 10's `two(&a.x, &a.y) //
accepted: distinct fields` gives an eight-column struct-of-arrays kernel restrict-quality
disjointness **unconditionally**, with no annotation and no runtime alias guard, where C++ must
be told or must emit a guard — and it beats safe Rust by eight bounds compares per element. Rule
8's affine release with no drop flags beats Rust by one stack byte and one branch per
conditionally-owned local. The `par` tree is source-determined, so a floating-point reduction is
bit-reproducible at no cost. A pool edge is 16 bytes against LLVM's 32-byte four-pointer `Use`,
in contiguous storage, with relocation by `memmove` and no fixup. And Rule 3 refuses, at compile
time, several programs where C++ dangles silently.

---

# Part 4 — Accepted-unsafe claims, re-traced

Nine claims were recorded across the cells. Seven survive the rule text; two do not.

### Survives — memory errors

**U1. A bystander reference over storage the call released (Gap 3).** Cells 1-10, 3-9, 3-13,
5-6. The program in Gap 3 above. Rule 3 and Rule 10 clause 3 invalidate only on a proper prefix
and never ask for an index-distinctness proof, while Rule 10 clause 1 demands exactly that proof
for its own comparison. With `i == j` at run time, `use(q)` reads freed heap storage. Cells 3-7
and 3-10 rely on the same permissive reading to keep ordinary bystanders alive, so the reading
is not an outlier. **Survives the text.**

**U2. A store through a path whose index was changed after formation (Gap 4).** Cell 2-7. The
program in Gap 4 above. Every step is admitted by a quoted rule and no rule re-checks the bound.
**Survives the text.**

**U3. A reference into a block-local `Box` after the block's release (Gap 2).** Constructed
during verification of rows 4/13. The program in Gap 2 above. Rule 3's invalidation list is
qualified "by a statement or by a call"; the compiler's release is neither. **Survives the text,
and is not contingent on any reading.**

### Survives — resource-obligation breaks, not memory errors

**U4. `truncate` discards linear elements unclosed (Gap 5).** Cells 6-8, 8-16. The program in
Gap 5 above. Accepted, and Rule 6 positively prescribes it. **Survives the text.**

**U5. A linear payload lost on the `Err` arm of `Box::new` (Gap 6).** Cell 8-14. The program in
Gap 6 above. Rule 9 records the consumption, so Rule 8's letter is satisfied while its guarantee
is not. **Survives the text.**

### Survives — identity errors that Rule 16 admits on purpose

**U6. A stale in-bounds handle names the slot's new occupant.** Cells 16-16, 2-16, 3-16, 4-16,
5-16, 6-16, 12-16.

```text
free_node(&g, id1)
id3 = alloc_reuse(&g, Node { .. })   // returns id1 again
x = g.buf[id1].op                    // reads the NEW occupant
```
Rule 16 states the disposition in terms: "A stale index that is still in bounds names the
current occupant of that slot: a logic error, not a memory error. Programs that need to detect
it keep a generation number as data." Memory-safe. It violates P3's stated requirement that a
relation to a deleted node must not silently become a relation to a replacement node, and P2's
requirement that every old pointer into a released block be refused afterwards, unless the
program pays RC9. Cell 16-16 additionally shows free-list reuse and `truncate(0)` arena reset
composing into identity confusion whose only check — the bounds test — fires nondeterministically
with input size, needing two detectors (generation plus epoch) rather than one.
**Survives the text, by design.**

**U7. A stale index survives a window operation because it is integer data.** Cell 4-6.

```text
n = len_of(v)
i = find(&v, key)?          // ensures i < len_of(v), i.e. i < n
x = pop(&v); consume(move x)
push_nogrow(&v, move fresh) // ensures chain re-derives len_of(v) == n
charge(&v[i], amount)       // Rule 7 discharged STATICALLY; writes a slot the search never saw
```
The fact `i < n` mentions only integer locals, so Rule 11's path-based invalidation never
touches it, and the two `ensures` re-derive the length. Memory-safe by Rule 12, silently wrong
in identity, and with **no** runtime check anywhere to notice. This one does not need Rule 16 —
it falls out of Rule 11 and Rule 6 alone. **Survives the text.**

**U8. A reference into an allocator-managed region survives that region's release.** Task
allocator. A user-level `free_words(&r.meta, b)` writes a *sibling* path (`r.meta`), which is
not a prefix of any payload path, so a live `&r.data[off..off+size]` is untouched. Writing the
payload range instead is an *equal* path, which Rule 3's content-write clause explicitly
excludes; writing `r.data` invalidates every block's references and conflicts with every
parallel fill. So P2's "every old pointer into b2 must be refused afterwards" is unmeetable at
any declarable release effect in the contiguous-region form. Memory-safe (the storage is
initialized `u64` the region still owns, and addresses are unobservable), identity-unsafe.
Worth flagging because Rule 16 says "logic error, not a memory error" about *indices*, and here
the same disposition silently extends to *references*. **Survives the text.**

### Does not survive

**U9. Two `par` arms draining adjacent ranges of one block (double free).** Cell 7-7's
construction. Rule 13's own example accepts the block, and both arms would decrement the one
`len_of` in the shared block header, returning the same `Box` twice. **It does not survive:**
the window operations are declared `fn pop<T>(buf: &DynBox<T>)`, and Rule 7 lists "a range
reference into either" as a thing distinct from "a `DynBox<T>`", so a range reference cannot
bind to `pop`. The typing gap behind it (Gap 9) is real; this consequence of it is not.

**U10. Two `par` arms racing through a closure's captured reference.** Cells 2-4, 10-13.
**It does not survive:** the closure fails its own body check under Rule 9 long before Rule 13
is consulted (see 2.1).

---

# Part 5 — What depends on Rule 16

Rule 16 is the only provisional rule in the candidate, and the dependence is sharply divided.

**Refusing Rule 16 refuses three tasks outright.** Rules 1 and 4 forbid a reference inside any
aggregate and forbid returning one; Rule 5 gives single-owner `Box` with no shared ownership.
So a cyclic or shared mutable structure has exactly one representation — a `DynBox` plus
indices — and Rule 16 is the ruling that such an index's staleness is a logic error rather than
a defect the rule set must prevent. The mutable-graph, multi-index and allocator tasks are not
achievable at all without it.

The dependence is on the **disposition, not the syntax**. Nothing in Rules 1–15 forbids indexing
a `DynBox` with a value read out of another `DynBox`; the program text still checks. Rule 16 is
what says the resulting hazard is acceptable.

**Not dependent on Rule 16:** the array-kernel task entirely; the sequential resource pipeline
and its two-resource `par`; the tree rewrite in its `Box` shape; the generic-algorithms boundary
mechanism, its four algorithms and all its parallelism. Within the cells, Rule 16 carries none
of RC1 (that is Rules 7 and 11), none of RC2 (Rule 6), none of RC3 (Rules 11 and 13), and none
of RC4 (Rules 3, 9, 10). It owns RC8 and RC9 and nothing else.

**What Rule 16 costs where it is used:** RC8 (allocation serializes against every pool access,
or a two-level handle at +1 dependent load per hop) and RC9 (the generation word). Cell 16-16
shows its two usage patterns — free-list reuse and `truncate(0)` arena reset — composing into a
failure whose only detector fires nondeterministically, which argues that if Rule 16 is
approved, the generation-and-epoch discipline should be written into it rather than left as
advice.

---

# Part 6 — Fixes, in order of value

1. **Gap 1** — rank Rule 6's enumeration against Rule 8's `close(move c.f)`, preferably by
   adding whole-local destructuring. Resolves 8 of 19 undecided cells and decides whether
   `Vector<Conn>` and `Box<File>` exist.
2. **Gap 2** — add the compiler's scope-exit release to Rule 3's invalidation list. One clause;
   closes the only unconditional memory-safety hole.
3. **Gap 3** — say that paths are compared by the storage they denote, with indices treated as
   overlapping unless proved distinct. Closes a use-after-free that four cells derived as legal.
4. **Gap 4** — say that a path's index is evaluated at formation. One clause; closes an
   out-of-bounds store.
5. **Gap 5 and Gap 6** — a `requires` on `truncate` and a value-returning `Err` arm on the
   fallible constructors. Two silent resource leaks that Rule 8 exists to prevent.
6. **Gap 8** — state the atomic update's left-hand side, its arity and its fallibility. It is
   the only zero-cost in-place primitive in the candidate and is currently defined by one
   example line.
7. **Gap 9 and Gap 10** — name the type of a range reference; say what holds after a `par` and
   what an early exit in an arm means. Both are needed before any code is written against the
   candidate.
8. **Gap 7, 11, 12, 13** — loop-carried target sets, field-assignment release, `entry(p)` fact
   survival, overflow. Smaller, and 13 belongs to the existing specification.
9. **Not a gap but the largest single cost:** add `exchange(buf, i, j)` to Rule 6's window
   operations (RC2). It restores in-place sort, in-place partition, stable retain, ordered
   growth and the O(1) list splice at zero cost, needs no new fact form, and protects no
   invariant by its absence.
10. **Correct two defective examples:** delete or rewrite `put_at(&buf[len_of(buf)], 3)` in Rule
    9 (2.5), and make Rule 8's linearity-propagation list match Rule 1's aggregate list (2.6).
