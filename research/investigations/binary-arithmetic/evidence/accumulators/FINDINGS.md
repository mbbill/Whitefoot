# Loop-carried bounds, Face A — can the writer state the bound their loop maintains?

Probe domain: accumulators and cursors whose loop-carried bound involves a
runtime factor. All verdicts below were produced by

```
WFC=/tmp/claude-0/-home-user-Whitefoot/1dea2c46-aae2-5e06-954f-b60e9c5b442c/scratchpad/target/release/whitefootc
$WFC --emit-llvm -o /dev/null <file>.wf; echo "exit=$?"
```

`RESULTS.txt` is the machine-generated verdict sweep over every `.wf` here;
`DIAGNOSTICS.txt` holds each one's first diagnostic line.

---

## Verdict

**Rare — in the sense that matters for the decision.** Face A is hit
*constantly*: 6 of 6 natural programs I wrote were rejected at invariant
formation, and every invariant containing `*` in the entire 491-case snapshot
corpus uses an inline literal factor, so no writer has ever escaped it. But in
6 of 6 cases the program was still expressible at **T0–T2**, and — the finding
that should drive the decision — **widening `[INV-1]`'s `*` alone would not
have unblocked a single one of the six.** In every case, once the invariant
formed, the proof still died on a *different* nonlinear obligation that a
formation widening does not address (detailed in "What widening `*` would
actually buy", below).

---

## Table

| # | Program | Face | Natural-form verdict | Best workaround | Tier |
|---|---------|------|----------------------|-----------------|------|
| A1 | `a1_gain_natural.wf` — scale samples by a runtime gain, accumulate | A | REJECT `[GRAM-4]` parse (`255*gain*i`); `a1b` REJECT `[INV-1] InvalidInvariant` (`gain*i`) | `a1_w3_narrow_product.wf` — land the product in u16, accumulate in u64, bound by `65535 * i` | **T0** |
| C2 | `c2_layout_natural.wf` — validate a runtime `records × width` layout, then fill | A | REJECT `[INV-1] InvalidInvariant` | `c2_w2_flat.wf` (flat loop over the validated total) **T0/T1**; `c2_w5_per_record_guard.wf` (keep the nesting, one dead guard per record + 3-`use` certificate) | **T1–T2** |
| D1 | `d1_fixedpoint_natural.wf` — accumulate u32 samples scaled by a runtime Q16.16 gain | A, then C | REJECT `[INV-1] InvalidInvariant`; without the invariant, REJECT `[OP-2] total +defined scaled` | `d1_w3_narrow_landing.wf` — do the product in u32, widen for the accumulator | **T1** |
| G1 | `g1_sum_items_natural.wf` — running total bounded by `count * max_item`, both runtime | A | REJECT `[INV-1] InvalidInvariant` | `g1_w2_requires_count.wf` — `requires len(items) <= limit`, bound by the item type's max | **T1** |
| H1 | `h1_stride_natural.wf` — write one channel of an interleaved buffer, runtime stride | A | REJECT `[INV-1] InvalidInvariant` | `h1_w1b_streaming_exact.wf` — advance while in range; the loop's own exit test *is* the bound | **T0** |
| J1 | `j0_counters_natural.wf` — sum u64 counters bounded by `n * cap`, cap runtime | A | REJECT `[INV-1] InvalidInvariant` | `j2_inline_digit_factor.wf` — inline-digit factor + per-item guard + restated step invariant | **T2** |
| F1 | `f1_hash_runtime_multiplier.wf` — rolling hash with a runtime base | — | **ACCEPT** | none needed — `*wrap`/`+wrap` are the intended semantics | **T0** |

---

## The two faces, as actually observed — plus a third

### Face A is a *parse* rejection as often as a semantic one

`[INV-1]`'s prose says two nonliteral operands are non-affine. The grammar is
stricter still: `affine_term := affine_factor ("*" affine_factor)?` admits **at
most one `*` per term, ever**. So the shipped worked-example shape generalises
worse than it looks:

```
$ $WFC --emit-llvm -o /dev/null a1_gain_natural.wf; echo "exit=$?"
whitefootc: Parsing/Source [GRAM-4]: SyntaxIssue { ... expected: ["{", ";", ")",
",", "<", ">", "+", "-", "==", "!=", "<=", ">="] }
  at a1_gain_natural.wf:9:53 in line
  "    invariant scaled_total: total <= 255_u64 * gain * i"
exit=1
```

`255_u64 * gain` parses. The *second* `*` is a syntax error — even though
`255_u64 * gain` is a perfectly affine term. This is not about nonlinearity at
all: `a2_two_multiplications_grammar.wf` writes `sum <= 5_u64 * 51_u64 * i`,
where **every factor is a literal**, and gets the identical `[GRAM-4]`
rejection at the second `*`. The bare two-atom form gives the semantic
diagnostic the brief predicted:

```
$ $WFC --emit-llvm -o /dev/null a1b_gain_natural_bare.wf; echo "exit=$?"
whitefootc: Semantics/Source [INV-1]: ... InvalidInvariant { reason: "an affine
multiplication has no direct integer-literal operand", mechanical_fix:
"multiply one affine factor by a directly written integer literal" }
  at a1b_gain_natural_bare.wf:9:38 in line
  "    invariant scaled_total: total <= gain * i"
exit=1
```

### Face B is *weaker* than the brief assumes — there is an automatic route

`[ENT-6]` has a "fixed interval product" rule: for `a * b` with both operands
nonconstant, the checker takes each operand's interval (tightened by **one**
premise each), forms the four endpoint products, and succeeds when all four fit
in T. So the very common u8×u8→u64 shape discharges with no writer effort:

