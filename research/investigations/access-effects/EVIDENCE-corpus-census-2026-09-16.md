# Evidence: corpus census of borrow and view shapes, 2026-09-16

Three read-only censuses of every `.wf` file outside `archive/`, produced by
research agents for owner decisions O7 and O12 of
[VERDICT-CORE.md](VERDICT-CORE.md): can a core model with no first-class
references (conventions, second-class projections, pool handles, indices)
express the existing corpus? Each site where a borrow or view is formed,
bound, passed, returned, stored or held is classified A (call argument), B
(block-local binding), C (returned), D (stored), E (held across a conflicting
write), F (split or partition across calls), G (loop-carried), H (other), with
the repair under the no-reference model or "unwritable". Counts were
reconciled against grep; the files read in full are named in each report.
Evidence, not decisions. Remove with the verdict it supports.

Totals: snapshot 1420 sites, conformance and codegen 1600, programs and
experiments 4782; unwritable 0 in each, conditional on the `with` form
admitting several sibling projections of one container with proved
disjointness, parent restoration when they close, nested re-projection
including recursion, a const origin, yielded projections with an origin set,
a `with` spanning a loop, and an exclusive-to-shared downgrade.


# File: census-programs.md

# Reference-site census — `tests/programs/`, `research/experiments/`, `research/investigations/`

Corpus read at `/private/tmp/whitefoot-access-effects-research`, branch
`research/access-effects`, read-only. No repository file was modified, no build
or `make check` was run.

Question this census serves: can a core model with **no first-class references**
— three call conventions (`let` / `inout` / `sink`), second-class projections
`with inout v[i] as p { ... }` that may be passed down but never stored or
returned, `Handle<P,T>` pool handles for shared or cyclic structure, and
index-based access — express the existing Whitefoot corpus?

---

## 0. Scope, unit of count, and method

**Files.** 189 `.wf` files:

| root | files |
|---|---|
| `tests/programs/` | 39 |
| `research/experiments/**` | 100 |
| `research/investigations/**` (excluding `binary-arithmetic/`) | 50 |

`research/investigations/binary-arithmetic/` and `archive/` were excluded as
instructed. 41 of the 189 files contain no borrow and no view at all.

**What counts as one site.** A site is one *reference-valued occurrence in
source*. Eight syntactic kinds exhaust the corpus; each occurrence is counted
once and only once, and the borrow operand consumed by a view constructor is
counted as part of that view's formation rather than separately:

| kind | what it is | count |
|---|---|---|
| `ARG_BORROW` | `&p` / `&uniq p` written in a call-argument position (`f(x: &uniq v)`) | 2536 |
| `VIEW_FORM` | `let v = slice_of(...)` / `let v = mut_slice_of(...)` | 521 |
| `BORROW_LET` | `let h = &p` / `let h = &uniq p` | 9 |
| `BORROW_RET` | `return &…` / `give &…` | 1 |
| `PARAM_BORROW` | a parameter whose written mode is `&` or `&uniq` | 579 |
| `PARAM_VIEW` | a parameter whose written type is `Slice<…>` / `MutSlice<…>` (incl. `&Slice`, `&uniq MutSlice`) | 158 |
| `MATCH_BINDER` | an arm binder of a borrowed `match deref(…)` — an OWN-13 arm-scoped child reborrow | 977 |
| `CALL_RESULT_BORROW` | `let r = <call whose result mode is a borrow>` | 1 |
| | **total sites** | **4782** |

**Exactness ledger.** Every count below is reconciled against raw `grep`
occurrence counts, not estimated:

* `&` characters in the corpus (excl. `binary-arithmetic`): **3717**; `&uniq`
  **2583**; shared `&` **1134**; `&&` **0**.
* `&` on `fn` declaration lines: **646** = 579 `PARAM_BORROW` + 66 `&` inside
  the 158 view parameters (`&uniq MutSlice<u8>` ×45, `&Slice<u8>` ×20,
  `&uniq MutSlice<u32>` ×1) + 1 borrow-mode *return type*
  (`chain-evidence.wf:1`).
* `&` in function bodies: **3071** = 2536 `ARG_BORROW` + 521 view-constructor
  operands + 9 `BORROW_LET` + 1 `BORROW_RET` + 4 mentions inside `doc` string
  literals (not sites).
* `slice_of` **281** + `mut_slice_of` **240** = **521**, and every one of the
  521 is `let`-bound, one per line (no view is ever constructed inline into an
  argument).
* All 152 `Slice<` / `MutSlice<` type mentions plus 6 legacy lowercase
  `slice<'s, u8>` mentions occur on `fn` declaration lines — **zero** in a
  struct field, enum payload, `let` annotation, or return type.

**Class assignment rules used (deterministic).**

* **A** — the reference exists only as a call argument or as the matching
  callee parameter, never bound to a name. `ARG_BORROW` + `PARAM_BORROW` +
  `PARAM_VIEW`.
* **B** — the reference is bound to a name (`let`, or a `match` arm binder),
  used inside that block, never stored or returned, and no call that writes the
  same container runs while it is live. All `VIEW_FORM` not in F, all
  `BORROW_LET`, all `MATCH_BINDER`.
* **C** — returned from a function, or bound from a function's borrow-mode
  result.
* **D** — stored into a struct field, enum payload, container element, or
  captured.
* **E** — held live across a call that writes the same container or storage.
* **F** — a view split or partition handed to helpers across calls or loop
  iterations: every three-operand (ranged) `slice_of` / `mut_slice_of`, plus
  every whole-container view bound outside a loop and passed to a call inside
  that loop.
* **G** — loop-carried across iterations.
* **H** — anything else.

**How B was verified, not assumed.** Class B requires "no mutating call on the
borrowed container while it is live". This was checked mechanically for all 521
views: for each, the true root container of the borrow operand was recovered
(stripping `&`, `&uniq`, a region name, `deref(…)`, and any field/index
suffix), the loan extent was taken as the spec gives it — VIEW-1/OWN-5: an
affine `MutSlice` ends at its consuming statement or its region's end, a copy
`Slice` ends at its **last use** — and the extent was scanned for `&uniq root`,
`set root…`, or `move root`. Twelve candidates surfaced; all twelve were read
and all twelve are explained without an E:

* `bfs.wf:74`, `bfs.wf:83` — the view is consumed (`output: move output`) by
  the call on the next line, so `return move next` / `move current` two lines
  later touches no live loan.
* `merge_sort.wf:110/132/134/135`, `radix_scatter.wf:119/121/209`,
  `range_split.wf:21` — the "conflict" is the *sibling* split on the following
  line, i.e. a second exclusive view of a **disjoint** range admitted by OWN-7.
  These are class F, not E.
* `range_split.wf:22` — `set output[0_u64]` at line 26 happens after `right`
  was consumed by `fill_recursive` at line 25.
* `run_views.wf:34` — a copy `Slice`; its last use is line 41 and the
  `place_back(vector: &uniq two, …)` is line 50. (The detector's hit was the
  *parameter name* `window:` at line 54, not a use of the value.)

---

## 1. Sites per class, per directory

Directories are grouped at the level of `tests/programs`, each top-level
experiment, and each top-level investigation. "files" is every `.wf` in that
directory; directories whose files contain no borrow and no view are listed
with all-zero rows.

| directory | files | A | B | C | D | E | F | G | H | total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `tests/programs` | 39 | 867 | 537 | 0 | 0 | 0 | 4 | 0 | 0 | 1408 |
| `research/experiments/blind-writer` | 12 | 283 | 25 | 0 | 0 | 0 | 0 | 0 | 0 | 308 |
| `research/experiments/buffer-initialization-cost` | 1 | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7 |
| `research/experiments/checked-law-channel` | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `research/experiments/codegen-vs-rust-c` | 5 | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 10 |
| `research/experiments/compute-bench` | 11 | 62 | 10 | 0 | 0 | 0 | 54 | 0 | 0 | 126 |
| `research/experiments/container-representation` | 40 | 696 | 47 | 0 | 0 | 0 | 0 | 0 | 0 | 743 |
| `research/experiments/crc32-swap-in` | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `research/experiments/effect-attrs-channel` | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `research/experiments/io-completion-bench` | 12 | 988 | 675 | 0 | 0 | 0 | 0 | 0 | 0 | 1663 |
| `research/experiments/literal-line-floor` | 1 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 |
| `research/experiments/park-on-miss-measurements` | 1 | 6 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 7 |
| `research/experiments/port-study` | 4 | 14 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 14 |
| `research/experiments/scoped-alias-channel` | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| `research/experiments/wfgrep-double-walk` | 3 | 209 | 75 | 0 | 0 | 0 | 0 | 0 | 0 | 284 |
| `research/experiments/wfgrep-scan-floor` | 2 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| `research/experiments/zlib-core-kernels` | 2 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| `research/investigations/arith-dissolution` | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| `research/investigations/compute-model` | 1 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 3 |
| `research/investigations/division-dissolution` | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `research/investigations/o11-composition` | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `research/investigations/proof-derived-parallelism` | 28 | 111 | 77 | 0 | 0 | 0 | 0 | 0 | 0 | 188 |
| `research/investigations/reborrow-extension` | 1 | 3 | 1 | **3** | 0 | 0 | 0 | 0 | 0 | 7 |
| `research/investigations/strict-clause-retirement` | 9 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| **TOTAL** | **189** | **3272** | **1449** | **3** | **0** | **0** | **58** | **0** | **0** | **4782** |

Composition of the two large classes:

* **A = 3272** = 2535 argument borrows + 579 borrow-mode parameters + 158 view
  parameters.
* **B = 1449** = 463 whole-container view bindings + 9 `let`-bound borrows +
  977 borrowed-`match` arm binders.

---

## 2. Every site in classes C, D, E, G, H, and every class-F site that spans a call boundary

### 2.1 Class C — returned from a function (3 sites, all in one probe file)

The entire corpus contains **one** function with a borrow-mode result and
**zero** functions with a view result. There is no `Slice`/`MutSlice` return
type anywhere in the 189 files.

**`research/investigations/reborrow-extension/chain-evidence.wf:1–2`, `:14`**

```
fn passthru['r0](x: &uniq 'r0 i32) -> &uniq 'r0 i32 pure {
  return &uniq 'r0 deref(x);
}
...
    let h = &uniq v;
    let r = passthru(x: &uniq deref(h));
    region {
      bump(n: &uniq deref(r));
    }
```

Three sites: line 1 (the borrow-mode result declaration), line 2 (`return
&uniq 'r0 deref(x)`), line 14 (`let r = …`, the caller-side holder, plus the
candidate-position child reborrow `&uniq deref(h)` that feeds it).

*Is the returned value a projection of a parameter?* **Yes** — `deref(x)` is
the complete resolved place of the single parameter `x`, so the origin set is
the singleton `{x}`.

*Repair:* **yielded projection with `origin`**. `passthru` becomes a function
that yields a projection of `x` with `origin = {x}`; the caller's `let r = …`
becomes the binder of a `with`-like yielded projection whose origin is the
`with inout v as h` block already open at line 13. Expressible under the model.
This file is an evidence probe for the reborrow-extension investigation, not a
program; it is the sole reason class C is non-empty.

### 2.2 Class D — stored into a field, payload, element, or captured: **0 sites**

Verified three ways, not assumed from STOR-5:

* No struct field, enum variant payload, or run-element type in the corpus has
  a `&`, `Slice<`, or `MutSlice<` type (all 158 such type mentions are `fn`
  parameters).
* No borrow expression ever appears inside a constructor call: of the 1949
  lines carrying argument borrows, **0** match `return <Ctor>(`, `give <Ctor>(`,
  or `= <Ctor>(`.
* The language has no closures, so nothing can capture a reference.
  (`proof-derived-parallelism/probes/d1_closure_div.wf` is named for divergence
  in a recursive fold, not for a closure form.)

### 2.3 Class E — held live across a call that writes the same container: **0 sites**

