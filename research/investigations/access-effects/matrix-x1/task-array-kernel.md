# Candidate x1 — engineering task: array / Struct-of-Arrays compute kernel

Derived against the frozen rule set in `CANDIDATE-X1.md` (2026-09-18) and only that
file. Engineering-task wording and the discriminating programs are quoted from
`PROGRAMS.md` where a step touches them. Nothing under "Not in this candidate",
"Pending owner ruling" (other than Rule 16, and this task does not use it — see
§10) or "Deferred" is assumed.

## 0. The question this task tests

Under candidate x1 — value-only aggregates, references as local path names,
effects declared on reference parameters, `par` blocks judged by path
disjointness, and no quantified facts over array elements — can a
Struct-of-Arrays compute kernel keep same-index updates across several columns,
hand adjacent runtime-width partitions to a helper, and carry an admitted
reduction, while preserving every parallel form the current Whitefoot
specification admits, and at what runtime cost against an idiomatic C++ or Rust
implementation?

The four parallel forms that must survive, expanded here so the rest of the file
is self-contained (they are the capability floor rows of `PROGRAMS.md`):

- **Independent declared calls may overlap** (`PROGRAMS.md` row "Independent
  declared calls, including recursive callees"): two calls whose declared access
  information does not conflict may run at the same time, the caller deciding
  from the signatures alone, and the callees may be recursive.
- **Counted loops with element maps** (row "Element-wise loops with same-index
  read/modify/write"): a loop `for i in 0..n` whose body reads and writes index
  `i` of several buffers may have its iterations overlapped, with the bounds
  already proved.
- **Runtime-width adjacent ranges passed to a helper** (row "Runtime-width
  adjacent ranges passed to a helper"): partitions whose widths are computed at
  run time may be handed to a helper, and the helper's signature alone tells the
  caller which partition it writes.
- **Admitted reductions** (row "Accumulator recombination"): a combining loop
  over wrapping, bitwise, minimum, maximum or Boolean operations may be
  recombined out of partial results.

Notation. Everything is candidate x1's pseudocode. `&path` forms a reference,
`own T` is a by-value parameter, an effect row follows the signature,
`contract { requires ...; ensures ...; }` is the contract block, `len_of` and
`cap_of` are the measures, `entry(p)` is a parameter's pre-state, and
`len_of(deref(entry(p)))` is the entry length of a `&DynBox<T>` parameter exactly
as Rule 6 writes it. The one symbol borrowed from outside the rule file is
`+wrap`, the wrapping addition of `PROGRAMS.md` program P15
(`s = s +wrap a[i]`), because the rule file names no arithmetic operator and the
reduction has to be written with a total one (§7.5 explains why it must be
total).

---

## 1. The layout, and what "lockstep" can and cannot mean here

```text
struct Cols {
    a: DynBox<u64>,  b: DynBox<u64>,  c: DynBox<u64>,  d: DynBox<u64>,
    e: DynBox<u64>,  f: DynBox<u64>,  g: DynBox<u64>,  h: DynBox<u64>,
}
```

Rule 1: "Every struct, enum, tuple, array, slice-like value, Box, DynBox, and
generic instantiation holds only owned values." A `DynBox<u64>` is an owned
value, so eight of them in one struct is the ordinary case, not a borderline
one. This is the layout of `PROGRAMS.md` program P5 ("struct Cols { a, b, c, d,
e, f, g, h: buffer<u64> }"), which additionally requires that "the eight columns
are provably distinct storages inside the loop so the backend can be told so
(per-load alias facts, no runtime guards)". Under x1 that distinctness is free
and total: the eight columns are eight distinct fields, and Rule 10's own
example settles field disjointness — "`two(&a.x, &a.y)` // accepted: distinct
fields". No `__restrict` annotation and no runtime alias guard exists or is
needed, because Rule 1's consequence ("any value can be relocated by copying its
bytes ... because nothing inside it points anywhere") means no column can
contain a reference into another column, and there is no way to construct one.

Two different things are called "lockstep" and only one of them is machine
checked here.

1. **Length lockstep**: `len_of(cols.a) == len_of(cols.b) == ... == len_of(cols.h)`.
   This is a conjunction of affine comparisons over measures, which is inside
   Rule 11's fact vocabulary ("Facts are the existing WF forms: affine
   comparisons over measures and integer values ..."). It is machine checked,
   carried across calls by `requires`/`ensures`, and costs nothing at run time.

2. **Row correspondence**: "the value at index `r` of column `a` and the value at
   index `r` of column `b` belong to the same logical row." This is a relation
   holding for every `r`, and Rule 11 is explicit: "There are no quantified facts
   over array elements ('for all i ...')". So row correspondence is **not** a
   machine-checked fact under x1. It is an invariant the writer maintains by
   construction, exactly as in C++ and in Rust, neither of which checks it
   either. No capability is lost against the baselines; the honest statement is
   that x1 checks the memory safety of the Struct-of-Arrays, not its row
   semantics.

Throughout this file, the phrase

```text
// lockstep(c, n) abbreviates these eight clauses, and nothing more:
//   len_of(c.a) == n;  len_of(c.b) == n;  len_of(c.c) == n;  len_of(c.d) == n;
//   len_of(c.e) == n;  len_of(c.f) == n;  len_of(c.g) == n;  len_of(c.h) == n;
```

is a **textual abbreviation in this document**, not a language form. It is
expanded in full at its first two uses below; later uses write the abbreviation
so the programs stay readable. Duplicating the eight clauses is verbosity, and
the protocol is explicit that "verbosity, extra parameters, and duplicated code
are not costs".

One writer-form detail that turns out to be load bearing: the eight clauses
anchor each column's length to the **integer local `n`**, not to each other
pairwise. Rule 11 says "A fact that mentions a path is invalidated when that
path is written". If lockstep were written as the chain
`len_of(c.b) == len_of(c.a)`, then any operation writing `c.a` would invalidate
the seven facts that mention it, and the program would have to re-establish all
of them. Anchored to `n`, a write to `c.a` invalidates only the clause about
`c.a`. §5.3 uses this.

---

## 2. The strongest program first: data-determined row indices

Per the protocol, step 2: "Write the strongest program first: the case where
static analysis provably cannot help (data-determined indices, lost conditions,
mixing, separate compilation, joins), not a convenient variant."

The strongest on-task case is the **active-set kernel**: the same-index update
across several columns is applied only to the rows named by a list computed at
run time. This is the shape of a particle system, an entity-component update, or
a sparse solver's active rows. Nothing static can bound the contents of the
list, and the list crosses a signature boundary as ordinary data.

### 2.1 The element functions

```text
fn cell_a(av: u64, cv: u64, dv: u64) -> u64
    // empty effect row: no reference parameters (Rule 9, "Effects are declared
    // only on reference parameters"; "A by-value parameter has no effect entry")
    contract { }

fn cell_b(bv: u64, ev: u64, fv: u64, gv: u64, hv: u64) -> u64
    // empty effect row
    contract { }
```

Both return an owned `u64` (Rule 15: "A function returns owned values only"), and
`u64` is Copy (stated in the rule file's notation paragraph), so passing the
column values in by value is a read of each place and not a move-out.

### 2.2 The active-set kernel

```text
fn kern_active(
    pa: &DynBox<u64>, pb: &DynBox<u64>, pc: &DynBox<u64>, pd: &DynBox<u64>,
    pe: &DynBox<u64>, pf: &DynBox<u64>, pg: &DynBox<u64>, ph: &DynBox<u64>,
    act: &DynBox<u64>
) -> u64
    writes(pa), writes(pb),
    reads(pc), reads(pd), reads(pe), reads(pf), reads(pg), reads(ph),
    reads(act)
    contract {
        requires len_of(pb) == len_of(pa);
        requires len_of(pc) == len_of(pa);
        requires len_of(pd) == len_of(pa);
        requires len_of(pe) == len_of(pa);
        requires len_of(pf) == len_of(pa);
        requires len_of(pg) == len_of(pa);
        requires len_of(ph) == len_of(pa);
        ensures  len_of(pa) == len_of(deref(entry(pa)));
        ensures  len_of(pb) == len_of(deref(entry(pb)));
    }
{
    n = len_of(pa)
    m = len_of(act)
    s = 0
    for t in 0..m {
        invariant la: len_of(pa) == n;
        invariant lb: len_of(pb) == n;
        invariant lc: len_of(pc) == n;
        invariant ld: len_of(pd) == n;
        invariant le: len_of(pe) == n;
        invariant lf: len_of(pf) == n;
        invariant lg: len_of(pg) == n;
        invariant lh: len_of(ph) == n;
        invariant lk: len_of(act) == m;

        i = act[t]                      // Rule 7: requires t < len_of(act)
        if i < n {                      // Rule 11: a dominating branch gives i < n
            pa[i] = cell_a(pa[i], pc[i], pd[i])
            pb[i] = cell_b(pb[i], pe[i], pf[i], pg[i], ph[i])
            s = s +wrap pa[i]
        }
        t = t + 1
    }
    s
}
```

Nine invariants and seven `requires` clauses; none of them costs an instruction.

### 2.3 What makes this the strongest case

`i = act[t]` is a value loaded from a buffer that some other translation unit
filled. Rule 11's fact vocabulary offers nothing that can bound it in advance:
"affine comparisons over measures and integer values, refinement facts from a
dominating branch, loop-header invariants `invariant name: affine_expr
compare_op affine_expr`, explicit `use` steps inside an `invariant`, and callee
contracts." A loop-header invariant relates affine expressions, and `act[t]` is
a load, not an affine expression over the loop counter. A callee contract on
whoever produced `act` could only say "every entry is below `n`", which is a
quantified fact over elements, and Rule 11 refuses those outright.

So the fact `i < n` can be obtained only where Rule 11 offers it: from a
dominating branch, once per active row. That is the whole cost of this program,
and §7 prices it.

---

## 3. The ordinary case: the dense kernel, and where the reduction lives

```text
fn kern_dense(
    pa: &DynBox<u64>, pb: &DynBox<u64>, pc: &DynBox<u64>, pd: &DynBox<u64>,
    pe: &DynBox<u64>, pf: &DynBox<u64>, pg: &DynBox<u64>, ph: &DynBox<u64>
) -> u64
    writes(pa), writes(pb),
    reads(pc), reads(pd), reads(pe), reads(pf), reads(pg), reads(ph)
    contract {
        requires len_of(pb) == len_of(pa);
        requires len_of(pc) == len_of(pa);
        requires len_of(pd) == len_of(pa);
        requires len_of(pe) == len_of(pa);
        requires len_of(pf) == len_of(pa);
        requires len_of(pg) == len_of(pa);
        requires len_of(ph) == len_of(pa);
        ensures  len_of(pa) == len_of(deref(entry(pa)));
        ensures  len_of(pb) == len_of(deref(entry(pb)));
    }
{
    n = len_of(pa)
    s = 0
    for i in 0..n {
        invariant la: len_of(pa) == n;
        invariant lb: len_of(pb) == n;
        invariant lc: len_of(pc) == n;
        invariant ld: len_of(pd) == n;
        invariant le: len_of(pe) == n;
        invariant lf: len_of(pf) == n;
        invariant lg: len_of(pg) == n;
        invariant lh: len_of(ph) == n;

        pa[i] = cell_a(pa[i], pc[i], pd[i])
        pb[i] = cell_b(pb[i], pe[i], pf[i], pg[i], ph[i])
        s = s +wrap pa[i]
    }
    s
}
```

This is `PROGRAMS.md` program P5's body verbatim in shape
(`for i in 0..n { a[i] = f(a[i], c[i], d[i]); b[i] = g(b[i], e[i], f[i], g[i], h[i]) }`)
with the admitted reduction folded in, which is the combination the task
requires. P5's requirement that "obvious shape is the fast shape" is met: this is
the obvious shape, it carries no guard, and §7 shows it emits the same
instructions as the C++ loop.

The reduction accumulator `s` is a plain owned `u64` local and the function
returns it. This is the only mechanism the reduction needs; §5.5 shows the
parallel recombination falls out of Rule 13 and Rule 15 with no reduction
primitive at all.

---

## 4. The range helper and the recursive partition

The task's "range helper for adjacent partitions" is `PROGRAMS.md` program P6:
`for w in 0..k { helper(slice_of(out, w*s .. (w+1)*s), input) }`, required to
permit "iteration overlap ... from the arithmetic of the ranges", with "the
helper's signature alone" telling the caller what it writes. §6.2 shows the
literal strided form is not admitted; this is the form that is.

### 4.1 The helper

```text
fn kern_range(
    pa: &DynBox<u64>, pb: &DynBox<u64>, pc: &DynBox<u64>, pd: &DynBox<u64>,
    pe: &DynBox<u64>, pf: &DynBox<u64>, pg: &DynBox<u64>, ph: &DynBox<u64>,
    grain: u64
) -> u64
    writes(pa), writes(pb),
    reads(pc), reads(pd), reads(pe), reads(pf), reads(pg), reads(ph)
    contract {
        requires len_of(pb) == len_of(pa);
        requires len_of(pc) == len_of(pa);
        requires len_of(pd) == len_of(pa);
        requires len_of(pe) == len_of(pa);
        requires len_of(pf) == len_of(pa);
        requires len_of(pg) == len_of(pa);
        requires len_of(ph) == len_of(pa);
        requires grain >= 1;
        ensures  len_of(pa) == len_of(deref(entry(pa)));
        ensures  len_of(pb) == len_of(deref(entry(pb)));
    }
{
    n = len_of(pa)
    if n <= grain {
        return kern_dense(pa, pb, pc, pd, pe, pf, pg, ph)
    }

    mid = n / 2                     // carried as: 2 * mid <= n;  n <= 2 * mid + 1
    par {
        s1 = kern_range(&pa[0..mid],  &pb[0..mid],  &pc[0..mid],  &pd[0..mid],
                        &pe[0..mid],  &pf[0..mid],  &pg[0..mid],  &ph[0..mid],
                        grain);
        s2 = kern_range(&pa[mid..n],  &pb[mid..n],  &pc[mid..n],  &pd[mid..n],
                        &pe[mid..n],  &pf[mid..n],  &pg[mid..n],  &ph[mid..n],
                        grain)
    }
    s1 +wrap s2
}
```

### 4.2 The driver, judging by signatures alone

```text
fn run(cols: &Cols, grain: u64) -> u64
    writes(cols.a), writes(cols.b),
    reads(cols.c), reads(cols.d), reads(cols.e),
    reads(cols.f), reads(cols.g), reads(cols.h)
    contract {
        requires len_of(cols.b) == len_of(cols.a);
        requires len_of(cols.c) == len_of(cols.a);
        requires len_of(cols.d) == len_of(cols.a);
        requires len_of(cols.e) == len_of(cols.a);
        requires len_of(cols.f) == len_of(cols.a);
        requires len_of(cols.g) == len_of(cols.a);
        requires len_of(cols.h) == len_of(cols.a);
        requires grain >= 1;
    }
{
    kern_range(&cols.a, &cols.b, &cols.c, &cols.d,
               &cols.e, &cols.f, &cols.g, &cols.h, grain)
}
```

Rule 9 allows the member paths in `run`'s row: "each path starts at a reference
parameter and may continue through fields", with the rule's own example
`fn update(o: &Obj, c: Bool) writes(o.a), writes(o.b) // member paths are
allowed`.

---

## 5. Rule-by-rule trace at the interesting points

### 5.1 The element map is not a partial move, and needs no guard

`pa[i] = cell_a(pa[i], pc[i], pd[i])`.

- Rule 7: "Every index must be proved in bounds". `pa[i]` needs `i < len_of(pa)`.
  From `invariant la: len_of(pa) == n` and the loop's `i` range `0..n`, this is an
  affine comparison over a measure and an integer value, which Rule 11 admits as
  a fact form. `pc[i]` and `pd[i]` need `i < len_of(pc)` and `i < len_of(pd)`,
  which come from `invariant lc` and `invariant ld` the same way. **Eight
  columns, eight facts, one range analysis, zero instructions.**
- Rule 6, on the assignment: "Assigning `buf[k] = x` where the old value is
  affine releases the old value; where it is linear the assignment is rejected
  unless written as the atomic update whose function consumes the old value."
  `u64` is Copy, so neither clause bites and the assignment is a plain store. The
  atomic-update form `buf[k] = f(buf[k])` is available but not required here.
- Rule 6, on the reads: `pa[i]` as an argument to `cell_a` is a duplication of a
  Copy value, admitted by Rule 8 ("Copy values may be duplicated"). It is **not**
  a partial move out, so Rule 6's prohibition — "There is no `take` operation and
  no partial move out of any place" — is not engaged. §8.3 handles the case where
  the column element type is affine instead.
- Rule 10 clause 2 at the call to `cell_a`: "A by-value argument contributes a
  consumption (`move`) or a read (copy) of its place to this comparison."
  So the call contributes reads of the places `pa[i]`, `pc[i]`, `pd[i]`. Clause 1
  compares them pairwise: all three are reads, and "Read/read overlap is
  allowed", so no index fact is needed even if two of the columns were the same
  buffer. The write to `pa[i]` is the enclosing assignment, sequenced after the
  call returns, not one of the call's effects.

### 5.2 Why `len_of` survives an element write — the zero-cost claim depends on this

The invariants `len_of(pa) == n` must survive the body's write to `pa[i]`, or
every iteration would have to re-load the length from the block header and the
loop would carry a dependent load per iteration. Two sentences decide it.

Rule 11: "A fact that mentions a path is invalidated when that path is written
(by statement or call) unless the callee's `ensures` re-establishes it." The path
written is `pa[i]`. The fact `len_of(pa) == n` mentions the path `pa`, which is a
proper prefix of `pa[i]`, not `pa[i]` itself.

Rule 6 settles the direction: "The boundary `len_of` is a runtime number stored
in the block header, readable by the program, and **changed only by the built-in
operations below**." The operations below are `push_nogrow`, `pop`,
`swap_remove` and `truncate`. An element assignment is not among them, and the
atomic update `buf[k] = f(buf[k])` commits a value back into an occupied slot,
so it does not change the boundary either. Therefore `len_of(pa)` is unchanged by
`pa[i] = ...` and the invariant is preserved at the back edge.

Rule 3 makes the same distinction for reference validity, which confirms the
reading: "Writing the storage at p's path or below it (a content write) does not
invalidate p."

No gap here; the narrow reading of Rule 11's invalidation clause is forced by
Rule 6's "changed only by".

### 5.3 The same-index update as a call across columns

The task's "same-index updates across several columns" also has a call form,
which is where Rule 9's no-index-in-signatures restriction is tested:

```text
fn step(pa_i: &u64, pb_i: &u64, cv: u64, dv: u64)  writes(pa_i), writes(pb_i)
    contract { }

step(&cols.a[i], &cols.b[i], cols.c[i], cols.d[i])
```

- Rule 9: "Signatures never contain index expressions; an index enters an effect
  only through an argument, evaluated once at the call." The row says
  `writes(pa_i)`, naming the parameter; the index is fixed by the argument. This
  is the rule's own `put_at` idiom: "`put_at(&buf[len_of(buf)], 3)` // the
  argument fixes the slot at the call; the row itself names no index."
- Rule 10 clause 1, after substitution: `writes(cols.a[i])`, `writes(cols.b[i])`,
  read of `cols.c[i]`, read of `cols.d[i]`. "Two effects on overlapping paths
  where at least one is a write must be proved disjoint (different roots, or
  indices or ranges proved distinct)." The four paths run through four **distinct
  fields**, which Rule 10's `two(&a.x, &a.y)` example establishes as disjoint. No
  fact about `i` is needed at all. **Accepted, zero instructions.**
- Rule 10 clause 3: "A live reference outside the call whose path has a proper
  prefix among the call's write paths becomes invalid after the call." The write
  paths are `cols.a[i]` and `cols.b[i]`. A reference formed earlier as
  `q = &cols.a[j]` has path `cols.a[j]`; `cols.a[i]` is not a proper prefix of
  it, so `q` stays valid. The columns are not invalidated by element writes,
  which is what lets the writer hoist column references out of a loop.

**The dangerous variant** the task set asks for is the same call with two slots of
one column:

```text
step(&cols.a[i], &cols.a[j], cols.c[i], cols.d[i])
```

Rule 10 clause 1 now compares `writes(cols.a[i])` with `writes(cols.a[j])`: same
root, same field, so it needs "indices ... proved distinct", i.e. the fact
`i != j`. Where the writer has it from a dominating branch (Rule 11) it costs one
compare outside the call; where the writer does not have it, the call is rejected
and the diagnostic names the missing fact. C++ compiles the aliased form and
silently produces a wrong answer; this is a safety gain at the cost of one
predicted compare in the case where the fact is not already present.

`step(&cols.a[i], &cols.a[i], ...)` is rejected outright — two writes to the same
path cannot be proved disjoint under any fact.

### 5.4 The range reference and the recursive split

`&pa[0..mid]` where `pa` is itself a reference parameter.

- Rule 7: "`part = &buf[lo..hi]` // requires `lo <= hi <= len_of(buf)`;
  `len_of(part) == hi - lo`". For the first arm: `0 <= mid` and
  `mid <= n == len_of(pa)` from `2 * mid <= n`. For the second arm:
  `mid <= n <= len_of(pa)`. For `pb..ph`, `len_of` equals `len_of(pa)` by the
  seven `requires` clauses. All affine comparisons over measures and integers,
  inside Rule 11.
- **Range of a range.** In the recursive call, `pa` is a parameter that may itself
  already name a range. Rule 7's list of indexable things is "an inline
  `array<T, N>` ..., a `DynBox<T>` ..., a range reference into either, and a
  `const` table" — a range into a *range reference* is not literally listed. Rule
  2 resolves it without adding a rule: "A reference is a local name for a path. A
  path starts at a local variable or a parameter and continues through fields,
  `*`, `[i]` ..., or `[lo..hi]` ...", and "a reference extends a path, it does not
  point at" the reference variable. So if `pa` names `cols.a[u..v]`, then
  `&pa[0..mid]` names `cols.a[u..u+mid]`, which **is** a range reference into a
  `DynBox<T>` and therefore is in Rule 7's list. The recursion is inside the
  rules by path substitution, not by extending them.
- **Why the columns cannot be bundled.** The alternative signature
  `fn kern_range(c: &Cols, lo: u64, hi: u64) writes(c.a), ...` fails: the row
  names the whole column `c.a`, so the two `par` arms both carry `writes(c.a)`
  and Rule 13 rejects them. Partition parallelism therefore **requires** the
  narrowed reference to cross the signature, which is precisely what Rule 13's
  own example does: `par { kernel(&v[0..mid], &out[0..mid]); kernel(&v[mid..n],
  &out[mid..n]) } // disjoint ranges: accepted`. §9.1 records the one open
  question this raises about the parameter's declared type, and §7 prices the
  worst reading.
- **Rule 13 disjointness between the arms.** "A's write paths are disjoint from
  B's read and write paths and vice versa, using the same path-overlap and
  index/range-disjointness judgment as Rule 10." Arm 1 writes `pa[0..mid]` and
  `pb[0..mid]`; arm 2 writes `pa[mid..n]` and `pb[mid..n]`. Same root, so the
  ranges must be "proved distinct": `0 <= mid` and `mid <= n` give it by affine
  comparison alone. Arm 1's reads `pc[0..mid]` ... `ph[0..mid]` versus arm 2's
  writes `pa[mid..n]`, `pb[mid..n]`: different roots, disjoint. Arm 1's reads
  versus arm 2's reads: "Read/read overlap is allowed". `s1` and `s2` are
  distinct locals, hence distinct roots. **Accepted.**
- **Recursion.** Rule 10: "Recursion is checked through contracts, never by
  unfolding bodies." The recursive call's seven `requires` clauses hold because
  `len_of(&px[0..mid]) == mid - 0` for every column `x` by Rule 7, so all eight
  narrowed lengths are `mid`; likewise all eight are `n - mid` in the second arm.
  `grain >= 1` is passed through unchanged.
- **Termination.** The writer supplies the measure. From `n > grain` and
  `grain >= 1` we get `n >= 2`. From `n <= 2 * mid + 1` and `n >= 2` we get
  `2 * mid >= 1`, hence `mid >= 1`. From `2 * mid <= n` and `mid >= 1` we get
  `n >= mid + mid >= mid + 1`, hence `mid < n`, and `n - mid < n`. Every step is
  an affine comparison over integer values, inside Rule 11; the halving is
  carried as the pair `2 * mid <= n`, `n <= 2 * mid + 1` rather than as the
  non-affine term `n / 2`.

### 5.5 The reduction needs no reduction rule

`par { s1 = kern_range(...); s2 = kern_range(...) }` followed by `s1 +wrap s2`.

- Rule 15: "A function returns owned values only." `u64` is owned, so each arm
  hands back a partial sum with no reference leaving the callee — Rule 4's
  "cannot be returned" never comes up.
- Rule 13's own example binds results inside a `par` block:
  `par { s1 = stats(&v); s2 = stats(&v) } // read/read: accepted`. So the binding
  form used here is the rule file's, not an invention.
- The two arms' writes to `s1` and `s2` are on distinct local roots, hence
  disjoint under Rule 10 clause 1.

The consequence is worth stating plainly, because it removes a whole design
question: **under x1 the writer writes the reduction tree, so there is no
compiler reassociation to admit.** `PROGRAMS.md` program P15 asks for a fixed
operation table admitting `s = s +wrap a[i]` as "any tree" while the floating
sum `x = x fadd a[i]` is "admitted only under a named level ... or stays
sequential". Under x1 the shape of the tree is source text: `kern_range`'s split
point `mid` is in the program, so the association is determined, the result is
bit-reproducible for any operation including floating addition, and no
"associative and commutative" permission is needed from the language. The
fixed-table question does not arise for this task. What the writer gives up is
the compiler's freedom to *choose* a different tree; the writer chooses it
instead, which is the same freedom exercised in a different place.

### 5.6 Independent declared calls (the first parallel form)

```text
fn kern_ab(pa: &DynBox<u64>, pb: &DynBox<u64>,
           pc: &DynBox<u64>, pd: &DynBox<u64>, grain: u64) -> u64
    writes(pa), writes(pb), reads(pc), reads(pd)
    contract { requires len_of(pb) == len_of(pa);
               requires len_of(pc) == len_of(pa);
               requires len_of(pd) == len_of(pa);
               requires grain >= 1; }

fn kern_ef(pe: &DynBox<u64>, pf: &DynBox<u64>,
           pg: &DynBox<u64>, ph: &DynBox<u64>, grain: u64) -> u64
    writes(pe), writes(pf), reads(pg), reads(ph)
    contract { requires len_of(pf) == len_of(pe);
               requires len_of(pg) == len_of(pe);
               requires len_of(ph) == len_of(pe);
               requires grain >= 1; }

par {
    s1 = kern_ab(&cols.a, &cols.b, &cols.c, &cols.d, grain);
    s2 = kern_ef(&cols.e, &cols.f, &cols.g, &cols.h, grain)
}
total = s1 +wrap s2
```

Substituted rows: `writes(cols.a)`, `writes(cols.b)`, `reads(cols.c)`,
`reads(cols.d)` against `writes(cols.e)`, `writes(cols.f)`, `reads(cols.g)`,
`reads(cols.h)`. Every pair runs through distinct fields, so every pair is
disjoint by Rule 10's field rule, and Rule 13 accepts. Both callees recurse
internally and split into `par` blocks of their own; Rule 10's "Recursion is
checked through contracts, never by unfolding bodies" means the caller never
looks inside. This is the capability floor row "Independent declared calls,
including recursive callees ... Preserve permitted overlap using caller-resolved
access information", satisfied with no annotation beyond the two effect rows.

### 5.7 The counted element map (the second parallel form)

x1 has no `par for`. Rule 13 defines only the two-armed block: "`par { A; B }` is
accepted when ...". The element map's iteration overlap is obtained from the
same recursive split as §4.1, terminating in `kern_dense` at the grain. Rule 13's
sentence "The existing counted-loop forms (per-element maps, adjacent ranges
passed to a helper, admitted reductions) are expressed with range references"
states that this is the intended route, and the per-element two-arm case is
given directly: `par { bump(&v[i]); bump(&v[j]) } // needs i != j`.

Cost against C++ and Rust: none. A recursive halving fork-join is what Intel
Threading Building Blocks' `parallel_for` and Rust's Rayon already generate from
their flat loop surface; x1 makes the writer spell the same tree. The extra
work is `k - 1` calls and `k - 1` combines for `k` leaves — `O(k)`, against
`O(n)` element work, and identical to the baselines' own scheduler.

---

## 6. Parallel opportunities: admitted and refused

### 6.1 Admitted

| Form | Rule that admits it | Extra runtime work |
|---|---|---|
| Two independent column-group kernels over one `Cols` (§5.6) | Rule 13 + Rule 10 field disjointness | none |
| Recursive adjacent-range split of all eight columns (§4.1) | Rule 13 range disjointness from `0 <= mid <= n` | none beyond the fork-join tree the baselines also build |
| Reduction recombination out of `par` (§5.5) | Rule 13 + Rule 15 | none |
| Two named elements of one column | Rule 13's `par { bump(&v[i]); bump(&v[j]) }` with `i != j` | one compare, only when the fact is absent |
| Read-only passes over the same column in parallel arms | Rule 13 "Read/read overlap is allowed" | none |
| Eight simultaneous `push_nogrow`, one per column | Rule 13, eight distinct fields | none (but see §9.3) |
| Allocation inside parallel arms (the privatization rewrite of §8.1) | Rule 14: "Allocation and release carry no effect entry and never make two parallel arms conflict" | none from the rules; the allocator's own cost remains |

### 6.2 Refused: the literal strided worker loop of program P6

```text
for w in 0..k { s_w = helper(&out[w * stride .. (w + 1) * stride], &input) }
```

Two independent problems.

First, **there is no form**. Rule 13 states an acceptance condition for
`par { A; B }` and nothing else: "`par { A; B }` is accepted when A's write paths
are disjoint from B's read and write paths and vice versa". No sentence in the
candidate admits overlap between two iterations of a counted loop, and the
protocol forbids me supplying one.

Second, even granting such a form, the disjointness obligation between iteration
`w` and iteration `w'` with `w < w'` is `(w + 1) * stride <= w' * stride`, a
comparison containing the product of two run-time values. Rule 11's vocabulary is
"affine comparisons over measures and integer values"; `w * stride` with both
operands run-time is not affine. The one fact form that might reach it is
"explicit `use` steps inside an `invariant`", and §9.2 records that the candidate
does not say what a `use` step may do. Note the boundary precisely: when `stride`
is a **compile-time constant** the term is affine in `w` and the obligation is
inside Rule 11; the task specifies runtime widths, which is the hard side.

**Best rewrite: the recursive binary split of §4.1.** Every `par` in it has
exactly two arms whose ranges meet at one point, so the obligation degenerates to
`0 <= mid <= n`, purely affine, and the product never appears. The capability
floor row "Runtime-width adjacent ranges passed to a helper — Preserve
origin/range information across the signature and separate iterations; no
descriptor-as-fresh-backing shortcut" is preserved: the range crosses the
signature as a range reference into the original column (Rule 2 makes it a name
for `cols.a[u..v]`, not a fresh buffer), and the helper's row alone tells the
caller it writes that partition.

**Cost: none.** The rewrite is the tree that Threading Building Blocks and Rayon
build from the flat form anyway, so the machine code is the same shape. Verdict
for this sub-case: `rejected-zero-cost`.

### 6.3 Refused: the write column selected at run time

```text
col = if flag { &cols.a } else { &cols.b }
par { s1 = kern_one(col, &cols.c); s2 = kern_one(&cols.b, &cols.d) }
```

Rule 2: "At a control-flow join, a reference variable's target is the set of
paths it may name; every check on it must hold for every member of the set." So
`col` names one of `{cols.a, cols.b}`, and Rule 13's disjointness check must pass
for both members. For the member `cols.b` the two arms are `writes(cols.b)`
against `writes(cols.b)` — the same path, not provably distinct. Rejected.

**Best rewrite: hoist the branch out of the parallel region.**

```text
if flag {
    par { s1 = kern_one(&cols.a, &cols.c); s2 = kern_one(&cols.b, &cols.d) }
} else {
    s1 = kern_one(&cols.b, &cols.c)          // the two arms genuinely conflict
    s2 = kern_one(&cols.b, &cols.d)          // on cols.b; sequential is correct
}
```

The duplicated body is not a cost by the protocol. In the `flag` arm the code is
identical to the unconditional version plus one predicted branch taken once per
kernel invocation, amortized over `n` elements. In the other arm the conflict is
real — both calls write `cols.b` — so C++ would be wrong to overlap them too, and
nothing is lost. Verdict for this sub-case: `rejected-zero-cost`.

### 6.4 Refused, correctly: growth inside the parallel region

```text
par { push_nogrow(&cols.a, x); s = kern_range(&cols.a, ..., grain) }
```

Rejected by Rule 13's own example, `par { push(&v, 1); stats(&v) } // rejected:
writes(v) overlaps reads(v)`. And a `Vector` growth, which per Rule 6 "allocates
a larger DynBox, moves `[0, len_of)` across, and replaces the old one", writes
the column path itself, so Rule 3 invalidates every reference formed from it:
"It is established when p is formed and invalidated when any proper prefix of p's
path is written, moved out of, replaced, or freed, by a statement or by a call."
A live `p = &cols.a[i]` across the growth is rejected, where C++ leaves a dangling
pointer and safe Rust also refuses. No capability lost; a safety gain over C++ at
zero runtime cost.

### 6.5 Refused, at parity: the in-place parallel stencil

`pa[i] = f(pa[i-1], pa[i], pa[i+1])` split into adjacent ranges gives arm 1
`writes(pa[0..mid])` and arm 2 `reads(pa[mid-1..n])`, which overlap at `mid-1`
and cannot be proved distinct. Rejected. The sequential in-place form is
accepted (the reads are Rule 10 clause 2 by-value reads at the call, all
read/read, and the store follows), so only the parallel form needs the standard
double-buffer rewrite — which C++ and Rust also need for a correct parallel
stencil. Parity; not a candidate cost.

### 6.6 The dangerous partition variant

```text
par { s1 = kern_one(&cols.a[0..mid], &cols.c[0..mid]);
      s2 = kern_one(&cols.a[lo..hi],  &cols.c[lo..hi]) }      // lo, hi run-time
```

Rule 13 needs the two ranges "proved distinct". Where the writer has a dominating
test — Rule 11's "refinement facts from a dominating branch" — establishing
`mid <= lo`, the block is accepted at the cost of one compare evaluated once,
outside the parallel region. Without it the block is rejected and the diagnostic
names the missing range fact. C++ with OpenMP accepts the same code and races
silently. This is the clearest safety win in the task and it costs one predicted
compare per partition, not per element.

---

## 7. Cost table against idiomatic C++ and Rust

Baselines: **C++** is eight `std::vector<uint64_t>` members (or eight raw
`uint64_t*` with `__restrict`) plus a Threading Building Blocks
`parallel_reduce`. **Rust** is eight `Vec<u64>` fields plus Rayon. Costs are per
the named operation; "per call" means once per partition handed to a helper,
which covers `hi - lo` elements.

| Operation | C++ | Rust (safe) | Candidate x1 | Delta vs C++ | Delta vs Rust |
|---|---|---|---|---|---|
| `a[i] = f(a[i],c[i],d[i])` inside the counted loop, per element | 3 loads, 1 store; vectorizes only if all eight pointers are `__restrict` or the compiler emits a runtime alias guard | 3 loads, 1 store; 3 bounds checks unless rewritten with zipped iterators | 3 loads, 1 store; bounds from the eight loop invariants; alias facts free from Rule 1 and distinct fields | **0**, and x1 gets `__restrict` unconditionally where C++ must be told or must guard | **−3 compares per element** on the plain indexing form |
| `b[i] = g(b[i],e[i],f[i],g[i],h[i])`, per element | 5 loads, 1 store | same + 5 bounds checks | 5 loads, 1 store | **0** | **−5 compares** |
| reduction step `s = s +wrap a[i]`, per element | 1 add | 1 wrapping add | 1 add | **0** | 0 |
| column base setup, per helper call | 8 pointer loads from the struct | 8 slice loads (pointer + length already in the value) | 8 `DynBox` pointer loads + 8 header loads for `len_of` | **+8 dependent loads per call**, hoisted out of the element loop, amortized to ~0 for `hi - lo` ≫ 8 | **+8 dependent loads per call** (Rust's length travels in the slice; x1's is in the block header per Rule 6) |
| forming `&pa[lo..hi]`, per call | pointer arithmetic, 1 add | slice split, 2 registers | Rule 2 makes the reference a name for a path, so `base + lo` and `hi - lo` in registers; no descriptor allocated | **0** | **0** |
| passing a partition across the signature, per call | 1 struct pointer + 2 counts = 3 registers | 8 slices = 16 registers, spills | 8 range references = 16 registers, spills; **or** 1 `&Cols` + `lo`, `hi` if §9.1 resolves that way | **+~10 stack stores per call** under the conservative reading; **0** under the other; amortized over the partition | **0** |
| k-way partition of the whole kernel | flat `parallel_for` over `w*stride` | Rayon `par_chunks` | recursive binary split, `k−1` internal calls | **0** (both baselines build the same tree internally) | **0** |
| reduction recombination | `parallel_reduce` join | `reduce` | `s1 +wrap s2` after each `par`, `k−1` joins | **0** | **0** |
| reduction determinism | scheduler-dependent tree | scheduler-dependent tree | tree fixed in source; bit-reproducible, and the same holds for floating addition | **0** cost, stronger guarantee | same |
| active-row update `i = act[t]; a[i] = ...`, per active row | 1 load, no check | 1 load + 1 check per column access (8 checks unless the lengths are proved equal, which safe Rust does not do across separate `Vec`s) | 1 load + **1** check; all eight column bounds follow from the single `i < n` plus the seven length equalities | **+1 compare and 1 predicted branch per active row**; blocks vectorization of the gathered update | **−7 compares per active row** |
| active-row update after compaction (§8.2) | dense loop | dense loop | dense loop, no check | **0** | **0** |
| `swap_remove` across eight columns to compact one row | 8 moves + 8 length stores | 8 moves + 8 length stores | 8 moves + 8 length stores | **0** | **0** |
| parallel permutation scatter `out[perm[i]] = in[i]` | 1 store per element, parallel, on the programmer's word that `perm` is injective | `unsafe` or per-element atomics | **refused**; best rewrite is the inverse-gather of §8.1 | **+8n bytes for the inverse table, +1 O(n) sequential build pass (amortizable), +1 compare per element** | 0 vs safe Rust, which is also refused or atomic |
| holding `p = &cols.a[i]` across a column growth | compiles, dangles | refused by the borrow checker | refused by Rule 3 | 0 cost, removes undefined behavior | 0 |
| aliased same-index call `step(&a[i], &a[j], ...)` | compiles, wrong answer when `i == j` | compiles, wrong answer when `i == j` (two `&mut` through indices require `split_at_mut`, which is itself a check) | rejected without `i != j`; one compare where a dominating branch supplies it | **+1 compare**, removes a silent wrong answer | comparable |
| cross-column move of an affine payload (§8.3) | 1 pointer move | `Option::take` + 1 branch | atomic update with a `writes(dest)` helper; `Option` niche | **+1 null check per later read** (already recorded in the candidate's "Known costs") | **0** |

Reading the table: **the task's core — same-index updates across eight columns, a
range helper for adjacent partitions, and an admitted reduction — costs nothing
against C++ per element, and beats safe Rust by removing eight bounds checks per
element.** The only per-call costs are the eight header loads for `len_of` and,
under the conservative reading of §9.1, the argument spill; both are `O(1)` per
partition against `O(hi − lo)` element work. The only per-element cost anywhere
in the task is the single bounds compare in the un-compacted active-set variant,
and §8.2 removes even that by adopting the representation the fast C++ uses
anyway.

---

## 8. The strongest counterexample, and the two rewrites that answer it

### 8.1 Parallel permutation scatter

This is the sharpest thing I can construct against x1 for an array kernel, and it
is genuinely on-task: a column-store or radix partition step permutes rows in
bulk, and it is the one place where the programmer's knowledge exceeds anything
the candidate can state.

```text
fn scatter(pout: &DynBox<u64>, pin: &DynBox<u64>, perm: &DynBox<u64>)
    writes(pout), reads(pin), reads(perm)
    contract { requires len_of(perm) == len_of(pin); }
{
    n = len_of(pin)
    m = len_of(pout)
    for i in 0..n {
        invariant li: len_of(pin) == n;
        invariant lp: len_of(perm) == n;
        invariant lo: len_of(pout) == m;
        k = perm[i]
        if k < m { pout[k] = pin[i] }
        i = i + 1
    }
}
```

The programmer knows `perm` is a permutation of `0..n` and that `m == n`. In C++
this licenses `#pragma omp parallel for` with no synchronization: distinct `i`
write distinct `k`, so there is no conflict.

Under x1 the parallel form is refused, permanently:

```text
par { scatter(pout, &pin[0..mid], &perm[0..mid]);
      scatter(pout, &pin[mid..n], &perm[mid..n]) }
```

Rule 13 requires arm 1's write paths to be disjoint from arm 2's. Both arms carry
`writes(pout)` — the **whole** output column, because the destination index is
data and Rule 9 can only narrow a path through "whole-index or range positions
supplied as arguments", and there is no argument that names the set of slots
this arm will touch. Even if the writer could name them, proving the two sets
distinct is the statement "for all `i < mid` and all `j >= mid`,
`perm[i] != perm[j]`", and Rule 11 closes that door in one sentence: "There are
no quantified facts over array elements ('for all i ...') and no per-slot
occupancy facts."

This is not a gap. The rules are clear and they refuse the program.

**Rewrite (a): privatize.** Give each arm its own output column and merge. For a
*reduction* scatter (a histogram, `hist[k] += v[i]`) this works and is what
production C++ histograms do anyway, so it is close to parity. For a
*permutation* scatter it does not work: the merge would have to know which arm
wrote which slot, which is per-slot occupancy data — `m` bits plus `m` selects
per merge level, `O(m log k)` extra work. Worse than the alternatives.

**Rewrite (b): invert the permutation and gather.** Build `inv` once with
`inv[perm[i]] = i`, then

```text
fn gather(pout: &DynBox<u64>, pin: &DynBox<u64>, inv: &DynBox<u64>, grain: u64)
    writes(pout), reads(pin), reads(inv)
    contract { requires len_of(inv) == len_of(pout); requires grain >= 1; }
{
    n = len_of(pout)
    if n <= grain { gather_seq(pout, pin, inv); return }
    mid = n / 2                       // 2 * mid <= n;  n <= 2 * mid + 1
    par {
        gather(&pout[0..mid],  pin, &inv[0..mid],  grain);
        gather(&pout[mid..n],  pin, &inv[mid..n],  grain)
    }
}
```

Now each arm writes an adjacent range of `pout` — disjoint by `0 <= mid <= n`
(Rule 13, affine) — and both arms *read* all of `pin`, which Rule 13 permits
explicitly: "Read/read overlap is allowed." Accepted.

Cost of rewrite (b) against the C++ parallel scatter:

- `8n` bytes for the `inv` table, one allocation (Rule 14 makes it fallible:
  `DynBox::new(n)?`).
- One `O(n)` **sequential** pass to build `inv`, since building it is itself a
  scatter. Where the permutation is reused across many kernel invocations — the
  usual case for a column store's row order — this amortizes to zero.
- **One compare per element** in the steady state: `gather_seq` still needs
  `inv[j] < len_of(pin)` from a dominating branch, for the same Rule 11 reason.
- The gather has sequential writes and random reads where the scatter had
  sequential reads and random writes; on current hardware this is usually the
  faster side, so this term is a wash or a gain.

**Rewrite (c): bucket first.** One counting pass plus one permutation pass to
group the input by destination partition, after which each arm scatters only
into its own range and the ranges are adjacent. Two extra `O(n)` passes — and
this is exactly what a parallel radix sort already does, so on the workload
where the counterexample bites hardest, the baseline pays the same price.

Verdict for the counterexample: `rejected-real-cost`. The cost is bounded and
amortizable — one extra `n`-element table, one amortizable sequential pass, one
compare per element — and the fast C++ form on the same workload frequently pays
it too. It is a real loss, not a fatal one, and the candidate's own "Known costs
already recorded" section already anticipates the compare ("Open-addressing hash
tables with non-Copy payloads pay one null check per hit versus hashbrown;
measure before deciding" is the same shape of cost in a different place).

### 8.2 The active-set program's own cost, and how to remove it

The strongest program of §2 pays one compare and one predicted branch per active
row for `if i < n`, and the branch blocks vectorization of the gathered update.
The zero-cost answer is to change the representation so the active rows are a
prefix of the columns, which makes §4.1's dense range kernel apply directly. Rule
6 supplies exactly the operation needed, and the eight columns stay in length
lockstep because they take the same `k`:

```text
fn deactivate(cols: &Cols, k: u64, n: u64)
    writes(cols.a), writes(cols.b), writes(cols.c), writes(cols.d),
    writes(cols.e), writes(cols.f), writes(cols.g), writes(cols.h)
    contract {
        requires k < n;
        requires len_of(cols.a) == n;   requires len_of(cols.b) == n;
        requires len_of(cols.c) == n;   requires len_of(cols.d) == n;
        requires len_of(cols.e) == n;   requires len_of(cols.f) == n;
        requires len_of(cols.g) == n;   requires len_of(cols.h) == n;
        ensures  len_of(cols.a) == n - 1;   ensures len_of(cols.b) == n - 1;
        ensures  len_of(cols.c) == n - 1;   ensures len_of(cols.d) == n - 1;
        ensures  len_of(cols.e) == n - 1;   ensures len_of(cols.f) == n - 1;
        ensures  len_of(cols.g) == n - 1;   ensures len_of(cols.h) == n - 1;
    }
{
    _ = swap_remove(&cols.a, k)     // Rule 6: requires k < len_of(cols.a), i.e. k < n
    _ = swap_remove(&cols.b, k)     // requires k < len_of(cols.b): still n, untouched
    _ = swap_remove(&cols.c, k)
    _ = swap_remove(&cols.d, k)
    _ = swap_remove(&cols.e, k)
    _ = swap_remove(&cols.f, k)
    _ = swap_remove(&cols.g, k)
    _ = swap_remove(&cols.h, k)
}
```

The writer form matters here, and it is the §1 point paying off. Each
`requires` clause anchors a column's length to the **integer** `n`, not to
another column. After the first `swap_remove`, Rule 11 invalidates only the facts
mentioning `cols.a`, and `swap_remove`'s own `ensures len_of(buf) ==
len_of(deref(entry(buf))) - 1` re-establishes that one as `n - 1`. The other
seven clauses never mentioned `cols.a`, so they survive untouched and each
subsequent call's `requires k < len_of(cols.x)` discharges from `k < n`. Written
as the pairwise chain `len_of(cols.b) == len_of(cols.a)` instead, the first call
would have knocked out all seven and the program would not check.

`_ = swap_remove(...)` discards a returned `own u64`; Copy values may be dropped
(Rule 8). For an affine element type the returned value would have to be
consumed, and for a linear one it must be (Rule 8: "Linear values must be
consumed by an explicit operation on every exit path").

Cost of compaction against C++: identical — eight element moves and eight length
stores per deactivation, which is what the C++ `swap`-and-`pop_back` loop does.
The row correspondence across the eight `swap_remove` calls holds because all
eight apply the same permutation at the same `k`; as §1 says, that is the
writer's invariant, unchecked here and unchecked in C++ and Rust as well.

### 8.3 A column of affine payloads, and the cross-column move

If the columns hold owned payloads rather than `u64` — `DynBox<Option<Box<Row>>>`
— then `pa[i]` can no longer be read out as a value, because Rule 6 says "There
is no `take` operation and no partial move out of any place: the only ways to
move a value out of storage are consuming a whole local (`move x`), the window
operations, and the atomic update." A same-column transform uses the atomic
update directly, which Rule 6 blesses: `pa[i] = transform(pa[i])`.

A **cross-column** move — take the payload out of column `b` at row `i` and put
it into column `a` at row `i`, the column-store equivalent of moving a component
between archetypes — is expressible without any new mechanism:

```text
fn extract_into(old: own Option<Box<Row>>, dest: &Option<Box<Row>>)
        -> own Option<Box<Row>>
    writes(dest)
    contract { }
// body: releases whatever dest held, stores `old` there, returns None

pb[i] = extract_into(pb[i], &pa[i])
```

Rule 10 clause 2 makes the by-value argument a consumption of the place `pb[i]`;
clause 1 compares it with `writes(pa[i])`; the two run through distinct fields,
so they are disjoint and the call is accepted with no index fact. The payload
moves as one pointer, no deep copy. Rule 12 blesses the representation
explicitly: "`slots: DynBox<Option<Entry>>` // non-Copy payloads: Option, using a
null niche where the type has one." Cost: one null check per later read of the
column, which is the candidate's already-recorded cost for non-Copy payloads.
§9.4 notes the one thing the atomic-update sentence does not say about this
form, and the zero-cost fallback if it resolves the other way.

---

## 9. Rule gaps

Four, quoted exactly. Only the first two are material to this task's cost, and
none of them blocks the task.

### 9.1 The declared type of a range-reference parameter

Rule 9: *"An effect row lists `reads(path)` and `writes(path)` where each path
starts at a reference parameter and may continue through fields, `*`, and
whole-index or range positions supplied as arguments."*

Rule 9, two sentences later: *"Signatures never contain index expressions; an
index enters an effect only through an argument, evaluated once at the call."*

These pull in opposite directions for the Struct-of-Arrays helper.

- **Reading (i)** takes the first sentence at face value: a row may say
  `writes(c.a[lo..hi])` where `lo` and `hi` are `u64` parameters, because that is
  a "range position supplied as arguments". The helper then takes one `&Cols`
  plus two counts — 3 registers — and the `par` arms are disjoint because
  `[0..mid]` and `[mid..n]` are proved distinct.
- **Reading (ii)** takes the second sentence at face value: the row may not
  contain `[lo..hi]` at all, and a range enters only by passing the narrowed
  reference `&c.a[lo..hi]` as an argument, the row naming only the parameter.
  This is the form of every example in the rule file, including Rule 13's
  `kernel(&v[0..mid], &out[0..mid])` and Rule 9's `put_at(&buf[len_of(buf)], 3)`.

A second, smaller part of the same gap: under reading (ii) the narrowed
reference has to be *declared* somewhere. Rule 7 gives a range reference `len_of`
and indexing, and "slice types as first-class values" is listed under "Not in
this candidate", so the only spelling consistent with the rest is to declare the
parameter `&DynBox<u64>` and let it accept both a whole-window reference and a
range reference into one. The rule file never writes such a signature.

**Why it does not block the task**: the programs in §4 are written under reading
(ii), the conservative one, and they are accepted. The gap changes only the
argument-passing cost — reading (ii) spills roughly ten registers per partition
call where reading (i) passes three — and that cost is `O(1)` per partition
against `O(hi − lo)` element work. The task is achievable under both readings;
the derivations above hold under both.

### 9.2 What an explicit `use` step may establish

Rule 11: *"Facts are the existing WF forms: affine comparisons over measures and
integer values, refinement facts from a dominating branch, loop-header
invariants `invariant name: affine_expr compare_op affine_expr`, explicit `use`
steps inside an `invariant`, and callee contracts."*

The clause names `use` steps as a fact form but says nothing about what a step may
justify. The question is live for §6.2: whether a `use` step may carry the
monotonicity of multiplication, `w < w'` implies `w * stride < w' * stride`,
which is the missing premise for the runtime-stride worker loop. If it may, the
flat P6 form still needs a counted-loop parallel form that Rule 13 does not
provide, so the gap is not sufficient on its own either.

**Why it does not block the task**: the recursive binary split of §4.1 never
forms the product, and it costs nothing (§6.2). Whichever way this resolves, the
adjacent-range capability is preserved at zero cost.

### 9.3 Facts leaving a `par` block

Rule 13 gives an acceptance condition and nothing about the post-state:
*"`par { A; B }` is accepted when A's write paths are disjoint from B's read and
write paths and vice versa, using the same path-overlap and index/range
disjointness judgment as Rule 10."*

Rule 12 defines fact propagation only for alternatives: *"At a join, facts are
intersected (a fact survives only if it holds on every incoming edge)."* A `par`
block is not a join of alternatives — both arms run — so neither rule says which
of A's and B's `ensures` clauses hold afterwards. This matters for a
Struct-of-Arrays whose length lockstep must be re-established after eight
parallel `push_nogrow` calls, one per column.

**Why it does not block the task**: none of §2, §3, §4 or §5 needs a length fact
out of a `par` block. The eight kernels' `ensures` say the windows are unchanged,
and the reduction results are plain owned locals. If a writer did need parallel
lockstep pushes, the sequential form costs eight stores — parallelizing eight
stores is pointless — so the fallback is free.

### 9.4 Whether the atomic update's function may have effects of its own

Rule 6: *"`buf[k] = f(buf[k])` // atomic in-place update: the old value goes into
f by value, f's result is committed, no program point lies between; requires
`k < len_of(buf)`"*

"No program point lies between" is stated to keep the slot from being observed
empty. It does not say whether `f` may itself carry an effect row touching other
paths, which is what §8.3's `extract_into` does. Rule 9 and Rule 10 would govern
such a row in the ordinary way, and the slot is never observable as empty because
`f` owns the old value throughout, so the natural reading admits it — but the
sentence does not say so.

**Why it does not block the task**: the task's columns are `u64`, where no atomic
update is needed at all (§5.1). The affine-payload variant of §8.3 has a
zero-cost fallback if this resolves the other way: keep the payload columns as
columns of `u64` handles into a separate payload buffer, which is the
higher-performance Struct-of-Arrays form regardless.

---

## 10. Dependence on Rule 16

**This task does not depend on Rule 16.** Rule 16 (pools and arenas as usage,
a stale but in-bounds index naming the current occupant) is the candidate's one
provisional rule, and none of the programs above needs it:

- The columns are allocated with `DynBox::filled(n, 0)?` or `DynBox::new(n)?` and
  indexed under proved bounds; no handle outlives a `truncate`.
- The compaction of §8.2 uses `swap_remove`, a Rule 6 window operation, not an
  arena reset.
- The privatization and inverse tables of §8.1 are plain `DynBox` values
  allocated per partition and returned as owned values (Rule 15), not pooled.

Rule 16 would enter only if a per-worker scratch buffer were reused across kernel
invocations by `truncate(&scratch, 0)` instead of being reallocated. Both forms
are available; the programs above use the reallocating form, so the derivation
stands whichever way the owner rules. The one place where Rule 16's *spirit*
appears without its mechanism is §1's row correspondence: a Struct-of-Arrays row
index that has been invalidated by a `swap_remove` still names a valid occupant
of every column, which is a logic error and not a memory error — but that is a
consequence of Rule 11's refusal of quantified facts, not of Rule 16.

---

## 11. Rules consistent, and task achievable

The protocol asks these separately: "a rule set that rejects a task cleanly has
not achieved it."

**Rules consistent: yes, for this task.** Every acceptance and every rejection
above follows from a quoted clause, and no two clauses needed for this task
contradict each other. The four gaps in §9 are places where the text is silent or
pulls two ways, not places where it says two incompatible things; each has a
derivation that holds under every reading, and §9 records which. The one
apparent tension — Rule 11's fact-invalidation clause against the need for
`len_of` to survive an element write — is resolved inside the rules by Rule 6's
"changed only by the built-in operations below" (§5.2), not by a choice of mine.

**Task achievable: yes, at zero cost for its stated content.** All four parallel
forms survive:

| Required form | Survives as | Cost |
|---|---|---|
| Independent declared calls, including recursive callees | `par` over two column-group kernels, judged from the substituted rows alone (§5.6) | zero |
| Counted loops with element maps, same-index read/modify/write | recursive range split terminating in `kern_dense` (§3, §5.7) | zero; the split is the tree the baselines build |
| Runtime-width adjacent ranges passed to a helper | `kern_range` taking range references into the original columns (§4.1) | zero; the literal strided form is refused and rewritten at zero cost (§6.2) |
| Admitted reductions | owned partial sums out of `par`, combined with `+wrap` (§5.5) | zero, and the tree is source-determined so the result is bit-reproducible |

Struct-of-Arrays columns in lockstep are preserved for lengths, machine checked,
at zero runtime cost; row correspondence is the writer's invariant, as it is in
both baselines. Per element, the dense kernel is instruction-for-instruction the
C++ loop and beats safe Rust by eight bounds checks. The residual costs are
`O(1)` per partition (eight header loads for `len_of`; argument spill under the
conservative reading of §9.1) and one compare per active row in the
un-compacted active-set variant, which §8.2 removes by adopting the compacted
representation that fast C++ uses anyway.

The one real loss is outside the task's stated content and is recorded as the
counterexample: a **parallel scatter whose destination indices are data** is
refused permanently, because its disjointness obligation is a quantified fact
over elements and Rule 11 has none. The best rewrite costs one extra `n`-element
table, one amortizable sequential pass, and one compare per element.

---

## 12. Verdicts

From the fixed vocabulary.

| Item | Verdict |
|---|---|
| **Array / Struct-of-Arrays compute kernel — the task as stated** (same-index updates across several columns, range helper for adjacent partitions, admitted reduction, all four parallel forms, columns in lockstep) | **`accepted-fine`** |
| Dense same-index element map over eight columns, with the reduction (§3) | `accepted-fine` |
| Adjacent-range helper by recursive binary split (§4.1, §5.4) | `accepted-fine` |
| Independent declared calls over disjoint column groups (§5.6) | `accepted-fine` |
| Reduction recombination out of `par` (§5.5) | `accepted-fine` |
| Length lockstep across eight columns, including `swap_remove` compaction (§1, §8.2) | `accepted-fine` |
| Active-set kernel with data-determined row indices (§2) | `accepted-fine`, at one compare and one predicted branch per active row; `rejected-zero-cost` in effect, since §8.2's compacted form removes the compare |
| Literal strided worker loop `for w in 0..k { helper(&out[w*stride..], ...) }` (§6.2) | `rejected-zero-cost` |
| Run-time-selected write column inside `par` (§6.3) | `rejected-zero-cost` |
| Growth or `push` inside a `par` arm touching the same column (§6.4) | `rejected-zero-cost` (the baselines refuse or are undefined) |
| In-place parallel stencil (§6.5) | `rejected-zero-cost` (parity: both baselines need the double buffer) |
| Aliased same-index call `step(&a[i], &a[j], ...)` without `i != j` (§5.3) | `rejected-zero-cost` where a dominating branch supplies the fact; one compare, and it removes a silent C++ wrong answer |
| **Parallel permutation scatter — the strongest counterexample** (§8.1) | **`rejected-real-cost`** |
| Cross-column move of an affine payload (§8.3) | `accepted-fine`, at the already-recorded one null check per later read |
| Declared type and row spelling of a range-reference parameter (§9.1) | `undecided-rule-gap` — affects argument-passing cost only, `O(1)` per partition; task achievable under both readings |
| What an explicit `use` step may establish (§9.2) | `undecided-rule-gap` — not on the critical path; the binary split avoids the product entirely |
| Facts leaving a `par` block (§9.3) | `undecided-rule-gap` — unused by this task; zero-cost sequential fallback |
| Effects of the atomic update's function (§9.4) | `undecided-rule-gap` — unused by the `u64` columns; zero-cost fallback for payload columns |

**Rule 16 dependence: none** (§10).
