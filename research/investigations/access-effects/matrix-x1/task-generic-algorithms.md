# Engineering task under candidate x1 — generic algorithms through function-typed parameters

Derived against the frozen rule set in `CANDIDATE-X1.md` (2026-09-18) and only that
file. Where the engineering framing or a discriminating program is touched it is
quoted from `PROGRAMS.md` (row "Generic algorithm through helpers", plus P5, P6,
P16, P17). Nothing under "Not in this candidate", "Deferred", or the second
"Pending owner ruling" item is used. Rule 16 is used in exactly one place, §10,
and the dependence is marked there and in §12.

Task text: *Generic algorithms: sort, binary search, partition, and a map/filter
over `DynBox<T>` written once with function-typed parameters carrying effect rows
and contracts; show that the bounds and disjointness facts flow through the
generic boundary and what a caller must supply.*

Cost convention, from the protocol's item 7: runtime performance is the only cost.
Verbosity, extra parameters and duplicated code are not costs. "One **move**" means
one relocation of `sizeof(T)` bytes; Rule 1's stated consequence — "any value can be
relocated by copying its bytes (memmove, realloc), because nothing inside it points
anywhere" — makes every move a plain byte copy with no fixup, so move counts are
directly comparable with C++ move constructors and Rust `ptr::copy`.

Baselines named throughout: **C++** means libstdc++ `std::sort` / `std::lower_bound`
/ `std::partition` / `std::remove_if` with an inlined functor, including
`__unguarded_partition`'s sentinel scan; **Rust** means `slice::sort_unstable`
(pdqsort), `slice::binary_search_by`, `Vec::retain`, `Iterator::map().collect()`
with a monomorphized closure and no `get_unchecked`. Where the two baselines differ
the table says which one a cost is measured against, because several of this
candidate's costs are parity with safe Rust and a real loss only against C++.

---

## 1. The question this task tests

Three questions, which turn out to have three different answers:

1. **Does a generic algorithm need to see its callback's body?** Rule 10's last
   paragraph says no — "Function-typed parameters carry a full signature with its
   own row and contract, and a call through one uses that row." The test is whether
   the *declared row alone* is enough to (a) check the generic's body once, (b) let
   the generic declare its own row, and (c) let a caller decide acceptance without
   looking inside the generic. §5 and §6 answer: yes, at zero runtime cost, and the
   mechanism that makes it work is that every aliasing question is decided at the
   caller after substitution, never inside the generic.
2. **Do bounds facts cross the boundary?** Rule 7 requires a fact for every index
   and Rule 11 fixes the fact language. The test is whether a generic can prove its
   own indices in bounds from its parameters' measures, and whether a *callback*
   can hand the generic a bounds fact it could not derive. §6.2 answers: yes for
   both directions, at zero cost, with one open question about integer halving
   (§11, G2) that only binary search touches.
3. **Can the four algorithms be written at all for a non-Copy element type?**
   Rule 6 closes the set of ways to move a value out of storage. The test is
   whether in-place reordering survives that closure. §4 and §9 answer: **no** —
   two-slot exchange is not expressible for any non-Copy `T`, in-order draining
   *is* expressible by an indirect route, and every reordering algorithm over a
   non-Copy element type pays a real runtime cost. This is the load-bearing
   negative result of the task.

---

## 2. Why the generic surface has the shape it has, before any algorithm is written

Two rules together force the shape of every signature below, so they are settled
first rather than re-argued at each algorithm.

### 2.1 A function value may not carry a captured reference, so context is an explicit parameter

The idiomatic C++ and Rust form of every one of these algorithms captures state in
the callback:

```cpp
// C++ / Rust shape that this candidate cannot have
std::sort(idx.begin(), idx.end(), [&keys](u64 a, u64 b){ return keys[a] < keys[b]; });
```

Rule 1 states the closure is impossible as a *value*: "Every struct, enum, tuple,
array, slice-like value, Box, DynBox, and generic instantiation holds only owned
values. This is recursive and closed under wrapping: there is no type parameter,
wrapper, or variant payload through which a reference can be stored." Rule 4 states
the same from the reference's side: a reference "cannot be captured by a function
value that is stored or returned."

The two sentences do not settle the case that actually matters here — a function
value that is *passed as an argument* and neither stored nor returned (§11, G1).
It does not matter for cost. The form used throughout this file threads the
captured state as an ordinary extra reference parameter:

```text
// instead of a closure over `keys`, the comparator takes the context by reference
fn less_by_key(a: &u64, b: &u64, ctx: &DynBox<u64>) -> Bool
    reads(a), reads(b), reads(ctx)
```

A closure environment pointer and an explicit `ctx` reference are the same machine
word passed in the same register. The rewrite is **zero cost** under either reading
of G1, and every signature below is written in the explicit-context form so that
the derivation does not depend on G1 being resolved.

This has a second, better consequence, exploited in §6.2: because the context is a
*declared parameter* of the callback, the callback's effect row and its contract can
both mention it. A closure's captured state can appear in neither.

### 2.2 The generic's own row is the caller's entire interface

Rule 9: "A function body is checked against its own row: every statement's effect
and every callee's substituted row must be covered by the declared row." Applied to
a generic whose callee is a parameter, this means the generic must declare, on its
own reference parameters, the union of everything the supplied function can do.
That union is computable from the function-typed parameter's declared row alone,
by substituting the generic's own arguments into it — the same substitution Rule 10
clause 1 performs at any call.

So the shape of every algorithm below is:

```text
fn algorithm<T, C>(buf: &DynBox<T>,            // the container
                   ctx: &C,                     // everything the callback needs
                   f: fn(...) <row> <contract>) // the callback, row and contract written out
    <the union row over buf and ctx>
    contract { <the measure relations the caller needs after the call> }
```

and `PROGRAMS.md`'s requirement for this task row — "Express representative
container/tree/range tasks over user types and passed functions without inspecting
callees at every call site" — is met exactly by that union row. §5.3 traces a call
site to show the caller consults nothing else.

---

## 3. Shared declarations

```text
// ---- element types, one per value class of Rule 8 ----
// Rule 8: "A type is copy, affine, or linear."
struct Key   { hi: u64, lo: u64 }                 // copy: two u64 fields, no owning part
struct Node  { key: u64, bytes: DynBox<u8> }      // affine: owns a DynBox (Rule 8, via containment)
linear type File;   fn close(f: own File)         // linear, Rule 8's own declaration form
struct Conn  { f: File, id: u64 }                 // linear by containment (Rule 8)

// ---- the context type used by the derivations ----
struct Ranker { weights: DynBox<u64>, bias: u64 }   // Rule 1: owned fields only
```

