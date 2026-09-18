# Engineering task under candidate x1, revision 4 — generic algorithms through function-typed parameters

Round two. Derived against the frozen rule set in `CANDIDATE-X1.md` (revision 4,
2026-09-18) and only that file. Round one of this task was derived against
revision 2; this file says at §1 exactly which of its conclusions revision 4
withdraws, which it keeps, and which are new, and then re-derives the program from
scratch in revision 4's notation so that nothing here has to be read against the
older file.

Task: *Generic algorithms: in-place sort with `swap`, binary search, partition, and
a map/filter over `Slots` written once with function-typed parameters carrying
effect rows and contracts, operating on `&[T]` range references; show that the
bounds and disjointness facts flow through the generic boundary and what a caller
must supply.*

Cost convention, from the protocol's item 7: runtime performance is the only cost.
Verbosity, extra parameters and duplicated code are not costs. Costs are counted in
**loads and stores of `sizeof(T)` bytes**, not in "moves", because Rule 1's stated
consequence — "any value can be relocated by copying its bytes (memmove, realloc),
because nothing inside it points anywhere" — makes every relocation a plain byte
copy with no fixup, so a three-statement exchange and a two-statement hole rotation
are directly comparable at the memory-traffic level and differ by exactly one load
and one store.

Baselines. **C++** means libstdc++ `std::sort` (introsort: pdqsort-style partition,
`__unguarded_partition`, final insertion sort with a moved-out temp),
`std::lower_bound`, `std::partition`, `std::unique`, `std::remove_if`, with an
inlined functor. **Rust** means `slice::sort_unstable`, `slice::binary_search_by`,
`Vec::retain`, `Vec::swap_remove`, `Vec::insert`, `Iterator::map().collect()` with a
monomorphized closure and no `get_unchecked`. Where the two differ, the table says
which one a cost is measured against, and whether safe Rust (user code without
`unsafe`) or Rust's standard library (which uses `unsafe` internally) is meant.

---

## 1. What revision 4 changes for this task

| Round-one conclusion | Status under revision 4 | Ground |
|---|---|---|
| **"There is no two-slot exchange for any non-Copy `T`."** In-place sort, in-place partition and stable in-place compaction were held inexpressible for every affine and linear element type; that was the load-bearing negative result and the whole counterexample. | **Withdrawn.** All three are now expressible, in place, with no allocation. | Rule 6: `swap(p: &T, q: &T) writes(p), writes(q) // built in; p and q may be the same place, then nothing happens`, applying "to any owned place, not only to window slots". |
| The partition loop needed an `i != j` fact (or a branch) before any exchange. | **Withdrawn.** No such fact and no branch. | Rule 6: `swap(&r[i], &r[j]) // no i != j branch needed in a partition loop`; Rule 10 clause 1: "The built-in `swap` is the one operation whose two arguments may be the same place." |
| Round one's **G3** — may a range reference be re-ranged, so that a recursive divide-and-conquer sort can hand sub-ranges to its recursive calls? Left `undecided-rule-gap`; the sort was written iteratively with an inline stack to avoid it. | **Closed, permissively.** The recursive form is written directly in §3. | Rule 7: `&x[lo..hi]` "has the parameter type `&[T]`: a reference kind with measure `len`, formed only from an indexable or from another range reference", with the worked line `sub = &part[a..b] // requires a <= b <= part.len`. |
| Round one's **G4** — may the atomic in-place update's function take further arguments, so that a contextual in-place map over a non-Copy element type is expressible? Left `undecided-rule-gap` with a `+1 allocation, +n moves` cost attached to the strict reading. | **Closed, permissively.** `part[k] = f(part[k], ctx)` is written directly and the cost disappears. | Rule 6: `place = f(place, args...) writes(place) // atomic in-place update`. |
| Round one's **G1** (may a function value that is passed but neither stored nor returned capture a reference?) and **G2** (does `mid = lo + (hi - lo) / 2` yield affine bounds on `mid`?). | **Both still open, both still zero cost.** Restated at §8 with their unchanged sentences. | Rule 4 and Rule 1 name only the stored and returned cases; Rule 11's fact list is unchanged. |
| Every reordering signature carried `ensures len_of(buf) == len_of(deref(entry(buf)))`. | **Gone.** A range-generic function needs no length-preservation clause at all: no operation in the language takes a `&[T]`, so `part.len` is a constant of the function body. | Rule 7 gives `&[T]` the single measure `len`; every length-changing operation in Rule 6 is declared on `&r` (a `Slots`/`Ring`) or `&b` (a `Box<Slots<T>>`). |
| `map_new` declared `writes(dst)`, so every caller reference into `dst` died across the call. | **Improved to zero-cost.** `map_into` declares `writes(dst.free), writes(dst.len)`, and a caller's reference into the already-filled part of `dst` survives the whole map. | Rule 6: "a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`". |
| `swap_remove` and `truncate` were built-ins whose internals the derivation could not see. | **Now derivable in source**, and `swap_remove` needs no `k != r.len - 1` branch. | Rule 6 lists them as "zero-cost compositions of the above"; §3.7 writes one out. |
| `par { A; B }` notation. | **Gone.** §5 writes two adjacent statements and says whether they may overlap. | Rule 13; "Not in this candidate": "a `par` statement". |
| Ordered insertion had to be hand-shifted; concatenation had to be a push loop. | **Parity with `Vec::insert` and `Vec::extend`.** | Rule 6: `insert_at` and `append`, each "one memmove" / "one memcpy". |
| Counterexample: the missing exchange. | **Replaced.** §7's counterexample is the in-place *fold* — the atomic update may not read a second element of the container it updates, which blocks `std::unique`-with-merge over a non-Copy element type. Narrower than round one's, and still unrescuable at zero cost. | Rule 6: "f's row must not overlap any prefix of place"; Rule 6: "A move out of a window slot or an array element is rejected". |
| Verdicts: partition and sort over non-Copy elements `rejected-real-cost`; stable filter over non-Copy elements `rejected-real-cost` (+1 allocation, ≈+1.5n moves). | **Both become `accepted-fine`** and **`rejected-real-cost` at +1 load +1 store per relocated element** respectively. | §6's table. |
| New gaps found only in round two: **G5**, whether a supplied function argument must match the function-typed parameter's declared signature exactly; **G6**, whether a window part such as `r.last` may be used as a *place*, not only inside an effect row. | New, both zero cost. | §8. |

Unchanged from round one and not re-argued here beyond a one-line restatement: the
generic boundary mechanism itself (§4), the refusal of the unguarded sentinel
partition (§6, still the largest surviving cost), the absence of element-level facts,
and the conditional status of the indirect call through a function-typed parameter.

---

## 2. The question this task tests

Whether one generic surface — four reordering and searching algorithms plus a map
and a filter, each written once against `&[T]` or `&Slots<T, N>` and a function-typed
parameter — type-checks with bounds and disjointness carried entirely by declared
rows and contracts, and whether the element's value class still costs anything now
that `swap` exists.

Three sub-questions, with three different answers:

1. **Does the generic need to see its callback's body?** No, at zero cost; every
   aliasing question is decided at the caller by substitution. §4.
2. **Do bounds facts cross the boundary in both directions?** Yes, at zero cost:
   outward as an `ensures` on a returned index, inward as an `ensures` on a
   callback, internally from a loop invariant over the parameter's own measure. §4.3.
3. **Does the element's value class still cost anything?** Almost nothing. `swap`
   removes the whole of round one's negative result. What remains is one asymmetry:
   `swap` is an *exchange*, and three C++/Rust idioms (insertion-sort rotation,
   hole-based compaction, in-place fold) are written with a *moved-out temporary*
   that this candidate has no way to create for a non-Copy `T`. §6, §7.

---

## 3. The program

All of it, in revision 4's notation. Measures are pseudo-fields (`part.len`,
`r.room`, `entry(r).len`). Every function carries a row; contracts appear where a
caller or a callee needs the relation.

### 3.1 Types

```text
struct Key   { hi: u64, lo: u64 }                       // copy (Rule 8)
struct Node  { key: u64, bytes: Box<Slots<u8>> }        // affine by containment (Rule 8)
linear type File;   fn close(f: own File)
struct Conn  { f: File, id: u64 }                       // linear by containment (Rule 8)

struct Ranker { weights: Box<Slots<u64>>, bias: u64 }   // a context; Rule 1: owned fields only
struct Counts { kept: u64, dropped: u64 }               // an accumulating context
```

### 3.2 Range-generic: partition

```text
fn partition_range<T, C>(
        part: &[T],
        ctx:  &C,
        less: fn(a: &T, b: &T, c: &C) -> Bool  reads(a), reads(b), reads(c))
    -> u64
    writes(part), reads(ctx)
    contract {
        requires part.len > 0;
        ensures  result < part.len;
    }
{
    piv = part.len - 1              // Lomuto: the pivot sits at the last position
    w = 0
    k = 0
    while k < piv {
        invariant p1: w <= k;
        invariant p2: k <= piv;
        invariant p3: piv < part.len;
        if less(&part[k], &part[piv], ctx) {
            swap(&part[w], &part[k])
            w = w + 1
        }
        k = k + 1
    }
    swap(&part[w], &part[piv])
    w
}
```

### 3.3 Range-generic: insertion sort and quicksort

```text
fn insertion_sort<T, C>(
        part: &[T],
        ctx:  &C,
        less: fn(a: &T, b: &T, c: &C) -> Bool  reads(a), reads(b), reads(c))
    writes(part), reads(ctx)
{
    i = 1
    while i < part.len {
        invariant i1: i <= part.len;
        j = i
        while j > 0 && less(&part[j], &part[j - 1], ctx) {
            invariant j1: j <= i;
            invariant j2: i < part.len;
            swap(&part[j], &part[j - 1])
            j = j - 1
        }
        i = i + 1
    }
}

fn sort_range<T, C>(
        part: &[T],
        ctx:  &C,
        less: fn(a: &T, b: &T, c: &C) -> Bool  reads(a), reads(b), reads(c))
    writes(part), reads(ctx)
{
    if part.len <= 16 {
        insertion_sort(part, ctx, less)
    } else {
        m = partition_range(part, ctx, less)        // m < part.len, from its ensures
        lo_half = &part[0..m]
        hi_half = &part[m + 1..part.len]
        sort_range(lo_half, ctx, less)              // these two statements
        sort_range(hi_half, ctx, less)              // may overlap; see §5.1
    }
}
```

### 3.4 Range-generic: binary search, and the one division in the program

```text
fn halve(w: u64) -> u64
    contract { requires w > 0; ensures result < w; ensures result + result <= w; }

fn lower_bound<T, K, C>(
        part: &[T],
        key:  &K,
        ctx:  &C,
        less_kv: fn(a: &T, k: &K, c: &C) -> Bool  reads(a), reads(k), reads(c))
    -> u64
    reads(part), reads(key), reads(ctx)
    contract { ensures result <= part.len; }
{
    lo = 0
    hi = part.len
    while lo < hi {
        invariant b1: lo <= hi;
        invariant b2: hi <= part.len;
        h   = halve(hi - lo)            // h < hi - lo, purely affine from here on
        mid = lo + h                    // lo <= mid and mid < hi by affine arithmetic
        if less_kv(&part[mid], key, ctx) { lo = mid + 1 } else { hi = mid }
    }
    lo
}
```

### 3.5 Range-generic: map in place

```text
fn map_inplace<T, C>(
        part: &[T],
        ctx:  &C,
        f: fn(v: own T, c: &C) -> own T  reads(c))
    writes(part), reads(ctx)
{
    k = 0
    while k < part.len {
        invariant m1: k <= part.len;
        part[k] = f(part[k], ctx)       // Rule 6's atomic in-place update
        k = k + 1
    }
}

fn map_inplace_acc<T, C>(
        part: &[T],
        ctx:  &C,
        f: fn(v: own T, c: &C) -> own T  writes(c))
    writes(part), writes(ctx)
{ ... the same body; only the callback's row and the generic's row differ ... }
```

### 3.6 Container-generic: map into a fresh window

`place_back` needs `r.room`, and a `&[T]` has only `len` (Rule 7), so anything that
changes a length takes the container.

```text
fn map_into<T, U, C>(
        src: &[T],
        dst: &Slots<U, N>,
        ctx: &C,
        f: fn(v: &T, c: &C) -> own U  reads(v), reads(c))
    reads(src), writes(dst.free), writes(dst.len), reads(ctx)
    contract {
        requires dst.room >= src.len;
        ensures  dst.len == entry(dst).len + src.len;
    }
{
    k = 0
    while k < src.len {
        invariant n1: dst.len == entry(dst).len + k;
        invariant n2: k <= src.len;
        place_back(&dst, f(&src[k], ctx))
        k = k + 1
    }
}
```

### 3.7 Container-generic: filter, stable and unstable, and `swap_remove` written out

```text
// swap_remove, as library code (Rule 6 lists it among the "zero-cost compositions").
fn swap_remove<T>(r: &Slots<T, N>, k: u64) -> own T
    writes(r.filled), writes(r.last), writes(r.len)
    contract { requires k < r.len; ensures r.len == entry(r).len - 1; }
{
    swap(&r[k], &r.last)        // no k != r.len - 1 branch: Rule 10 clause 1's swap exemption
    take_back(&r)
}

// Stable, in place, no allocation, all three value classes.
fn retain<T, C>(
        r:   &Slots<T, N>,
        ctx: &C,
        keep:    fn(v: &T, c: &C) -> Bool  reads(v), reads(c),
        discard: fn(v: own T, c: &C)       writes(c))
    writes(r.filled), writes(r.len), writes(ctx)
    contract { ensures r.len <= entry(r).len; }
{
    n = r.len
    w = 0
    k = 0
    while k < n {
        invariant f1: w <= k;
        invariant f2: k <= n;
        invariant f3: r.len == n;
        if keep(&r[k], ctx) {
            if w != k { swap(&r[w], &r[k]) }    // see §6 on why the guard is written
            w = w + 1
        }
        k = k + 1
    }
    while r.len > w {                            // the tail now holds exactly the discards
        invariant f4: w <= r.len;
        discard(take_back(&r), ctx)              // linear T is consumed here, as Rule 8 demands
    }
}

// Unstable, one pass, one relocation per removed element.
fn retain_unstable<T, C>(
        r:   &Slots<T, N>,
        ctx: &C,
        keep:    fn(v: &T, c: &C) -> Bool  reads(v), reads(c),
        discard: fn(v: own T, c: &C)       writes(c))
    writes(r.filled), writes(r.last), writes(r.len), writes(ctx)
{
    k = 0
    while k < r.len {
        invariant u1: k <= r.len;
        if keep(&r[k], ctx) { k = k + 1 }
        else { discard(swap_remove(&r, k), ctx) }
    }
}
```