- `b1_u8_product_domain.wf` — `sample * gain`, both from `cvt::<u8,u64>` — **ACCEPT**
- `b2_u32_product_domain.wf` — `count * width`, both full-width u32 — REJECT
  `[OP-2] ... UndischargedIntegerDomainObligation { residual: "count *defined width" }`
- `b3_u32_product_guarded.wf` — same, after two dominating range guards — **ACCEPT**

Face B is therefore a **T1** problem (two range guards, usually validations you
would write anyway), not a wall. In `k1_grid_index_exact.wf` the same trick
makes the specimen's `row *checked width` unnecessary: with
`requires height <= 4096` / `requires width <= 4096` the **exact** `row * width`
discharges, deleting one of the specimen's two dead `Err(overflow)` arms
(`k2_grid_index_exact_guarded.wf`, ACCEPT).

### Face C — the one that actually blocked my programs

**A product's result carries no bound, even when its own domain obligation was
discharged.** `[ENT-6]` mints a fresh full-type atom for it, and the interval
rule explicitly "publishes no product inequality or intermediate premise".

```
$ $WFC --emit-llvm -o /dev/null b4_product_result_bound.wf; echo "exit=$?"
whitefootc: Semantics/Source [INV-1]: ... UndischargedLocalInvariant { name:
"product_bound", ... }
  at b4_product_result_bound.wf:17:3 in line
  "  invariant product_bound: bytes <= 64000_u32;"
exit=1
```

`count <= 1000` and `width <= 64` are both established by dominating branches;
`bytes <= 64000` is still not provable. `d1_w1_operand_guards_only.wf` shows the
same thing end to end: guarding `raw <= 4095` and `gain <= 262144` does not let
`total + scaled` discharge — REJECT `[OP-2] total +defined scaled`.

**Guards on operands do not reach the result.** You must re-guard the result
itself, *and* restate the guard as an `invariant_stmt` — a branch fact lands in
the L0 relation state, which `AUTO` can use alone but cannot pair with a header
invariant:

- `d1_w2_result_guard.wf` — guard on `scaled`, no restatement → REJECT
  `[INV-1] UndischargedLoopInvariant { name: "scaled_total", obligation:
  Backedge, required_relation: "total <= (16384_u64 * (i + 1_u64))" }`
