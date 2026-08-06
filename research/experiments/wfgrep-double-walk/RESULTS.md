# WFGREP-DOUBLE-WALK results

Status: COMPLETE — S2 (fused scan+match) credited at 1.15x on the scan
cases and landed; the residual divergence re-attributed from instruction
count to a measured ~3.8 cycles/byte serial-walk latency floor, with the
S3 witness that no legal shape can widen the step

The protocol, shape catalog, expected code-shape consequences, falsifiers,
inherited corpus digests, statistics, and bands were frozen in the commit
carrying `PROTOCOL.md` before any comparative number, counter, or
shape IR/assembly was observed (pre-freeze correctness-only development
disclosed there). The single run `wfgrep-double-walk-1` executed on
2026-08-06 in protocol order (verify, code-shape, null-before, bench-grep,
bench-s1, bench-s2, bench-s3, null-after, counters, confirm-s2). Raw
evidence: `raw/wfgrep-double-walk-1.jsonl`, 2,303 lines, 2,100 samples,
SHA-256
`912ed5eee244d01e66f39ea86c07a188f69c25f0582decdd970759910ef17654`.
No sample was deleted, extended, or rerun. Host: Apple M4, 16 GiB, macOS
26.5.2 (25F84), AC power at all sixteen phase boundaries, Low Power Mode
off. Binaries (SHA-256): B0
`96d1ec975924aaa475a6402c1de9d7ba4e3b3c5463fc1beb9146ba2cb8d003d3`, S1
`6a21ab795fc7d66140db46973342dba30808c55dd89e1845b1c5580e30a7d442`, S2
`1051f524fb6840fa15ac7bbdf0d86a975188bc718cb22331bd0d711d5cfff5c2`, S3
`098f690e744973222e82b5754b92ed809c85945b9329631e496a48dc72a2dd7a`,
comparator `/usr/bin/grep` `569588bf…f50d8f1f` (the baseline's pinned
binary). The verify phase matched all 25 subject×case outputs and exit
codes against the inherited WFGREP-BASELINE manifest pins byte for byte.

## Result

Shape phases: ratio = B0 elapsed / shape elapsed; above 1.0 means the
shape is faster. Medians of 30 paired warm rounds; 95% bootstrap
intervals; `w` from the null gates below.

| Case | vs B0 | S1 hoisted first byte | S2 fused scan+match | S3 SWAR word scan |
|---|---|---|---|---|
| `large` | ratio [CI] | 1.140 [1.128, 1.149] **material improvement** | 1.150 [1.141, 1.152] **material improvement** | 0.896 [0.893, 0.899] **material regression** |
| `nomatch` | ratio [CI] | 1.139 [1.135, 1.144] **material improvement** | 1.145 [1.141, 1.149] **material improvement** | 0.899 [0.893, 0.902] directional regression |
| `dense` | ratio [CI] | 1.084 [1.080, 1.091] directional | 1.062 [1.058, 1.066] directional | 1.016 [0.992, 1.025] practical parity |
| `many` | ratio [CI] | 1.012 [0.999, 1.022] precision-limited | 1.011 [1.003, 1.019] precision-limited | 0.998 [0.976, 1.025] precision-limited |
| `floor` | ratio [CI] | 0.999 [0.941, 1.045] diagnostic | 1.014 [0.997, 1.058] diagnostic | 0.980 [0.947, 1.008] diagnostic |

Product comparison (baseline orientation: grep elapsed / subject elapsed;
below 1.0 means the subject is slower than grep):

| Case | B0 fresh baseline (bench-grep) | S2 (confirm-s2) | Movement |
|---|---|---|---|
| `large` | 0.650 [0.646, 0.659] material loss | 0.753 [0.747, 0.776] material loss | +0.10 |
| `nomatch` | 0.657 [0.653, 0.660] material loss | 0.762 [0.755, 0.774] material loss | +0.11 |
| `dense` | 1.087 [1.069, 1.096] directional win | **1.160 [1.149, 1.175] material win** | +0.07 |
| `many` | 0.490 [0.482, 0.501] material loss | 0.512 [0.501, 0.520] material loss | +0.02 |
| `floor` | 1.132 [1.029, 1.215] diagnostic | 1.026 [0.981, 1.054] diagnostic | — |