### 3.8 The two generics that show the boundary carrying a fact inward

```text
// Adjacent-duplicate removal: the callback reads two elements of the container the
// generic writes. Accepted; see §4.4.
fn unique_by<T, C>(
        r:   &Slots<T, N>,
        ctx: &C,
        same:    fn(a: &T, b: &T, c: &C) -> Bool  reads(a), reads(b), reads(c),
        discard: fn(v: own T, c: &C)              writes(c))
    writes(r.filled), writes(r.len), writes(ctx)
{
    if r.len > 0 {
        n = r.len
        w = 0
        k = 1
        while k < n {
            invariant q1: w < k;
            invariant q2: k <= n;
            invariant q3: r.len == n;
            if !same(&r[w], &r[k], ctx) {        // reads/reads: no w != k fact needed
                w = w + 1
                if w != k { swap(&r[w], &r[k]) }
            }
            k = k + 1
        }
        while r.len > w + 1 { discard(take_back(&r), ctx) }
    }
}

// Scatter: the callback's contract supplies the bound the generic cannot derive.
fn scatter<T, U, N>(
        src: &[T],
        dst: &Slots<U, N>,
        slot_of: fn(v: &T, d: &Slots<U, N>) -> u64  reads(v), reads(d)
                     contract { ensures result < d.len; },
        make:    fn(v: &T) -> own U  reads(v))
    reads(src), writes(dst.filled)
{
    k = 0
    while k < src.len {
        invariant s1: k <= src.len;
        j = slot_of(&src[k], &dst)      // j < dst.len arrives as a callee contract
        set(&dst[j], make(&src[k]))     // Rule 7's index premise discharged by that contract
        k = k + 1
    }
}
```

---

## 4. Trace, at the interesting points only

### 4.1 `partition_range`: bounds, swap, and the callback

**Rule 7, the three indices.** "Every index must be proved in bounds". `p3`
(`piv < part.len`) holds at entry from `requires part.len > 0` and `piv = part.len - 1`,
both affine, and holds at every iteration because **nothing in the body can change
`part.len`**: every length-changing operation in Rule 6 is declared on `&r` or `&b`,
never on a `&[T]`, and Rule 7 gives `&[T]` exactly one measure, `len`, on a "reference
kind ... formed only from an indexable or from another range reference". `p2` refined
by the loop test gives `k < piv`; `p1` gives `w <= k`. So `w <= k < piv < part.len`
and all of `part[w]`, `part[k]`, `part[piv]` are in bounds. `p1`, `p2`, `p3` are Rule
11's admitted header form, "loop-header invariants `invariant name: affine_expr
compare_op affine_expr`", every side affine in `w`, `k`, `piv` and the measure
`part.len`.

At the final `swap(&part[w], &part[piv])` the loop has exited with `k >= piv`, and
with `p2` that gives `k == piv`, so `w <= piv < part.len`.

**Rule 6, the exchange.** `swap(&part[w], &part[k])` is the built-in, whose row is
`writes(p), writes(q)`, substituted here to `writes(part[w]), writes(part[k])`. Rule
10 clause 1 would normally demand `w != k`, and the loop does not have it when `w == k`
(the common case in a nearly-sorted prefix). It is not needed: clause 1 says "The
built-in `swap` is the one operation whose two arguments may be the same place", and
Rule 6 states the consequence for exactly this program — "`swap(&r[i], &r[j])  // no
i != j branch needed in a partition loop". **This is the sentence that makes the
whole of round one's §4 obsolete**, and it costs nothing: `swap` on the same place
"then nothing happens", so a sensible lowering of `t = a; a = b; b = t` with `a == b`
is value-preserving without any branch.

**Rule 9, coverage.** `writes(part[w])` and `writes(part[k])` are covered by the
declared `writes(part)` because `part` is a prefix of both; the callback's substituted
`reads(part[k]), reads(part[piv]), reads(ctx)` are covered by `writes(part)` (a write
permission covers the read of the same storage — the body's read of `part[k]` is at a
path the row already names) and by `reads(ctx)`. Rule 9's own requirement, "Signatures
never contain index expressions; an index enters an effect only through an argument,
evaluated once at the call", is met: the row says `writes(part)`, and `w`, `k`, `piv`
appear only in statements inside the body.

**Rule 10 clause 1, at the callback.** Substituted: `reads(part[k])`, `reads(part[piv])`,
`reads(ctx)`. "Read/read overlap is allowed", so no `k != piv` fact is needed even
though one is available. Accepted, and accepted *without knowing whether the caller
will alias `ctx` with the container the range came from* — that question is asked once,
at the caller, in §4.5.

**Rule 10, the callback's body.** "Function-typed parameters carry a full signature
with its own row and contract, and a call through one uses that row." Nothing above
consults `less`'s body.

### 4.2 `sort_range`: the recursion and the two sub-ranges

**Rule 7, forming the halves.** `&part[0..m]` requires `0 <= m <= part.len`, from
`partition_range`'s `ensures result < part.len` (Rule 11: "callee contracts").
`&part[m + 1..part.len]` requires `m + 1 <= part.len`, the same fact rearranged
affinely. Both are Rule 7's `sub = &part[a..b]` line, which revision 4 states
explicitly and revision 2 did not — this is round one's G3, closed.

**Rule 3, does the first recursive call invalidate `hi_half`?** The call substitutes
to `writes(part[0..m])`. Rule 3 invalidates a reference when "a proper prefix of p's
path is written", judged by the rule that "two indexed positions on the same storage
are taken to overlap unless their indices or ranges are proved distinct". `hi_half`'s
path is `part[m + 1..part.len]`. The written path is not a prefix of it — the common
prefix `part` is not itself written — and the two ranges are proved distinct from
`m <= m + 1`, affine. **`hi_half` survives the first recursive call.** Without this
step the recursive form would have to re-form the second half after the first call,
which is one address computation, not a cost; with it the two calls are adjacent and
independent, which §5.1 uses.

