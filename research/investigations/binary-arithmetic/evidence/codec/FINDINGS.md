# Codecs, parsers, serialization: does the affine-only proof surface bite?

Domain: codecs, parsers, serialization, buffer-capacity arithmetic.
Compiler: `whitefootc` v0.44 (`--emit-llvm -o /dev/null`). Every verdict below was
compiled; the `.wf` files are in this directory and are named in each row.

---

## Verdict

**Rare, with one recurring exception.** Streaming/sequential codec work — which is
most codec code — does not hit the nonlinear wall at all: the cursor-walk
restructuring that avoids the product is as natural as the flattened form and costs
the same instructions. The wall bites reliably in exactly one shape: **random access
into an array whose stride is a runtime value** (fixed-width record tables, image row
strides, bit-packed pages). That shape costs **T3** every single time, and there is no
cheaper form.

Separately, and more surprisingly, I found a **common** gap next door that is not
[INV-1] at all: **a contract clause cannot state any arithmetic relation between
measures**, not even `len(out) >= 2 * len(src)` or `len(out) >= len(src) + 1`. That
blocks the *affine* half of capacity math at every function boundary, and it hits the
ordinary literal-factor codecs (hex, base64, RLE) that the nonlinear question misses.

---

## Table

| # | program (file) | domain shape | face | natural-form verdict | best workaround | tier |
|---|---|---|---|---|---|---|
| C1 | `c1_escape_natural.wf` | escaper, output = in_len × runtime `width` | A | **reject** `[OP-2] base +defined k` | `c1_w3_hoisted_limit.wf` — hoisted span limit, one extra compare/branch per input byte | **T2** |
| C2 | `c2_record_natural.wf` | fixed-width record table, sequential column scan | A | **reject** `[OP-2] base +defined field` (and `[OP-4] at < len(data)` in `c2_record_natural_wrapadd.wf`) | `c2_wB_limit_walk.wf` — record-cursor walk, same instruction count | **T0** |
| C3 | `c3_random_natural.wf` | fixed-width record table, **random** access | A | **reject** `[OP-2] base +defined field` | `c3_w1_option_guard.wf` — 2 guards + `u8`→`Option<u8>` | **T3** |
| C4 | `c4_frame_walk.wf` | length-prefixed frame, body = `count * elem_size` from the wire | — | **accept as written** | none needed | **none** |
| C5 | `c5_hex_encode.wf` | hex encode, `out_len = in_len * 2` (literal) | — | **accept** | none (caller-side length guard, see negatives) | **none** |
| C6 | `c6_base64_encode.wf` | base64, `groups = (n+2)/3`, `out_len = groups*4` | — | **accept** with two proof-only `[PRF-1]` certificates | none | **T0** |
| C7 | `c7_chunk_natural.wf` | chunked reader, runtime `chunk_size`, `count = len/chunk_size` | **B** | **reject** `[OP-2] c *defined chunk_size` | `c7_w1_chunk_walk.wf` — cursor walk, removes both the division and the product | **T0/T1** |
| C8 | `c8_bitpack_natural.wf` | bit-packed page, runtime `bits_per_value`, random value | A | **reject** `[OP-4] byte_index < len(page)` | `c8_w1_option_guard.wf` — guard + `u32`→`Option<u32>` | **T3** |
| C9 | `c9_stride_natural.wf` | raw image, `y*stride + x*bpp`, random pixel | A | **reject** `[OP-2] row_base +defined column_base` | `c9_w1_guard_chain.wf` — 4 guards + `Option<u8>`; sequential scan `c9_w3_row_cursor_scan.wf` is free | **T3** (random) / **T0-T1** (scan) |

Contract-machinery probes (`p*.wf`) are tabulated in "The contract route" below.

---

## The strongest case: raw image row stride (C9)

A raw-image decoder reads pixel `(x, y)` at `y*row_stride + x*bytes_per_pixel + channel`.
Both `row_stride` and `bytes_per_pixel` come from the file header, so both products
have two nonliteral operands. This is the single most common nonlinear index in all of
codec work (BMP/TGA/PNG post-filter, framebuffers, YUV planes), and nobody would write
it differently if the rule did not exist.

### Natural form — `c9_stride_natural.wf`