The full derivation is in §0 ("How B was verified"). All twelve mechanical
candidates were read and every one is either a sibling *disjoint* split (class
F) or a loan that had already ended by the spec's own extent rule. The P4
cursor shape does not occur: the blind-writer P4/B1/C/D/E probes discipline
every write into `region 'x { let view = slice_of(&'x src); region 'y { … } }`,
which closes the loan before the container is next written, and
`probe_e_hoisted_length.wf` hoists a *measure* (`len_of`), not a view, out of
its loop.

### 2.4 Class G — loop-carried across iterations: **0 sites**

Every `loop`/`for` body is its own region block (OWN-11), so no bound borrow
and no exclusive view survives an iteration boundary. Exactly five views are
bound *outside* a loop and used inside it; none is created in one iteration and
consumed in the next. Four of the five are passed to a call inside the loop and
are counted as class F (§2.6); the fifth,
`research/experiments/container-representation/foundation/large-result.wf:38`,
is only indexed:

```
      region {
        let bytes = slice_of(&record.bytes);
        for (at in 0_u64..4096_u64) {
          let byte = bytes[at];
```

which is class B — repaired by `with let record.bytes as bytes { for … }`.

### 2.5 Class H — anything else: **0 sites**

The eight syntactic kinds in §0 exhaust every `&`, `Slice`, and `MutSlice`
occurrence in the corpus; the occurrence ledger balances exactly against raw
grep counts with no residue. Three shapes deserve a note even though each lands
cleanly inside A or B:

* **`&uniq MutSlice<T>` parameters (46) and `&Slice<T>` parameters (20)** are
  *borrows of a view descriptor* — a reference to a reference. They exist only
  so a caller can hand a view down without consuming the affine descriptor;
  VIEW-4 forbids writing the descriptor itself. Under the model these are
  ordinary projection arguments, so they are class A.
* **113 of the 521 views have a `const` item as their origin**
  (`slice_of(&usage_text)`, `slice_of(&reason_missing)`, …), i.e. the
  `immutable-const` origin rather than a projection of a parameter or local.
  The model needs a static-origin projection for these; nothing else about
  them is unusual.
* **22 arm binders are written through** (`set deref(bslot) = total;` in
  `par_layout.wf`, `bt.wf`, `q4.wf`, `a2r_layout*.wf`, `p1a/p1b.wf`,
  `g3_dep.wf`, `a2_bubble.wf`). These write one field of a `Box<'s, LNode<'s>>`
  cell reached through `&uniq`, inside a recursive tree fold. This is the
  corpus's clearest `Handle<P,T>` case: the tree is store-allocated and the
  fold writes each node's own slot.

### 2.6 Class F sites that span a call boundary (57 of 58)

All 54 ranged (three-operand) view formations plus the 4 loop-spanning view
bindings; 53 + 4 = 57 cross a call boundary. The one that does not is
`research/experiments/compute-bench/programs/bfs.wf:48`
(`let cell = mut_slice_of(&uniq output, vertex, after); set cell[0_u64] =
result;`) — a one-element partition written in place, repaired by
`set output[vertex] = result;` (plain index write, no projection needed).

Sites are grouped by the region block that creates them; each group's line
numbers enumerate the individual sites.

---

**F-1 — `research/experiments/compute-bench/programs/bfs.wf:39`** (1 site)

```
      let first = vertex * 4_u64;
      let end = first + 4_u64;
      invariant row_end: end <= covered {
        use 4 times (vertex + 1_u64 <= count);
      }
      let neighbors = slice_of(&edges, first, end);
      let found = pull_distance(neighbors: neighbors, previous: previous, level: level);
```

*Repair:* `with let edges[first..end] as neighbors { pull_distance(neighbors, …) }`
— one shared projection, one call, range proof already written as
`invariant row_end`.

---

**F-2 — `research/experiments/compute-bench/programs/histogram.wf:66, 67`** (2 sites) and **`:72, 73`** (2 sites)

```
    region {
      let part = slice_of(&input, first, end);
      let workspace = mut_slice_of(&uniq counters, row_start, row_end);
      let counted = count_block(input: part, counters: move workspace);
    }
```

*Repair:* nested `with` — `with let input[first..end] as part { with inout
counters[row_start..row_end] as workspace { count_block(part, workspace) } }`.
Two different containers, so no disjointness proof between them is needed; the
`invariant workspace_end` already discharges the range goal.

---

**F-3 — `research/experiments/compute-bench/programs/merge_sort.wf:106–111`** (6 sites)

```
  region {
    let a0 = slice_of(&first, 0_u64, middle);
    let a1 = slice_of(&first, middle, n);
    let b0 = slice_of(&second, 0_u64, rank);
    let b1 = slice_of(&second, rank, m);
    let out0 = mut_slice_of(&uniq output, 0_u64, split);
    let out1 = mut_slice_of(&uniq output, split, total);
    let left = merge_values(first: a0, second: b0, output: move out0);
    let right = merge_values(first: a1, second: b1, output: move out1);
  }
```

*Repair:* a **simultaneous disjoint split** form —
`with inout output[0..split] as out0, inout output[split..total] as out1 { … }`
— plus two shared splits of each input. A strictly nested `with` cannot express
it: `out0` and `out1` must be live at the same time so the two `merge_values`
calls remain an independent pair. This is the one form requirement the census
puts on the model.

---

**F-4 — `research/experiments/compute-bench/programs/merge_sort.wf:132–140`** (8 sites)

```
  region {
    let in0 = mut_slice_of(&uniq input, 0_u64, middle);
    let in1 = mut_slice_of(&uniq input, middle, count);
    let out0 = mut_slice_of(&uniq output, 0_u64, middle);
    let out1 = mut_slice_of(&uniq output, middle, count);
    let left = sort_values(input: move out0, output: move in0);
    let right = sort_values(input: move out1, output: move in1);
    let sorted0 = slice_of(&input, 0_u64, middle);
    let sorted1 = slice_of(&input, middle, count);
    let destination = mut_slice_of(&uniq output, 0_u64, count);
    let merged = merge_values(first: sorted0, second: sorted1, output: move destination);
  }
```

*Repair:* simultaneous disjoint split of two containers, then **re-projection
of the parent after the children die** — the `with` block must close `in0/in1`
and `out0/out1` before `sorted0/sorted1/destination` open. Expressible with
sequenced `with` blocks provided each split form admits two simultaneous
sibling bindings.

---

**F-5 — `research/experiments/compute-bench/programs/merge_sort.wf:158, 159`** (2 sites)

```
  region {
    let source = mut_slice_of(&uniq scratch, 0_u64, count);
    let destination = mut_slice_of(&uniq output, 0_u64, count);
    let sorted = sort_values(input: move source, output: move destination);
  }
```

*Repair:* nested `with inout` over two distinct buffers; no disjointness proof
needed.

---

**F-6 — `research/experiments/compute-bench/programs/prefix.wf:41`** (1 site), **`:58, 59`** (2 sites), **`:65, 66`** (2 sites)

```
    region {
      let part = slice_of(&input, first, end);
      let destination = mut_slice_of(&uniq output, first, end);
      let base = sums[block];
      let scanned = scan_block(input: part, output: move destination, base: base);
    }
```

*Repair:* nested `with` inside the `for` body, one shared and one exclusive
projection of two distinct buffers, block range discharged by the written
`invariant block_end`.

---

**F-7 — `research/experiments/compute-bench/programs/radix_scatter.wf:110`** (1 site) and **`:119–122`** (4 sites)

```
          let first_low = mut_slice_of(&uniq lows, 0_u64, low_count);
          let rest_low = mut_slice_of(&uniq lows, low_count, low_capacity);
          let first_high = mut_slice_of(&uniq highs, 0_u64, high_count);
          let rest_high = mut_slice_of(&uniq highs, high_count, high_capacity);
          let copied_low = copy_run(values: &payload.low, output: move first_low);
          let copied_high = copy_run(values: &payload.high, output: move first_high);
          let remaining = pack_chunks(chunks: move tail, lows: move rest_low, highs: move rest_high);
```

*Repair:* the hardest shape in the corpus — **four simultaneous exclusive
projections of two containers plus a fifth (`tail`) of a third**, with the
`rest_*` halves handed to a recursive call while the `first_*` halves are
handed to two other calls. A two-binding `with … as a, … as b` per container
suffices; strictly nested single-binding `with` does not.

---

**F-8 — `research/experiments/compute-bench/programs/radix_scatter.wf:154, 155`** (2 sites) and **`:160, 161`** (2 sites)

```
      region {
        let source = slice_of(&input, first, end);
        let destination = mut_slice_of(&uniq chunks, block, after);
        let wrote = write_chunk(input: source, bit: bit, output: move destination);
      }
```

*Repair:* nested `with`, distinct containers, per-iteration one-element
destination partition.

---

**F-9 — `research/experiments/compute-bench/programs/radix_scatter.wf:194–196`** (3 sites)

```
  region {
    let source = mut_slice_of(&uniq chunks, 0_u64, blocks);
    let low_output = mut_slice_of(&uniq lows, 0_u64, capacity);
    let high_output = mut_slice_of(&uniq highs, 0_u64, capacity);
    let packed = pack_chunks(chunks: move source, lows: move low_output, highs: move high_output);
  }
```

*Repair:* three nested `with inout` over three distinct buffers; no
disjointness proof needed.

---

**F-10 — `research/experiments/compute-bench/programs/radix_scatter.wf:207–210`** (4 sites)

```
  region {
    let low_source = slice_of(&lows, 0_u64, low_total);
    let high_source = slice_of(&highs, 0_u64, high_total);
    let low_output = mut_slice_of(&uniq output, 0_u64, low_total);
    let high_output = mut_slice_of(&uniq output, low_total, output_count);
    let copied_low = copy_values(input: low_source, output: move low_output);
    let copied_high = copy_values(input: high_source, output: move high_output);
  }
```

*Repair:* simultaneous disjoint split of `output` at `low_total`, plus two
shared projections; the disjointness goal is already carried by
`invariant full_low_bound` / `full_high_bound` / `output_limit`.

---

**F-11 — `research/experiments/compute-bench/programs/range_split.wf:21, 22`** (2 sites)

```
  region {
    let left = mut_slice_of(&uniq output, 0_u64, middle);
    let right = mut_slice_of(&uniq output, middle, count);
    let a = fill_recursive(output: move left, depth: remaining);
    let b = fill_recursive(output: move right, depth: remaining);
    let restored = output[0_u64];
    set output[0_u64] = restored;
  }
```

*Repair:* simultaneous disjoint split, **and parent restoration**: after both
children are consumed the parent must become readable and writable again inside
the same block. A `with inout output[0..middle] as left, inout
output[middle..count] as right { … }` whose close restores the parent
expresses it exactly. This file is the deliberate witness for both properties.

---

**F-12 — `research/experiments/compute-bench/programs/range_split.wf:34`** (1 site) and **`:47`** (1 site)

```
  region {
    let output = mut_slice_of(&uniq values, 0_u64, 17_u64);
    let filled = fill_recursive(output: move output, depth: 32_u64);
```

*Repair:* a single `with inout values[0..17] as output { … }`. The `:47` twin
is the empty-range case (`0..0`), which the model must admit.

---

**F-13 — `research/experiments/compute-bench/programs/stencil.wf:57–60`** (4 sites) and **`:77–80`** (4 sites)

```
        region {
          let top = slice_of(&current, before, first);
          let middle = slice_of(&current, first, end);
          let bottom = slice_of(&current, end, after);
          let row = mut_slice_of(&uniq next, first, end);
          let written = stencil_row(top: top, middle: middle, bottom: bottom, output: move row);
        }
```

*Repair:* three *overlapping-free but adjacent* shared projections of one
container and one exclusive projection of the other. Shared projections may
coexist without a disjointness proof, so a nested `with let … as top { with let
… as middle { … } }` suffices; the exclusive `row` targets a different buffer.
Ping-pong between `current` and `next` alternates by loop parity, so the model
must allow the same `with` shape to name either buffer.

---

**F-14 — `tests/programs/adaptive_quadrature.wf:96`** (1 site, loop-spanning)

```
  region {
    let published = slice_of(&report);
    let sent = 0_u64;
    loop @write {
      if sent >= 64_u64 { break @write; }
      region {
        match write_once(factory: &uniq files, output: &uniq out, source: &published, start: sent, end: 64_u64) {
```