- `d1_w2b_result_guard_restated.wf` — one added line, `invariant step_bound:
  scaled <= 16384_u64;` → **ACCEPT**

---

## The derived-variable rewrite: tested carefully, and it is a dead end

The brief asked whether `set budget = budget + factor;` + `invariant acc <=
budget` rescues the bound. Three probes, same loop, same accumulation:

| file | form | verdict |
|------|------|---------|
| `i1_budget_uncapped.wf` | carry `budget`, state `acc <= budget` | REJECT `[OP-2] acc +defined w` |
| `i2_budget_capped.wf` | + `invariant budget_cap: budget <= 255_u64 * i` | REJECT `[INV-1] UndischargedLoopInvariant { name: "within", obligation: Backedge, required_relation: "acc <= budget" }` |
| `i4_budget_guarded.wf` | + a per-step runtime guard `w <= max_weight`, restated | **ACCEPT** |
| `i3_no_budget_literal.wf` | **delete the budget**, state `acc <= 255_u64 * i` | **ACCEPT** |

Read those four rows together and the escape collapses:

1. `acc <= budget` alone establishes nothing — `budget` is a loop-carried
   mutable, so it is a fresh unbounded header atom. The accumulator's own
   `+defined` obligation is exactly as undischarged as before.
2. `acc <= budget` **is not even inductive**. The backedge needs
   `step <= factor`, which nothing knows. That per-step relation is the real
   content of the bound, and the derived variable does not supply it — it
   *assumes* it.
3. Once you *do* supply it (a runtime guard, `i4`), you have paid T2 — and at
   that point `i3`'s one-line literal invariant works with none of the
   scaffolding. **The budget variable is either insufficient or redundant, never
   the thing that closes the gap.**

Where a derived variable *does* work is when it is a genuine runtime cursor
whose relation to a **length** is affine — `c2_w4_streaming.wf` and
`c2_w5_per_record_guard.wf` carry `limit = room - width` plus a snapshot and
close the subscript with a three-step certificate:

```wf
invariant in_room: written + 1_u64 <= room {
  use cell;
  use start <= limit;
  use c + 1_u64 <= width;
}
```

That is the honest positive result of this probe, and it is the only place a
`use` block earned its keep.

## The other escapes, all compiled

- **`[PRF-1]` `use` does not help.** A `use` source is formed by `[INV-1]`'s
  rules, so it fails identically:
  `e1_prf1_use_probe.wf` → `[PRF-1] InvalidSourceProof { reason: "an affine
  multiplication has no direct integer-literal operand" }` at
  `use written <= records * width;`.
- **A contract cannot carry the relation — and this is far broader than
  nonlinearity.** `[FN-8]`: "No proof-required exact operation, computed
  arithmetic result, ... becomes a relation term."
  - `e2_ensures_product.wf` — `ensures written <= records * width;` →
    `[GRAM-5]` **parse** rejection.
  - `e3_ensures_sum.wf` — `ensures written <= records + width;` → the *same*
    `[GRAM-5]` rejection. **Contract clauses admit no arithmetic at all.**
  - `e4_contract_define_product.wf` — routing it through
    `define needed = records * width;` → `[FN-8] InvalidRequires`.
  - What *does* work is the length form: `g1_w2_requires_count.wf` uses
    `define count = len(items); requires count <= item_limit;` — **ACCEPT**.
    So the function-boundary escape exists only for `term <op> term`, and in
    practice that means **make the bound a `len()`**.
- **A named const cannot be the literal factor.** `[INV-1]`: "Named constants
  ... are not admitted as affine atoms in this version."
  `j1_named_const_factor.wf` → `[INV-1] UnresolvedUse { spelling: "item_limit",
  role: InvariantValue }`. Binding it to a local first turns it straight back
  into Face A (`j3_local_limit_factor.wf` → `InvalidInvariant`). The digits must
  be duplicated inline in every invariant (`j2_inline_digit_factor.wf`,
  ACCEPT). Small, but it is a standing maintenance hazard: a limit constant and
  its invariants can silently drift apart.

