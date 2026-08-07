# WIDE-SCAN-LOWERING results

Status: COMPLETE — route (b) credited: the check-aware wide probe is a
material 1.43x on both scan cases with every required check preserved
observably, and the confirm rerun moves wfgrep past the system grep on
every compute-bound case

The protocol was frozen at `3920e71` before any implementation or
measurement; the transform was then implemented at `49e2663` and the
harness at `6c20826`, the compiler revision this run measured. The single
run `wide-scan-lowering-1` executed on 2026-08-06 in protocol order
(build, gen, verify, code-shape, oracle+gates, null-before, bench-grep,
bench-wide, null-after, counters, confirm). Raw evidence:
`raw/wide-scan-lowering-1.jsonl`, 1,629 lines, 1,500 samples, SHA-256
`73c6330069480726b423fe7470e46eeac4ea935979520f8bd2123c95915aa73f`.
No sample was deleted, extended, or rerun. Host: Apple M4, macOS 26.5
(Darwin 25.5.0), `/usr/bin/clang` Apple clang 21.0.0 (clang-2100.1.1.101);
AC power and Low Power Mode off at all twelve phase boundaries. Binaries
(SHA-256): base
`9e5315146ee606bf253450c26371628995afb2c66bc06d2ecaf18d330456fdb6`, wide
`a41991c3e972109c3bcfce93e8e6864b11089a7ffaa7de39e4ee7478374172ee`,
comparator `/usr/bin/grep` `569588bf…f50d8f1f` (the baseline's pinned
binary). Both subjects are the identical landed `tests/programs/wfgrep.wf`
bytes; only the compiler differs (base revision `23bb7b0` versus this
branch). The verify phase matched the inherited WFGREP-BASELINE corpus
pins and all 15 subject-case output/exit pins byte for byte.

## Route (a) — closed as preregistered