*Repair:* `with let report as published { loop @write { … write_once(…,
published, sent, 64) … } }` — the projection is formed once outside the loop
and passed down on each iteration. Note the call also takes `&uniq files` and
`&uniq out` as `inout` capabilities while `published` is live; those are
different storage.

---

**F-15 — `tests/programs/raw_deflate_boundary.wf:346`** (1 site, loop-spanning)

```
                      region {
                        let scratch_window = mut_slice_of(&uniq scratch);
                        loop @chunks {
                          ...
                          region {
                            match read_at(factory: &uniq deref(files), file: &uniq file, destination: &uniq scratch_window, file_offset: file_offset, start: filled, end: 4097_u64) {
```

*Repair:* `with inout scratch as scratch_window { loop @chunks { read_at(…,
scratch_window, …, filled, 4097) } }` — an exclusive projection held across
many iterations, each writing a different `[filled, 4097)` window of it by
**index re-supply** rather than by re-projection. This shape is already
index-based and transfers directly.

---

**F-16 — `tests/programs/raw_deflate_dynamic_decode.wf:200, 201`** (2 sites, loop-spanning)

```
        region {
          let literal_window = mut_slice_of(&uniq literal_lengths);
          let distance_window = mut_slice_of(&uniq distance_lengths);
          loop @expand_lengths {
            ...
                      let stored = propagate store_dynamic_length(literal_lengths: &uniq literal_window, distance_lengths: &uniq distance_window, literal_count: literal_count, position: position, value: length);
```

*Repair:* two nested `with inout` blocks, both enclosing the loop, with the
write position re-supplied per iteration as `position`. Distinct containers, no
disjointness proof needed.

---

## 3. Summary

| question | count | share of 4782 |
|---|---:|---:|
| trivially expressible — class A (argument/parameter position only) | 3272 | 68.4 % |
| trivially expressible — class B (bound, block-local, no conflicting write) | 1449 | 30.3 % |
| **A + B together** | **4721** | **98.7 %** |
| need a `with … as` block (every class-B binding: 463 views, 9 `let` borrows, 977 arm binders) | 1449 | 30.3 % |
| need a `with` block **plus a proved range partition** (class F) | 58 | 1.2 % |
| of which need **two or more *exclusive* sibling projections of one container live at once** (7 pairs) | 14 | 0.29 % |
| of which need **two or more *shared* sibling projections of one container live at once** (no disjointness proof needed) | 12 | 0.25 % |
| need an **index re-supply** (a projection held across loop iterations whose written window moves per iteration) | 4 | 0.08 % |
| need a **pool handle** (`Handle<P,T>`) — the `Box<'s,T>` / `Vector<'s,T>` recursive-tree folds | 22 arm-binder writes across 10 files | — |
| need a **yielded projection with an `origin` set** (class C) | 3 | 0.06 % |
| **unwritable** | **0** | **0 %** |

**Nothing in the corpus is unwritable under the no-reference model**, subject to
one design requirement the census makes concrete:

1. **`with` must admit simultaneous disjoint sibling projections of one
   container.** Fourteen sites in exactly seven pairs —
   `merge_sort.wf:110/111` (`output`), `:132/133` (`input`), `:134/135`
   (`output`); `radix_scatter.wf:119/120` (`lows`), `:121/122` (`highs`),
   `:209/210` (`output`); `range_split.wf:21/22` (`output`) — form two
   **exclusive** projections of the *same* buffer and hand them to *different*
   calls that must remain an independent pair. Strictly nested single-binding
   `with` blocks sequence those calls and destroy the property these programs
   exist to demonstrate. A two-or-more-binding form —
   `with inout v[a..m] as p, inout v[m..b] as q { … }` — with the disjointness
   goal discharged the way OWN-7 already discharges it, covers all fourteen.
   A further twelve sites (`merge_sort.wf:106/107`, `:108/109`, `:138/139`;
   `stencil.wf:57/58/59`, `:77/78/79`) need two or three **shared** sibling
   projections of one container live at once, which needs no disjointness
   proof but does need the `with` form to admit more than one binding.
2. **The parent must be restored when the sibling projections close.**
   `range_split.wf:23–26` and `merge_sort.wf:136–140` read and re-project the
   parent after the children are consumed, inside the same block.
3. **Projections of projections must nest.** `merge_sort.wf:132–135` and
   `radix_scatter.wf:119–122` split a `MutSlice` *parameter*, not a local
   buffer; the corpus already has 46 `&uniq MutSlice<T>` and 20 `&Slice<T>`
   parameters that pass a projection down for further projection.
4. **A static/`const` origin is needed.** 113 of 521 views are views of a
   `const` table (`immutable-const` origin), not of a parameter or local.

Class C is 3 sites in a single evidence probe
(`research/investigations/reborrow-extension/chain-evidence.wf`), and the
returned reference there *is* a projection of the function's own parameter, so
the "yielded projection with `origin`" form covers it. No real program in the
corpus returns a reference or a view, and no function anywhere declares a
`Slice` or `MutSlice` result.

---

## 4. PAR-1 / PAR-2 overlap status of the corpus programs

PAR-1 (pairwise statement overlap) and PAR-2 (counted-loop overlap) are
*implementation permissions over an already accepted sequential program*, not
source obligations (`spec/kernel-spec.md:1962`). The verdict surface is
`whitefootc --par-ledger`, which prints one line per analyzed site. **No
ledger snapshot for the corpus as a whole is stored in the tree**, so for most
programs the answer is **unknown without running the compiler** (out of scope
for this read-only census). What *is* recorded:

**Pinned in a test — `tests/programs/par_layout.wf`.**
`compiler/tests/programs/parallel.rs:88–96` asserts
`pair(layout, layout)  eligible` and `pair(layout_banded, layout_banded)
eligible` from `program_permission_ledger("par_layout.wf")`, and the same file
asserts both folds are handed out under `--par --par-scalar-leaf-limit off`.
`research/investigations/proof-derived-parallelism/RESULTS.md:52–73` records the
full transcript, including a third permitted pair `pair(build, build)` at
line 19 and five condition-1 denials.

**Recorded PAR-2 loop ledgers — 12 files under
`research/experiments/blind-writer/2026-08-28/ledger/*.txt`.** These are the
most directly relevant evidence for the next study, because the denial reason
is *exactly* a held loan:

| file | permitted loops | denial reason that matters here |
|---|---|---|
| `probe_a_staged_permitted` | `:34` | `:17` — "an iteration holds an exclusive loan on storage the iteration does not introduce, at `&uniq 'f files`" |
| `probe_b_staged_denied` | `:34` | same, at `:17` |
| `probe_b1_write_after_loop` | `:34` | same, at `:17` |
| `probe_c_helper_denied` | `:47` | same, at `:26` |
| `probe_c_inline_same_regions` | `:39` | same, at `:17` |
| `p1_tree_wc` | `:226` | `:53`, `:65`, `:335` — "the body contains a statement that forms a borrow of storage the iteration does not introduce" |
| `p4_copy_count` | `:149` | same, at `:38` |
| `p2_tree_grep` | none | `:28`, `:40`, `:139`, `:307` same; plus condition 4 (return leaves loop) and condition 1 (non-associative carried write) |
| `p3_checksum` | none | `:39`, `:54` same; `:166`, `:189` condition 1 |
| `p5_two_outputs` | none | `:35`, `:114` same |
| `probe_e_hoisted_length` | none | `:35`, `:116` same |
| `probe_d_reborrow_two_statements` | (no loop) | — |

**The check the next study must run.** PAR-2 condition 2 denies a counted loop
whose body *forms a borrow of*, or *holds an exclusive loan on*, storage the
iteration does not introduce. Every `with inout` block in the no-reference
model is such a held exclusive loan. The three class-F loop-spanning sites
(F-14, F-15, F-16) and every `with` block that encloses a loop sit squarely in
that condition. Whoever ports the corpus must re-take these twelve ledgers plus
`par_layout.wf`'s pair ledger and confirm the permitted rows are still
permitted — a `with` block that hoists a loan *out* of the loop body may in
fact convert some of these denials into grants, which would be a result worth
recording either way.

**Everything else.** For the remaining 176 `.wf` files, including the whole of
`research/experiments/compute-bench/programs/` where all 54 range-partition
sites live, the PAR-1/PAR-2 verdicts are **unknown** — no snapshot exists in
the tree and `--par-ledger` was not run. `research/experiments/compute-bench/`
does carry a `verdict.awk` / `verdict-test.sh` pair for reading ledgers, so
re-taking them is cheap.

---

## 5. Provenance notes

* Six files under `research/experiments/blind-writer/2026-08-28/` use the
  legacy lowercase `slice<'s, u8>` view spelling and the legacy
  `match cvt<u64, u8>(…)` form; they predate the current spec and may not
  compile against it. Their sites are counted, and their shapes are the
  region-discipline shapes the current corpus still uses.
* `research/investigations/proof-derived-parallelism/probes/README.md` states
  that seventeen of the twenty-eight probes there carry pre-v0.34 `claim` forms
  and no longer compile. They are archived evidence, not a gated corpus; their
  sites are counted because their shapes (recursive `Box` tree folds writing
  per-node slots) are the ones the pool-handle question turns on.
* 41 of the 189 files contain no borrow and no view at all — mostly
  `division-dissolution`, `strict-clause-retirement`, `o11-composition`,
  `codegen-vs-rust-c`, and the arithmetic probes.


# File: census-snapshot.md

# Census: `tests/snapshot/` under the no-reference core model

Scope: every `.wf` file under `tests/snapshot/cases/` on branch `research/access-effects`
in `/private/tmp/whitefoot-access-effects-research` — **484 files, 12 categories**.
Read-only census; no repository file was modified.

Question being answered (M11): can a core with **no first-class references** —
three call conventions `let` / `inout` / `sink`, second-class projections
`with inout v[i] as p { … }` that may be passed down but never stored or returned,
pool handles `Handle<P,T>` for shared or cyclic structure, and index-based access —
express what this corpus expresses, and which recorded verdicts change?

## 0. What counts as a site, and how it was counted

A **site** is one syntactic occurrence at which a borrow (`&`, `&uniq`, `&'r`,
`&uniq 'r`) or a view (`Slice<'r,T>`, `MutSlice<'r,T>`) is formed, bound, passed,
returned, stored or held. Six disjoint site kinds were counted; each was produced by
`grep` and then confirmed by reading every file that contributes a non-`place_back`
site (231 files contain a borrow or view token; 67 contain a view token).

| kind | what it is | count |
| --- | --- | --- |
| A1 | borrow expression in a call-argument position (`f(x: &uniq v)`) | 1269 |
| A2 | borrow-mode parameter declaration (`fn f(out: &uniq MutSlice<u8>)`) | 23 |
| A3 | own-mode view parameter declaration (`fn g(b: own Slice<u8>)`) | 59 |
| A4 | view value passed by value at a call (`g(b: wp)`) | 25 |
| B1 | view formation bound with `let` (`slice_of` / `mut_slice_of`) | 42 |
| B2 | borrow bound with `let` (`let handle = &uniq view_target;`) | 2 |
| | **total sites** | **1420** |

Reconciliation of the raw token counts: 1295 `&` tokens occur in the corpus; one of
them (`kills__adversary-r2__13-struct-field-ref-write-kill.wf:19`) is inside a `doc`
string and is not a site, leaving 1294 = A1 (1269) + A2 (23) + B2 (2). Of the 1269
A1 sites, 1166 are `place_back(vector: &uniq v, value: …)`; the remaining 103 are 25
`arena_vector_proved(store: &uniq workspace, …)` / 1 `heap_vector(store: &uniq heap, …)`,
17 borrows of a `MutSlice` descriptor, 7 `args_count`/`arg_get` on `Args`, 3
reborrows through a held borrow (`&uniq deref(out)`), and helper calls on structs and
views. View formations: 26 `slice_of` + 16 `mut_slice_of` = 42, of which the 26 shared
views account for 25 A4 passes plus one `&payload` borrow, and the 16 exclusive views
account for 15 direct `&uniq view` arguments plus 2 B2 bindings.