```wf
fn sample_pixel(pixels: own buffer<u8>, row_stride: own u64, bytes_per_pixel: own u64,
                x: own u64, y: own u64, channel: own u64) -> value: own u8 reads(pixels) contract {
  define room = len(pixels);
  requires row_stride >= 1_u64;      requires row_stride <= 65536_u64;
  requires bytes_per_pixel >= 1_u64; requires bytes_per_pixel <= 8_u64;
  requires channel < bytes_per_pixel;
  requires x <= 65535_u64;           requires y <= 65535_u64;
  requires room >= row_stride;
} {
  let row_base = y * row_stride;
  let column_base = x * bytes_per_pixel;
  let pixel_base = row_base + column_base;
  let at = pixel_base + channel;
  return pixels[at];
}
```

```
$ whitefootc --emit-llvm -o /dev/null c9_stride_natural.wf ; echo exit=$?
whitefootc: Semantics/Source [OP-2]: SemanticIssue { rule: Op2, ...
  kind: UndischargedIntegerDomainObligation { residual: "row_base +defined column_base",
  disposition: Unproved, mechanical_fix: "when the relation must hold, establish the fixed
  `.defined` normalization with a verified requirement, a source invariant, or explicit finite
  proof steps; ..." } } at c9_stride_natural.wf:15:20
  in line "  let pixel_base = row_base + column_base;"
exit=1
```

Note what is **not** rejected. Both `y * row_stride` and `x * bytes_per_pixel` are
*admitted*: `[ENT-6]`'s fixed interval-product rule multiplies the four endpoint pairs
of the operands' proved intervals, and the `requires` bounds make all four fit u64.
The rejection lands on the **addition of two products**, because that same rule
"publishes no product inequality or intermediate premise" — the checker computes
`y*row_stride <= 65535*65536` in order to admit the multiply and then **throws the
bound away**, so `row_base` re-enters the state as a fresh atom over the whole u64
range. With the bounds already in the contract, no overflow is possible at any of the
three additions; the compiler simply refuses to remember why.

### Best workaround — `c9_w1_guard_chain.wf` (accepted, exit=0)

```wf
-> value: own Option<u8>                       // (1) widened result
  let row_base = y * row_stride;
  let row_start_ok = row_base <= room;         // (2) dead guard
  if row_start_ok { } else { return None<u8>(); }
  let row_left = room - row_base;
  let row_fits = row_stride <= row_left;       // (3) dead guard
  if row_fits { } else { return None<u8>(); }
  let column_base = x * bytes_per_pixel;
  let column_start_ok = column_base <= row_stride;  // (4) dead guard
  if column_start_ok { } else { return None<u8>(); }
  let column_left = row_stride - column_base;
  let pixel_fits = bytes_per_pixel <= column_left;  // (5) dead guard
  if pixel_fits { } else { return None<u8>(); }
  invariant pixel_in_room: row_base + column_base + channel < room;   // proof-only
  let pixel_base = row_base + column_base;
  let at = pixel_base + channel;
  let byte = pixels[at];
  return Some<u8>(value: byte);
```

Four dominating compares and branches, all provably never taken, plus two subtractions,
plus `u8` → `Option<u8>` which forces a `match` on every caller (see `main` in the same
file). Exactly the `grid_index` shape the brief names as the specimen for T3.

The cheaper variant `c9_w2_wrap_one_guard.wf` (accepted) drops to **one** guard by
forming the offset with `+wrap`, but that trades a compile-time-detectable overflow for
a silent wrong-pixel read; it keeps the `Option` return anyway, so it is still T3 and it
is worse code.

Contrast `c9_w3_row_cursor_scan.wf` (accepted): a **full-image scan** with a row cursor
and a pixel cursor pays nothing — the two loop tests are the loop conditions you need
anyway and the three `invariant` lines are erased. The wall is about *seeking*, not
about *streaming*.

### Corroboration inside the project's own corpus

`tests/snapshot/cases/indexing/indexing__writer-r2__06_row_col_from_flat_index.wf` is an
accepted case that pays this price already: **six** dominating guards
(`width *defined height`, `area == capacity`, `row < height`, `col < width`,
`row *defined width`, `row_width +defined col`, `flat < area`), each with a
`return 0_u8` dead edge — an invented dummy value, T3 — for one flattened pixel read.
The project has already written the receipt.