**Rule 10, the recursion itself.** "Recursion is checked through contracts, never by
unfolding bodies." `sort_range` has no `ensures`, and needs none: it cannot change
`part.len`. Rule 2's static-path-shape clause — "a loop-carried rebinding may change
only the index values inside the path, never extend the path through itself" — is
about a rebinding inside a loop and does not reach the recursion, whose callee names
a fresh parameter.

### 4.3 `map_inplace` and `map_into`: the atomic update and the window parts

**Rule 6, the update.** `part[k] = f(part[k], ctx)` is `place = f(place, args...)`,
whose row is `writes(place)` and whose side condition is that "f's row must not
overlap any prefix of place". The prefixes of `part[k]` are `part`. `f`'s declared row
is `reads(c)`, substituted to `reads(ctx)`; `ctx` and `part` are different roots, so
nothing overlaps. Accepted. In `map_inplace_acc` the row is `writes(ctx)`, still a
different root, still accepted — and the caller pays for that difference in §4.5.

That `f` here is a function-typed *parameter* rather than a named function is read off
Rule 10's "a call through one uses that row" together with Rule 6's phrasing, which
constrains `f` only by "f is total and returns the place's type" and by the
non-overlap side condition. No reading of Rule 6 restricts the update's `f` to a named
function.

**Rule 6, the window parts, in `map_into`.** `place_back`'s row is
`writes(r.next), writes(r.len)`, substituted to `writes(dst.next), writes(dst.len)`.
Rule 9's coverage check is discharged against the declared `writes(dst.free)` and
`writes(dst.len)` because `dst.next` is "the append slot, at index `r.len`" and
`dst.free` is "all slots from `r.len` up", so the first is contained in the second by
the definitions Rule 6 gives. The payoff is at the caller:

```text
p = &(*db)[0]                                   // valid: 0 < (*db).len
map_into(&src[0..n], &(*db), &ranker, build)    // writes((*db).free), writes((*db).len)
use(p)                                          // still valid
```

Rule 6: "a live `r[i]` (which has `i < r.len`) never overlaps `r.next` or `r.free`".
`p`'s path `(*db)[0]` has no proper prefix among the call's write paths under the
may-overlap judgment, so Rule 10 clause 3 does not kill it. Under round one's
`writes(dst)` it died. **New capability, zero cost, and it is the concrete reason the
window-parts vocabulary is worth having on a user-written generic**, which is exactly
what Rule 6 claims for it: "This vocabulary is ordinary: the rows below use nothing a
user function cannot write."

**Rule 11, the append invariant.** `n1: dst.len == entry(dst).len + k` is affine over
a measure and an integer. It re-establishes at the back edge from `place_back`'s own
`ensures r.len == entry(r).len + 1`, carried by Rule 11's sentence: "Across a call, a
fact known before the call about a measure of an argument survives as a fact about
`entry(p)` of that argument". `place_back`'s `requires r.room > 0` is discharged from
`n1`, `n2` refined by the loop test, and the entry `requires dst.room >= src.len`:
`dst.len - entry(dst).len = k < src.len <= entry(dst).room`.

### 4.4 `unique_by` and `scatter`: two elements of one container, and a fact arriving inward

`same(&r[w], &r[k], ctx)` substitutes to `reads(r[w]), reads(r[k]), reads(ctx)`. Two
indexed positions on the same storage, and Rule 10 clause 1's "Read/read overlap is
allowed" settles it with no `w != k` fact. So **a callback may inspect two elements of
the container the generic is mutating, provided it only reads them** — `std::unique`'s
shape, at zero cost. §7 is what happens when it needs to *write* one of them.

`scatter` shows the inward direction of fact flow. Rule 9 permits the callback's row
to name `d` because "each path starts at a reference parameter" and `d` is one of the
callback's own reference parameters; Rule 11 admits "callee contracts" as a fact form;
so `ensures result < d.len` substitutes at the call to `j < dst.len`, which is
precisely Rule 7's premise for `dst[j]`. **The bound crosses the generic boundary as a
contract clause and costs nothing at runtime.** Without it, Rule 7's stated fallback
applies — "when the proof is unavailable the program tests the measure, which is
ordinary data" — for one compare and one predicted branch per element.

Two limits, both from quoted text and both without cost. A callback's `ensures` can
mention only the callback's own parameters and result, which is why `slot_of` takes
`d`; widening a callback's parameter list is not a cost (protocol item 7). And whoever
*establishes* the fact pays once: if `slot_of`'s body is `h % d.len`, Rule 11's fact
forms do not supply a residue fact, so the body tests once and returns `0` otherwise —
the same single branch, moved inside the callback, not an extra one.

### 4.5 The caller: where aliasing is actually decided

```text
r: Ranker
v: Box<Slots<Node>>
part = &(*v)[0..(*v).len]

sort_range(part, &r, less_by_weight)                  // (i)
map_inplace_acc(part, &r, rescale_and_count)          // (ii)
map_inplace_acc(part, &(*v), fold_into_self)          // (iii)
retain(&(*v), &counts, keep_hot, drop_cold)           // (iv)
```

- **(i)** substitutes to `writes(part), reads(r)`. Rule 10 clause 1, "different
  roots". Accepted.
- **(ii)** substitutes to `writes(part), writes(r)`. Different roots. Accepted.
- **(iii)** substitutes to `writes(part), writes((*v))`. `part`'s path is
  `(*v)[0..(*v).len]`, so `(*v)` is a prefix of it: the two overlap and neither is
  proved distinct. "Two effects on overlapping paths where at least one is a write
  must be proved disjoint ... otherwise the call is rejected." **Rejected at the
  caller.**
- **(iv)** is accepted, and afterwards `part` is invalid: `retain`'s substituted row
  writes `(*v).filled` and `(*v).len`, and `(*v).filled` is "all slots below `r.len`",
  which overlaps the range `(*v)[0..(*v).len]`. Rule 10 clause 3 kills it. Re-forming
  it is one address computation and a measure load, not a cost.

This is the whole interface. The generic's body was checked once under the assumption
that its reference parameters are different roots; the caller that violates that
assumption is caught by substituting the generic's *own declared row*, one line the
caller reads. The generic's body is never re-examined and the callback's body is never
examined at all.

### 4.6 What crosses the boundary, closed list

| Fact | Direction | Crosses? | Ground |
|---|---|---|---|
| A measure of a reference parameter (`part.len`, `r.room`) | both | yes | Rule 11, "affine comparisons over measures" |
| A relation in an `ensures`, including `entry`-relative ones | outward | yes | Rule 11, "callee contracts"; Rule 6's `place_back` contract is the file's own example |
| A result index's bound (`ensures result < part.len`) | outward | yes | Rule 4's `find` signature |
| A bound established by a *callback*'s `ensures` | inward | yes | §4.4's `scatter` |
| Variant-routed relations | outward | yes | Rule 11, "`ensures when Variant:`" |
| Disjointness of the actual argument paths | inward only, as an obligation | yes | Rule 10 clause 1; never derived inside a generic |
| Anything about element *values* — sortedness, injectivity of an index array, "some element ahead exceeds the pivot" | neither | **no** | Rule 11: "There are no quantified facts over array elements ("for all i ...") and no per-slot occupancy facts" |
| A reference into a container | outward | **no** | Rule 4: a reference "cannot be returned" |