## 1. Per-category table

Accept/reject columns are the `verdict` column of `tests/snapshot/index.tsv` — the
gate's recorded expectation for this compiler today. "Expected-rejected" cases are
**not** capabilities the model must preserve.

| category | files | accept | expected-reject | A | B | C | D | E | F | G | H |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| accumulators | 51 | 31 | 20 | 190 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| arithmetic-domain | 53 | 37 | 16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| certificates | 34 | 11 | 23 | 12 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| contracts | 24 | 17 | 7 | 96 | 10 | 0 | 0 | 0 | 0 | 0 | 0 |
| cursor-loops | 54 | 40 | 14 | 214 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| diagnostics | 21 | 0 | 21 | 43 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| header-forms | 38 | 26 | 12 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| indexing | 52 | 38 | 14 | 453 | 19 | 0 | 0 | 0 | 0 | 0 | 0 |
| joins | 37 | 22 | 15 | 44 | 3 | 0 | 0 | 0 | 0 | 0 | 0 |
| kills | 56 | 34 | 22 | 217 | 5 | 0 | 0 | 0 | 0 | 0 | 0 |
| real-programs | 14 | 14 | 0 | 107 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| signed | 50 | 27 | 23 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| **total** | **484** | **297** | **187** | **1376** | **44** | **0** | **0** | **0** | **0** | **0** | **0** |

Per-category A breakdown (A1/A2/A3/A4) and B breakdown (B1/B2):

| category | A1 | A2 | A3 | A4 | B1 | B2 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| accumulators | 186 | 0 | 2 | 2 | 2 | 0 |
| certificates | 12 | 0 | 0 | 0 | 0 | 0 |
| contracts | 82 | 3 | 5 | 6 | 8 | 2 |
| cursor-loops | 208 | 0 | 3 | 3 | 3 | 0 |
| diagnostics | 35 | 3 | 5 | 0 | 2 | 0 |
| indexing | 422 | 6 | 12 | 13 | 19 | 0 |
| joins | 10 | 3 | 31 | 0 | 3 | 0 |
| kills | 207 | 8 | 1 | 1 | 5 | 0 |
| real-programs | 107 | 0 | 0 | 0 | 0 | 0 |
| **total** | **1269** | **23** | **59** | **25** | **42** | **2** |

`arithmetic-domain`, `header-forms` and `signed` (141 files) contain no borrow and no
view token at all: they are scalar-domain, loop-header and signedness programs.

## 2. Sites in classes C, D, E, F, G, H

**There are none. C = D = E = F = G = H = 0 across all 484 files.**

The negative results were each established by a mechanical sweep and then confirmed by
reading:

- **C (returned borrow/view).** `grep -c 'return *&'` → 0. Every function result in the
  corpus is scalar, `unit`, `Bool`, an enum, or an owned `FixedVector`; the full
  signature list (62 distinct signatures involving `&` or `Slice`) contains no
  `-> r: own Slice<…>` or `-> r: own MutSlice<…>`. The language *does* admit a yielded
  view with an origin ceiling (`spec/kernel-spec.md` [FN-1] line 1591 and [VIEW-…]
  line 1227); this corpus never uses it.
- **D (stored into a field, payload, element or capture).** All 128 lines of `struct`
  and `enum` declarations were dumped; the complete set of field types is `u64`, `u32`,
  `u8`, `FixedVector<u32,4>`, `FixedVector<u8,4>`, `Position`, `DelayLine`. No
  reference or view field exists. The language already forbids it — [STOR-5]
  (spec lines 1285–1294) makes any region-bearing type in stored content a hard error.
  The corpus has no closures.
- **E (borrow/view live across a call or write that touches the same container by
  another path).** A per-file sweep matched each `slice_of(&X)` / `mut_slice_of(&uniq X)`
  binding against every later `place_back(vector: &uniq X …)` or `set X[…]` in the same
  file: **zero hits**. Every direct container access (12 occurrences) happens after the
  `region` block holding the view has closed. The two `let`-bound borrows (B2) are each
  consumed by the very next statement.
- **F (view split or partition handed to helpers).** The three-operand forms
  `slice_of(&p, start, end)` / `mut_slice_of(&uniq p, start, end)` that the spec admits
  (line 1197) appear **zero** times; all 42 formations are whole-container. No two views
  of the same container are ever live at the same time. Range restriction is always
  expressed the way the model already wants it — by passing `start`/`end` indices
  alongside the whole view (`copy_window(src: read_src, dst: &uniq view_dst, start: 2, end: 5)`).
- **G (loop-carried across iterations).** No view or borrow variable is ever reassigned:
  `grep` for `set <viewvar> =` → 0, and the language forbids it outright ([VIEW-4],
  spec line 1222: `set p = e` at a view-typed target is a hard error). No `let x = <view>`
  aliasing copy exists either — all 15 `let a = b;` statements copy scalars.
- **H (anything else).** Nothing left over. The exotic-looking sites are all A or B; see
  §3.

Consequently the "every accepted-case site in C/D/E/G/H, and every F site spanning a
call boundary" list that this census was asked to produce is **empty**, and the
**unwritable count is 0**.

## 3. Closest approaches — the sites a reader would expect to land in C–H

These are the seven shapes that come nearest to needing a first-class reference. Each
is recorded with its actual class and its repair, because the value of this census is in
showing *why* they stay inside A/B.

### 3.1 Reborrow of a projection parameter, once per loop iteration — class A

`tests/snapshot/cases/contracts/contracts__writer-r1__10_append_within_capacity.wf:9-13`
(recorded verdict: **accept**)

```
 9  fn fill_all(out: &uniq MutSlice<u8>, value: own u8) -> result: own unit reads(out), writes(out) {
10    let spare = len_of(deref(out));
11    for (i in 0_u64..spare) {
12      store_at(out: &uniq deref(out), offset: i, value: value);
13    }
14    return unit;
```

This is the strongest demand in the corpus: the projection must stay usable across a
loop whose body calls a helper that *writes through it*, and `spare`, read once before
the loop, must survive every such call. Repair: `fn fill_all(inout out: [u8], …)` with
`store_at(inout out, …)` — an `inout` parameter re-passed as `inout`. Requires only
that the model's projection measure be killed by a write to the origin's *length*, not
by a write through the projection; this is exactly what the current [CALL-3] rule
already gives and what the row's own `doc` records. Not G: no binding crosses a
backedge; a fresh reborrow is formed and dies inside each iteration.

### 3.2 Borrow of a view bound to a local and then moved into a call — class B

`tests/snapshot/cases/contracts/contracts__writer-r1__10_append_within_capacity.wf:31-37`
(accept) and `…/contracts__writer-r1__02_buffer_capacity_copy.wf:37-47` (expected-reject, FN-8)

```
31        region {
32          let view_target = mut_slice_of(&uniq target);
33          region {
34            let handle = &uniq view_target;
35            fill_all(out: move handle, value: 5_u8);
36          }
37        }
```

The only two `let`-bound borrows in the corpus (B2 = 2), and the only place a *reference
to a projection* is named. Repair: delete the intermediate binding —
`with inout target as p { fill_all(inout p, 5_u8); }`. Pure syntax; the `handle`
binding is a no-op holder that the model removes rather than needs.

### 3.3 One exclusive projection, two mutating calls, measure must survive both — class A/B

`tests/snapshot/cases/indexing/indexing__writer-r2__07_bucket_index_from_hash.wf:30-37`
(accept)

```
30      region {
31        let view_counts = mut_slice_of(&uniq counts);
32        region {
33          bucket_of(counts: &uniq view_counts, hash: 19_u64);
34          bucket_of(counts: &uniq view_counts, hash: 27_u64);
35        }
36      }
37      let landed = counts[3_u64];
```

`bucket_of` has `requires buckets > 0` over `len_of(deref(counts))`; the second call
needs that measure to have survived the first. Repair:
`with inout counts as p { bucket_of(inout p, 19); bucket_of(inout p, 27); }`. Not E:
the writes go *through* the projection, not around it; `counts[3]` at line 37 is after
the projection's block closed.

### 3.4 Two sequential exclusive views of one container — class B (not F)

`tests/snapshot/cases/joins/joins__writer-r1__r12_writes_join_common_bound_accept.wf:36-48`
(accept)

```
36      region {
37        let view_scratch = mut_slice_of(&uniq scratch);
38        region { store_byte(out: &uniq view_scratch, i: 3_u64, value: 42_u8, pick: take_direct); }
41      }
42      region {
43        let view_scratch = mut_slice_of(&uniq scratch);
44        region { store_byte(out: &uniq view_scratch, i: 5_u64, value: 9_u8, pick: take_reversed); }
47      }
48      let observed = scratch[3_u64];
```

Two exclusive views of the same storage, never simultaneously live. Repair: two
successive `with inout scratch as p { … }` blocks. No partition proof is required
because no partition is formed.

### 3.5 Exclusive projection live across a loop of element writes, then a direct read — class A/B

`tests/snapshot/cases/kills/kills__writer-r1__kill03_buffer_write_loop_invariant.wf:33-39`
(accept)

```
33      region {
34        let view_scratch = mut_slice_of(&uniq scratch);
35        region {
36          let done = fill_running_sum(out: &uniq view_scratch, count: 5_u64);
37        }
38      }
39      if scratch[4_u64] == 10_u64 {
```

The case exists to record that element writes through a view kill element facts but not
the run's measure. Repair: `with inout scratch as p { fill_running_sum(inout p, 5); }`
followed by the direct `scratch[4]` read once the block closes. The kill discipline the
case pins is a property of the effect rules, not of reference representation.

### 3.6 Projection live across nested `match` arms, region blocks and an intervening call — class A/B

`tests/snapshot/cases/diagnostics/diagnostics__writer-r1__r13_system_range_unproved.wf:18-26`
(expected-reject, FN-8 — the destination-length premise is missing, by design)

```
18        region {
19          let window = mut_slice_of(&uniq bytes);
20          region 'a {
21            match arg_get(args: &args, position: 1_u64) {
22              Ok(value: text) => {
23                region 'v {
24                  region {
25                    let end = args_count(args: &'a args);
26                    match host_copy_bytes(value: &'v text, destination: &uniq window, start: 0_u64, end: end) {
```

The deepest nesting in the corpus: an exclusive projection of `bytes` held across four
block levels, two `match` scrutinees and two calls on *other* objects (`args`). Repair:
`with inout bytes as w { … host_copy_bytes(let text, inout w, 0, end) … }`; the
intervening calls touch `args` and `text`, never `bytes`, so no E condition arises. The
explicit regions `'a` / `'v` become plain block scopes.

### 3.7 Three borrows and a borrow-of-a-view at one call — class A

`tests/snapshot/cases/diagnostics/diagnostics__writer-r2__r06_world_value_sysrange_no_branch.wf:1-4, 26-29`
(expected-reject, FN-8)

```
 1  fn publish(factory: &uniq HandleFactory, output: &uniq OutputStream, source: &Slice<u8>, start: own u64, end: own u64) -> …
 4      match write_once(factory: &uniq deref(factory), output: &uniq deref(output), source: source, start: start, end: end) {
…
26      region 'o {
27        let payload = slice_of(&bytes);
28        region {
29          let result = publish(factory: &uniq factory, output: &uniq 'o output, source: &payload, start: 0_u64, end: n);
```

The only `&Slice<…>` parameter (a shared borrow *of a projection*) and the only
`&uniq 'r` with an explicit region. Repair: two `inout` providers, one `let`-convention
projection argument; `source` is then passed down as an ordinary projection argument,
which is precisely the "passed down but never stored" permission. Three simultaneous
`inout`s on three distinct objects nest as three `with` blocks or, more naturally, as
three `inout` parameters at one call.

## 4. Construct adjacent to the census that the model must still answer

Not a borrow and not a view, so outside classes A–H, but it is the one place in the
corpus where something region-bearing *escapes a call*, and M11 should see it:

```
tests/snapshot/cases/indexing/indexing__writer-r2__01_offset_length_window_copy.wf:18-20
18      let workspace = arena_frame::<16, 1, 'w>();
19      region {
20        let src = arena_vector_proved::<u8>(store: &uniq workspace, count: 8_u64);
```