---

## The contract route: it is writable, dischargeable, and inert

The brief flagged `[MSR-5]` as the most likely escape. I tested it hard. It is not one.

| probe | what it tests | result |
|---|---|---|
| `p01_contract_product_exact.wf` | `define needed = count * width;` in a `contract` | **reject** `[FN-8] InvalidRequires` — exact `*` is a proof-required partial op, inadmissible in a clause |
| `p13_contract_literal_scale.wf` | `define needed = count * 2_u64;` | **reject** `[FN-8] InvalidRequires` — even a **literal** factor is inadmissible |
| `p15_contract_literal_add.wf` | `define needed = count + 2_u64;` | **reject** `[FN-8] InvalidRequires` — even **addition** of a literal is inadmissible |
| `p14`, `p16` | `*sat` / `+wrap` in a clause | pass `[FN-8]`: total rows *are* admitted in a clause |
| `p07`, `p08`, `p10_sat_requires_pure.wf` | can a caller discharge `requires room >= count *sat width`? | **reject** `[FN-8] UndischargedCallRequirement`, *even with ground constants* (`32 >= 4*3`) and an explicit dominating branch on the identical expression — the goal-origin expansion is all-or-nothing, so a let-bound literal folds the operands away and the trees stop matching |
| `p11_sat_requires_params.wf` | same, with the operands as **parameters** | **accept** (exit=0) — so the clause *is* dischargeable, by a runtime branch on a syntactically identical expression |
| `p12_sat_requires_callee_use.wf` | can the **callee** turn that requirement back into a fact? | **reject** `[INV-1] UndischargedLocalInvariant` on `recomputed <= room` — no |

So the contract product is a **write-only clause**. Spec-wise this is forced:
`[ENT-3.S4]` projects a requirement into L0 only "when ... its operands ... are each an
admitted term, constant, or `len(P)` length term", and `[MSR-5]` says explicitly that
"a clause operand that is neither an [ENT-2] term nor a constant stays an ordinary pure
total operand contributing no L0 projection". A `*sat` product is not a term. The
callee gains an opaque signed goal that no bounds or domain obligation consults.

**The bigger consequence, which is not about products at all:** because `[FN-8]` bans
*every* proof-required exact operation in a clause and `[S4]` projects only whole terms,
a contract can relate measures **only by comparison**. `requires len(out) >= len(src)`
works (`percent_decode` uses it); `requires len(out) >= 2 * len(src)` and
`requires len(out) >= len(src) + 1` are both `[FN-8] InvalidRequires`. Every
expansion codec in the world — hex, base64, RLE, escaping, UTF-8 transcode, PCM
widening — has a precondition of exactly that shape, and none of them can state it. The
callee must therefore re-check capacity at runtime even when the factor is the literal 2.