---

## The strongest case: `c2_layout_natural.wf`

A runtime record layout validated against a fixed arena, then filled. This is
ordinary parser/decoder code — the layout comes from a header, the arena does
not.

```wf
let records = cvt::<u8, u64>(header[0_u64]);
let width   = cvt::<u8, u64>(header[1_u64]);
let out  = buffer_new(16_u64, 0_u8);
let room = len(out);
let needed = records * width;          // discharges: interval product, u8-derived
let layout_ok = needed <= room;
if layout_ok { } else { return exit_status(code: 2_u8); }

let written = 0_u64;
for (r in 0_u64..records, invariant record_base: written <= r * width) {
  for (c in 0_u64..width, invariant cell_offset: written <= r * width + c) {
    set out[written] = 7_u8;
    set written = written + 1_u64;
  }
}
```

```
$ $WFC --emit-llvm -o /dev/null c2_layout_natural.wf; echo "exit=$?"
whitefootc: Semantics/Source [INV-1]: ... InvalidInvariant { reason: "an affine
multiplication has no direct integer-literal operand", mechanical_fix:
"multiply one affine factor by a directly written integer literal" }
  at c2_layout_natural.wf:17:39 in line
  "    invariant record_base: written <= r * width"
exit=1
```

Three workarounds, all compiled:

- `c2_w1_guard.wf` (**ACCEPT**) — keep the nesting, pay a provably-true
  `written < room` compare and a dead arm **per output byte**. This is the
  corpus's own idiom (`real-programs__writer-r1__rle_decode_expand.wf` does
  exactly this). **T2.**
- `c2_w2_flat.wf` (**ACCEPT**) — one flat `for (k in 0_u64..needed)`; the binder
  and `needed <= room` discharge the subscript with nothing else. **T0/T1** —
  and it is better code. But `c2_w3_flat_with_indices.wf` (**ACCEPT**) shows the
  price when the body genuinely needs `r` and `c`: a `/` and a `%` per element
  plus a `width >= 1` guard, i.e. two integer divisions where W1 paid one
  compare. **T2, and slower than W1.**
- `c2_w5_per_record_guard.wf` (**ACCEPT**) — best of the three: keep the
  nesting, hoist the dead guard from per-byte to **per record**, and close the
  subscript with the three-`use` certificate above. **T1–T2**, six lines of
  proof scaffolding.

The residual, honest cost: the arena-fits check `needed <= room` is proved at
runtime and then **thrown away** — nothing downstream can use it, because
`needed` is an opaque product result (Face C). The writer re-establishes at
runtime, once per record or once per byte, a fact the program already
validated.

---

## What widening `*` would actually buy — the adversarial check

I walked each of the six programs asking: *if `[INV-1]` admitted `a * b` with
monomial normalisation, would the program compile?*

| # | Invariant would form? | Would the proof close? | What still blocks it |
|---|---|---|---|
| A1 | yes | **no** | backedge needs `sample * gain <= 255 * gain` — multiplication **monotonicity**, not formation |
| C2 | yes | **no** | subscript needs `r*width + c < records*width` from `r < records`, `c < width` — monotonicity again |
| D1 | yes | **no** | same as A1 |
| G1 | yes | **no** | `+defined` needs `(i+1) * max_item <= u64::MAX` — a nonlinear range fact |
| H1 | yes | **partly** | header invariant `cursor <= channel + k*channels` *is* provable under monomial normalisation (`(k+1)*channels = k*channels + channels`); the consumer, `cursor < room`, still needs `frames*channels + channel <= room`, whose only source is a runtime guard on an opaque product |
| J1 | yes | **no** | same as G1 |