Three measures are used, all from the rule file: `len_of`, `cap_of`, and the
pre-state form `entry(p)` (Rule 11: "the pre-state of a parameter is written
`entry(p)`"). Where a reference parameter `p: &DynBox<T>` is dereferenced in a
contract the file's own spelling `len_of(deref(entry(p)))` is used, copied from
Rule 6's `push_nogrow` contract.

---

## 4. The strongest program first: a generic in-place reorder over a non-Copy element type

The protocol's item 2 asks for the case where static analysis provably cannot help.
For this task that is not a data-determined index — it is the **element class**.
Every reordering algorithm in the assignment (sort, partition, and the compacting
half of filter) is built from one operation, and that operation is where the
candidate breaks. So it is derived first, in full, before anything that works.

### 4.1 The program

```text
// The one primitive every in-place reordering algorithm needs, written generically.
fn exchange<T>(buf: &DynBox<T>, i: u64, j: u64)   writes(buf)
    contract {
        requires i < len_of(buf);
        requires j < len_of(buf);
        ensures  len_of(buf) == len_of(deref(entry(buf)));
    }
{
    // ??? — §4.2 shows every route the rules admit and why each fails for non-Copy T
}

// Its three consumers, all of which are otherwise ordinary:
fn sort_range<T, C>(buf: &DynBox<T>, ctx: &C,
                    less: fn(a: &T, b: &T, c: &C) -> Bool  reads(a), reads(b), reads(c))
    writes(buf), reads(ctx)
    contract { ensures len_of(buf) == len_of(deref(entry(buf))); }
{ ... quicksort whose only mutation is exchange(&buf, i, j) ... }

fn partition_range<T, C>(buf: &DynBox<T>, lo: u64, hi: u64, ctx: &C,
                         pred: fn(v: &T, c: &C) -> Bool  reads(v), reads(c)) -> u64
    writes(buf), reads(ctx)
    contract { requires lo <= hi; requires hi <= len_of(buf);
               ensures lo <= result; ensures result <= hi;
               ensures len_of(buf) == len_of(deref(entry(buf))); }
{ ... Lomuto whose only mutation is exchange(&buf, w, k) ... }

fn retain_inplace<T, C>(buf: &DynBox<T>, ctx: &C,
                        keep: fn(v: &T, c: &C) -> Bool  reads(v), reads(c))
    writes(buf), reads(ctx)
{ ... compaction whose only mutation is a move of buf[r] into buf[w], w <= r ... }
```

Instantiate with `T = Box<Node>` — an eight-byte slot holding an affine owning
pointer. In C++ this is `std::vector<std::unique_ptr<Node>>` and `std::sort` on it
performs ordinary eight-byte `std::swap`s; in Rust it is `Vec<Box<Node>>` and
`sort_unstable_by` does the same with `ptr::swap`. Both are memory-safe today.

### 4.2 Every route the rules admit, and why each one fails

Rule 6 closes the set: "There is no `take` operation and no partial move out of any
place: the only ways to move a value out of storage are consuming a whole local
(`move x`), the window operations, and the atomic update." An exchange of two slots
requires reading two non-Copy values *out* of storage. The routes are therefore:

**(a) `t = buf[i]`, a plain read.** For Copy `T` this is a copy and the whole
problem disappears (§7). For non-Copy `T` it is a move out of the place `buf[i]`,
which is not a whole local, not a window operation and not the atomic update.
Refused by the sentence above, and separately by Rule 12: "No place is ever
partially moved: every path is either wholly present or the program cannot name
it."

**(b) `buf[i] = buf[j]`.** Same refusal on the right-hand side. It would also lose
`buf[i]`'s old value: Rule 6, "Assigning `buf[k] = x` where the old value is affine
releases the old value."

**(c) The atomic update, Rule 6's fourth route.** Its whole text is:

> `buf[k] = f(buf[k])                   // atomic in-place update: the old value goes into f by value, f's result is committed, no program point lies between; requires k < len_of(buf)`

This *is* a read-out — the old value genuinely reaches `f` by value. The problem is
that it is a read-out with no exit. `f`'s only outputs are its return value, which
Rule 6 commits back into slot `k`, and whatever its effect row permits it to write.
So try to give it an exit:

```text
fn hand_over<T>(old: own T, dst: &T, replacement: own T) -> own T   writes(dst)
{
    *dst = move old          // writing through a reference: Rule 9's `put_at` shape
    move replacement         // committed into slot k by the atomic update
}

buf[i] = hand_over(buf[i], &buf[j], ???)
```

Two things go wrong, and only one of them is repairable.

*First*, `*dst = move old` **destroys** the old contents of `buf[j]` — Rule 6's
release clause again. An exchange needs that value, and there is no route to it:
reading `*dst` is refused by (a), and the atomic update cannot be applied to `*dst`
because `*dst` is a reference target, not a `DynBox` slot in a place the writer can
name as `buf[k]`.

*Second*, `???` must be some owned `T`, and the generic has none. `T` is a type
parameter with no constructor, no default, and — because Rule 12 forbids partially
present places — no uninitialized local to fill later.

The second problem is repairable: `pop` yields a donor.

```text
donor = pop(&buf)                          // Rule 6: -> own T, requires len_of(buf) > 0
buf[i] = hand_over(buf[i], &buf[j], move donor)
```

At this call Rule 10 clause 1 is satisfied — the atomic update writes `buf[i]`, the
argument's substituted row writes `buf[j]`, and they are "indices or ranges proved
distinct" given `i != j`. The program is *accepted*. It is simply not an exchange:
`buf[j]`'s old value has been released, `buf[i]` now holds the donor, and the value
that was in `buf[i]` is in `buf[j]`. One element of the container has been
destroyed. Restoring it needs a read-out of `*dst`, and the circle closes.

**(d) Nesting an atomic update inside `f`.** For `f`'s body to write slot `j` as a
window operation it must hold `&buf`, not `&buf[j]`. Then at the outer atomic
update the substituted effects are `writes(buf[i])` from the update and
`writes(buf)` from the argument, and Rule 10 clause 1 refuses them — this is
exactly the rule file's own worked rejection:

> ```text
> fn bad(outer: &DynBox<DynBox<Int>>, inner: &DynBox<Int>)  writes(outer), writes(inner)
> bad(&vv, &vv[0])                             // rejected: writes(vv) and writes(vv[0]) overlap
> ```

**(e) `swap_remove`.** It is a genuine interior read-out —
`fn swap_remove<T>(buf: &DynBox<T>, k: u64) -> own T writes(buf)` — but it
simultaneously relocates the last element into slot `k` and decrements `len_of`. It
cannot express an exchange of two chosen slots at fixed length; §8 shows the one
thing it *can* express, which is more useful than it first appears.

### 4.3 The finding

**There is no two-slot exchange for any non-Copy `T` under Rules 1-15.** The
consequence is not a diagnostic about one program; it removes in-place `sort`,
in-place `partition`, and stable in-place `retain` from the generic surface for
every affine and every linear element type, which is every element type that owns
anything.

Note carefully what this is *not*. It is not a safety requirement. Rule 1's own
stated consequence licenses the operation at the machine level: "any value can be
relocated by copying its bytes (memmove, realloc), because nothing inside it points
anywhere." A byte exchange of two disjoint slots of a `DynBox` preserves every
invariant the candidate states — the window `[0, len_of)` is unchanged, no slot is
ever observed empty ("no program point can observe a slot inside the window as
empty"), each value keeps exactly one owner throughout, and no reference into the
block can be live across it because Rule 10 clause 3 kills every reference whose
path has `buf` as a proper prefix. The operation is refused by the *enumeration* in
Rule 6, not by any property the enumeration protects. §9 develops this into the
counterexample.

---

## 5. The generic boundary: what crosses it, traced rule by rule

This section derives the three mechanical claims of §2.2 on a program that works,
so that §7 and §8 can use them without re-tracing.

### 5.1 The program

```text
// In-place map over every element, with a context the callback may read and write.
fn map_inplace<T, C>(buf: &DynBox<T>,
                     ctx: &C,
                     f: fn(v: own T, c: &C) -> own T  reads(c))
    writes(buf), reads(ctx)
    contract { ensures len_of(buf) == len_of(deref(entry(buf))); }
{
    i = 0
    while i < len_of(buf) {
        invariant m1: 0 <= i;
        invariant m2: i <= len_of(buf);
        buf[i] = f(buf[i], ctx)
        i = i + 1
    }
}

// The same algorithm when the callback accumulates into the context instead.
fn map_inplace_acc<T, C>(buf: &DynBox<T>,
                         ctx: &C,
                         f: fn(v: own T, c: &C) -> own T  writes(c))
    writes(buf), writes(ctx)
    contract { ensures len_of(buf) == len_of(deref(entry(buf))); }
{ ... identical body ... }
```

### 5.2 Trace of the generic's body

**Rule 7, the index.** `buf[i]` in the atomic update: "buf: DynBox<Int>;  buf[i]
// requires i < len_of(buf)". Discharged from `m2` (`i <= len_of(buf)`) refined by
the loop test `i < len_of(buf)` — Rule 11's "refinement facts from a dominating
branch". `m1` and `m2` are exactly Rule 11's admitted header form, "loop-header
invariants `invariant name: affine_expr compare_op affine_expr`"; both sides are
affine in `i` and the measure `len_of(buf)`. `m2` re-establishes at the back edge
because `i = i + 1` runs after the refined `i < len_of(buf)`, and `len_of(buf)` is
unchanged across the atomic update, which Rule 6 lists among the operations that do
not move the boundary (only `push_nogrow`, `pop`, `swap_remove`, `truncate` and
replacement change it: "The boundary `len_of` is a runtime number stored in the
block header, readable by the program, and changed only by the built-in operations
below").

**Rule 6, the update itself.** The written form is `buf[i] = f(buf[i], ctx)`, which
carries an argument beyond the old value. Rule 6 shows the schema with one argument;
whether further arguments are admitted is G1 in §11. The derivation below is under
the permissive reading, and §11 states what is lost under the strict one.

**Rule 9, effect coverage.** "A function body is checked against its own row: every
statement's effect and every callee's substituted row must be covered by the
declared row." The body's effects are the atomic update's write of `buf[i]` and the
callback's substituted row. In `map_inplace`, `reads(c)` substitutes to `reads(ctx)`;
`writes(buf[i])` is covered by the declared `writes(buf)` because `buf` is a prefix
of `buf[i]`, and `reads(ctx)` is covered by the declared `reads(ctx)`. In
`map_inplace_acc`, `writes(c)` substitutes to `writes(ctx)`, which is why that
version's row must say `writes(ctx)` — and that single character is the entire
difference the caller will see in §5.3.

Note the index does not appear in either row, as Rule 9 requires: "Signatures never
contain index expressions; an index enters an effect only through an argument,
evaluated once at the call." The row says `writes(buf)`; the index `i` enters only
the statement's own effect inside the body.

**Rule 10 clause 1, at the callback call.** The substituted effects are
`writes(buf[i])` (from the update) and `reads(ctx)` or `writes(ctx)` (from `f`).
`buf` and `ctx` are distinct reference parameters, and clause 1's first disjunct is
"different roots". Accepted inside the generic, for both versions, and — this is the
point — accepted *without knowing whether the caller will pass overlapping
arguments*. That question is not asked here.

**Rule 10, the callback itself.** "Function-typed parameters carry a full signature
with its own row and contract, and a call through one uses that row." Nothing in
the trace above consults `f`'s body. The generic is checked once, as written,
against the row in its own parameter list.

### 5.3 Trace at three call sites — where aliasing is actually decided

```text
r: Ranker
v: DynBox<Node>

// (i) read-only context
map_inplace(&v, &r, rescale)                 // rescale: fn(v: own Node, c: &Ranker) -> own Node reads(c)

// (ii) accumulating context, disjoint from the container
map_inplace_acc(&v, &r, rescale_and_count)   // writes(c)

// (iii) the caller aliases the two
map_inplace_acc(&v, &v, fold_into_self)
```

**(i)** Substituting `map_inplace`'s row gives `writes(v), reads(r)`. Rule 10
clause 1: different roots, disjoint. Accepted.

**(ii)** Substituting `map_inplace_acc`'s row gives `writes(v), writes(r)`.
Different roots. Accepted.

**(iii)** Substituting gives `writes(v), writes(v)` — "Two effects on overlapping
paths where at least one is a write must be proved disjoint (different roots, or
indices or ranges proved distinct); otherwise the call is rejected." Same root,
same path, not distinct. **Rejected at the caller.**

This is the mechanism claimed in §2.2. The generic body was checked once under the
assumption that its two reference parameters are different roots; the caller that
violates that assumption is caught by substituting the *generic's own declared row*,
which is one line the caller can read. The generic's body is never re-examined, and
`f`'s body is never examined at all. It is `PROGRAMS.md`'s requirement for this row
— "without inspecting callees at every call site" — discharged by Rule 10's
substitution alone, at zero runtime cost, because nothing in the check survives into
the lowered code.

---

## 6. What the boundary transmits, and what a caller must supply

The assignment asks for this explicitly, so it is stated as a closed list with its
grounds, rather than left implicit in the traces.

### 6.1 What crosses outward (generic → caller), and what does not

| Fact | Crosses? | Ground |
|---|---|---|
| The measure `len_of(p)` of any reference parameter `p` | yes | Rule 11 admits "affine comparisons over measures"; the measure is generic in `T` |
| Relations in the generic's `ensures`, including `entry`-relative ones | yes | Rule 11: "callee contracts"; Rule 6's `push_nogrow` contract is the file's own example |
| A result index's bound, `ensures result < len_of(v)` | yes | Rule 4's worked `find` signature is exactly this |
| Variant-routed relations | yes | Rule 11: "`ensures when Variant:` for result-routed relations" |
| Anything about *element values* — sortedness, injectivity of an index array, "some element to the right exceeds the pivot" | **no** | Rule 11: "There are no quantified facts over array elements ("for all i ...") and no per-slot occupancy facts" |
| A reference into the container | **no** | Rule 4: a reference "cannot be returned" |

The last two rows are the ones that cost something, in §7.4 and §7.5.

### 6.2 What crosses inward (caller → generic), and the one non-obvious direction

Three things cross inward, and the third is the interesting one.

**(a) Bounds the generic proves for itself.** A generic never needs the caller to
supply an index bound for indices it generates: `map_inplace` derived `i < len_of(buf)`
from its own loop invariant and the measure of its own parameter. This holds for
every algorithm below except where a *data-determined* index is involved (§7.4).

**(b) Disjointness of the actual argument paths.** The caller must supply this and
cannot delegate it, because inside the generic two distinct reference parameters are
"different roots" by construction (§5.2) and the generic has no way to observe
otherwise. Rule 10's substitution is the entire enforcement, as traced in §5.3(iii).
This is the answer to "what must a caller supply": *the disjointness of everything
it passes, judged against the generic's declared row and nothing else.*

**(c) A bounds fact supplied by the callback's contract.** This is the direction
that the closure form of §2.1 cannot express at all, and it is what makes the
explicit-context signature strictly more capable than a closure rather than merely
equivalent.

```text
// A generic scatter: place each element of `src` into a slot of `dst` chosen by `slot_of`.
fn scatter<T, U>(src: &DynBox<T>,
                 dst: &DynBox<U>,
                 slot_of: fn(v: &T, d: &DynBox<U>) -> u64  reads(v), reads(d)
                     contract { ensures result < len_of(d); },
                 make: fn(v: &T) -> own U  reads(v))
    reads(src), writes(dst)
{
    i = 0
    while i < len_of(src) {
        invariant s1: i <= len_of(src);
        k = slot_of(&src[i], &dst)        // k < len_of(dst) arrives as a callee contract
        dst[k] = make(&src[i])            // Rule 7's index premise discharged by that contract
        i = i + 1
    }
}
```

Rule 11 admits "callee contracts" as a fact form, and Rule 9 permits the callback's
row to name `d` because "each path starts at a reference parameter" — `d` is one of
the callback's own reference parameters. So the callback's `ensures result < len_of(d)`
substitutes at the call to `k < len_of(dst)`, which is precisely Rule 7's premise
for `dst[k]`. **The bound crosses the generic boundary as a contract clause and
costs nothing at runtime.** Without it the same program falls back to Rule 7's other
route — "when the proof is unavailable the program tests the measure, which is
ordinary data" — for one compare and branch per element (§7.4).

Two limits on (c), both from quoted text and both without cost:

- The callback's `ensures` can only mention the callback's *own* parameters and
  result. A key-extraction callback written `fn key_of(v: &T) -> u64 reads(v)` can
  say nothing about a container it does not take, which is why `slot_of` above takes
  `d`. The fix is always to widen the callback's parameter list, which Rule 9
  permits and which item 7 of the protocol says is not a cost.
- Whoever ultimately *establishes* the fact pays for it once. If `slot_of`'s body is
  `h % len_of(d)`, its `ensures` needs `len_of(d) > 0` plus a residue fact for `%`,
  neither of which Rule 11's listed forms supply; the body then tests once
  (`if r < len_of(d) { r } else { 0 }`). That is one compare per element, paid
  inside the callback instead of inside the generic — the same single branch, not an
  extra one. The boundary crossing itself remains free.

### 6.3 Summary answer to the assignment's question

- **Bounds flow both ways and cost nothing.** Outward as `ensures` on a returned
  index (Rule 4's `find`), inward as `ensures` on a callback (§6.2(c)), and
  internally from a loop invariant over the parameter's own measure.
- **Disjointness flows only inward, and only as an obligation on the caller.** The
  generic declares a row; the caller substitutes; Rule 10 clause 1 decides. No
  disjointness fact is ever *derived* inside a generic about its own parameters, and
  none needs to be.
- **Nothing about element values flows in either direction**, because Rule 11
  admits no quantified facts over elements. That single sentence is the source of
  three of the five real costs in §12's table.

---

## 7. The four algorithms over a Copy element type

With §5's mechanism established, the Copy cases are short. `T = Key` throughout;
`u64` and `Int` behave identically. Rule 8: "Copy values may be duplicated."

### 7.1 Partition

```text
fn partition_copy<T, C>(buf: &DynBox<T>, lo: u64, hi: u64, ctx: &C,
                        less: fn(a: &T, b: &T, c: &C) -> Bool  reads(a), reads(b), reads(c))
    -> u64
    writes(buf), reads(ctx)
    contract {
        requires lo < hi;
        requires hi <= len_of(buf);
        ensures  lo <= result;
        ensures  result < hi;
        ensures  len_of(buf) == len_of(deref(entry(buf)));
    }
{
    // Lomuto; the pivot is the element at hi-1 and is never moved until the end.
    piv = hi - 1
    w = lo
    k = lo
    while k < piv {
        invariant p1: lo <= w;
        invariant p2: w <= k;
        invariant p3: k <= piv;
        invariant p4: piv < len_of(buf);
        if less(&buf[k], &buf[piv], ctx) {
            t = buf[w]; buf[w] = buf[k]; buf[k] = t      // Copy: three copies, no move-out
            w = w + 1
        }
        k = k + 1
    }
    t = buf[w]; buf[w] = buf[piv]; buf[piv] = t
    w
}
```

**Rule 7.** `buf[k]`, `buf[w]`, `buf[piv]` each need their index below `len_of(buf)`.
`p4` gives `piv < len_of(buf)` (established at entry from `requires hi <= len_of(buf)`
and `piv = hi - 1`, both affine). `p3` refined by the loop test gives `k < piv`, and
`p2` gives `w <= k`, so all three are below `len_of(buf)` by affine chaining. Every
invariant is Rule 11's admitted header form.

**Rule 10 clause 1 at `less(&buf[k], &buf[piv], ctx)`.** Substituted:
`reads(buf[k])`, `reads(buf[piv])`, `reads(ctx)`. "Read/read overlap is allowed", so
no `k != piv` fact is needed, although one is available. Accepted.

**The three-copy swap.** `t = buf[w]` is a copy, not a move-out, because `T` is
Copy; §4.2(a)'s refusal does not apply. `buf[w] = buf[k]` reads a Copy and writes a
slot. Rule 6's release clause is about affine and linear old values, so nothing is
released. Accepted.

**Cost.** Identical to C++ Lomuto and to safe Rust: three register moves per swap,
one comparison per element, no extra branch. The comparison is an indirect call
through `less` unless the implementation specializes; see §7.6.

**The one refused variant, and it is a real cost.** Hoare's unguarded partition, the
form libstdc++ actually uses:

```text
while true {
    while less(&buf[i], &pivot, ctx) { i = i + 1 }     // no i < hi test: relies on the pivot as a sentinel
    while less(&pivot, &buf[j], ctx) { j = j - 1 }
    if i >= j { return j }
    exchange(&buf, i, j); i = i + 1; j = j - 1
}
```

The inner loop indexes `buf[i]` with no bound available: the sentinel argument is
"there exists an element at or before `hi` that is not less than the pivot", a fact
quantified over elements. Rule 11: "There are no quantified facts over array
elements ("for all i ...") and no per-slot occupancy facts". The parenthetical
illustrates the universal case; the sentinel premise is existential, and on either
reading the word "quantified" excludes it. **Refused.**

The rewrite is the guarded scan, `while i < j && less(&buf[i], &pivot, ctx)`, which
adds one compare and one branch per scanned element. Versus libstdc++'s
`__unguarded_partition` that is a real cost — roughly `n` extra compares per
partition level, so about `n log n` over a full sort. Versus Rust's pdqsort, which
also bounds its scans, it is parity. Verdict for this variant:
**`rejected-real-cost`**, cost quantified in §12.

It is worth recording *why* this is the right trade even though it costs: the
sentinel argument in C++ is exactly what makes `std::sort` with an inconsistent
comparator run off the end of the array. Under this candidate, `partition_copy`'s
bounds come from `p1`-`p4` and the measure alone, and never from `less`. A lying
comparator produces a wrong order and cannot touch memory outside the window. That
is a genuine safety gain over C++, bought with the branch.

### 7.2 Sort

```text
fn sort_copy<T, C>(buf: &DynBox<T>, ctx: &C,
                   less: fn(a: &T, b: &T, c: &C) -> Bool  reads(a), reads(b), reads(c))
    writes(buf), reads(ctx)
    contract { ensures len_of(buf) == len_of(deref(entry(buf))); }
{
    // Iterative, with an inline stack of (lo, hi) pairs. Rule 7 admits
    // "an inline array<T, N> (constant length, always fully initialized)".
    stack: array<Span, 64> = filled(Span { lo: 0, hi: 0 })   // Span is two u64: Copy
    sp = 0
    stack[0] = Span { lo: 0, hi: len_of(buf) }
    sp = 1
    while sp > 0 {
        invariant t1: sp <= 64;
        sp = sp - 1
        s = stack[sp]                                    // Copy read
        if s.hi - s.lo > 1 {
            m = partition_copy(&buf, s.lo, s.hi, ctx, less)
            // m's bounds arrive from partition_copy's ensures: s.lo <= m < s.hi
            if sp + 2 <= 64 {
                stack[sp]     = Span { lo: s.lo,   hi: m }
                stack[sp + 1] = Span { lo: m + 1,  hi: s.hi }
                sp = sp + 2
            } else {
                insertion_sort(&buf, s.lo, s.hi, ctx, less)   // depth guard: bounded fallback
            }
        }
    }
}
```

**Why iterative rather than recursive.** Not because recursion is refused — Rule 10
states "Recursion is checked through contracts, never by unfolding bodies", and the
recursive form checks fine. It is because the recursive form wants to hand each half
to the recursive call as a *sub-range of a range reference*, and whether that is
admitted is G3 in §11. Writing the sequential sort iteratively over `buf[k]` sidesteps
G3 entirely, at zero cost: the stack array is inline, so there is no allocation, and
the loop is what every production quicksort compiles to after tail-call elimination
anyway. Verbosity is not a cost (protocol item 7).

**Rule 7 on the stack.** `stack[sp]` needs `sp < 64`, from `t1` and the guarded
push. `array<Span, 64>` is Rule 7's first indexable kind and is "always fully
initialized", which the `filled` initializer supplies; `Span` is Copy so this is
legal without any element-wise fact.

**Rule 11, the bound on `m`.** `partition_copy`'s `ensures lo <= result` and
`ensures result < hi` cross the boundary as callee contracts and are exactly what
the two pushed spans need to stay within `[0, len_of(buf)]`. Nothing about the
*contents* is needed, and nothing about the contents is available.

**Cost.** Parity with C++ and Rust on everything except the partition guard inherited
from §7.1 and the indirect call of §7.6.

### 7.3 Binary search

```text
fn lower_bound<T, K, C>(buf: &DynBox<T>, key: &K, ctx: &C,
                        less_kv: fn(a: &T, k: &K, c: &C) -> Bool  reads(a), reads(k), reads(c))
    -> u64
    reads(buf), reads(key), reads(ctx)
    contract { ensures result <= len_of(buf); }
{
    lo = 0
    hi = len_of(buf)
    while lo < hi {
        invariant b1: 0 <= lo;
        invariant b2: lo <= hi;
        invariant b3: hi <= len_of(buf);
        mid = lo + (hi - lo) / 2
        if less_kv(&buf[mid], key, ctx) { lo = mid + 1 } else { hi = mid }
    }
    lo
}
```

**Rule 10 clause 1.** `reads(buf[mid])`, `reads(key)`, `reads(ctx)` — all reads,
"Read/read overlap is allowed", so a caller may even pass `key` as a reference into
`buf` itself and the call still stands. Accepted.

**Rule 7, and the gap.** `buf[mid]` requires `mid < len_of(buf)`. From `b2` refined
by the loop test, `lo < hi`; from `b3`, `hi <= len_of(buf)`. What remains is
`mid < hi`, which follows from `mid = lo + (hi - lo)/2` only if the checker derives
affine bounds for a halved difference. Rule 11's fact list is:

> "Facts are the existing WF forms: affine comparisons over measures and integer values, refinement facts from a dominating branch, loop-header invariants `invariant name: affine_expr compare_op affine_expr`, explicit `use` steps inside an `invariant`, and callee contracts."

The *fact* wanted, `mid + 1 <= hi`, is an affine comparison over integer values and
is plainly in the language. What is not stated is whether the assignment
`mid = lo + (hi - lo) / 2`, whose right-hand side is not affine, yields it. Marked
**`undecided-rule-gap`**, G2 in §11.

The gap is confined and its cost is bounded on both sides:

- *Permissive reading* (a halving assignment yields `lo <= mid` and `mid < hi` when
  `lo < hi`): zero cost, identical to `std::lower_bound`.
- *Confinement route, zero cost in the loop*: move the only division in the program
  behind a contract.

  ```text
  fn halve(w: u64) -> u64
      contract { requires w > 0; ensures result < w; ensures result + result <= w; }
  ...
  h = halve(hi - lo)            // h < hi - lo, purely affine from here on
  mid = lo + h                  // mid < hi by affine arithmetic
  ```

  `halve`'s `ensures` are affine comparisons, so Rule 11 carries them; the whole
  question is displaced into one function whose body the trusted base can supply.
  Cost in the search loop: zero.
- *Pessimistic reading, neither of the above*: Rule 7's stated fallback — "when the
  proof is unavailable the program tests the measure, which is ordinary data" —
  gives `if mid < len_of(buf) { ... } else { return lo }`. One compare and one
  perfectly predicted branch per probe, about `log2 n` per search. Real, small, and
  quantified in §12.

**Sortedness is not expressible, and this costs nothing.** `lower_bound` cannot
write `requires sorted(buf)`; that is a quantified fact over elements and Rule 11
refuses it. The consequence is that the *contract* promises only `result <= len_of(buf)`
and the ordering premise is carried informally by the caller. This is exactly the
status quo in C++ and Rust, whose `lower_bound` and `binary_search_by` document the
precondition and do not check it, so it is a parity item rather than a cost — and
the memory-safety consequence is strictly better than C++'s, since a non-sorted
input here yields a wrong index and never an out-of-range probe.

### 7.4 Map to a new element type

```text
fn map_new<T, U, C>(src: &DynBox<T>, dst: &DynBox<U>, ctx: &C,
                    f: fn(v: &T, c: &C) -> own U  reads(v), reads(c))
    reads(src), writes(dst), reads(ctx)
    contract {
        requires cap_of(dst) >= len_of(src);
        requires len_of(dst) == 0;
        ensures  len_of(dst) == len_of(deref(entry(src)));
    }
{
    i = 0
    while i < len_of(src) {
        invariant q1: len_of(dst) == i;
        invariant q2: i <= len_of(src);
        push_nogrow(&dst, f(&src[i], ctx))
        i = i + 1
    }
}
```

**Rule 6 on `push_nogrow`.** Its `requires len_of(buf) < cap_of(buf)` is discharged
from `q1` (`len_of(dst) == i`), `q2` refined by the loop test (`i < len_of(src)`),
and the entry requirement `cap_of(dst) >= len_of(src)`: `len_of(dst) = i < len_of(src) <= cap_of(dst)`.
All affine over measures. `q1` re-establishes at the back edge from `push_nogrow`'s
own `ensures len_of(buf) == len_of(deref(entry(buf))) + 1`, which is Rule 11's
"callee contracts" carrying the increment.

**Rule 10 clause 1.** `writes(dst)` from `push_nogrow` versus `reads(src[i])` and
`reads(ctx)` from the argument expression, plus clause 2's contribution — "A by-value
argument contributes a consumption (`move`) or a read (copy) of its place to this
comparison" — for the temporary holding `f`'s result. `dst`, `src` and `ctx` are
different roots. Accepted. A caller that passes the same buffer as `src` and `dst`
is rejected by substituting `reads(src), writes(dst)`, as in §5.3(iii).

**Cost.** One allocation on each side; Rust's `collect` with a known size hint also
allocates once. The one difference is `push_nogrow` updating the block header's
`len_of` on every iteration, where Rust's `collect` writes the length once at the
end. That is one store per element to a hot line. Rule 6 says `len_of` is "changed
only by the built-in operations", and Rule 1 guarantees no element write can reach
the header (no aggregate contains a reference, and the header is the compiler's own
layout), so the store is hoistable out of the loop by an ordinary backend. Recorded
in §12 as conditional: zero if hoisted, `+1 store` per element if not.

### 7.5 Filter, Copy elements

```text
fn retain_copy<T, C>(buf: &DynBox<T>, ctx: &C,
                     keep: fn(v: &T, c: &C) -> Bool  reads(v), reads(c))
    writes(buf), reads(ctx)
    contract { ensures len_of(buf) <= len_of(deref(entry(buf))); }
{
    n = len_of(buf)
    w = 0
    r = 0
    while r < n {
        invariant r1: w <= r;
        invariant r2: r <= n;
        invariant r3: len_of(buf) == n;
        if keep(&buf[r], ctx) { buf[w] = buf[r]; w = w + 1 }
        r = r + 1
    }
    truncate(&buf, w)                  // Rule 6: requires n <= len_of(buf); here w <= r <= n
}
```

Stable, in place, no allocation, one comparison and at most one copy per element —
byte-for-byte the shape of `Vec::retain` and `std::remove_if`. **Zero cost.** The
bounds are `r1`-`r3`, all affine; `truncate`'s `requires n <= len_of(buf)` is
discharged by `r1` and `r2` chained (`w <= r <= n`) against `r3`.

### 7.6 The indirect call, and why it is not charged as a rule cost

Every algorithm above calls through a function-typed parameter, where C++ templates
and Rust generics monomorphize and inline. The candidate says nothing about
lowering; Rule 10 fixes only how the call is *checked* ("a call through one uses
that row"), and proofs "are erased before lowering". A call whose callee argument is
a known function at the call site is an ordinary direct call after the usual
specialization, and a call whose callee is selected at runtime is an indirect call
in every one of the three languages — C++ pays exactly the same through
`std::function`. So this is an implementation obligation on the compiler, not a cost
the rules impose, and it is recorded as parity in §12 with that condition stated.

---

## 8. The four algorithms over a non-Copy element type

§4 removed exchange. This section derives what remains, and finds that one
non-obvious route survives.

### 8.1 In-order draining is expressible, using `swap_remove` alone

`swap_remove(&buf, k)` returns slot `k` and relocates the last live element into it.
Applied at `k = 0, 1, 2, ...` it emits the original elements *in order*, because it
only ever disturbs the tail:

```
n = 5:  [x0 x1 x2 x3 x4]
swap_remove(0) -> x0      [x4 x1 x2 x3]      slot 1 still holds x1
swap_remove(1) -> x1      [x4 x3 x2]         slot 2 still holds x2
swap_remove(2) -> x2      [x4 x3]            slot 3 no longer exists — the phase ends here
                                              and the remainder is exactly reversed
pop()          -> x3      [x4]
pop()          -> x4      []
```

The phase boundary is where `j` reaches `len_of(buf)`, and that condition is affine:

```text
fn drain_in_order<T, C>(buf: &DynBox<T>, sink: &C,
                        emit: fn(v: own T, s: &C)  writes(s))
    writes(buf), writes(sink)
    contract { ensures len_of(buf) == 0; }
{
    n = len_of(buf)
    j = 0
    while j + j < n {
        invariant d1: len_of(buf) + j == n;       // affine over a measure and an integer
        invariant d2: 0 <= j;
        // swap_remove requires j < len_of(buf); from d1 that is j < n - j, i.e. j + j < n,
        // which is the loop test refined by Rule 11's "refinement facts from a dominating branch"
        emit(swap_remove(&buf, j), sink)
        j = j + 1
    }
    while len_of(buf) > 0 {                        // Rule 11's own example: the loop condition is the test
        emit(pop(&buf), sink)
    }
}
```

The second loop emits the reversed tail, which is the original forward order. `d1`
re-establishes because `swap_remove`'s contract states
`ensures len_of(buf) == len_of(deref(entry(buf))) - 1` while `j` increases by one.

**This is the one thing that keeps the non-Copy cases alive.** Order-preserving
extraction of owning elements from a `DynBox` is expressible with the window
operations alone, with no allocation, no unsafe, no `take`, and purely affine
bounds.

**Its cost.** Each `swap_remove` performs one relocation of the last element plus
one move-out: two moves. The drain runs about `n/2` of them and about `n/2` pops at
one move each, so about `1.5n` moves where C++ draining a vector front to back does
`n`. **Extra: about `0.5n` relocations of `sizeof(T)` bytes.** For `Box<Node>` that
is `4n` bytes of extra traffic and is noise; for a large inline element it is not.

### 8.2 Stable filter over non-Copy elements

Built on §8.1, out of place, with a supplied disposer so that linear elements are
discharged (Rule 8: "Linear values must be consumed by an explicit operation on
every exit path; the compiler never releases them"):

```text
fn filter_into<T, C>(src: &DynBox<T>, dst: &DynBox<T>, ctx: &C,
                     keep:    fn(v: &T, c: &C) -> Bool  reads(v), reads(c),
                     discard: fn(v: own T, c: &C)       reads(c))
    writes(src), writes(dst), reads(ctx)
    contract { requires cap_of(dst) >= len_of(src);
               requires len_of(dst) == 0;
               ensures  len_of(src) == 0; }
{
    // identical two-phase drain; each extracted element is tested once and routed
    // ... if keep(&e_as_ref, ctx) { push_nogrow(&dst, move e) } else { discard(move e, ctx) }
}
```

One wrinkle the rules force: `keep` takes `&T`, but after `swap_remove` the element
is an owned local `e`, and `&e` is a reference to a whole local — Rule 2's "A path
starts at a local variable or a parameter". Accepted. Alternatively `keep` is given
the signature `fn(v: &T, c: &C) -> Bool` and called as `keep(&e, ctx)` before the
routing branch, which is what the body does.

**Cost against `Vec::retain`.** `retain` is in place, stable, allocates nothing, and
does zero moves when nothing is removed. `filter_into` allocates one buffer of
`n * sizeof(T)`, always performs the `1.5n`-move drain of §8.1 plus `n` pushes, and
holds both buffers live at once. So: **+1 allocation, +`n * sizeof(T)` peak memory,
and about `+1.5n` moves in the common case where little is removed.** A cheap
mitigation exists and is worth naming: a read-only counting pass first (`n` predicate
calls, zero moves) and skip the drain entirely when nothing fails the predicate,
which turns the common case into `+n` predicate evaluations and no moves. Either way
this is `rejected-real-cost` against `Vec::retain`.

The **unstable** variant is free, and is worth recording because it is what most
call sites actually need:

```text
i = 0
while i < len_of(buf) {
    invariant u1: i <= len_of(buf);
    if keep(&buf[i], ctx) { i = i + 1 }
    else { discard(swap_remove(&buf, i), ctx) }
}
```

In place, one allocation-free pass, one move per removed element — identical to
`Vec::swap_remove`-based retention in Rust. **Zero cost**, at the price of a
different element order, which `PROGRAMS.md`'s "Observable behavior to preserve"
column makes a behavioral difference rather than a free choice.

### 8.3 Sort and partition over non-Copy elements: the two rewrites

Neither is expressible directly (§4). Two rewrites exist.

**Rewrite A — sort a Copy permutation, then apply it.** Sorting is over
`DynBox<u64>` with a comparator that dereferences into the payload container:

```text
fn less_by_payload(a: &u64, b: &u64, c: &DynBox<Node>) -> Bool
    reads(a), reads(b), reads(c)
{
    // Rule 7 premise for c[*a] and c[*b] is NOT available: the values are data.
    // Rule 7's fallback applies: "when the proof is unavailable the program tests the measure"
    if *a < len_of(c) && *b < len_of(c) { c[*a].key < c[*b].key } else { false }
}
```

Applying the permutation is where the cost concentrates. Extraction of an arbitrary
index is only `swap_remove`, which relocates the last element and therefore
invalidates every other permutation entry, so the apply phase must maintain a
forward and a reverse index:

```text
perm:    DynBox<u64>     // sorted original positions
pos:     DynBox<u64>     // pos[o]  = current slot of original element o
orig_at: DynBox<u64>     // orig_at[s] = original identity now living in slot s
out:     DynBox<Node>

k = 0
while k < n {
    invariant a1: len_of(buf) + k == n;
    o = perm[k]
    s = pos[o]                                  // extra dependent load
    if s < len_of(buf) {                        // data-determined: Rule 7 fallback
        e = swap_remove(&buf, s)
        moved = orig_at[len_of(buf)]            // whoever was relocated into slot s
        pos[moved] = s; orig_at[s] = moved      // two extra stores
        push_nogrow(&out, move e)
    }
    k = k + 1
}
```

Cost per element: **+1 extra dependent load** (`pos[o]`), **+1 bounds branch**,
**+2 stores** for index maintenance, **+1 relocation** from `swap_remove`, and
**+3 allocations** of `n * 8` bytes plus one of `n * sizeof(T)`. Against
`std::sort` on `vector<unique_ptr<Node>>`, which does about `n log n` eight-byte
in-place swaps and allocates nothing, this is a large constant-factor loss on the
apply phase, plus **+1 dependent load per comparison** and **+1 bounds branch per
comparison** during the sort itself — and the sort's memory access becomes random
into the payload container, losing the sequential locality that makes a
cache-resident sort fast. `rejected-real-cost`.

**Rewrite B — never store the owning values in the sorted container.** See §10; it
is the Rule 16 route and it makes the sort itself free.

---

## 9. The strongest counterexample against the candidate for this task

**Claim: Rule 6's closed move-out list removes a capability with no safety
justification, and every generic reordering algorithm over an owning element type
pays for it.**

The counterexample program is four lines:

```text
v: DynBox<Box<Node>>                          // eight-byte owning slots, nothing more
sort_range(&v, &ctx, less_by_key)             // one generic sort, written once
// requires exactly one operation the rules do not have:
exchange(&v, i, j)   with  i != j,  i < len_of(v),  j < len_of(v)
```

The argument, with each step grounded:

1. **The operation is refused.** §4.2 walks every route Rule 6's sentence leaves
   open — plain read, slot assignment, the atomic update with and without a donor,
   nesting, `swap_remove` — and each one fails. No route remains.
2. **The refusal protects nothing the candidate claims.** Rule 1 already states the
   machine-level licence: "any value can be relocated by copying its bytes (memmove,
   realloc), because nothing inside it points anywhere." Rule 6's two invariants
   about `DynBox` — that `len_of` changes "only by the built-in operations" and that
   "no program point can observe a slot inside the window as empty" — are both
   preserved by a byte exchange of two in-window slots at fixed length. Rule 8's
   affine and linear obligations are preserved: the exchange creates no value and
   destroys none, so every value still has exactly one owner on every path. Rule 3's
   reference-validity story is unaffected, because Rule 10 clause 3 already kills
   every live reference whose path has `v` as a proper prefix at any call declaring
   `writes(v)`. There is no invariant to point at.
3. **It removes an existing capability, which `PROGRAMS.md` treats as a floor
   failure.** That file's capability floor says "Losing an existing capability is a
   failure of this floor, not merely a price that may be noted and ignored." The
   research worktree's own kernel specification carries the operation today as
   `[SET-2] Affine-place replacement`, "`let x = replace p = e;` atomically exchanges
   the affine value stored at a writable place with a same-typed replacement, binding
   the previous value as the fresh `own` binding x", and notes it is "sound because
   the exchange leaves no program point at which the referent place lacks exactly one
   valid owner." *(That quotation is from `spec/kernel-spec.md` in this worktree, not
   from the rule file; it is cited to show the operation is not hypothetical, and it
   is not used as an authority for any acceptance decision above.)* Candidate x1's
   Rule 6 has one operation of this kind, the atomic update, and it differs in
   exactly one respect: the extracted value cannot leave `f`.
4. **The cost is real and is not recoverable by rewriting.** §8.3 Rewrite A costs a
   dependent load and a bounds branch per comparison, destroyed locality during the
   sort, four allocations, and five extra memory operations per element in the apply
   phase. §10's Rewrite B removes the sort cost but requires Rule 16 and moves the
   dependent load to every subsequent payload access. The protocol's criterion — "A
   rejection counts as expressiveness loss only when no equal-performance rewrite
   exists" — is met: there is no equal-performance rewrite.

**The minimal change the counterexample argues for**, stated as an observation and
not as a rule: adding a two-slot exchange to Rule 6's enumerated window operations,
with the contract
`fn exchange<T>(buf: &DynBox<T>, i: u64, j: u64) writes(buf) contract { requires i < len_of(buf); requires j < len_of(buf); ensures len_of(buf) == len_of(deref(entry(buf))); }`,
restores sort, partition and stable in-place retention over every element class at
zero cost, needs no new fact form, needs no change to Rules 1-5 or 7-15, and does
not require `i != j`. Whether it is added is the owner's ruling, not this
derivation's.

**Two weaker counterexamples, recorded for completeness:**

- *Parallel scatter through a data-determined index array is refused and cannot be
  rescued.* `par` over `out[idx[k]] = f(in[k])` needs `idx` injective, which is a
  quantified fact over elements that Rule 11 excludes. There is no rewrite that
  recovers the parallelism without changing the algorithm. Against safe Rust this
  is parity (safe Rust refuses it too); against C++ with a programmer assertion it
  is a real loss of parallelism.
- *Recursive divide-and-conquer with `par` inside the recursion* is gated on whether
  a range reference may be re-ranged (G3). §10 shows the loss is recoverable at zero
  runtime cost by forming all parallel sub-ranges at the top level, so this one is
  `rejected-zero-cost` under the pessimistic reading of G3.

---

## 10. Parallelism: what the rules admit and what they refuse

### 10.1 Admitted

**(a) Parallel map over adjacent ranges.** Rule 13 states this case verbatim:

> `par { kernel(&v[0..mid], &out[0..mid]); kernel(&v[mid..n], &out[mid..n]) }   // disjoint ranges: accepted`

Through the generic boundary it becomes:

```text
par { map_inplace(&buf[0..mid], &r, rescale);
      map_inplace(&buf[mid..n], &r, rescale) }
```

Substituted rows: `writes(buf[0..mid]), reads(r)` against `writes(buf[mid..n]), reads(r)`.
The ranges are "proved distinct" from `0 <= mid <= mid <= n`, purely affine; the two
`reads(r)` overlap and "Read/read overlap is allowed". **Accepted, zero cost.** Note
the caller decided this from `map_inplace`'s one-line row and never opened `rescale`.

**(b) The `k`-worker chunking of P6.** `PROGRAMS.md` P6 asks for `k` workers over
`out` with runtime-width slices. Rule 13's last sentence covers it: "The existing
counted-loop forms (per-element maps, adjacent ranges passed to a helper, admitted
reductions) are expressed with range references." The chunk arithmetic
`lo = w * s; hi = lo + s` uses only multiplication and addition, so every
disjointness premise is affine and no division appears. **Accepted, zero cost**, and
notably the division question of §7.3 does not touch this case.

**(c) Parallel searches and parallel scans on one container.** `par { i = lower_bound(&v, &k1, ...); j = lower_bound(&v, &k2, ...) }` — every substituted effect is a read.
Rule 13: "Read/read overlap is allowed." **Accepted, zero cost.**

**(d) Parallel sort by top-level static fanout.** Form all `2^d` sub-ranges directly
from `buf` at the top level, sort each sequentially with §7.2's iterative
`sort_copy`, then merge:

```text
p1 = partition_copy(&buf, 0, n, ctx, less)                       // sequential split
par { sort_copy(&buf[0..p1], ctx, less); sort_copy(&buf[p1..n], ctx, less) }
```

`p1`'s bounds arrive as `partition_copy`'s `ensures 0 <= result` and
`ensures result < n`, so the two ranges are provably disjoint. **Accepted, zero
cost per arm.** What is lost relative to a work-stealing recursive parallel sort is
load balance: the fanout is fixed at the top, so a skewed partition leaves one arm
long. That is the same Amdahl structure as any parallel quicksort whose first
partition pass is sequential, and the standard mitigation (sample pivots, do the
first `d` partition levels sequentially, then fan out over `2^d` provably ordered
split points `0 <= p1 <= p2 <= ... <= n`) is expressible here because every one of
those disjointness premises is affine.

**(e) Parallel reduction with per-arm accumulators.** Rule 13 names "admitted
reductions" among the counted-loop forms. Each arm writes its own accumulator and a
combine step follows. Parity with rayon.

### 10.2 Refused, with the rewrite and its cost

**(a) A shared writable context across arms.** `par { map_inplace_acc(&buf[0..mid], &acc, f); map_inplace_acc(&buf[mid..n], &acc, f) }`
substitutes to `writes(acc)` in both arms. Rule 13 refuses. The rewrite is 10.1(e),
per-arm accumulators plus a combine — **zero cost**, parity with rayon's `fold`/`reduce`.

**(b) Parallel filter into one output.** Each arm's output length is data-determined,
so the output sub-ranges are not known in advance and cannot be proved disjoint. The
rewrite is the standard two-pass form: a parallel counting pass, a sequential prefix
sum over `k` counts, then a parallel scatter into now-provably-disjoint output
ranges. Cost: **one extra full read pass and one extra predicate evaluation per
element**, or `n` stored predicate bits to avoid re-evaluating. This is exactly what
parallel STL and rayon's `par_iter().filter().collect()` do, so it is **parity**.

**(c) Parallel in-place partition.** Both arms write the same range. The rewrite is
block-based partitioning into a separate output buffer with prefix sums: **+1 buffer
of `n * sizeof(T)` and `+n` moves**, again the parallel-STL shape, **parity**.

**(d) Parallel scatter through a data-determined index array.** Refused, no rewrite.
See §9. Parity with safe Rust, a real loss against C++.

**(e) Parallel recursion inside a generic.** Gated on G3, and recoverable at zero
cost by 10.1(d).

### 10.3 Where Rule 16 enters, and only here

**Rewrite B of §8.3.** If the payload objects live in an arena and the sorted
container holds `u64` handles rather than owning values, then the element type of
the sorted container is Copy and every algorithm in §7 applies unchanged and at zero
cost — the exchange problem of §4 disappears because `u64` can be copied.

```text
struct Arena<T> { buf: DynBox<T> }                                // Rule 16, verbatim
fn alloc<T>(a: &Arena<T>, x: own T) -> u64   writes(a.buf)
    contract { requires len_of(a.buf) < cap_of(a.buf);
               ensures result == len_of(deref(entry(a.buf)));
               ensures len_of(a.buf) == result + 1; }

ids: DynBox<u64>                                                  // Copy elements: sortable at zero cost
sort_copy(&ids, &arena, less_by_arena_key)
```

This is Rule 16 as written — "A pool is a DynBox plus indices used as handles" — and
it is the *only* place in this task where the rule is used. Its own recorded price
applies: one dependent load per payload access thereafter, plus one bounds branch
per dereference unless a contract carries the bound as in §6.2(c).

The dependence is asymmetric and worth stating precisely, because it is what §12's
Rule 16 marking rests on:

- **Rules 1-15 alone** give: the entire Copy-element surface (§7), the generic
  boundary mechanism (§5, §6), all admitted parallelism (§10.1), the in-order drain
  (§8.1), both filter forms (§8.2), and Rewrite A of §8.3.
- **Rule 16 adds**: the only rewrite that restores a zero-cost sort over an owning
  payload type. Without it, sorting a container of owning values costs §8.3's full
  Rewrite A; with it, the sort is free and the cost moves to payload dereference.
- **Rule 16 is also the rule set's only statement about handle identity after a
  mutation** — "A stale index that is still in bounds names the current occupant of
  that slot: a logic error, not a memory error" — which matters here because a
  binary-search result index is renumbered by any subsequent filter. Rule 11 already
  kills the *bounds* fact when the container is written, so there is no memory
  question; without Rule 16 the candidate simply has no stated position on what the
  stale index means.

---

## 11. Rule gaps

Four, each with the exact sentence and the cost on both readings.

**G1 — may a function value passed as an argument capture a reference?**

Rule 4's heading and clause:

> "Rule 4. References are never stored, returned, or captured into anything that escapes"
>
> "It cannot be assigned into any aggregate (Rule 1), cannot be returned, and cannot be captured by a function value that is stored or returned."

Rule 1:

> "Every struct, enum, tuple, array, slice-like value, Box, DynBox, and generic instantiation holds only owned values. This is recursive and closed under wrapping: there is no type parameter, wrapper, or variant payload through which a reference can be stored."

Missing: the case of a function value that is neither stored nor returned — passed
directly as an argument to `sort_copy` or `map_inplace` and consumed there. Rule 4's
clause names only the stored and returned cases; Rule 1's list does not name a
function value among its aggregates but its closure clause ("no type parameter,
wrapper, or variant payload") reads as if it should. **`undecided-rule-gap`.**

**Cost: zero on either reading.** Every signature in this file threads context as an
explicit reference parameter, which is one machine word in one register — the same
word a captured environment pointer would be. §2.1. The permissive reading would
only shorten the source, and protocol item 7 says that is not a cost.

**G2 — does a halving assignment yield affine bounds on its result?**

Rule 11:

> "Facts are the existing WF forms: affine comparisons over measures and integer values, refinement facts from a dominating branch, loop-header invariants `invariant name: affine_expr compare_op affine_expr`, explicit `use` steps inside an `invariant`, and callee contracts."

Missing: whether `mid = lo + (hi - lo) / 2`, whose right-hand side is not an affine
expression, establishes `lo <= mid` and `mid < hi` when `lo < hi`. The *facts*
wanted are affine comparisons over integer values and are plainly in the language;
what the sentence does not settle is whether a non-affine assignment produces them,
and it defers the answer to "the existing WF forms" without naming them.
**`undecided-rule-gap`.** Only `lower_bound` (§7.3) touches it; the chunk arithmetic
of §10.1(b) uses multiplication only and is unaffected.

**Cost: zero under the permissive reading or via the `halve` contract of §7.3;
otherwise one compare and one predicted branch per probe, about `log2 n` per search,
via Rule 7's stated fallback "when the proof is unavailable the program tests the
measure, which is ordinary data."**

**G3 — may a range reference be re-ranged?**

Rule 7:

> "Indexable things: an inline `array<T, N>` (constant length, always fully initialized), a `DynBox<T>` (Rule 6), a range reference into either, and a `const` table."

Missing: whether "either" extends to a range reference into a *range reference*. The
recursive form of a generic divide-and-conquer sort needs it — a callee holding
`part: &DynBox<T>` that is itself a range wants `&part[0..m]` and `&part[m..len_of(part)]`
to hand to its recursive calls. Rule 2's path grammar admits `[lo..hi]` as a path
continuation generally, which pulls the other way. **`undecided-rule-gap`.**

**Cost: zero, via §7.2 and §10.1(d).** Under the pessimistic reading the sequential
sort is written iteratively over `part[k]` with an inline stack (no allocation), and
the parallel sub-ranges are formed directly from the container at the top level. The
sole residue is load balance under static fanout, discussed in §10.1(d), and the
sampled-pivot mitigation restores it. `rejected-zero-cost` for this gap.

**G4 — may the atomic update's function take further arguments?**

Rule 6:

> `buf[k] = f(buf[k])                   // atomic in-place update: the old value goes into f by value, f's result is committed, no program point lies between; requires k < len_of(buf)`

Missing: whether `buf[k] = f(buf[k], ctx)` and `buf[k] = f(buf[k], move donor)` are
the same operation. The clause describes the semantics of the old value and the
result and says nothing about arity, but the schema shows exactly one argument.
**`undecided-rule-gap`**, and it is the only gap here with a cost attached.

**Cost.** Under the permissive reading: zero, and §5's `map_inplace` and §4.2's
donor construction both stand as written. Under the strict reading: a contextual,
*consuming* in-place map over a non-Copy element type becomes inexpressible — the
alternative shape `f(&buf[i], ctx) writes(x)` mutates through a reference and so
cannot consume the old value to build the new one, and §2.1 forbids folding the
context into a closure. The rewrite is to build a second `DynBox` with §7.4's
`map_new` and replace: **+1 allocation of `n * sizeof(T)` and `+n` moves**, where the
in-place update is zero of both. For a Copy element type the strict reading costs
nothing, since `f(&buf[i], ctx) writes(x)` works there.

---

## 12. Rules consistent, and task achievable — recorded separately

Protocol item 5 requires these apart.

### 12.1 Rules consistent for this task: yes

Every acceptance and every rejection above is decided by a quoted clause, and no two
clauses were found to contradict each other on this task's programs. One apparent
tension was examined and resolves:

Rule 9 says "`writes` covers writing, replacing, moving out of, and freeing the
storage at the path", while Rule 6 says "the only ways to move a value out of storage
are consuming a whole local (`move x`), the window operations, and the atomic
update." Read together, Rule 9 describes the *breadth an effect entry must have* when
one of Rule 6's licensed operations moves a value out — `pop(&buf)` moves out of
`buf[len-1]` and declares `writes(buf)` — and does not itself license a writer-authored
move-out of a reference target. §4.2(c) depends on this reading; under the opposite
reading `exchange` would be writable as
`fn exch<T>(old: own T, cell: &T) -> own T writes(cell)` and §4's whole negative
result would vanish. The reading taken is the one Rule 6 states in the imperative
("the only ways are"), and Rule 12 independently confirms it: "No place is ever
partially moved: every path is either wholly present or the program cannot name it."

The four gaps of §11 are underspecification, not inconsistency: in each case the rule
file is silent, not self-contradicting, and in three of the four the silence costs
nothing.

### 12.2 Task achievable: yes, with cost

The assignment asks for four algorithms plus map/filter, written once with
function-typed parameters carrying effect rows and contracts, and for the bounds and
disjointness story.

| Sub-task | Copy element type | Non-Copy element type |
|---|---|---|
| map, in place | derived, zero cost (§5) | derived, zero cost under G4's permissive reading; +1 alloc and +`n` moves under the strict one |
| map, to a new type | derived, zero cost (§7.4) | derived, zero cost |
| filter, stable | derived, zero cost (§7.5) | derived, +1 alloc, +`n·sizeof(T)` peak, ≈+1.5`n` moves (§8.2) |
| filter, unstable | derived, zero cost | derived, zero cost (§8.2) |
| partition | derived; Lomuto zero cost, unguarded Hoare refused at +1 branch/element (§7.1) | **not expressible in place** (§4); Rewrite A or B |
| sort | derived, zero cost modulo the partition guard (§7.2) | **not expressible in place** (§4); Rewrite A or B |
| binary search | derived; zero cost, or +1 branch/probe under G2's pessimistic reading (§7.3) | derived, same |
| bounds across the boundary | derived, zero cost, both directions (§6.2) | same |
| disjointness across the boundary | derived, zero cost; decided entirely at the caller (§5.3) | same |
| parallel map / chunked map / parallel search / parallel sort | admitted, zero cost (§10.1) | same, over handles |

So: **achievable with cost.** The generic-boundary half of the assignment — the part
the task text emphasizes, "show that the bounds and disjointness facts flow through
the generic boundary and what a caller must supply" — is achieved at **zero** runtime
cost and with a clean mechanism. The cost is entirely in the element-class half:
in-place reordering of owning values.

### 12.3 Cost table

Per operation, against the baselines named at the top. "-" means no difference.

| Operation | Extra loads | Extra branches | Extra copies / moves | Extra allocations | Lost parallelism | Against |
|---|---|---|---|---|---|---|
| map in place, Copy or non-Copy (G4 permissive) | - | - | - | - | - | both |
| map in place, non-Copy, G4 strict | - | - | +`n` moves | +1 of `n·sizeof(T)` | - | both |
| map to a new type | - | - | - | - | - | both, if the `len_of` store is hoisted; else +1 store/element vs Rust `collect` |
| filter, Copy, stable, in place | - | - | - | - | - | both |
| filter, non-Copy, stable | - | - | ≈+1.5`n` moves | +1 of `n·sizeof(T)` | - | vs `Vec::retain` |
| filter, non-Copy, unstable | - | - | - | - | - | both |
| partition, Copy, Lomuto | - | - | - | - | - | vs Rust pdqsort; vs libstdc++ Lomuto |
| partition, Copy, scan guard | - | +1 cmp+branch per scanned element | - | - | - | vs libstdc++ `__unguarded_partition`; parity vs Rust |
| sort, Copy | - | inherits the partition guard, ≈`n log n` branches | - | - | - | vs C++; parity vs Rust |
| sort, non-Copy, Rewrite A comparison phase | +1 dependent load/comparison, plus loss of sequential locality | +1 bounds branch/comparison | - | +1 of `8n` | - | both |
| sort, non-Copy, Rewrite A apply phase | +1 dependent load/element | +1 bounds branch/element | +2 stores and +1 relocation per element | +2 of `8n`, +1 of `n·sizeof(T)` | - | both |
| sort, non-Copy, Rewrite B (Rule 16) | +1 dependent load per payload access thereafter | +1 bounds branch per payload access unless carried by §6.2(c) | - | - | - | both |
| binary search, G2 permissive or via `halve` | - | - | - | - | - | both |
| binary search, G2 pessimistic | - | +1 cmp+branch per probe, ≈`log2 n` per search | - | - | - | both |
| generic scatter, index bound via §6.2(c) | - | - | - | - | - | vs C++; better than safe Rust |
| generic scatter, no such contract | - | +1 bounds branch/element | - | - | - | vs C++; parity vs safe Rust |
| parallel map over adjacent ranges | - | - | - | - | - | both |
| parallel `k`-worker chunking | - | - | - | - | - | both |
| parallel filter | - | +1 predicate evaluation per element (or `n` stored bits) | - | - | - | parity both |
| parallel in-place partition | - | - | +`n` moves | +1 of `n·sizeof(T)` | - | parity both |
| parallel scatter, data-determined indices | - | - | - | - | **all of it** | vs C++; parity vs safe Rust |
| parallel recursive divide-and-conquer (G3 pessimistic) | - | - | - | - | load balance under static fanout only | both |
| call through a function-typed parameter | - | - | - | - | - | parity, conditional on the compiler specializing a statically known callee (§7.6) |

### 12.4 Dependence on Rule 16

**Dependent: yes, and confined to §10.3.**

The four algorithms, the generic boundary mechanism, the bounds and disjointness
story, the parallelism, and the in-order drain all derive under Rules 1-15 with no
use of Rule 16, and this is stated explicitly at the end of §10.3. Rule 16 is used in
exactly one place, for one purpose: it is the only rewrite that restores a **zero-cost
sort over an owning payload type**, by putting the payloads in an arena and sorting
Copy handles. Without Rule 16 that route is unavailable and the only remaining route
is §8.3 Rewrite A, whose cost is the largest single entry in the table above. Rule 16
is also the rule set's only statement about what a stale-but-in-bounds index means
after a filter renumbers the container, which a binary-search result index becomes.

### 12.5 Verdicts

| Item | Verdict |
|---|---|
| Generic surface with function-typed parameters carrying rows and contracts | **`accepted-fine`** |
| Bounds facts across the generic boundary, both directions | **`accepted-fine`** |
| Disjointness across the generic boundary, decided at the caller | **`accepted-fine`** |
| map in place, map to a new type, unstable filter | **`accepted-fine`** |
| Filter, Copy elements, stable, in place | **`accepted-fine`** |
| Filter, non-Copy elements, stable | **`rejected-real-cost`** (+1 alloc, ≈+1.5`n` moves) |
| Partition and sort, Copy elements | **`accepted-fine`** |
| Unguarded sentinel partition | **`rejected-real-cost`** (+1 branch per scanned element vs libstdc++) |
| Partition and sort, non-Copy elements, in place | **`rejected-real-cost`** — the §9 counterexample |
| Binary search | **`undecided-rule-gap`** (G2); `accepted-fine` under the permissive reading or the `halve` contract |
| Parallel map, chunked map, parallel search, parallel sort by top-level fanout | **`accepted-fine`** |
| Parallel scatter through data-determined indices | **`rejected-real-cost`** vs C++, parity vs safe Rust |
| Recursive `par` inside a generic | **`undecided-rule-gap`** (G3); `rejected-zero-cost` under the pessimistic reading |
| Closure capture of a reference | **`undecided-rule-gap`** (G1); zero cost either way |
| Atomic update with extra arguments | **`undecided-rule-gap`** (G4) |
| **Overall task** | **`accepted-fine`, achievable with cost.** The generic-boundary machinery is free; the cost is concentrated in in-place reordering of owning element types, and §9 argues that cost is removable by one addition to Rule 6's window operations. |