`arena_vector_proved(store: &uniq Arena<'s,…>, count) -> result: own Vector<'s,T>`
(spec line 1048) takes a provider by unique borrow and **returns a region-confined
owned run**. 24 `arena_vector_proved` sites + 1 `heap_vector` site + 18 `arena_frame`
sites, in 19 files across `accumulators`, `contracts`, `diagnostics`, `indexing`,
`joins`, `kills` and `real-programs`. The `&uniq workspace` argument itself is class A (`inout`); the
*result* is exactly the `Handle<P,T>` shape — an owned handle into a pool whose
lifetime the pool controls. If the no-reference model keeps `Handle<P,T>`, these 43
sites are covered; if it did not, they would be the only unwritable construct in the
corpus.

Separately, `indexing__writer-r1__merge_two_pointer.wf` threads an owned
`FixedVector<u32,3>` in and out of a recursive helper (`out: own …` moved in,
returned). Under the model that is `inout` with no return — a restructure, not a
capability gap; its recorded verdict (reject, FN-9) is about recursive summaries and is
unaffected.

## 5. Summary of totals

| quantity | value |
| --- | ---: |
| files censused | 484 |
| accepted today | 297 |
| expected-rejected today | 187 |
| files containing a borrow or view token | 231 |
| files containing a view (`Slice`/`MutSlice`) token | 67 |
| total borrow/view sites | 1420 |
| class A (projection argument / projection parameter) | 1376 |
| class B (`with … as` block) | 44 |
| class C (returned) | 0 |
| class D (stored) | 0 |
| class E (held across a foreign write) | 0 |
| class F (split/partition across calls) | 0 |
| class G (loop-carried) | 0 |
| class H (other) | 0 |
| **unwritable under the no-reference model** | **0** |

Every one of the 1420 sites has a mechanical repair: 1376 become a projection argument
or an `inout` / `let` parameter, and 44 become a `with … as` block. Not one needs a
reference to be stored, returned, or held across a write it does not itself perform.

## 6. Acceptance-set diff (the M11 answer)

**Forced flips: 0.** No recorded verdict changes as a consequence of removing first-class
references. Two independent checks support this:

1. *No accept depends on a removed capability.* All 1420 sites are A or B, so every
   accepted program is rewritable with the same statements, the same proofs and the same
   obligations.
2. *No rejection is caused by a borrow rule.* The 187 rejections cite OP-2 (60), OP-4
   (47), PRF-1 (30), INV-1 (22), FN-8 (10), FN-9 (7), FN-1 (4), TYPE-5 (2), and one each
   of TYPE-6, SET-1, OWN-1, OP-6, OP-1. None cites an aliasing, view or call-transport
   rule (`OWN-5`, `VIEW-*`, `CALL-*`, `BLK-4`, `STOR-5`). The single `OWN-1` rejection,
   `kills__adversary-r1__p6_consuming_move_kills_fact`, is a *move* after consumption,
   which the `sink` convention reproduces unchanged. The four `FN-1` rejections are all
   non-total `cvt` pairs, not slice results.

**Conditional flips: 2, both reject → accept, and only under one specific model choice.**

| case | today | would flip to | condition |
| --- | --- | --- | --- |
| `accumulators__writer-r2__p13_slice_param_sum` | reject OP-2 | accept | projection parameters publish their origin set's measure into the callee |
| `accumulators__adversary-r1__p12_per_byte_widened_checked_sum` | reject OP-2 | accept | same |

Both reject for the same reason, recorded in their own rows: *"no caller fact crosses
into the callee, so `len_of(data)` is unbounded here"* and *"`len_of(deref(weights))` is
unbounded at this signature"*. Each is a whole-container projection of a fixed-extent
origin (`const FixedVector<u8,5>` and a 4-element `FixedVector<u8,4>` respectively),
passed to a helper that sums bytes with a checked `+`. A second-class projection carries
an **origin set** that is statically known and cannot escape; if the model lets a
projection parameter publish its origins' common length bound, both helpers get the
premise their checked add needs and both cases accept. If projection parameters stay
opaque in extent — as `own Slice<T>` is today — both stay rejected. Note that both rows
already carry `finder_expectation = accept, agreement = no`: the authors expected these
to pass, so the flip would resolve two of the corpus's 22 recorded disagreements.

Cases deliberately examined and found **not** to flip, despite looking like candidates:

- `contracts__writer-r1__02_buffer_capacity_copy` (reject FN-8). The missing premise is
  `len_of(source) <= len_of(target)`. The `place_back` fill loops export only
  `len_of(source) >= 3` and `len_of(target) >= 5`; an exact origin extent does not supply
  an upper bound on `len_of(source)`. Same obligation, same failure, under either model.
- `indexing__writer-r2__05_binary_search_loop` (reject INV-1) and
  `indexing__adversary-r2__14-chunk-carry-shift-wrong-start-reject` (reject INV-1): loop
  header joins, no reference content.
- `kills__writer-r2__09_region_exit_kills_borrow_fact` (accept). Its point is that a
  region-local borrow holder's fact dies at region exit and a fresh owned `len_of`
  re-establishes it. A `with` block gives the same block-scoped kill, so the accept holds.
- The six `agreement = no` rows whose `doc` records a B7c4b-1 move to the view surface
  (`indexing__writer-r1__subslice_copy`, `indexing__writer-r2__01`, `…__07`, `…__11`,
  `joins__writer-r1__r12`, `kills__writer-r1__kill03`) all accept *because* a write
  through a view kills element storage and not the origin's measure [CALL-3]. A
  projection with an origin set reproduces that rule exactly; they stay accepted.

**Bottom line for M11:** on this corpus the no-reference core is not merely sufficient,
it is strictly under-exercised — the corpus never returns, stores, splits, loop-carries
or aliases a borrow, and 97% of its 1420 borrow/view sites are plain call arguments. The
corpus therefore supplies *no evidence against* the model and, equally, *no evidence for*
the harder mechanisms (`origin`-carrying yielded projections, proved range partitions,
pool handles for cyclic structure): those must be justified from `tests/conformance/`,
`research/`, or new programs, not from here. The single construct here that genuinely
needs a non-`with` mechanism is the arena/heap allocation result (`Vector<'s,T>` escaping
a call that took `&uniq Arena<'s,…>`), 43 sites, which `Handle<P,T>` covers.


# File: census-conformance.md

# Census: `tests/conformance` + `tests/codegen` against a no-first-class-reference core

Worktree `/private/tmp/whitefoot-access-effects-research`, branch `research/access-effects`.
Read-only census. Nothing in the repository was modified.

**Model under test (M11).** Three call conventions `let` / `inout` / `sink`; second-class
projections `with inout v[i] as p { ... }` that may be passed down but never stored or
returned; pool handles `Handle<P, T>` for shared or cyclic structure; index-based access.
No `&`, `&uniq`, `&'r`, `&uniq 'r`, no `Slice<'r,T>` / `MutSlice<'r,T>` values, no regions.

---

## 1. Corpus table

### 1.1 Files and verdicts

| | conformance | codegen | total |
|---|---|---|---|
| `.wf` files | 805 | 95 | **900** |
| accepted (manifest `expect.kind` = `run` or `accept`) | 426 | 95 | **521** |
| expected rejection (`expect.kind` = `reject`) | 379 | 0 | **379** |
| accepted files containing ≥1 borrow/view site | 244 | 47 | **291** |
| borrow/view sites in accepted files | 1532 + 8\* | 60 | **1600** |
| borrow/view sites inside expected-rejection cases (not classified here) | 561 | 0 | 561 |

\* 8 caller-side sites that hold a *returned* borrow/view (`let r = f(...)`) carry no `&`
or `Slice` token and are added by hand; see §2.1.

Manifest: `tests/conformance/manifest.jsonl`, one JSON object per line, `id` →
`cases/<id>.wf`, `rules[]` = spec rule ids exercised, `expect` = `{kind: run|accept|reject,
rule: <id>}`, `status` = `runnable` (803) / `pending` (1) / `xfail` (1).
`tests/codegen` has **no reject verdicts at all**: its `cases.json` files record
`proof_classification` (`elided` / `checked` / `proved` / `retained` / `mixed`), i.e. whether
a bounds check is removed. The `n*`-prefixed cases are *negative controls for proof
elision*, not rejected programs, so all 95 codegen sources count as accepted.
Confirmed by reading `tests/codegen/README.md` and
`tests/codegen/cases/bounds/derived-range/n15-remainder-tail-uniq-alias-mutation.wf`.

### 1.2 Sites per class (accepted cases only)

Site = one textual occurrence where a borrow or a view is formed, bound, passed, returned,
stored or held. The `&` operand of `slice_of` / `mut_slice_of` is folded into the view
formation it feeds (VIEW-2: "the formed value, not the argument borrow, holds the loan"),
so it is not double counted.

| class | description | sites | share |
|---|---|---:|---:|
| **A** | passed only as a call argument (projection argument) | **1409** | 88.1 % |
| **B** | `let`-bound, used only inside its block, never stored or returned | **101** | 6.3 % |
| **C** | returned from a function | **23** | 1.4 % |
| **D** | stored into a struct field, enum payload, container element, or captured | **0** | 0 % |
| **E** | held live across a call that writes the same container (P4 cursor) | **0** | 0 % |
| **F** | view split or partition handed to helpers across calls | **64** | 4.0 % |
| **G** | loop-carried across iterations | **2** | 0.1 % |
| **H** | anything else | **1** | 0.06 % |
| | **total** | **1600** | |

Class A breaks down as 1032 call arguments + 317 borrow-mode formal parameters
+ 60 view-typed formal parameters. The formal-parameter half is the receiving end of the
same convention; it is where `let` / `inout` / `sink` would be written.

**Operand shape of the 1426 borrow sites** (what the `&` is taken of) — this is the number
that decides how much projection machinery M11 actually needs:

| operand | sites | meaning under M11 |
|---|---:|---|
| whole variable `v` (845 call args, 317 formals, 37 `let` holders) | 1199 | plain `let` / `inout` argument; no projection needed |
| `deref(p)` reborrow of a borrow-mode parameter | 191 | pass the projection down |
| `deref(p).f` / `deref(p)[i]` | 16 | re-project an incoming projection |
| field `v.f` | 9 | `with inout v.f as p` |
| element `v[i]` | 1 | `with inout v[i] as p` |
| returned-borrow expression `&'r deref(x)` | 4 | yielded projection (class C) |
| borrow in a result *type* position (not an operand) | 6 | the yield declaration itself |

**Only 26 of 1426 borrow sites (1.8 %) project a field or an element at all.** The corpus
is overwhelmingly whole-variable argument passing.

### 1.3 Expected rejections per ownership rule id

Ownership-family rejections (OWN-\*, VIEW-\*, STOR-5, LIV-\*): **73 of 379** (19.3 %).

| rule | cases | rule | cases |
|---|---:|---|---:|
| OWN-5 | 27 | OWN-11 | 3 |
| OWN-1 | 18 | OWN-12 | 3 |
| STOR-5 | 5 | LIV-2 | 2 |
| OWN-4 | 4 | OWN-3 | 2 |
| OWN-10 | 4 | VIEW-2 | 2 |
| VIEW-4 | 1 | VIEW-6 | 1 |
| LIV-1 | 1 | | |

Adjacent but outside the requested set, listed for completeness: STOR-1 4, STOR-4 2.
The remaining 306 rejections cite non-ownership rules (TYPE-5 23, OP-4 20, PROV-6 18,
OP-1 16, BLK-0 15, FN-8 15, TYPE-7 15, INV-1 14, … 65 distinct rule ids in all).

### 1.4 The headline finding on class D

**Class D is empty, and it is empty by construction.** STOR-5 states: *"No struct field,
enum variant payload, `array`/`buffer`/run element, or `box`/`arena` content may be a borrow
or a region-bearing type"*, and *"the `field`/`vfield` grammar admits only `type`, and
`type` has no borrow (`&` / `&uniq`) production [GRAM-3]"*. VIEW-1 adds: *"Neither view is
ever stored in a nominal field, an enum payload, or a run slot [STOR-5], and neither is a
generic type argument [FN-2]; a view crosses a function boundary only as one direct
parameter or one direct `own` result."*