Headline: the fused single-pass shape S2 is a credited 1.15x/1.145x
improvement on the scan-dominated cases with byte-identical behavior and
all §9.1 gates green; it moves the product ratio from 0.65 to 0.75–0.76
on those cases and flips `dense` from a fragile edge to a clean material
win over the system grep. The remaining 0.75 loss does not close by any
attempted legal shape, and the counters attribute why (below). The
`many` case barely moves in ratio because its divergence is the host
per-open identity cost the baseline already attributed outside Whitefoot
(B0's fresh product ratio 0.490 vs 0.605 in the 0022 run is that same
host cost varying between days; its widened-threshold material-loss
classification holds).

## Demonstrated precision and machine-state disclosures

Null half-widths (B0 vs itself): before 0.16% / 0.27% / 0.43% / 1.03%
(large/nomatch/dense/many), after 0.16% / 0.29% / 0.34% / **2.22%**. All
null intervals contain 1.0. `many`'s null-after exceeds the 2% gate, so
per the inherited degradation rule its parity and directional
classifications are unavailable — the three `many` shape rows above are
recorded as precision-limited, and the bench-grep `many` material loss
stands only because its whole interval [0.482, 0.501] clears the widened
threshold 0.90 × (1 − 0.0222) = 0.880. Two absolute-time drifts are
disclosed rather than reinterpreted: in bench-s3's `dense` case both
sides shifted together to ~226 ms (base's other-phase median is ~181 ms),
and in confirm-s2 both sides ran ~8–10% above their bench-phase absolutes
(machine-state drift late in the run); the paired within-round ratios,
which are the frozen statistics, are unaffected in design and the
null-after gates bound the achieved precision.

## Code shape against the preregistered expectations

Retained artifacts: `BUILD/{base,s1,s2,s3}.{raw.ll,opt.ll,s}` under the
work root, digests printed by the `code-shape` phase and recorded in the
freeze-inherited Makefile flow. Verdicts against the frozen falsifiers:

- **S1 — structural delta PRESENT; quantitative sub-prediction missed.**
  The optimized candidate fast path (`bb11.i.i` →
  `buffer.index.cont.v65.i.i` → `bb12.i.i` in `s1.opt.ll`) holds exactly
  one memory load (the source byte) compared against the register-held
  first pattern byte (`%v26.i.i`); the per-candidate pattern reload of
  the baseline is gone, and LLVM strength-reduced the bounds check into
  the loop bound (`usub.sat(4096, start)`). The preregistered "≥ 2
  instructions/byte drop" leg, however, measured 1.69 (22.30 → 20.61 on
  `nomatch`), so the falsifier's quantitative leg fired even though the
  wall ratio cleared the material band.
- **S2 — structural delta PRESENT; quantitative sub-predictions missed.**
  The optimized `main` contains one per-byte fast-path walk (`bb57.i` →
  `buffer.index.cont.v664.i` → `bb60.i`/`bb63.i` → `bb61.i` in
  `s2.opt.ll`): one source-byte load, a newline compare, a register
  first-byte compare (`%v124.i`), with the verify subloop entered only on
  candidates; `wf_line_matches` does not exist in the module (0
  occurrences). The quantitative legs missed: `nomatch`
  instructions/byte measured 17.67 (> the 16 falsifier bar; the static
  model of ~11 undercounted the loop-carried state the ANF/match lowering
  keeps), and the expected ≥ 1.4 wall ratio measured 1.145 — the
  shortfall is explained by the latency floor in the counters section,
  and is itself the slice's main finding.
- **S3 — obstruction prediction CONFIRMED exactly.** `s3.opt.ll`
  contains no `load i64` anywhere; the word step keeps all eight separate
  byte loads (`%v624.i`…`%v645.i`), each behind its own trap-guarded
  bounds branch against a distinct constant (4095…4089) with a distinct
  trap record (`.wf_trap.15`–`.wf_trap.22`); LLVM strength-reduced the
  SWAR predicate arithmetic but would not fuse loads across the
  trap-carrying branches, because each guarded index owns a distinct
  noreturn side effect that a wide load would reorder. Wall consequence
  as predicted: `large` 0.896 material regression, `nomatch` 0.899
  directional regression. This is the minimal witness that the one legal
  word-at-a-time spelling cannot reach a wide-load lowering on the
  ordinary path. The conditional S4 (fused+widened) therefore never
  opened, per its preregistered condition.

## Counters and the re-attribution

Medians of five (`/usr/bin/time -l`), `nomatch` (268,435,456 bytes):