**What a caller must supply**, exactly three things: the range-formation premises
(`lo <= hi <= r.len`), the disjointness of every path it passes against the generic's
substituted row, and any `requires` in the generic's contract (`part.len > 0`,
`dst.room >= src.len`). Nothing else, and none of the three survives into lowered
code.

---

## 5. Overlaps the rules admit and refuse

Rule 13: "Two adjacent statements of one block may overlap when the first's write
paths are disjoint from the second's read and write paths and vice versa, using the
same path-overlap and index/range-disjointness judgment as Rule 10". Written as two
adjacent statements; there is no `par`.

### 5.1 Admitted

```text
sort_range(&part[0..m], ctx, less)                     // may overlap with the next statement:
sort_range(&part[m + 1..part.len], ctx, less)          // ranges proved distinct from m <= m + 1;
                                                       // reads(ctx) meets reads(ctx), allowed

i = lower_bound(part, &k1, ctx, less_kv)               // may overlap: every substituted effect
j = lower_bound(part, &k2, ctx, less_kv)               // is a read

map_inplace(&part[0..mid], ctx, f)                     // may overlap: disjoint ranges,
map_inplace(&part[mid..part.len], ctx, f)              // shared read-only context

map_into(&src[0..mid], &(*d1), ctx, build)             // may overlap: different dst roots;
map_into(&src[mid..src.len], &(*d2), ctx, build)       // reads(src) meets reads(src), allowed

lo_box = Box::new(Slots::new<U>(mid))?                 // may overlap: "Allocation and release
hi_box = Box::new(Slots::new<U>(n - mid))?             // are not effects" (Rule 14)
```

Chunking to `k` workers uses only multiplication and addition (`lo = w * s; hi = lo + s`),
so every disjointness premise is affine and the division question of §8 never arises.

### 5.2 Refused, with the best rewrite and its cost

| Refused pair | Why | Best rewrite | Cost |
|---|---|---|---|
| `map_inplace_acc(&part[0..mid], &acc, f)` ; `map_inplace_acc(&part[mid..n], &acc, f)` | `writes(acc)` in both | per-arm accumulator, then a combine statement | zero; parity with rayon `fold`/`reduce` |
| `map_into(&src[0..mid], &(*d), ...)` ; `map_into(&src[mid..n], &(*d), ...)` | both write `(*d).free` and `(*d).len` | two windows, then `append(&(*d1), &(*d2))` | **one memcpy of the second half** (Rule 6: `append` is "one memcpy"); parity with rayon `collect`, and strictly better than round one, which had no `append` and re-pushed element by element |
| `retain(&(*v), ...)` ; `s = stats(&(*v))` | `writes((*v).filled)` meets `reads((*v))` | none — the dependence is real | none to pay; the sequential order is the meaning |
| `sort_range(&part[0..m], ...)` ; `j = lower_bound(part, ...)` | `writes(part[0..m])` meets `reads(part)` | search the untouched half, or sequence them | zero |
| two arms partitioning one range in place | both write `part` | block partition into a scratch window plus a prefix sum | +1 window of `n * sizeof(T)`, +1 load +1 store per element; parity with parallel STL |
| parallel filter into one window | each arm's output length is data-determined, so the output sub-ranges cannot be proved disjoint | counting pass, sequential prefix sum over `k` counts, then scatter into now-disjoint ranges | +1 read pass and +1 predicate evaluation per element; parity with rayon `par_iter().filter().collect()` |
| scatter through a data-determined index array | injectivity of the index array is a quantified fact over elements, which Rule 11 excludes | invert-and-gather, or bucket-first | +1 pass; parity with safe Rust, a real loss of parallelism against C++ with a programmer assertion |

---

## 6. Cost table

Per operation, against the baselines named at the top. "—" means no difference. One
**relocation** is one load plus one store of `sizeof(T)` bytes.