So Whitefoot's references are **already second-class in the storage dimension**. The scan
confirms it: zero field/payload/element sites in 900 files, accepted or rejected. The only
two powers the current model has that M11 removes are (C) returning a borrow or a view and
(B/F) binding one to a named `let` holder whose extent is a region rather than a block.

---

## 2. Every accepted-case site in classes C, D, E, G, H, and F across a call

### 2.1 Class C — returned (23 sites, 9 distinct return points, 9 files)

Under M11 each of these is expressible **only** as a yielded projection carrying an `origin`
set. Every one of them is a projection of a parameter except C-4, which is a projection of a
`const`.

**C-1.** `tests/conformance/cases/own4-pos-return-caller-borrow.wf:1` (sig) and `:2` (return)

```
fn passthru['r0](x: &'r0 i32) -> return_value: &'r0 i32 pure {
  return &'r0 deref(x);
}
```

Projection of a parameter. **Repair:** `fn passthru(let x: i32) -> yields let i32 origin(x)`,
consumed by the caller in a `with` block. Expressible iff M11 admits yielded projections (R1).

**C-2.** `tests/conformance/cases/own6-pos-callresult-borrow-chain.wf:1`, `:2` (callee) and
`:15` (caller holds the returned borrow)

```
15:      let r = passthru(x: &uniq deref(h));
16:      region {
17:        bump(n: &uniq deref(r));
18:      }
```

Projection of a parameter, then re-projected one more level (`&uniq deref(r)`) and passed to
`bump`. **Repair:** `with inout v as h { with yield passthru(h) as r { bump(r) } }` — the
returned projection must bind in a `with`, not a `let`. Expressible iff yielded projections
may be re-projected.

**C-3.** `tests/conformance/cases/fn1-pos-result-provenance-distinct-regions.wf:1`, `:3`
(callee) and `:11` (caller)

```
 1: fn pick['r](a: &uniq 'r i32, b: &uniq i32) -> result: &uniq 'r i32 pure {
 3:   return &uniq 'r deref(a);
11:      let r = pick(a: &uniq 'a v, b: &uniq w);
12:      set deref(r) = 7_i32;
```

Projection of parameter `a` only; the second region on `b` is exactly FN-1's "one candidate"
repair. **Repair:** `origin(a)` on the yielded projection; the caller writes through it
inside the `with`. Expressible.

**C-4.** `tests/conformance/cases/fn1-pos-result-provenance-zero-candidate.wf:2`

```
formal ConstantSource {
  fn base['r]() -> result: &'r i32 pure;
}
```

Zero-candidate arm: no parameter is an origin, so the only legal source is `const` storage.
**Repair:** `origin(const)` — a projection of immutable static storage, which never dangles.
Expressible; M11 needs one distinguished `const` origin.

**C-5.** `tests/conformance/cases/form8-pos-related-pair-written.wf:1` (sig) and `:11`
(caller)

```
 1: fn pass['r](value: &'r i32) -> result: &'r i32 pure {
 3:   return value;
10:    let p = &a;
11:    let held = pass(value: p);
12:    let observed = deref(held);
```

Projection of a parameter, passed in and handed straight back. **Repair:** `origin(value)`.
Expressible.

**C-6.** `tests/conformance/cases/fn1-pos-returned-slice-const-run.wf:3` (sig), `:5`
(return of the bound view), `:10` (caller)

```
1: const fixed_bytes: FixedVector<u8, 3> =[7_u8, 13_u8, 19_u8];
3: fn fixed_view['r]() -> result: own Slice<'r, u8> pure {
4:   let view = slice_of(&'r fixed_bytes);
5:   return view;
6: }
```

A view of a `const` returned from a parameterless function. **Repair:** yielded projection
with `origin(const)`, same mechanism as C-4. Expressible.

**C-7.** `tests/conformance/cases/fn1-pos-returned-slice-inputs-run.wf:1`, `:5` (sigs),
`:34`, `:49` (callers)

```
 1: fn pass_slice['r](value: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
 5: fn choose_slice['r](take_left: own Bool, left: own Slice<'r, u8>, right: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
 6:   if take_left {
 7:     return left;
 8:   } else {
 9:     return right;
10:   }
```

**This is the case the `origin` set exists for.** `choose_slice` returns one of two
parameters chosen at run time, so the yielded projection's origin is the *set*
`{left, right}` and the caller must treat the result as aliasing both. **Repair:**
`-> yields let u8[] origin(left, right)`; the caller opens both `with` blocks and consumes
the yield inside them. Expressible, and it is the strongest argument for making `origin` a
set rather than a single name.

**C-8.** `tests/conformance/cases/view6-pos-a-helper-publishes-the-child-of-its-destination.wf:1`
(sig), `:7` (return), `:25` (caller)

```
1: fn fill_and_publish['r](destination: &uniq MutSlice<'r, u8>, value: own u8) -> filled: own Slice<'r, u8> writes(destination) contract {
2:   requires 2_u64 <= len_of(deref(destination));
3: } {
5:   set deref(destination)[0_u64] = value;
6:   set deref(destination)[1_u64] = value;
7:   return slice_of(&'r deref(destination));
8: }
```

Fill-and-publish: an `inout` projection in, a *shared* projection of the same storage out.
**Repair:** `fn fill_and_publish(inout destination: u8[], let value: u8) -> yields let u8[]
origin(destination)`, i.e. an exclusive projection may yield a shared child of itself. This
is a real requirement on M11: the yield's strength must be allowed to differ from the
parameter's. Expressible if that downgrade is admitted.

**C-9.** `tests/codegen/cases/bounds/output-capacity-lockstep/n31-helper-return-alias-escape.wf:1`
(sig), `:2` (return)

```
1: fn ident['r](x: &uniq 'r buffer<u8>) -> result: &uniq 'r buffer<u8> pure {
2:   return &uniq 'r deref(x);
3: }
```

Identity pass-through, used to build an alias that must defeat bounds-fact propagation.
**Repair:** delete the helper and project `out` directly (see G-2). Expressible; the case's
purpose is a codegen negative control, not a language capability.

### 2.2 Class D — stored (0 sites)

None. Forbidden by STOR-5 and by the `type` grammar, which has no borrow production; views
are additionally excluded from fields, payloads and run slots by VIEW-1. Nothing to repair.

### 2.3 Class E — held live across a call that writes the same container (0 sites)

None. OWN-5's resolved-place exclusivity makes the P4 cursor shape unrepresentable today: a
live `&uniq v` excludes every other usable access path to `v`, and a live shared borrow
excludes writes. The eight candidates the scan surfaced (a holder live while its root is
touched again) are all *sibling view formations*, i.e. class F, not writes through a second
path: `x-buffer-borrowed-columns-run.wf:92`/`:109`,
`view2-pos-two-shared-views-of-one-place.wf:16`, `view2-pos-a-view-over-a-run.wf:13`,
`view2-pos-a-shared-child-reborrow-of-an-exclusive-view.wf:17`,
`own5-pos-a-unique-borrow-of-a-parent-view-after-its-child.wf:19`,
`own5-pos-recursive-adjacent-child-ranges.wf:11`/`:12`.

The corresponding *rejections* are `exclusive-neg-view-live-during-call`,
`prov3-neg-an-append-while-a-copy-view-is-still-used` and
`view2-neg-an-element-write-while-a-child-view-lives`. **Consequence for the owner: dropping
first-class references costs the corpus nothing in class E, because the current language
already refuses it.**

### 2.4 Class G — loop-carried (2 sites)

**G-1.** `tests/conformance/cases/run-sysin-read-to-end.wf:25`

```
24:        region {
25:          let window = mut_slice_of(&uniq chunk);
26:          loop @chunks {
27:            let ended = 0_u8;
28:            region {
29:              match read_next(factory: &uniq entry_factory, input: &uniq input, destination: &uniq window, start: 0_u64, end: 8_u64) {
```

One exclusive view of a reusable 8-byte chunk, formed once and reused by every iteration of
an unbounded read loop. **Repair:** hoist the `with` above the loop —
`with inout chunk as window { loop @chunks { read_next(..., destination: window, ...) } }`.
Expressible, and it requires only that a `with`-bound projection be usable inside a loop
nested in its block. Re-forming the projection per iteration would also work here, but only
because the chunk is not stateful across iterations.

**G-2.** `tests/codegen/cases/bounds/output-capacity-lockstep/n31-helper-return-alias-escape.wf:9`

```
 5: fn probe(out: &uniq buffer<u8>, src: own buffer<u8>) -> result: own u64 writes(out) {
 9:   let alias = ident(x: &uniq 'r deref(out));
10:   loop @groups {
11:     let rem = n -wrap i;
12:     if rem < 3_u64 { break @groups; }
15:     set deref(alias)[o] = 0_u8;
```

A returned exclusive borrow bound outside the loop and written through inside it — the one
place in the corpus where a *returned* reference is also loop-carried. **Repair:** drop the
`ident` indirection and write `with inout out as alias { loop @groups { set alias[o] = 0_u8;
... } }`. Expressible.

### 2.5 Class H — anything else (1 site)

**H-1.** `tests/conformance/cases/own5-pos-rhs-borrow-is-disjoint-from-captured-target.wf:17`

```
 1: fn change(value: &uniq u64) -> result: own u64 writes(value) {
 2:   set deref(value) = 9_u64;
16:   region {
17:     set values[0_u64] = change(value: &uniq values[1_u64]);
18:     set values[1_u64] = values[1_u64] +wrap 1_u64;
19:   }
```