| Binary | instructions | instr/byte | cycles | cycles/byte | IPC | real |
|---|---:|---:|---:|---:|---:|---:|
| B0 | 5.987e9 | 22.30 | 1.173e9 | 4.37 | 5.10 | 0.29 |
| S1 | 5.532e9 | 20.61 | 1.030e9 | 3.84 | 5.37 | 0.25 |
| S2 | 4.743e9 | 17.67 | 1.024e9 | 3.81 | 4.63 | 0.25 |
| S3 | 5.369e9 | 20.00 | 1.303e9 | 4.86 | 4.12 | 0.32 |
| grep | 4.141e9 | 15.43 | 0.807e9 | 3.01 | 5.13 | 0.19 |

(`large` is the same picture: B0 22.32/4.49, S1 20.68/4.39, S2
17.68/3.81, S3 20.02/4.88, grep 15.39/2.96 instr/byte and cycles/byte.)

The same-source ablation is decisive. S1 removed 1.69 instructions/byte
and gained 0.53 cycles/byte; S2 removed a further 2.94 instructions/byte
beyond S1 and gained a further **0.02** cycles/byte. Instruction removal
saturated: both improved shapes land on the same ≈ 3.8 cycles/byte, at
falling IPC (5.37 → 4.63). The walk is bound not by instruction
throughput but by its serial per-byte structure — a loop-carried
increment→compare→branch dependence with two-plus taken branches per
byte — so the double-walk's *instruction* redundancy (the attributed
cause this slice addressed) is now closed as far as any attempted legal
shape can close it, and the residual product loss (0.75–0.76) is
re-attributed to the per-byte *step width*: grep's memchr consumes the
buffer in 16-byte SIMD strides (its 3.0 cycles/byte includes its heavier
per-line machinery; its scan stride is the structural difference), while
every legal Whitefoot spelling steps one byte per iteration, and the S3
witness shows the one legal widening cannot lower to a wide step because
each index carries its own trap-guarded check. Closing the remainder is
therefore a FLOOR/lowering question (a check-aware wide-scan lowering or
a proof-driven guard hoist), not a source-shape question.

On the bounds-trap secondary: this run neither implements nor measures
check elision, and the ablation gives no ground to re-attribute the
residual to the traps — S2's per-byte trap compare/branch is 2 of 17.67
instructions/byte in a loop whose cycles did not respond to a 2.94
instruction/byte removal. The secondary stays untouched and unclaimed,
per the plan boundary.

Credit assignment per the frozen rules: S2 is credited as a whole patch
(hoisted first byte + fusion) at 1.150/1.145 on `large`/`nomatch`; the
single-variable S1 measurement bounds the hoist's own share at
1.140/1.139, so fusion's wall margin beyond the hoist is ~1%, though it
removes 2.94 instructions/byte and one whole walk — further evidence of
the latency floor. S2 additionally never regresses: `dense` 1.062
directional (its extra per-byte hit-flag branch costs ~2% of S1's dense
gain), `many` directional-positive under the precision limit, `floor`
parity.

## Landed outcome

S2's bytes land as `tests/programs/wfgrep.wf` (the credited shape; SHA
above, source `shapes/s2-fused-scan-match.wf`). At landing, the nine-case
oracle, the ten §9.1 gates, and the full repository gate were rerun green
on the landed bytes, with one gate constant re-derived from source per
task 0016's rule: `DECLARED_FUNCTIONS` in
`compiler/src/backend/tests/cost_shape.rs` drops `line_matches`, which
the landed source no longer declares (five declared helpers remain; no
row weakened, no count relaxed). The §9.1 cost-shape properties
(allocation, transfer, batching, initialization rows) are unchanged by
the walk fusion, as the green gates state.

## What this run does and does not mean

It means: on this host, one attributed cause — the scalar double-walk's
redundant instruction work — is closed by a legal source shape on the
ordinary compiler path, worth 1.15x end-to-end on scan-dominated
workloads with byte-identical output and every required check intact;
the match-dense workload now beats the system grep materially (1.16);
and the remaining scan-case loss to grep (0.75–0.76) is measured and
attributed to per-byte stepping against SIMD striding, with a concrete
witness that no legal source spelling can widen the step under the
current lowering. The next lever on this gap belongs to FLOOR/lowering
work (check-aware wide scanning), not to further source-shape search.

It does not mean: any language-versus-C machine-quality claim; any
ripgrep, GNU grep, regex, traversal, cold-cache, or cross-host claim;
any bounds-trap cost claim (unmeasured here); and no fraction of any
flagship claim. The `many` product ratio remains dominated by the host's
per-open treatment of unsigned binaries, outside any Whitefoot layer.