One more boundary gap found in passing, from `[CALL-4]`'s own DEFERRED note: a function
returning `own buffer<T>` can publish **no** relation about its length ("no measure of a
result is an admitted operand in this version"). In `c5_hex_encode.wf` the caller must
add `let produced = len(encoded); if produced >= 4_u64 { } else { ... }` before touching
the buffer the encoder just sized itself. Every allocate-and-return codec API pays this.

Finally, `c3_w2_offset_param.wf` shows a related sharp edge: a **proved `[INV-1]`
invariant is affine-only and never becomes an L0 fact**, so `invariant slot_in_room:
base + field < room;` does *not* discharge a callee's `requires offset < len(data)`
(`[FN-8] UndischargedCallRequirement: at < len(table)`); only a real runtime compare
does. Pushing the offset across the boundary as a parameter therefore relocates the T3
cost into the caller rather than removing it.

---

## Division and remainder

Checked as a first-class question, per the brief.

* **Literal divisor is fine and carries a usable fact.** `[ENT-3.S7]` gives
  `let q = a / k;` (k a positive literal) both `q <= a` in L0 and the scaled affine
  image `k*q <= a`. `c6_base64_encode.wf` rides exactly that: `groups = (count+2)/3`
  then `out_length = groups*4`, and both the source and destination subscripts discharge
  with two proof-only `[PRF-1]` certificates:

  ```wf
  invariant group_start: 3_u64 * g < count { use 3 * (g < groups); use 3_u64 * groups <= padded; }
  invariant dest_slot: 4_u64 * g < out_length { use 4 * (g < groups); }
  ```

  Zero runtime cost. `out_len = in_len * 4 / 3` capacity math is **comfortable**.
  (AUTO alone cannot do it: `AUTO` pairs only *listed premises*, never two L0 images, so
  the coefficient-3 and coefficient-4 steps need the explicit `use N * (...)` multiplier.
  That multiplier must be a bare decimal, which is why the same trick is unavailable the
  moment the factor is a variable.)

* **Runtime divisor carries nothing at all,** and that is where division hits the wall —
  harder than multiplication does. `[ENT-3.S7]` requires a literal divisor, so
  `let chunk_count = room / chunk_size;` establishes *no* relation, not even
  `chunk_count <= room`. In `c7_chunk_natural.wf` the failure is therefore
  `[OP-2] c *defined chunk_size` — pure **Face B**: the loop binder `c` inherits the
  division result's unbounded interval, so the interval-product rule cannot close the
  multiply. This is the one candidate where the multiply itself, not its result, is the
  rejection.

  Workaround `c7_w1_chunk_walk.wf` (accepted) deletes the division *and* the product:
  walk with a chunk cursor and a hoisted `limit = room - chunk_size`. Both loop tests
  are semantically required. **T0/T1, and the resulting code is better** — no division
  in the hot path.

* The `indexing__adversary-r1__07-chunk-count-accept` doc the brief pointed me at is
  about the same asymmetry from the other side: `data_len / 3` with a *literal* 3
  behaves well enough that a dominating `chunk_count == 0` branch suffices "without
  needing a nonlinear division-image certificate". Change the 3 to a runtime value and
  nothing in that case survives.

---

## Negative results (things I expected to break that did not)

1. **`count * elem_size` read out of the wire is fine.** `c4_frame_walk.wf` compiles
   **as naturally written**, no workaround. The parser must validate an untrusted
   product against the bytes that remain anyway, and that validation branch produces
   exactly the L0 fact the cursor advance needs. Whenever the multiplicands are
   attacker-controlled, the runtime check is inherent and the wall is invisible. This
   covers a large fraction of real parser code.

2. **Sequential fixed-width record walking is free.** `c2_wB_limit_walk.wf` costs
   *exactly* one compare and one add per record — identical to the counted
   `for i in 0..count { data[i*w + f] }` it replaces — because the hoisted
   `limit = room - record_width` turns the loop test into the bounds proof. Two
   proof-only `invariant` lines, zero runtime cost. It is also more honest code: it
   never trusts the header's record count against the real byte length.

3. **Literal expansion factors are fine.** `c5_hex_encode.wf` (×2) and
   `c6_base64_encode.wf` (÷3 then ×4) both compile. `[INV-1]`'s "at least one direct
   operand is an integer literal" covers the whole classical capacity-math family.

4. **Face B is much weaker than the brief implies.** `[ENT-6]`'s interval-product rule
   discharges `a * b` with two nonliteral operands whenever both operands have proved
   intervals — which a `requires x <= 65535_u64` supplies for free. In C1, C2, C3, C8
   and C9 the multiply is *accepted*; the rejection is always downstream. Face B only
   appeared standalone in C7, where a runtime division upstream destroyed the interval.

5. **The cursor restructuring is the project's own house style.** `tests/programs/raw_deflate.wf`'s
   `read_bits` maintains a `hold`/`bits` accumulator and never forms
   `field_index * bits_per_field`. `percent_decode.wf` and `tlv_frame_parse.wf` are both
   cursor walks. So the workaround I keep reaching for is not a contortion invented to
   satisfy the checker; it is what this codebase already writes.

6. **A workaround that is genuinely better than the natural form:** C7's chunk walk.
   Removing `room / chunk_size` removes a 20–40 cycle division from the setup and makes
   the truncated-tail case explicit instead of implicit. I would send that as a review
   comment in C.

---

## Prevalence read

Extrapolating from the nine programs here plus the project's existing corpus:

* **Streaming/sequential codecs — roughly 70–80% of codec code by volume** (encoders,
  decoders, framers, scanners, escapers, checksummers, bit readers): **no cost**. The
  cursor form is natural, equal-cost, and already idiomatic here. C2, C4, C7, C9-scan.
* **Literal-factor capacity math** (hex, base64, RLE, UTF-8 widening): **no cost**,
  though it needs one or two `[PRF-1]` `use N * (...)` certificates that a writer has to
  know to reach for. C5, C6.
* **Random access with a runtime stride** — record tables seeked by index, image
  pixels, bit-packed pages, sorted index files searched by binary search: **T3, every
  time.** I could not find a cheaper form; the offset-parameter API relocates the cost
  rather than removing it. I would guess this is **one function in a nontrivial
  format reader** — an accessor, a `get_record`, a `pixel_at` — rather than something
  spread everywhere. It is a small, sharply-located, recurring tax.
* **Runtime expansion factors** (C1): T2, one dead compare per input byte in a hot loop.
  Less common than the random-access shape in my domain, because the factor is usually a
  per-encoding constant.

So for the nonlinear question as asked: **rare**, and I had to reach for random access
to make it hurt.

But two adjacent facts change the picture and I think they matter more than the
nonlinear fragment:

* **The contract clause cannot express any arithmetic at all** — not `2 * len(src)`,
  not `len(src) + 1`. That is not the nonlinear wall; widening `[INV-1]` to polynomials
  would not fix it. It hits the *common* literal-factor codecs, not the rare nonlinear
  ones, and it forces the callee to re-derive capacity at runtime on every expansion
  codec. Widening `[FN-8]`/`[S4]` to admit `len(P) ± k` and `k * len(P)` as clause terms
  looks like a much better return on the same effort.
* **The interval-product rule already computes the bound it then discards.** Publishing
  the four-endpoint interval as an ordinary fact about the product's result would, on
  its own, turn C1/C2/C3/C8/C9's `[OP-2]` rejections into `[OP-4]` bounds rejections —
  i.e. it would delete every dead *arithmetic* guard in the workarounds above and leave
  only the single real bounds guard. That is a strictly smaller change than a polynomial
  fragment and it removes most of the measured cost.

---

## File inventory (all 36 `.wf` compiled; `wf.sh` is the harness)

Rejections that are **the finding**:
`c1_escape_natural` `c2_record_natural` `c2_record_natural_wrapadd` `c3_random_natural`
`c7_chunk_natural` `c8_bitpack_natural` `c9_stride_natural`
`p01` `p07` `p08` `p10` `p12` `p13` `p14` `p15` `p16`.

Rejections that are **scaffolding**, kept so the argument can be re-run:

* `c4_frame_natural.wf` — my first draft of C4. It fails on `[OP-4] cursor < len(wire)`
  for an ordinary *affine* reason (I used `+wrap` for the header end, which drops the
  relation `[ENT-3.S7]` gives only for exact `+`). `c4_frame_walk.wf` is the same program
  written carefully and is the accepted C4 result. Nothing nonlinear is involved either way.
* `c1_w2_cursor_guard.wf` and `c1_w2a_cursor_no_republish_reject.wf` — these two isolate a
  separate, purely mechanical fact worth knowing: the L0 fact a body branch establishes is
  killed at the loop body's scope exit, so the backedge obligation fails
  (`[INV-1] UndischargedLoopInvariant ... obligation: Backedge`). Adding one proof-only
  `invariant span_fits: cursor + width <= room;` immediately before the update republishes
  it and the identical program compiles (`c1_w2a_cursor_republished_accept.wf`, exit=0).
  Every cursor workaround in this report needs one such line. It costs nothing at runtime,
  but a writer who does not know the rule will read the diagnostic as "my loop is wrong".

Accepted workarounds: `c1_w1_guard_each_byte` `c1_w2b_cursor_remaining` `c1_w3_hoisted_limit`
`c2_wA_cursor_walk` `c2_wB_limit_walk` `c3_w1_option_guard` `c3_w2_offset_param`
`c4_frame_walk` `c5_hex_encode` `c6_base64_encode` `c7_w1_chunk_walk` `c8_w1_option_guard`
`c9_w1_guard_chain` `c9_w2_wrap_one_guard` `c9_w3_row_cursor_scan` `p11`.
