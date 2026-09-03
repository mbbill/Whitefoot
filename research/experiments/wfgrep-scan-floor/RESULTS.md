# WF-SCAN-FLOOR results

Status: COMPLETE — both frozen shapes classify as practical parity

The exact scanner shapes, runtime inputs, correctness oracle, ordinary compiler
path, experiment-only linkage boundary, code-shape inspection, paired schedule,
statistics, interpretation bands, and stop condition were frozen in commit
`a965cb4b611b03273a68d1e98e7670c3ae4626e6` before comparative timing.

The single create-once run `wf-scan-floor-1` completed on AC power on
2026-08-05. Its 184-line raw record is retained at
`raw/wf-scan-floor-1.jsonl` with SHA-256
`c25a170e8de0181914ca0a7dde1e77d7703c770f6bfed45532b6f36f56ec2d28`.
It contains one header, all 180 preregistered samples, two summaries, and one
completion record; power was AC before and after. No sample was removed,
extended, or rerun.

## Replay status (2026-09-03)

This bundle is frozen v0.17-era evidence. `full_scan.wf` and `early_scan.wf`
no longer parse under the active specification: a function's result binding
is now named (`-> result: own u64`, the active grammar's `result_binding`),
and the `traps` effect both kernels declare was retired in v0.40 together
with every runtime trap, the very machinery whose guard-dominated bounds
traps the result below credits LLVM with removing. The Makefile that
compiled the kernels through the current compiler was removed on 2026-09-03,
so no research driver names a program the compiler rejects. The kernels, the
C and Rust controls, the harness, `runner.rs`, and the raw evidence stay as
the run's record; a replay starts from the freeze commit named above.

## Result

The primary ratio is `C elapsed / Whitefoot elapsed`; below 1 means Whitefoot
is slower. Both complete 95% descriptive bootstrap intervals lie inside the
frozen `[0.95, 1.05]` practical-parity band and contain 1.0.

| Shape | Comparator | Median ratio | 95% interval | Relative half-width | Classification |
| --- | --- | ---: | ---: | ---: | --- |
| full | same-Clang C | 0.999258 | [0.996948, 1.002261] | 0.266% | practical parity |
| full | safe Rust | 0.999059 | [0.993214, 1.001932] | 0.436% | practical parity |
| early | same-Clang C | 1.000848 | [0.998128, 1.008784] | 0.532% | practical parity |
| early | safe Rust | 0.998996 | [0.997804, 1.003521] | 0.286% | practical parity |

Absolute medians provide scale, not the causal statistic. The full kernel did
8 GiB of logical byte work per process: Whitefoot took 2.092759 s (3.823
GiB/s), C 2.090435 s (3.827 GiB/s), and Rust 2.092210 s (3.824 GiB/s). The
early kernel did 4,429,185,072 byte examinations: Whitefoot took 1.117020 s
(3.693 logical GiB/s), C 1.118720 s (3.687 logical GiB/s), and Rust 1.115805 s
(3.697 logical GiB/s). These early rates count repeated byte examinations, not
unique memory traffic.

This is positive floor evidence: the active language and ordinary compiler can
express these two same-algorithm, single-buffer scanner shapes without a
material machine-quality loss to C. It is not evidence that Whitefoot beats C,
Rust, ripgrep, or an optimized `memchr`/SIMD/substring algorithm, and it says
nothing about traversal, I/O, matching, line reconstruction, output, or
parallel scheduling. In particular, it supplies no fraction of the flagship
2x-ripgrep claim beyond removing these two elementary loops as an observed
language-floor blocker.

The full result tests the preregistered primary hypothesis. The early result is
an exploratory practical-parity classification under the same frozen rule.

## Frozen pre-timing code shape

The full-pass Whitefoot raw LLVM retains one explicit bounds trap in the source
index loop. Apple Clang `-O2` removes that trap from `wf_scan`, builds a main
`<16 x i8>` vector loop plus a `<4 x i8>` vector epilogue, and emits no helper or
library call. The same-Clang C control has the same 16-byte load, two byte
comparisons, widening, accumulation, and scalar-tail structure; register
allocation and assembly metadata differ.

The early-exit Whitefoot helper likewise contains one raw bounds trap. The
optimized `wf_scan` contains no trap call and inlines four scalar byte-search
loops. The C control has the same four-loop structure and no helper or library
call. The safe-Rust control retains four calls to its cross-crate `find_byte`,
so its early result is secondary toolchain/source-boundary evidence rather than
a substitute for the Whitefoot/C causal comparison.

These observations establish the expected final-code consequences before
timing. The completed timing is consistent with that mechanism: there is no
retained-check or final-code gap to explain for either frozen shape.

## Correctness and work identity

All six executables passed the independent empty, one-byte, mixed-marker,
all-byte, and generated-buffer cases immediately before timing. Every full
sample reported checksum `34282752274381056`, data hash
`4380160126096607260`, 128 repetitions, and length 67,108,864. Every early
sample reported checksum `4429185000`, data hash `13441015133733895468`, 24
repetitions, and the same length. The runner would have stopped on any mismatch,
unexpected output, crash, trap, or work-identity change.

The measured executable hashes were:

| Shape | Whitefoot | C | Rust |
| --- | --- | --- | --- |
| full | `cb9de8e888f2c65f49eec0432c67f13983c1735a6c738749a23d339ac4a0b8af` | `dcb4d6cf500ba5d1788962cdd2d10cfc47ad0f7b10fd72384a6b657e56b02d3a` | `fc1ff1fda24d51b3224d2cd39a54f966c721a0ebd662703add5df1354be87411` |
| early | `0cb3bfe8c1760eb04c88aa9732e32d08606f2b243b80aca9da740df54e65b5ce` | `3534d26b327efa6ed4c0608ddf0484e38f0759bbc5624af4cd5b9d5ff93c62da` | `71f6d351066d48bef81490655d472b8df55de0fb58f3e1b060f69cbf15c8db8c` |

## Noise and hostile review

Each variant occupied each process position ten times. Position-specific
median elapsed times stayed within about 1.1% of each variant's overall median;
there was no monotone advantage shared by all variants. Full-scan within-round
C ratios ranged from 0.9271 to 1.0875 and early ratios from 0.9916 to 1.0390.
Those visible excursions were retained; 28 of 30 full C ratios and all 30 early
C ratios remained inside +/-5%. The six five-round full-order subgroup medians
ranged from 0.9739 to 1.0039; the early subgroups ranged from 0.9982 to 1.0149.
The slow full subgroup was the order with C first and Whitefoot last, compatible
with third-position or machine-state drift, but no frequency or temperature
telemetry can establish that cause. The schedule gives every order exactly
equal weight. Removing any one order left the full C median between 0.9987 and
1.0008 and the early C median between 1.0006 and 1.0011.

The intervals are descriptive bootstrap intervals over complete rounds, not a
claim of host-independent uncertainty. This run cannot separate small
sub-percent differences from the disclosed sequential-process noise. That
noise, the lack of stratification in the bootstrap, and possible serial
correlation forbid ranking the three implementations from their point
estimates. The balanced schedule and order-removal sensitivity check leave the
frozen practical-parity classification intact.

## Decision

Stop under the preregistered rule. Record the P7 Boolean-dataflow full scanner
as current-compiler efficiency evidence and record that ordinary loop guards
let LLVM recover both frozen bounds checks. Do not propose a language,
compiler, proof, intrinsic, or runtime change from this result. The next
wfgrep performance experiment must come from a measured end-to-end or
algorithmic gap, not from adding favorable scanner microbenchmarks.