An exclusive *element* projection is an argument of the call whose result commits to a
*different element of the same run*, accepted only because OWN-7 proves `0 ≠ 1`. It is
neither purely an argument (the statement's captured target overlaps the root) nor a bound
holder. **Repair:** sequence the two halves —
`let r = 0_u64; with inout values[1] as p { set r = change(p); } set values[0] = r;`.
Expressible without any disjointness proof at all; keeping the one-statement form would
require M11 to keep index disjointness for a commit target beside an open projection.

### 2.6 Class F across a call — 64 sites in 15 groups

Definition used: two or more borrows/views live at the same program point where at least one
is handed to a helper call during the overlap. 36 sites are `let`-bound holders, 28 are
formed inline in a multi-projection call argument list. Marked **SAME** where two of the
simultaneous projections share one origin container — those are the true splits/partitions;
the rest are independent projections of disjoint containers or fields.

**F-1 SAME.** `own5-pos-recursive-adjacent-child-ranges.wf:11,12` (2 sites)

```
 9:   let middle = count / 2_u64;
10:   region {
11:     let left = mut_slice_of(&uniq output, 0_u64, middle);
12:     let right = mut_slice_of(&uniq output, middle, count);
13:     let a = fill(output: move left);
14:     let b = fill(output: move right);
```

Recursive divide-and-conquer over one exclusive view. **Repair:**
`with inout output[0..middle] as left { fill(left) } with inout output[middle..count] as right { fill(right) }`.
The two halves are used sequentially, so nesting is not even required. Expressible provided
M11 supports (a) range projections with proved `start <= end <= len`, and (b) re-projecting
an incoming `inout` projection parameter, which the recursion needs at every depth.

**F-2 SAME.** `x-borrowed-pool-tree-run.wf:168,169,175,176` (4 sites)

```
168:        let left_out = mut_slice_of(&uniq left);
169:        let right_out = mut_slice_of(&uniq right);
170:        region {
171:          let built = build(left: &uniq left_out, right: &uniq right_out, count: &uniq count, depth: depth);
...
175:              let left_in = slice_of(&left);
176:              let right_in = slice_of(&right);
177:              let checksum_result = checksum(left: left_in, right: right_in, root: root);
```

Two parallel index-based tree arrays plus a scalar cursor, all three handed to one builder;
then two shared views handed to a recursive walk. **Repair:** three nested `with` blocks
(`inout left`, `inout right`, `inout count`) around the `build` call, two around `checksum`.
Expressible; needs nested `with` on disjoint roots, and the shared projections must survive
being passed into a *recursive* call (`checksum` calls itself twice at lines 122–123).
Note this program is already a pool: nodes are `u64` indices, not references.

**F-3 SAME.** `x-buffer-borrowed-columns-run.wf:92,93,109,110` (4 sites)

```
 91:            region {
 92:              let left_out = mut_slice_of(&uniq columns.left);
 93:              let right_out = mut_slice_of(&uniq columns.right);
 94:              region {
 95:                fill(left: &uniq left_out, right: &uniq right_out, length: length);
 96:              }
 97:            }
```

Two exclusive projections of **two different fields of one struct**, live together across a
call. **Repair:** `with inout columns.left as l { with inout columns.right as r { fill(l, r, length) } }`.
Expressible only if M11 admits nested `with` on statically disjoint field paths of one
owner. This is the load-bearing requirement: refusing it costs the whole two-column idiom.

**F-4 SAME.** `own5-pos-a-unique-borrow-of-a-parent-view-after-its-child.wf:19,21` (2 sites)

```
19:        let writer = mut_slice_of(&uniq bytes);
20:        region {
21:          let view = slice_of(&bytes);
22:          let seen = observe(view: view);
...
28:          let done = fill(output: &uniq writer, value: 7_u8);
```

A shared child of a live exclusive parent, ended before the parent is used again. **Repair:**
`with inout bytes as writer { with let writer as view { observe(view) } fill(writer) }` —
reproject the open `inout` projection at `let` strength. Expressible if downgrade-reprojection
is admitted (the same power C-8 needs (R5)).

**F-5 SAME.** `fn1-pos-returned-slice-inputs-run.wf:46,47` (2 sites)

```
46:    let left_source = slice_of(&left);
47:    let right_source = slice_of(&right);
48:    let take_left = False();
49:    let selected = choose_slice(take_left: take_left, left: left_source, right: right_source);
```

Two shared views live at once and handed to the origin-set function of C-7. (The earlier
`pass_source` at line 33 is used alone and is class B.) Repair as in C-7; expressible.

**F-6.** `fn8-pos-affine-requirement-measures.wf:23,24,25` (3 sites)

```
22:  region {
23:    let a = slice_of(&first);
24:    let b = slice_of(&second);
25:    let destination = mut_slice_of(&uniq output, 0_u64, 8_u64);
26:    let count = fill(first: a, second: b, output: move destination);
```

Three projections of three disjoint buffers in one call, with a contract relating their
lengths. **Repair:** three nested `with` blocks. Expressible; the length contract is
unaffected.

**F-7.** `par1-pos-a-view-argument-is-a-footprint-on-its-origin.wf:34,35` (2 sites) — two
exclusive views of two disjoint runs handed to two successive `fill` calls; the point of the
case is that PAR-1 resolves each footprint to its origin. **Repair:** two nested `with`
blocks; each call's footprint is the projection's own path. Expressible.

**F-8.** `x-requires-output-capacity-run.wf:38,39` (2 sites)

```
37:      region {
38:        let window = slice_of(&source);
39:        let destination = mut_slice_of(&uniq output);
40:        let window_length = len_of(window);
41:        let destination_length = len_of(destination);
42:        if window_length <= destination_length {
47:          let written = copy_bytes(out: &uniq destination, source: window);
```

Classic in/out pair with a capacity guard read off both projections before the call.
**Repair:** nested `with`; `len_of` on a projection must remain available. Expressible.

**F-9.** `fn8-pos-requires-affine-row.wf:41,42` (2 sites) — exclusive out-view plus shared
in-view of two disjoint runs handed to `scaled`. Repair and verdict as F-8.

**F-10.** `x-base64-rfc-vectors-run.wf:226,227 / 269,270 / 315,316` (6 sites)

```
226:        let man_source = slice_of(&man_input);
227:        let man_window = mut_slice_of(&uniq man_output);
...
237:        match encode(out: move man_window, input: man_source) {
```

Three repeats of the same in/out pair for the three RFC 4648 vectors. **Repair:** nested
`with` per vector. Inside `encode` the two projections are passed down into
`read_index` / `write_index` from within a loop (lines 74–76, 104–113), which M11 permits as
pass-down. Expressible.

**F-11.** `accept-par3-staged-denied-hoisted-scratch.wf:155,173`,
`accept-par3-staged-denied-read-before-write.wf:155,161`,
`accept-par3-staged-denied-carried-scratch-byte.wf:164`,
`accept-par3-staged-iteration-own-scratch.wf:132`,
`par3-pos-a-per-iteration-run-from-the-store-is-iteration-own.wf:137` (7 sites; the outer
`window = slice_of(&name)` of each file is class B because no sibling is live when it is
passed)

```
148:      for @scan (index in 0_u64..4_u64) {
149:        let window = slice_of(&name);
151:          match open_file(factory: &uniq deref(files), root: cwd, name: &window, start: 0_u64, end: 4_u64) {
155:                  let destination = mut_slice_of(&uniq data);
157:                    match read_at(factory: &uniq deref(files), file: &uniq handle, destination: &uniq destination, ...)
173:                  let source = slice_of(&data);
174:                  set digest = fold_prefix(source: source, produced: 64_u64, seed: 0_u64);
```

A shared name projection live across the whole loop body while an exclusive scratch
projection and then a shared re-read projection of a *hoisted* buffer are opened and closed
inside it. **Repair:** `with let name as window { for @scan { ... with inout data as destination { read_at(...) } with let data as source { fold_prefix(...) } } }`.
Expressible; it needs a `with` block to span a loop (as G-1 does) and an exclusive and a
shared projection of one buffer to be opened in sequence, not simultaneously — which the
source already does.

**F-12.** `systcp-connection-two-halves.wf:156` (3 sites, formed inline)

```
155:                  region {
156:                    set outcome = pump(receive: &uniq link.receive, send: &uniq link.send, scratch: &uniq window);
157:                  }
```

**The hardest F in the corpus:** two exclusive projections of two disjoint fields of one
connection struct *plus* a third of an unrelated buffer, all in one call. **Repair:** three
nested `with` blocks on `link.receive`, `link.send`, `scratch`. Expressible only with nested
`with` on disjoint field paths (same requirement as F-3, R2). If M11 refuses it, the repair is a
`sink` of the whole `link` into `pump` and a return of it afterwards, which changes the
program's shape and its effect paths.

**F-13.** `systcp-connection-field-effect-paths.wf:97` (2 sites)

```
 90: fn drain(link: &uniq TcpConnection, scratch: &uniq MutSlice<u8>) -> result: own u8 reads(link.receive, scratch), writes(link.receive, scratch) contract {
 97:     match receive_next(receive: &uniq deref(link).receive, destination: &uniq deref(scratch), start: 0_u64, end: 32_u64) {
```

Re-projection of an incoming `inout` parameter down to one field, beside a re-projection of
a second `inout` parameter. **Repair:** `with inout link.receive as r { receive_next(r, scratch, ...) }`
inside the callee. Expressible if an `inout` projection parameter can be re-projected to a
field — the same power F-3 and C-2 need (R3).

**F-14.** `run-generic-priority-behavior.wf:92, 130, 135` (6 sites)

```
 92:      let order = Order::compare(env: env, left: &deref(queue).heap[parent], right: &deref(queue).heap[at]);
 96:      exchange_at::<T>(heap: &uniq deref(queue).heap, left: parent, right: at);
130:        let sibling_order = Order::compare(env: env, left: &deref(queue).heap[right], right: &deref(queue).heap[left]);
135:      let order = Order::compare(env: env, left: &deref(queue).heap[at], right: &deref(queue).heap[best]);
```

**Two shared element projections of the same array at two run-time indices, live in one
call.** This is the binary-heap sift comparison. **Repair:** because both are `let`
strength, M11 needs no disjointness proof — two shared index projections of one container
coexist, exactly as OWN-5 already admits shared borrows without limit. Expressible, and it
is the case that decides whether `with let v[i] as a, let v[j] as b { ... }` must be
writable (two open shared projections of one container at once).

**F-15.** `run-generic-owning-map-behavior.wf:154, 336, 338, 386, 388, 431, 451` (17 sites)

```
154:      set action = key_probe::<Key<K, E>>(slot: &deref(slots)[index], tag: tag, key: &key, env: env);
...
159:        let previous = replace deref(slots)[index] = move offered;
...
338:  let (found, examined) = key_find::<SeedKey>(slots: &map.slots, key: &query, env: &env);
```

The open-addressing hash map: a shared element projection `&deref(slots)[index]` beside a
shared projection of a key, and a field projection `&uniq map.slots` beside `&env`.
**Repair:** `with let slots[index] as slot { key_probe(slot, tag, key, env) }`; the
`replace` at line 159 stays an ordinary indexed exchange on the `inout` parameter.
Expressible and entirely index-based already. This is the largest borrow user in the corpus
(76 sites) and every one of them is A, B or F. `run-exclusive-owning-map-put.wf:127`
(`probe(slot: &deref(slots)[index], key: key)`) is the same shape with no simultaneous
sibling, so it counts as class A.

**Not class F, for contrast.** `x-wc-chunk-summary-run.wf:129,154`
(`combine(out: &uniq total, left: &left, right: &right)`) puts three borrows in one call,
but all three are whole variables with disjoint roots, so it is three ordinary call
conventions and no projection at all. 105 of the 117 multi-borrow call lines in the corpus
are of this kind; only the 12 lines in F-12…F-15 need projection machinery.

---

## 3. Totals and the unwritable count

| | count |
|---|---:|
| files censused | 900 |
| accepted files | 521 |
| accepted files with borrow/view sites | 291 |
| classified sites | **1600** |
| A / B | 1409 / 101 |
| C / D / E | 23 / 0 / 0 |
| F / G / H | 64 / 2 / 1 |
| sites needing a field or element projection | 26 |
| **sites unwritable under M11** | **0** |

**No site in the accepted corpus is unwritable under the no-reference model**, but that
verdict is conditional on five powers M11 must grant. Each is named by a site above; if any
is refused, the sites listed against it become unwritable.

| required power | sites that need it | if refused |
|---|---:|---|
| **R1.** Yielded projections with an `origin` set, bound by `with` in the caller | 23 (all of class C) | C-1…C-9 unwritable; 9 functions must be inlined or turned inside out |
| **R2.** Nested `with` on statically disjoint field paths of one owner | F-3 (4), F-12 (3), F-13 (2), F-15 (17) = 26, plus the one flip in §4.3 | the two-column, TCP-halves and map idioms lose their helper boundary |
| **R3.** Re-projecting an incoming projection parameter (incl. across recursion) | F-1 (2), F-2 (4), F-13 (2), C-2 (3), plus the 191 `deref(p)` reborrows in class A ≈ 202 | recursion over sub-ranges and every pass-down through a borrow parameter becomes unwritable |
| **R4.** Two simultaneous shared projections of one container at two indices | F-14 (6), F-5 (2) = 8, plus the two-shared-view rule tests in class B | binary-heap sift and every two-element comparison must copy elements out first |
| **R5.** A `with` block spanning a loop, and an exclusive projection downgraded to a shared child | G-1 (1), G-2 (1), C-8 (3), F-4 (2), F-11 (7) = 14 | read-to-end loops and fill-and-publish must re-open per iteration or lose the helper |

Nothing in the accepted corpus needs a stored reference (class D = 0) or a cursor held
across a conflicting write (class E = 0). Nothing in the accepted corpus needs a reference
in a generic type argument, an enum payload, or a container element. **The corpus's demand
on first-class references is confined to returning them (23 sites, 9 functions) and to
holding several at once across a call (64 sites, 16 groups).**

---

## 4. Acceptance-set diff (M11)

73 ownership-family rejections. Verdicts:

- **MOOT-U** — the rejected program is no longer *writable*: it binds a borrow to a named
  holder, forms a borrow or view value, commits at a view binding, or puts one in storage.
  Nothing is left to accept or reject.
- **MOOT-R** — the rule's whole subject (the written region and the borrow's independent
  lifetime) disappears.