So a pure formation widening closes **0 of 6**. To make any of them work you
would additionally need (i) products' results to carry derived bounds and
(ii) an admitted monotonicity rule `0 <= a, b <= c ⟹ a*b <= a*c`. Both are
much larger commitments than widening `affine_factor`, and (i) is the one my
programs actually kept dying on. **If any single change is worth costing, it is
publishing a bound for a product whose interval route already succeeded — not
`[INV-1]`'s `*`.**

---

## Negative results

- **Rolling hashes and checksums with a runtime multiplier do not hit this at
  all.** `f1_hash_runtime_multiplier.wf` (**ACCEPT**) — `*wrap`/`+wrap` are the
  *intended* semantics for a hash, and they carry no obligation. I expected this
  to be a strong candidate and it is a non-event.
- **FIR / weighted-sum with a runtime coefficient table dissolves.** The
  narrow-product-then-widen rewrite (`a1_w3_narrow_product.wf`) makes the
  runtime factor **vanish from the proof entirely**: once the product lands in
  u16, the type interval bounds the step and `total <= 65535_u64 * i` is all you
  need. This surprised me — it is arguably *better* code than the natural form,
  and it is what a careful C programmer writes anyway.
- **The corpus has already converged on the literal weakening.** Every
  invariant containing `*` across all 491 snapshot cases and 28 real programs
  uses an inline literal factor (`sum <= 255_u32 * i`, `2_u64 * mid + 1_u64 <=
  2_u64 * hi`, …). Not one names a runtime factor.
- **The 28 real programs contain no exact `*` and no `*checked` at all.** All 17
  multiplications are `*wrap`, and every one has a *literal* second operand
  (`index *wrap 128_u64`, `pixel *wrap 3_u64`, `left *wrap 1026_u64`) — static
  record strides. Runtime strides simply do not occur there.
- **Where the corpus does multiply two runtime values, Face B mostly resolves
  itself.** Exact `*` appears 23 times across the snapshot cases; only five have
  two nonliteral operands (`width * height`, `wide_a * factor`, `row * width`,
  `quotient * divisor`, `after * filter.gain`) and **four of the five are
  recorded ACCEPT** — the interval-product route carries them. The one recorded
  REJECT is `kills__writer-r2__06_chain_middle_replace` (`OP-2`), the case the
  brief quotes.

---

## Prevalence read

**The natural spelling is blocked essentially every time; the program almost
never is.**

I did not have to contrive anything: runtime gain, runtime record width,
runtime stride, and a runtime item bound are all ordinary. I wrote six programs
in the brief's territory and all six were rejected at formation on the first
try. If I extrapolate to a real codebase, any loop whose step size or weight is
data-dependent will hit `[INV-1]` on the writer's first attempt, and the
diagnostic's `mechanical_fix` ("multiply one affine factor by a directly
written integer literal") does not tell them what to do next.

But the *repair rate* is high and the repairs are cheap, because of one
structural fact: **a bound with a runtime factor is almost always weakenable to
one with a literal factor**, using the runtime factor's own type maximum, and
the weakened bound is almost always still strong enough for what the bound was
for (an overflow domain or a subscript). Two of my six repairs were T0. The
weakening fails only when the accumulator is already the widest type
(`j2`, u64 counters — T2), or when a *downstream* consumer needs the tight
product (the c2 arena case) — and there the contract surface cannot carry the
relation regardless of `[INV-1]`, so widening `*` would not deliver the
capability anyway.

What I would actually recommend on this evidence, in priority order:

1. **Publish a bound for a product whose domain already discharged** (Face C).
   It cost me a dead branch plus a restated invariant in 3 of 6 programs, it is
   the thing the interval rule has already computed and deliberately throws
   away, and it is far smaller than a polynomial fragment.
2. **Admit named consts as invariant atoms** (`j1`/`j3`). Tiny, and it removes a
   real drift hazard.
3. **Widening `[INV-1]`'s `*`.** On this evidence, last — it would not by itself
   have compiled any of the six programs I wrote.
