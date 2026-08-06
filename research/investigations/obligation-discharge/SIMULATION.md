# Hand-simulation: obligation-discharge semantics on three native programs

Status: falsifier #1 of DOSSIER.md §6, executed 2026-08-06 by hand (single
analyst, no mechanical verification). Verdict up front: **the design survives
and the numbers land close to the dossier's predictions.** Key caveat: all
three programs were written under this project's own P9/ERR-4 doctrine, so
their input guards were already branch-shaped; undisciplined code would show
more branch-forced sites.

## Method

Programs: `tests/programs/utf8parse.wf` (318 lines, input-parser shape),
the dynamic-deflate unit (`raw_deflate.wf` helpers + `raw_deflate_dynamic.wf`
+ `raw_deflate_dynamic_decode.wf`, ~1000 lines; functions on the dynamic
path only), `tests/programs/sha256_abc.wf` (156 lines, pure kernel).

Simulated checker levels:

- **L0 (v0.17-strength)**: path-sensitive dominating branch/match facts,
  constant propagation incl. const-array element ranges, linear arithmetic
  (transitivity, ± constants, halving), table-op facts (`len` truthful,
  `len(buffer_new(n,v)) = n`), fact kills on assignment/effect-row writes.
  NO loop induction, NO user-function postconditions, NO struct invariants.
- **L1**: + loop induction (initialization + preservation over back edges).
- **L2**: + `ensures` on user functions (used exactly once: `read_bits`
  result `< 1 << count`, derivable inside its body from the mask op).
- **L3**: + struct/witness invariants (not simulated; residue noted).

Every current trap site (index, buffer_new size; all arithmetic in these
programs is already `.wrap` = total) was classified: **proven** (check
deleted), **claim** (clean residual, runtime-checked, traps), **branch**
(tainted residual, else = Err path), plus threading-tax requires clauses
counted.

## Results

| | utf8parse | deflate-dynamic | sha256 | total |
|---|---|---|---|---|
| trap sites today (semantic) | 44 | 30 | 9 | 83 |
| — of which test assertions | 8 | 0 | 1 | 9 |
| **L0 proven** | 25 | 17 (57%) | 0 | 42 |
| **L0 claims (structural)** | 2 | ~8 | 3 | ~13 |
| — sites those claims cover | 11 | ~8 | 8 | 27 |
| **L0 branches (taint-forced)** | 0 | 4–5 regions | 0 | 4–5 |
| requires clauses added | 0 | 5–7 (max depth 3) | 0 | 5–7 |
| taint false positives | 0 | 0 | 0 | **0** |
| **L1: claims discharged** | 2/2 | ~6/8 | 3/3 | ~11/13 |
| **L2: remaining wrap-family** | — | all discharged | — | — |
| **L3 residue (permanent branches)** | 0 | 3 regions | 0 | 3 |

Residual size: every claim in all three programs is **1–2 comparisons, one
line**. No multi-clause residual appeared.

## Per-program notes

### utf8parse — the origin example, vindicated

The state machine matches on tainted bytes everywhere (control flow — no
taint propagation) while every index operand is an internal counter. Two
loop-head claims (`i <= source_length`, `count <= i`) plus the existing
requires axiom (`len(out) >= len(src)`) cover all 11 body bounds sites via
transitivity. The requires itself discharges at the sole call site by
constants, so the entry check vanishes. Hot loop: today 2 checks/iteration →
L0 2 claim-checks (parity) → **L1 zero**. Under a hostile-input reading,
taint cost is exactly zero: no tainted value ever reaches a trap operand.

### deflate-dynamic — the stress case

- **The code's own Err guards do the proving.** `read_bits`, `emit_byte`,
  `decode_length`, `copy_distance` already branch-validate (Truncated,
  OutputFull, InvalidHuffmanCode/Distance) before indexing; at L0 those
  guards discharge the adjacent bounds obligations for free. 6 of 17 proven
  sites are proven by pre-existing doctrine-shaped guards — the mechanism
  and P9 agree on real code.
- **One `ensures` line kills a whole family.** Without it, wrap pathology
  (`literal_count = bits + 257` where `bits` is formally unbounded) forces
  one branch and one claim; `ensures result < 1 << count` on `read_bits`
  discharges `literal_count ∈ [257,286]`, `distance_count ≤ 32`,
  `code_count ≤ 19`, `total_lengths` no-wrap — the first real-world case for
  the ensures construct, and it pays instantly.
- **Const-table element ranges prove a "tainted-looking" index.**
  `code_lengths[code_length_order[code_index]]` is proven at L0 because
  every element of the const order table is < 19.
- **The L3 residue is exactly 3 branch regions** (`decode_table_symbol`'s
  `symbols[ordered]`, `build_huffman_table`'s second-pass `offsets` and
  `symbols` writes), all guarding the canonical-Huffman well-formedness
  invariant — quantified, cross-structure, genuinely beyond any near-term
  fragment. As branches they cost one compare each and buy honest
  `InvalidHuffmanCode` Err paths where zlib trusts its own table
  construction. This is the entire "bucket 3" of a 1000-line decoder.
- **Threading tax is real but bounded**: 5–7 clauses (store_dynamic_length
  ×2, build_huffman_table ×2, copy_distance's `output_offset <= len(out)`
  which threads through decode_dynamic/inflate — the depth-3 specimen). All
  clauses discharge at every call site at L0 via allocation-length equality
  and washed constants; no clause needed a runtime check at any call site.

### sha256 — the pure-compute control

Claim consolidation at its best: ONE loop-head claim
(`16 <= extend_index < 64`) covers all five schedule accesses in the extend
loop (5 checks/iteration → 1), three claims cover all 8 sites. At L1 all
discharge: **the entire hash kernel runs with zero runtime checks**, versus
5 retained checks in the hottest loop today — a direct, measurable P0 payoff
tied to the roadmap's vectorization interest.

## Findings against the dossier's predictions

1. "Most bounds obligations discharge or reduce to one-line residuals" —
   **held** (L0: 57–59% proven outright on non-test sites; every residual
   one line; L1 takes structural claims to ~85–100% per program).
2. "Threading tax appears but stays shallow" — **held** (max depth 3, all
   call-site discharges free).
3. Taint saturation fear — **did not materialize** on parser/decoder-shaped
   code: zero false positives, and the 4–5 branch-forced sites are exactly
   where an input-triggered trap would otherwise be reachable or where the
   fact is dead-in-truth but unprovable. The forced Err paths are honest.
4. Unpredicted finding: **test assertions are claims** — the 9 existing
   `check ... else trap` test expectations map onto the claim construct
   unchanged (named, deliberate, unprovable facts whose violation = test
   failure). No special test mechanism needed.
5. Unpredicted finding: the **checker upgrade ladder has sharp, small
   steps** — loop induction (L1) is the single highest-value upgrade;
   ensures (L2) is the second and is needed exactly once per I/O-shaped
   helper; struct invariants (L3) can wait indefinitely because their
   residue is tiny and branch-shaped.

## Caveats

- Selection bias: corpus written under this project's own doctrine; guards
  were already in the right shape. A lazy-writer corpus (dossier §6 probe 2)
  remains untested.
- Single-analyst hand entailment; borderline judgments (path-sensitive
  match-join facts, kill granularity through `&uniq` calls) assumed a
  path-sensitive checker and signature-driven kills — both must be specced.
- Dynamic check counts are static reasoning, not measured runs; codegen
  fusion (dossier §4.5) untested.
- `main` inputs are constants in these tests; taint analysis used the
  hostile-input reading for parse/deflate as the design intends.