- **STILL** — the error survives, re-raised as a projection-overlap, projection-origin,
  liveness, or affinity error under a different id.
- **FLIP** — M11 would accept the program.

### 4.1 MOOT — rule gone or program unwritable (26 cases)

| rule | case | why |
|---|---|---|
| OWN-1 | `own1-neg-bare-uniq-copy` | MOOT-U: no `&uniq` value to copy |
| OWN-1 | `own1-neg-return-borrowed-box` | MOOT-U: returns a borrow held in a box |
| OWN-1 | `own1-neg-return-through-shared-borrow` | MOOT-U: moves out through a bound shared borrow |
| OWN-1 | `x-ownmove-borrow-after-match-move` | MOOT-U: `let p = &…` holder outliving a move |
| OWN-1 | `view1-neg-a-move-of-a-shared-view` | MOOT-U: a view is not a value, so `move` on one is unwritable |
| OWN-3 | `own3-neg-unknown-region` | MOOT-R: `&'z a` — no region syntax |
| OWN-3 | `own3-neg-undeclared-signature-region` | MOOT-R: undeclared signature region |
| OWN-4 | `form8-neg-elided-related-region` | MOOT-R: elided related region on a returned borrow |
| OWN-4 | `own4-neg-brand-not-shortened-by-loan` | MOOT-R: brand/loan region interaction |
| OWN-5 | `own5-neg-read-while-uniq` | MOOT-U: `let u = &uniq a; let b = a;` |
| OWN-5 | `x-borrow-two-uniq-same-place` | MOOT-U: two bound `&uniq` holders |
| OWN-5 | `x-borrow-write-through-shared-borrow` | MOOT-U: bound shared holder written through |
| OWN-5 | `own5-neg-owned-header-keeps-bound-borrow` | MOOT-U: bound holder survives a match header |
| OWN-5 | `own5-neg-owned-header-keeps-result-parent-suspended` | MOOT-U: bound holder + returned borrow chain |
| OWN-5 | `own13-neg-borrowed-header-keeps-parent-suspended` | MOOT-U: bound `&uniq` holder as match scrutinee |
| OWN-5 | `own5-neg-slice-value-match` | MOOT-U: a view bound by `if`-`give` as a value |
| OWN-5 | `sys14-list-handle-unique` | MOOT-U: bound holder over a handle |
| OWN-10 | `x-borrow-own-param-escape-no-return` | MOOT-U: `let p = &'r0 x` on an `own` parameter |
| OWN-11 | `own11-neg-borrow-outer-region` | MOOT-R: `&'r a` inside a loop naming an outer region |
| STOR-5 | `stor5-neg-box-new-region-bearing` | MOOT-U: `heap_box(store, value: own Slice<u8>)` — no view value to box |
| STOR-5 | `stor5-neg-arena-new-region-bearing` | MOOT-U: same, via `arena_new` |
| VIEW-4 | `view4-neg-a-set-at-a-view-binding` | MOOT-U: `set window = slice_of(&right)` — a `with` binder is not a settable place |
| VIEW-6 | `view6-neg-two-same-region-view-results` | MOOT-R: "two results may not share one region" has no region; replaced by an origin-set rule on yields |
| OWN-12 | (region-substitution half of all 3 cases) | MOOT-R for the region clause; the overlap clause survives — see 4.2 |
| OWN-4 / OWN-10 | (rule statements themselves) | MOOT-R: "borrow held to the end of `'a`'s block" and "borrow-storage duration" have no subject; their *cases* still reject under `origin` |

Count by rule: OWN-1 5, OWN-3 2, OWN-4 2, OWN-5 8, OWN-10 1, OWN-11 1, STOR-5 2, VIEW-4 1,
VIEW-6 1 = **23 cases moot**, plus the OWN-4 / OWN-10 / OWN-12 rule statements whose cases
survive under renamed rules.

### 4.2 STILL REJECTS under a renamed rule (49 cases)

| rule | cases | what re-raises the error under M11 |
|---|---:|---|
| LIV-1 | 1 | liveness join-check is unchanged (no borrows involved) |
| LIV-2 | 2 | `set (v[i], v[j]) = …` — index disjointness, unchanged |
| OWN-1 | 13 | affinity and move discipline survive intact |
| OWN-4 | 2 | `own4-neg-return-local-borrow`, `x-borrow-return-uniq-local-region`: a yield whose origin set contains no parameter |
| OWN-5 | 18 | projection-overlap: two exclusive projections of one range, a write while a shared projection is open, a parent read during an exclusive child, an argument overlapping a commit target |
| OWN-10 | 3 | same as OWN-4: no parameter origin for the yield |
| OWN-11 | 2 | `own11-neg-move-outer-in-loop`, `liv1-neg-loop-leaves-an-outer-binding-dead`: loop liveness, no borrows |
| OWN-12 | 3 | two overlapping projection arguments at one call, one exclusive |
| STOR-5 | 3 | providers (`arena`, `Heap`, `Arena`) remain unstorable; only the borrow/view half goes away |
| VIEW-2 | 2 | `start <= end <= len_of(source)` becomes the projection-range obligation, unchanged |

### 4.3 FLIP to accepted (1 case)

**`own5-neg-later-argument-uses-suspended-parent`** (cited OWN-5):

```
fn bad(pair: &uniq Pair) -> result: own u64 reads(pair.right), writes(pair.left) {
  region {
    return assign(value: &uniq deref(pair).left, prior: deref(pair).right);
  }
}
```

Rejected today because the exclusive child `pair.left` suspends its parent `pair` for the
whole call, so the later argument's read of `pair.right` is a suspended-parent access. Under
a path-disjointness discipline — the same one R2 requires for F-3 and F-12 —
`with inout pair.left as v { assign(v, pair.right) }` is admissible, because `pair.left` and
`pair.right` are statically disjoint paths. **This case flips exactly when M11 grants P2**,
and refusing the flip means refusing F-3, F-12 and F-13 as well. The owner should treat the
flip and those three F groups as one decision, not four.

### 4.4 Summary of the diff

| verdict | cases |
|---|---:|
| moot (rule gone or program unwritable) | **23** |
| still rejects under a renamed rule | **49** |
| flips to accepted | **1** |
| total ownership-family rejections | **73** |

The 306 non-ownership rejections are untouched by M11.

---

## 5. Method: what was read in full, what was counted by grep

### 5.1 Counted mechanically

All 900 `.wf` files were tokenised by a throwaway script (kept outside the repository, under
`/tmp/wfcensus/`, nothing written into the worktree) that, per line, located
`&uniq`, `&uniq 'r`, `&`, `&'r`, `Slice<`, `MutSlice<`, `slice_of(`, `mut_slice_of(`, and
assigned a syntactic context by paren depth at the token offset plus the line's leading
keyword (`fn` header before/after `->`, `let`, `set`, `return`/`give`, `struct`/`enum` field,
otherwise call argument). Confirmed there are no multi-line `fn` signatures (`grep -c '^\s*->'`
= 0) and no `&&` or infix `& ` in the corpus, so every `&` is a borrow. Token total 2301;
accepted-case tokens 1694; minus 102 `slice_of` / `mut_slice_of` operand borrows folded into
their formation = 1592 sites, plus 8 caller-held returned borrows/views added by hand = 1600.
Holder liveness, sibling overlap, loop containment and intervening-write detection were
computed from brace-depth blocks over the same files. Verdicts and rule ids come from
`tests/conformance/manifest.jsonl` (821 records: 805 cases + 16 coverage notes).

### 5.2 Read in full (35 accepted cases)

`tests/conformance/cases/`: `form8-pos-a-run-element-region-is-determined-by-the-actual.wf`,
`own4-pos-return-caller-borrow.wf`, `own6-pos-callresult-borrow-chain.wf`,
`fn1-pos-result-provenance-distinct-regions.wf`, `fn1-pos-result-provenance-zero-candidate.wf`,
`form8-pos-related-pair-written.wf`, `fn1-pos-returned-slice-const-run.wf`,
`fn1-pos-returned-slice-inputs-run.wf`,
`view6-pos-a-helper-publishes-the-child-of-its-destination.wf`,
`view2-pos-captured-relative-and-empty-ranges.wf`,
`par1-pos-a-view-argument-is-a-footprint-on-its-origin.wf`,
`par2-pos-runtime-stride-range-helper.wf`, `own5-pos-recursive-adjacent-child-ranges.wf`,
`run-sysin-read-to-end.wf`, `form3-pos-lexical-classes.wf`, `x-borrowed-pool-tree-run.wf`,
`x-child-reborrow-run.wf`, `own5-pos-a-unique-borrow-of-a-parent-view-after-its-child.wf`,
`view2-pos-a-shared-child-reborrow-of-an-exclusive-view.wf`,
`view2-pos-two-shared-views-of-one-place.wf`, `x-buffer-borrowed-columns-run.wf`,
`x-requires-output-capacity-run.wf`, `own7-pos-distinct-noverlap.wf`,
`x-borrow-two-shared-reads-run.wf`, `x-enum-borrow-payload-live.wf`,
`form8-pos-narrower-loop-region-block.wf`,
`view1-pos-a-shared-view-is-used-twice-without-move.wf`,
`fn8-pos-affine-requirement-measures.wf`,
`call3-pos-a-fill-through-an-exclusive-view-keeps-both-lengths.wf`,
`x-typ-uniq-deref-write-roundtrip.wf`, `x-integ-coin-borrow-match-score-twice.wf`,
`own5-pos-rhs-borrow-is-disjoint-from-captured-target.wf`.
`tests/codegen/cases/`: `requires-check-tautology.wf`,
`bounds/derived-range/n15-remainder-tail-uniq-alias-mutation.wf`,
`bounds/output-capacity-lockstep/n31-helper-return-alias-escape.wf`.

### 5.3 Read in substantial part (8 accepted cases)

`accept-par3-staged-denied-opaque-cursor.wf` (1–139),
`run-generic-owning-map-behavior.wf` (1–300 of ~460),
`x-base64-rfc-vectors-run.wf` (1–135, 200–250),
`systcp-connection-two-halves.wf` (1–60, 130–175),
`systcp-connection-field-effect-paths.wf` (90–100),
`accept-par3-staged-denied-hoisted-scratch.wf` (145–176),
`run-exclusive-owning-map-put.wf` (120–132),
`fn8-pos-requires-affine-row.wf` (30–55),
plus `view2-pos-a-view-over-a-run.wf` (1–32), `msr1-pos-the-four-measure-readers.wf` (1–38)
and every borrow line of `run-generic-priority-behavior.wf`.

**43 accepted cases read**, spanning classes A, B, C, F, G and H and every view-forming
family.

### 5.4 Rejection cases read (18)

`own5-neg-read-while-uniq`, `x-borrow-two-uniq-same-place`, `own11-neg-borrow-outer-region`,
`own3-neg-unknown-region`, `view4-neg-a-set-at-a-view-binding`,
`own12-neg-a-view-argument-beside-a-unique-borrow-of-itself`,
`liv2-neg-two-subscripts-of-one-run`, `own5-neg-commit-overlaps-rhs-temporary`,
`own5-neg-a-published-child-freezes-its-parent-view`,
`own5-neg-owned-header-keeps-bound-borrow`, `exclusive-neg-view-live-during-call`,
`prov3-neg-an-append-while-a-copy-view-is-still-used`,
`own5-neg-later-argument-uses-suspended-parent`, `stor5-neg-box-new-region-bearing`,
`own5-neg-owned-header-keeps-result-parent-suspended`,
`own13-neg-borrowed-header-keeps-parent-suspended`, `x-borrow-own-param-escape-no-return`,
`own5-neg-slice-value-match`.
The other 55 ownership rejections were classified from the manifest rule id plus a feature
scan of their source (does it bind a borrow holder, return a borrow/view, name a region,
form a view, or use a provider type).

### 5.5 Spec passages relied on

`spec/kernel-spec.md` lines 673–803 (OWN-1…OWN-14, LIV-1, LIV-2), 1185–1232 (VIEW-1,
VIEW-2, VIEW-4, VIEW-6), 1234–1300 (STOR-1, STOR-4, STOR-5).