| Operation | Extra loads | Extra branches | Extra copies / relocations | Extra allocations | Lost parallelism | Against |
|---|---|---|---|---|---|---|
| generic boundary itself: rows, contracts, fact flow, caller-side disjointness | — | — | — | — | — | both; nothing survives into lowered code |
| `partition_range`, Lomuto, any value class | — | — | — | — | — | both (`std::swap` is also an exchange: 2 loads + 2 stores) |
| the same partition's **scan guard** | — | **+1 compare + 1 predicted branch per scanned element** | — | — | — | vs libstdc++ `__unguarded_partition` and vs Rust std's unguarded scans (which use `unsafe`); parity vs safe Rust |
| `sort_range`, Copy element type | — | inherits the scan guard, ≈`n log n` extra branches | — | — | — | vs C++; parity vs safe Rust |
| `sort_range`, non-Copy element type | — | same | — | — | — | **now parity with C++/Rust on relocations**; round one had no in-place form at all |
| `insertion_sort` inner shift, Copy | — | — | — | — | — | a Copy `T` can be held in a local, so the hole rotation is written directly |
| `insertion_sort` inner shift, non-Copy | **+1 load +1 store per shift, unless the backend forwards the traveling value in a register across the swap chain** | — | — | — | — | vs libstdc++'s moved-out temp; same class as Rule 6's recorded "usually elided by the backend, not guaranteed" |
| `lower_bound` via `halve` | — | — | — | — | — | both |
| `lower_bound` without it (G2 pessimistic) | — | +1 compare + 1 branch per probe, ≈`log2 n` per search | — | — | — | both |
| `map_inplace`, any value class | — | — | — | — | — | both (round one charged +1 allocation and +`n` relocations under G4's strict reading) |
| `map_into` | — | — | — | — | — | both, **if** the per-element `dst.len` store is hoisted; else +1 store per element vs Rust `collect` |
| caller reference into `dst.filled` surviving a `map_into` | — | — | — | — | — | better than round one, which killed it |
| `retain`, stable, Copy | — | +1 predicted branch per kept element (the `w != k` guard) | — | — | — | parity: `Vec::retain` and `std::remove_if` branch too |
| `retain`, stable, non-Copy | — | same guard | **+1 load +1 store per relocated element** (`swap` exchanges; `Vec::retain` rotates through a hole) | — | — | vs `Vec::retain`; round one cost +1 allocation, +`n·sizeof(T)` peak and ≈+1.5`n` relocations |
| `retain_unstable` / `swap_remove` | +1 load | — | +1 store per removed element | — | — | vs `Vec::swap_remove` (2 loads + 1 store); same exchange-versus-hole gap |
| `unique_by` without a merge | — | — | — | — | — | both |
| **in-place fold / `unique` with a merge, non-Copy** | — | — | ≈+1.5 relocations per element | **+1 window of `n·sizeof(T)`** | — | vs `std::unique` with `*w = merge(move *w, move *r)`; **§7** |
| the same fold, Copy | — | — | — | — | — | both |
| binary search + `insert_at` (ordered insertion) | — | — | — | — | — | parity with `Vec::insert`: Rule 6 specifies "one memmove" |
| concatenating two mapped halves with `append` | — | — | one memcpy | — | — | parity with `Vec::extend_from_slice` |
| sorting handles whose comparator dereferences a second container | +1 dependent load per comparison, plus loss of sequential locality | +1 bounds branch per comparison, unless the callback carries the bound as in §4.4 | — | +1 of `8n` | — | vs C++; parity vs safe Rust. Round one was forced into this shape for every non-Copy element type; revision 4 needs it only for genuine index/multi-key sorts |
| call through a function-typed parameter | — | — | — | — | — | parity, conditional on the compiler specializing a statically known callee; C++ pays the same through `std::function` |
| parallel adjacent-range map, chunked map, parallel search, parallel sort by top-level fanout | — | — | — | — | — | both |
| parallel filter into one window | — | +1 predicate evaluation per element | — | — | — | parity both |
| parallel in-place partition | — | — | +1 relocation per element | +1 of `n·sizeof(T)` | — | parity both |
| parallel scatter through data-determined indices | — | — | — | — | **all of it** | vs C++; parity vs safe Rust |

Two entries in that table deserve their own sentence, because they are the only
genuine losses left in the task and both have the same root cause.

**`swap` is an exchange, not a hole rotation.** C++ and Rust write their rotation and
compaction loops by moving one value *out* into a temporary, leaving a moved-from
slot, and then shifting the rest by single relocations. Under this candidate there is
no moved-from state — Rule 12: "No place is ever partially moved: every path is either
wholly present or the program cannot name it" — and for a non-Copy `T` there is no way
to create the temporary, because Rule 6 states "A move out of a window slot or an array
element is rejected; use the operations above." So every relocation is an exchange:
two loads and two stores where a hole rotation needs one and one. In the insertion-sort
chain the extra traffic is recoverable by a backend that keeps the traveling value in a
register across consecutive swaps; in compaction, where the source and destination are
far apart and the displaced value genuinely has to be written back, it is not.

**The refused sentinel scan.** The unguarded inner loop

```text
while less(&part[i], &pivot, ctx) { i = i + 1 }      // no i < part.len test
```

needs the fact "some element at or before the end is not less than the pivot", which is
quantified over elements; Rule 11 states "There are no quantified facts over array
elements ("for all i ...") and no per-slot occupancy facts", and the `use` steps Rule 11
admits are steps inside an `invariant`, not a licence to introduce a fact form the same
sentence forbids. The guarded scan `while i < j && less(&part[i], &pivot, ctx)` adds one
compare and one predicted branch per scanned element, roughly `n` per partition level.
It is worth recording *why* this is the right trade: in C++ the sentinel argument is
exactly what lets `std::sort` with an inconsistent comparator run off the end of the
array. Here every bound in `partition_range` comes from `p1`–`p3` and the measure, never
from `less`, so a lying comparator produces a wrong order and cannot touch memory outside
the range. That is a real safety gain bought with the branch.

---

## 7. The strongest counterexample against the candidate for this task

**Claim: the atomic in-place update may not read a second element of the container it
updates, and for a non-Copy element type there is no other way to get one element's
value into a call that also owns another element's value. Every in-place *fold* — the
generic form of `std::unique` with a merge, run-length compression, group-by reduction
— is therefore refused, and the rewrite costs a window and about 1.5 relocations per
element.**

### 7.1 The program

```text
// Adjacent runs are folded into their leftmost member, in place, stably.
fn fold_runs<T, C>(
        r:   &Slots<T, N>,
        ctx: &C,
        same:  fn(a: &T, b: &T, c: &C) -> Bool  reads(a), reads(b), reads(c),
        merge: fn(acc: own T, add: own T, c: &C) -> own T  reads(c))
    writes(r.filled), writes(r.len), reads(ctx)
{
    w = 0
    k = 1
    while k < r.len {
        if same(&r[w], &r[k], ctx) {
            r[w] = merge(r[w], r[k], ctx)          // <<< the operation under test
        } else {
            w = w + 1
            if w != k { swap(&r[w], &r[k]) }
        }
        k = k + 1
    }
    // ... discard the tail ...
}
```

With `T = Key` (Copy) this is ordinary. With `T = Node` — an affine struct owning a
`Box<Slots<u8>>`, the `std::vector<std::unique_ptr<...>>` case — it is the interesting
one, and it is what a compiler's symbol-table or interval-merge pass actually is.

### 7.2 Every route the rules admit, and why each one fails

**(a) `r[k]` as a by-value argument.** For Copy `T` this is a copy and the problem
disappears. For non-Copy `T` it is a move out of a window slot: Rule 6, "`x = move r[k]`
// rejected: use take_back, remove_at, replace, or swap". Refused.

**(b) Widen the callback so it reads the second element through a reference:**
`r[w] = merge2(r[w], &r[k], ctx)` with `merge2`'s row `reads(add), reads(c)`. The atomic
update's side condition is Rule 6's "f's row must not overlap any prefix of place". The
place is `r[w]`; its prefixes include `r`. The substituted row contains `reads(r[k])`,
and under Rule 3's judgment — "two indexed positions on the same storage are taken to
overlap unless their indices or ranges are proved distinct" — `r[k]` overlaps the prefix
`r` unconditionally, since `r` is a prefix of `r[k]` rather than a sibling index.
**Refused**, and no `w != k` fact repairs it, because the clash is with the prefix, not
with the sibling.

**(c) Reach the second element through a `&[T]` instead of the container.** Let the
callback take `rest: &[T]` and an index. The place is still `r[w]` and the range
reference is still formed from `r`, so the substituted read path still overlaps the
prefix `r`. Refused for the same reason.

**(d) Bring `r[k]` into a local by `swap`.** `swap(&donor, &r[k])` requires `donor` to
be an owned local of type `T`, and a generic has no constructor for `T` — Rule 12 makes
an uninitialized local unnameable, and Rule 1 gives no default. A donor can be obtained
from the container itself (`donor = take_back(&r)`), and then the update is accepted,
because a by-value argument has no effect entry at all (Rule 9: "A by-value parameter
has no effect entry"):

```text
donor = take_back(&r)                    // one owned T, taken from the back
swap(&donor, &r[k])                      // donor now holds r[k]'s value; r[k] holds the old donor
r[w] = merge(r[w], move donor, ctx)      // accepted: the second argument is by value, no row
```

The program is accepted and is not a fold. `merge` **consumes** `donor`, so the next
iteration has no donor, and the only way to obtain another one is another `take_back`
from the back — which in a left-to-right stable scan removes an element that has not
been examined yet. Each fold consumes exactly one donor and produces none. The element
parked at `r[k]` is a real value, but getting it back into a local needs a donor, and
the circle closes. **One value can be in flight in a local at a time, and a merge
consumes it.**

**(e) `remove_at(&r, k)`.** A genuine interior extraction, returning `own T`, and it
type-checks. Its cost is Rule 6's own annotation, "one memmove", per removed element:
`O(n)` per fold, `O(n·m)` for `m` folds, against C++'s `O(n)` total. Refused on cost,
not on rules.

### 7.3 The best rewrite, and its cost

Out of place, in original order, using the in-order property of `swap_remove` applied at
ascending indices — `swap_remove(&r, j)` disturbs only the tail, so applying it at
`j = 0, 1, 2, ...` while `j + j < n` emits the original elements in order, and the
remaining reversed tail comes out in order under `take_back`:

```text
fn fold_runs_out<T, C>(
        r:   &Slots<T, N>,
        out: &Slots<T, N>,
        ctx: &C,
        same:  fn(a: &T, b: &T, c: &C) -> Bool             reads(a), reads(b), reads(c),
        merge: fn(acc: own T, add: own T, c: &C) -> own T  reads(c))
    writes(r.filled), writes(r.last), writes(r.len),
    writes(out.free), writes(out.len), reads(ctx)
    contract { requires out.room >= r.len; ensures r.len == 0; }
{
    n = r.len
    if n > 0 {
        acc = drain_next(&r, 0, n, 0)                     // the first element, in a local
        j   = 1
        while j < n {
            invariant g1: r.len + j == n;
            v = drain_next(&r, j, n, j)                   // the next element, in a local
            if same(&acc, &v, ctx) { acc = merge(move acc, move v, ctx) }
            else { place_back(&out, move acc); acc = move v }
            j = j + 1
        }
        place_back(&out, move acc)
    }
}

// The in-order extraction step, written out: swap_remove while the index is still
// inside the window, take_back afterwards.
fn drain_next<T, C>(r: &Slots<T, N>, j: u64, n: u64, idx: u64) -> own T
    writes(r.filled), writes(r.last), writes(r.len)
    contract { requires r.len > 0; ensures r.len == entry(r).len - 1; }
{
    if idx + idx < n && idx < r.len { swap_remove(&r, idx) } else { take_back(&r) }
}
```

Everything in the rewrite is accepted: `acc` and `v` are owned locals, so `merge`
receives both by value and has the empty row on its value parameters; Rule 12's join
rule ("a local consumed on one incoming edge is consumed after the join") makes `acc`
consumed-and-rebound on both branches, which is well formed because both branches
rebind it.

**Cost against `std::unique` with an in-place merge**, which does about `n` relocations,
zero allocations and no extra window:

- **+1 window of `n·sizeof(T)`** and the same peak memory.
- **≈+1.5 relocations per element**: each `swap_remove` is an exchange (2 loads +
  2 stores) plus the move-out, against C++'s single relocation; the `place_back` of each
  survivor is one more relocation that the in-place form does not do.
- **+1 predicted branch per element** in `drain_next`.

For a Copy element type all of this vanishes — `tmp = r[k]` is a copy and the direct
in-place form of §7.1 stands unchanged. So the cost is exactly the price of "there is no
moved-from state".

### 7.4 Why the refusal is worth weighing against

The refusal protects nothing the candidate elsewhere claims. A byte exchange of two
in-window slots is already licensed at machine level by Rule 1's stated consequence, and
`swap` licenses it at source level. What the atomic update's side condition prevents is
not aliasing damage but a *second read* of a sibling slot while one slot is in flight —
and the derivation above shows the same effect is reachable, at a price, by first moving
that sibling into a local. So the clause costs a window and half a pass, and buys
nothing that `swap`'s own aliasing exemption does not already concede for the
write/write case.

**The minimal change this argues for**, stated as an observation and not as a rule: the
atomic update's side condition could be relaxed from "f's row must not overlap any
prefix of place" to "f's row must not overlap `place` itself, judged by Rule 10 clause 1"
— which keeps every case the current clause is there to refuse (`f` writing the place it
is updating, or freeing its container) while admitting `r[w] = merge2(r[w], &r[k], ctx)`
under the ordinary proof `w != k`. Whether that is adopted is the owner's ruling, not
this derivation's.

---

## 8. Rule gaps

Four, two carried from round one unchanged and two new. Round one's G3 and G4 are
closed by revision 4's text, as §1 records.

**G1 — may a function value that is passed as an argument, and neither stored nor
returned, capture a reference?** Rule 4: "It cannot be assigned into any aggregate
(Rule 1), cannot be returned, and cannot be captured by a function value that is stored
or returned." Rule 1: "Every struct, enum, tuple, `Array`, `Slots`, `Ring`, `Box`, and
generic instantiation holds only owned values." Neither sentence names the passed-but-not-escaping
case, and Rule 1's list does not name a function value among its aggregates.
**`undecided-rule-gap`. Cost: zero on either reading** — every signature here threads
the context as an explicit reference parameter, which is the same machine word in the
same register that a captured environment pointer would be, and protocol item 7 says
source length is not a cost.

**G2 — does a halving assignment yield affine bounds on its result?** Rule 11: "Facts
are the existing WF forms: affine comparisons over measures and integer values,
refinement facts from a dominating branch or a `match` arm, loop-header invariants
`invariant name: affine_expr compare_op affine_expr`, explicit `use` steps inside an
`invariant`, and callee contracts." The *fact* wanted for `lower_bound`, `mid + 1 <= hi`,
is an affine comparison and is plainly in the language; what is unstated is whether the
assignment `mid = lo + (hi - lo) / 2`, whose right-hand side is not affine, produces it.
**`undecided-rule-gap`. Cost: zero**, because §3.4 displaces the whole question into
`halve`, whose `ensures` are affine comparisons that Rule 11 carries; the loop pays
nothing. Under the pessimistic reading *without* `halve`, Rule 7's fallback costs one
compare and one predicted branch per probe.

**G3 (round one) — closed.** Rule 7 now states `&[T]` is "formed only from an indexable
or from another range reference" and shows `sub = &part[a..b]`. The recursive sort of
§3.3 is written directly; round one's iterative-with-inline-stack workaround is dropped.

**G4 (round one) — closed.** Rule 6 now states the atomic update as
`place = f(place, args...)`. `map_inplace`'s contextual, consuming in-place map over a
non-Copy element type is written directly, and round one's `+1 allocation, +n moves`
fallback is dropped.

**G5 (new) — must a supplied function argument match the function-typed parameter's
declared signature exactly?** Rule 10: "Function-typed parameters carry a full signature
with its own row and contract, and a call through one uses that row." That sentence says
how a call *through* the parameter is checked and says nothing about what a caller may
supply for it: whether a function whose row is smaller (`reads(a), reads(b)` where
`reads(a), reads(b), reads(c)` is declared), whose `requires` is weaker, or whose
`ensures` is stronger, is acceptable. This is squarely inside the assignment's question
"what must a caller supply", and no other sentence in the file touches it.
**`undecided-rule-gap`. Cost: zero**, because under the strictest reading every callback
in §3 can be written to match its parameter's signature character for character, taking
an unused context parameter where one is declared, and protocol item 7 says an extra
parameter is not a cost. The permissive reading would only shorten the source.

**G6 (new) — may a window part be used as a *place*, and not only inside an effect
row?** Rule 6: "a window has four named parts that paths and effect rows may name, all
interpreted at call entry: `r.next` ..., `r.last` ..., `r.filled` ... and `r.free`".
Rule 2's path grammar, however, is a closed-looking list that does not mention them: "A
path starts at a local variable or a parameter and continues through fields, `*` (Box
content), `[i]` (index, Rule 7), `[lo..hi]` (range, Rule 7), or the payload of an enum
variant." §3.7 relies on `swap(&r[k], &r.last)`, which needs Rule 6's sentence to extend
Rule 2's grammar. **`undecided-rule-gap`. Cost: zero**: the fallback `swap(&r[k], &r[r.len - 1])`
is available under `r.len > 0`, needs no `k != r.len - 1` proof because of Rule 10 clause
1's swap exemption, and lowers to the same two loads and two stores.

---

## 9. Rules consistent, and task achievable — recorded separately

### 9.1 Rules consistent for this task: yes

Every acceptance and every rejection above is decided by a quoted clause, and no two
clauses were found to contradict each other on this task's programs. Round one recorded
one apparent tension, between Rule 9's "`writes` covers writing, replacing, moving out
of, and freeing the storage at the path" and Rule 6's closed list of ways a value leaves
storage; revision 4 removes it by stating the closure directly on the offending case —
"A move out of a window slot or an array element is rejected; use the operations above"
— so Rule 9 is read, unambiguously now, as describing the breadth an effect entry must
have when one of Rule 6's licensed operations moves a value out, not as licensing a
writer-authored move-out.

One new tension was examined and resolves: Rule 2's path grammar does not list the
window parts that Rule 6 says "paths and effect rows may name". Rule 6 is the later and
more specific statement, so the reading taken is that it extends the grammar; the
alternative reading costs nothing (G6).

The four gaps of §8 are underspecification, not inconsistency: in each case the file is
silent rather than self-contradicting, and in all four the silence costs nothing.

### 9.2 Task achievable: yes, with cost

| Sub-task | Copy element type | Non-Copy element type |
|---|---|---|
| in-place partition with `swap`, generic, on `&[T]` | derived, zero cost | derived, zero cost — **round one: not expressible** |
| in-place sort, recursive over `&[T]` sub-ranges | derived; zero cost except the scan guard | derived; same — **round one: not expressible** |
| insertion-sort shift | zero cost | zero cost when the backend forwards the traveling value, else +1 load +1 store per shift |
| unguarded sentinel partition | refused; +1 compare and branch per scanned element | same |
| binary search over `&[T]` | derived, zero cost via `halve` (G2) | same |
| binary search + `insert_at` | derived, zero cost | derived, zero cost — new in revision 4 |
| map in place, with a context | derived, zero cost | derived, zero cost — **round one: +1 allocation, +`n` relocations under G4 strict** |
| map into a fresh window | derived, zero cost if the length store is hoisted | same; and a caller reference into `dst.filled` now survives |
| filter, stable, in place | derived, zero cost | derived, +1 load +1 store per relocated element — **round one: +1 allocation, +`n·sizeof(T)` peak, ≈+1.5`n` relocations** |
| filter, unstable, in place | derived, zero cost | derived, +1 load +1 store per removed element |
| adjacent-duplicate removal without a merge | derived, zero cost | derived, zero cost |
| in-place fold / merge | derived, zero cost | **refused**; rewrite costs +1 window and ≈+1.5 relocations per element (§7) |
| bounds across the generic boundary, both directions | derived, zero cost | same |
| disjointness across the boundary, decided at the caller | derived, zero cost | same |
| admitted overlapped execution (§5.1) | derived, zero cost | same |

**Achievable with cost.** The half of the assignment the task text emphasises — "show
that the bounds and disjointness facts flow through the generic boundary and what a
caller must supply" — is achieved at zero runtime cost, and revision 4 makes the other
half nearly free too. The residual costs are three, all small and all named: the guarded
partition scan against libstdc++, the exchange-versus-hole gap for non-Copy relocations,
and the in-place fold.

### 9.3 Verdicts

| Item | Verdict | Round one |
|---|---|---|
| Generic surface with function-typed parameters carrying rows and contracts | `accepted-fine` | unchanged |
| Bounds facts across the generic boundary, both directions | `accepted-fine` | unchanged |
| Disjointness across the boundary, decided at the caller | `accepted-fine` | unchanged |
| In-place partition and sort over `&[T]`, **any** value class | `accepted-fine` | was `rejected-real-cost` for non-Copy |
| Recursive divide-and-conquer over `&[T]` sub-ranges | `accepted-fine` | was `undecided-rule-gap` (G3) |
| Map in place with a context, any value class | `accepted-fine` | was `undecided-rule-gap` (G4) |
| Map into a fresh window, with live caller references into `dst.filled` | `accepted-fine` | was `accepted-fine` without the surviving references |
| Filter, stable, Copy; filter, unstable; `unique_by`; ordered insertion; `append` | `accepted-fine` | improved or new |
| Filter, stable, non-Copy | `rejected-real-cost` (+1 load +1 store per relocated element) | was `rejected-real-cost` at +1 allocation and ≈+1.5`n` relocations |
| Insertion-sort shift, non-Copy | `rejected-real-cost`, backend-conditional | new |
| Unguarded sentinel partition | `rejected-real-cost` (+1 branch per scanned element vs libstdc++) | unchanged |
| **In-place fold / `unique` with a merge, non-Copy** | `rejected-real-cost` (+1 window, ≈+1.5 relocations per element) — the §7 counterexample | new |
| Parallel scatter through data-determined indices | `rejected-real-cost` vs C++, parity vs safe Rust | unchanged |
| Parallel filter, parallel in-place partition | `rejected-zero-cost` against safe baselines; parity with parallel STL and rayon | unchanged |
| Function value capturing a reference (G1) | `undecided-rule-gap`, zero cost | unchanged |
| Halving assignment yielding affine bounds (G2) | `undecided-rule-gap`, zero cost via `halve` | unchanged |
| Conformance of a supplied function to a function-typed parameter (G5) | `undecided-rule-gap`, zero cost | new |
| A window part used as a place (G6) | `undecided-rule-gap`, zero cost | new |
| **Overall task** | **`accepted-fine`, achievable with cost.** The generic-boundary machinery is free; `swap` removes round one's whole negative result; what remains is the exchange-versus-hole gap for non-Copy relocations, one refused sentinel scan, and the in-place fold of §7. | was `accepted-fine`, achievable with cost concentrated in in-place reordering |

### 9.4 Dependence on Rule 16

**Not dependent.** Round one used Rule 16 in one place: an arena of payloads plus a
window of `u64` handles was the only rewrite that restored a zero-cost sort over an
owning element type. Revision 4's `swap` makes the direct sort zero cost, so that
rewrite is no longer needed and Rule 16 is not used anywhere in this derivation. It
remains the only statement in the file about what a stale-but-in-bounds index means
after a filter renumbers a window, which a binary-search result becomes; that is a note
about callers, not a dependence of any program here.