The base build's code-shape artifacts show no vector fast path in the
optimized `@walk` (zero `load <16 x i8>` attributable to the walk; the
eight vector loads in both builds are LLVM's own vectorization of the
countable fill/copy loops, present identically in base and wide). The
three preregistered witnesses stand: the toolchain vectorizes no
early-exit loop even for same-Clang C (WF-SCAN-FLOOR), guarded byte loads
never fuse (0023's S3), and the fused walk is a multi-exit loop with a
nested verify loop. The reopening falsifier did not fire.

## Route (b) — the transform, and the frozen shape predictions

The compiler now recognizes, by semantic rule only, loops of the
byte-walk form (exit guard on an outside induction/bound pair, one
guarded `u8` load at the induction binding, a neutral middle whose
observable arms are dominated by equality tests of the loaded byte, the
single-step increment) and emits at the loop header one internally
guarded 16-byte probe, `buffer.probe.skip`, that returns how many
upcoming iterations are provably `i := i + 1` — 0 whenever
`index + 16 > min(bound, len)` or the first byte is interesting. Every
byte at which anything observable can happen, including every possible
trap, executes the unchanged scalar body. Implementation:
`compiler/src/lowering/builder/probe.rs` (recognizer and injection),
`IrOperation::BufferProbeSkip` in `compiler/src/lowering.rs`, emission in
`compiler/src/backend/emitter/buffer.rs`.

Frozen predictions against the retained artifacts (digests printed by the
code-shape phase; base.raw.ll `c5995f2d…`, base.opt.ll `20a2bf4a…`,
wide.raw.ll `288659d1…`, wide.opt.ll `845f327e…`):

- **E1 — CONFIRMED.** `wide.raw.ll` holds exactly one `load <16 x i8>`
  (the `@walk` probe) with exactly two needle compares (literal 10 and
  the hoisted first pattern byte); `base.raw.ll` holds zero; the
  `@.wf_trap.*` record tables of the two raw modules are byte-identical
  (19 records each).
- **E2 — CONFIRMED.** The optimized wide `main` keeps the probe as a
  reachable fast loop — `buffer.probe.load.v629.i`: one `<16 x i8>` load,
  `icmp eq … splat (i8 10)`, `icmp eq … %t88.i` (first-byte splat), `or`,
  `bitcast` to i16, then LLVM merged the frozen clean/found diamond into
  one zero-defined `cttz` plus a skip add on the fast backedge — and the
  scalar walk survives behind it with its trap-guarded byte load
  (`buffer.index.trap.v669.i` calling `@wf_trap(@.wf_trap.14, …)`).
  Both optimized modules retain the same seven `wf_trap` call sites and
  seven trap constants; the wide assembly alone contains the two `cmeq`
  vector compares.
- **E3 (binary-delta credit leg) — CONFIRMED.** `nomatch`
  instructions/byte 3.10 (bar ≤ 9; base 17.68) and cycles/byte 2.67
  (bar ≤ 2.7; base 3.87).

## Result

Null gates: null-before half-widths 0.21% / 0.19% / 0.41% / 1.27%
(large/nomatch/dense/many), null-after 0.98% / 0.18% / 0.29% / **2.17%**.
All null intervals contain 1.0. `many`'s null-after exceeds the 2% gate,
so per the inherited degradation rule its parity/directional
classifications are unavailable (precision-limited); material
classifications for `many` use the widened threshold 0.90 × (1 − 0.0217).

**bench-wide** — ratio = base elapsed / wide elapsed, above 1.0 means the
wide lowering is faster; medians of 30 paired warm rounds, 95% bootstrap
intervals:

| Case | base / wide | Classification |
|---|---|---|
| `large` | 1.431 [1.421, 1.455] | **material improvement** |
| `nomatch` | 1.428 [1.426, 1.432] | **material improvement** |
| `dense` | 1.156 [1.153, 1.161] | **material improvement** |
| `many` | 1.048 [1.034, 1.082] | precision-limited (directional-positive point) |
| `floor` | 0.980 [0.971, 1.003] | diagnostic |

**Product comparison** (grep elapsed / subject elapsed; above 1.0 means
the subject beats grep):

| Case | bench-grep (grep / base) | confirm (grep / wide) | Movement |
|---|---|---|---|
| `large` | 0.739 [0.735, 0.744] material loss | **1.069 [1.061, 1.096] directional win** | +0.33 |
| `nomatch` | 0.759 [0.750, 0.783] material loss | **1.071 [1.062, 1.084] directional win** | +0.31 |
| `dense` | 1.189 [1.171, 1.222] material win | **1.346 [1.337, 1.375] material win** | +0.16 |
| `many` | 0.503 [0.495, 0.517] material loss | 0.509 [0.499, 0.513] material loss | — |
| `floor` | 1.067 [1.044, 1.115] diagnostic | 1.102 [1.063, 1.148] diagnostic | — |

The fresh bench-grep baseline reproduces the 0023 rerun (0.753/0.762 on
large/nomatch) within same-day drift, so the movement is same-lineage:
the scan-dominated cases go from a material loss to a whole-interval win
over the system grep, and the match-dense case widens from 1.19 to 1.35.
`many` is unchanged, as attributed since WFGREP-BASELINE: its divergence
is the host's per-open treatment of unsigned binaries, outside any
Whitefoot layer.

Counters (medians of 5, `/usr/bin/time -l`): on `nomatch`
(268,435,456 bytes) base 17.68 instructions/byte at 3.87 cycles/byte,
wide 3.10 at 2.67, grep 15.43 at 2.81; `large` is the same picture
(3.11 at 2.67 versus grep's 15.39 at 3.03); `dense` wide 9.66 at 4.39
versus grep's 31.94 at 5.87. The walk's serial ~3.8-cycles/byte latency
floor from 0023 is broken by the 16-byte stride, and the wide binary now
retires fewer cycles per byte than the comparator on every compute case.

## Required checks, observably preserved

- Acceptance: unchanged by construction (the recognizer runs inside
  lowering, after checking; recognition failure is ordinary lowering).
  The compiler has no optimizer-fact channel; this single ordinary mode
  is the facts-off mode.
- Trap identity: the trap-identity oracle
  (`compiler/tests/programs/wide_scan.rs`) runs five probed walks and
  asserts exact per-byte results across lane boundaries (positions 0, 1,
  15, 16, 17, a newline, and the last byte; an absent needle; a bound
  below the length), and, on hostile bounds past the buffer length, the
  exact per-site OP-4 DIAG-3 record with SIGABRT — at the first byte of
  an empty buffer and one past the last byte at an unaligned offset
  inside a would-be stride — with every pre-trap published effect
  byte-identical to the scalar reference, and the two hostile sites'
  records distinct. Recognizer decline cases (quiet-path effect,
  non-single-step increment, inside-declared needle) are pinned in
  `compiler/src/lowering/tests.rs`.
- Gates: `make -C compiler check` EXIT=0 and `make check` EXIT=0 on the
  final tree (unpiped; 453 library tests + 28 program tests, including
  the nine-case wfgrep oracle and all ten §9.1 cost-shape gates, which
  needed no constant changes).

## Credit assignment

All credit-rule legs hold: E1+E2 present in the retained artifacts, E3's
binary-delta leg holds, bench-wide classifies material improvement on
both `large` and `nomatch` (and additionally on `dense`), the verify
phase shows zero pin divergence, all oracles and gates are green by
unpiped exit codes, and confirm moves the product ratio on
`large`/`nomatch` from 0.753/0.762 (the 0023 rerun) to 1.069/1.071.
F5 does not fire: no case regresses materially (`dense` improves
materially; `floor` is parity-band; `many` is precision-limited with a
positive point estimate). The wide probe is therefore a **credited win**
and the transform stands as the landed default on the single ordinary
lowering path.

## Demonstrated precision and disclosures

- Other agent workspaces were active on this host during the run; the
  null gates bound the achieved precision and came in under 2% on every
  case except `many`'s null-after (2.17%), handled by the inherited
  degradation rule above.
- Post-freeze development inspected emitted and optimized IR of unit
  fixtures and wfgrep for correctness, as the frozen protocol disclosed
  in advance; no timing, counter, or comparative measurement of any
  subject preceded the benchmark phases, which ran once, in order.
- LLVM merged the emitted clean/found diamond into a single zero-defined
  `cttz`; this is a semantics-preserving strengthening of the predicted
  shape, disclosed rather than silently reclassified: the E2 elements
  (one vector load, two compares, mask test, skip add, surviving scalar
  traps) are all present.
- The bench-phase absolute medians and the counter-phase `real` values
  differ by run context as in prior bundles; the frozen statistics are
  the paired within-round ratios.

## What this run does and does not mean

It means: this compiler can lower the fused trap-carrying byte walk to a
16-byte stride on the ordinary path with every required check preserved
observably — the FLOOR/lowering question the 0023 slice isolated is
answered YES by a general, semantics-preserving transform; the serial
per-byte latency floor is broken (3.87 → 2.67 cycles/byte on `nomatch`);
and the landed wfgrep now beats this host's `/usr/bin/grep -h -F` on
every compute-bound frozen case, including 1.35x on the match-dense one.

It does not mean: any ripgrep, GNU grep, regex, traversal, cold-cache, or
cross-host claim; any bounds-trap elision (no trap was removed — traps
were never in the hot path's skipped iterations to begin with, and every
reachable trap survives with its exact record); any claim about `many`,
whose divergence remains the host per-open identity cost; and no fraction
of any flagship claim. The probe applies only to loops in the recognized
byte-walk class; widening any other shape is future, separately
authorized work.
