# WF-LITERAL-LINE results

Status: COMPLETE — directional same-algorithm C advantage; large descriptive
library-algorithm signal

The semantics, runtime input, four implementations, correctness oracle,
machine-code inspection, balanced schedule, statistics, classification bands,
and claim boundary were frozen in commit
`7adb0faa55678997fdad3ddef15a311579c9d80a` before comparative timing.

The single create-once run `wf-literal-line-floor-1` completed on AC power on
2026-08-05. Its 130-line record is retained at
`raw/wf-literal-line-floor-1.jsonl` with SHA-256
`5221398928f3f7f7f78c5d885a250e7174c8eabebc9ecdc9f5d60e16a71fff0b`.
It contains one header, all 128 preregistered samples, and one completion
record. No sample was removed, extended, or rerun. Two earlier invocations
were refused before run-directory creation because AC power was not stable;
they produced no header or sample and did not consume the create-once run.

## Replay status (2026-09-03)

This bundle is frozen v0.17-era evidence. `literal_line.wf` no longer parses
under the active specification: a function's result binding is now named
(`-> result: own u64`, the active grammar's `result_binding`), and the
`traps` effect its kernel functions declare was retired in v0.40 together
with every runtime trap. The Makefile that compiled the kernel through the
current compiler was removed on 2026-09-03, so no research driver names a
program the compiler rejects. The kernel, the C and Rust controls, the
harness, `runner.rs`, and the raw evidence stay as the run's record; a
replay starts from the freeze commit named above.

## Result

The primary ratio is `C elapsed / Whitefoot elapsed`; below 1 means C is
faster. The algorithm ratio is `memmem Rust elapsed / naive Rust elapsed`.

| Comparison | Median ratio | 95% interval | Relative half-width | Frozen classification |
| --- | ---: | ---: | ---: | --- |
| C / Whitefoot | 0.953500 | [0.922344, 0.960942] | 2.024% | directional only: C faster |
| naive Rust / Whitefoot | 0.988283 | [0.976106, 1.002359] | 1.328% | diagnostic, includes 1.0 |
| memmem Rust / naive Rust | 0.136389 | [0.134852, 0.138298] | 1.263% | descriptive only |

The primary interval excludes 1.0 but is neither wholly inside the frozen
`[0.95, 1.05]` practical-parity band nor wholly below the `0.90` material-loss
threshold. In inverse form, Whitefoot took 1.048767x C at the point estimate,
with an inverse interval of [1.040646, 1.084194]. The exact result is therefore
a precise, directional roughly 5% same-algorithm loss, not practical parity
and not a material Whitefoot loss under the preregistered rule.

The pinned `memmem` implementation took 0.136389x naive Rust, corresponding to
7.331965x throughput at the point estimate with an inverse interval of
[7.230750, 7.415541]. This is a large and stable descriptive algorithm signal.
It is not promoted to the protocol's formal **material algorithmic-ceiling
gap**, because that classification required primary practical parity and the
primary interval missed that prerequisite. In particular, this is not a
ripgrep, end-to-end grep, language-feature, intrinsic, or 2x-wfgrep result.

Absolute medians provide scale, not causal statistics. Every fresh-process
sample consumed 16 scans of a 64 MiB input, or 1 GiB of logical haystack bytes.

| Variant | Median elapsed | Nominal logical rate |
| --- | ---: | ---: |
| Whitefoot scalar | 2.755058 s | 371.68 MiB/s |
| same-Clang C scalar | 2.648660 s | 386.61 MiB/s |
| safe naive Rust scalar | 2.698353 s | 379.49 MiB/s |
| pinned Rust `memmem` | 0.374866 s | 2731.64 MiB/s |

These rates count the frozen logical haystack identity. They are not unique
memory traffic: every implementation also locates and rescans matched lines.

## Correctness and evidence identity

All four controls passed the independent focused-case tuple oracle, exhaustive
small-domain cases, the 8 MiB high-candidate adversarial case, and the real
prefix before timing. Every timed sample reported the same aggregate digest
`14073950566171110000`, 74 records, 16 repetitions, and 67,108,864-byte input.
The real input also passed the frozen 1,324,231-LF, 20,172,286-first-byte, and
63,538,171-final-match-start invariants.

Independent post-run review found exactly one header, 128 samples, and one
completion record. Each variant has 32 samples covering rounds 0 through 31;
each occupies all four ordinal positions exactly eight times; every round has
all four variants and positions; and there are no missing or duplicate work
identities. The header and completion record both report AC power.

The header binds the following measured executables:

| Variant | SHA-256 |
| --- | --- |
| Whitefoot | `9ecddf08063afad03cb45906bae4ef121c6a140d6a1331e5fa4452d5144d3fac` |
| C | `1c760676f70ca0b76e1bfbbb2e58d244c366b3a09479094b83e8c0f6b1366982` |
| naive Rust | `f33d3bee6c4322049f96bb2dd15e1141f9e6033c562b8639b3949ae4fc12de85` |
| `memmem` Rust | `649646319ed334aed4b9129ad520b742c698083f4568915f386cc83b317762e1` |

The protocol SHA-256 is
`6163b222ef4cfa0bdc8c39de0a44f9b25a5316272797b65ea552005d898e27d2`,
the pre-timing code-shape record SHA-256 is
`d282efad817e1ae31cd79ec99ad35085cb21ba474428e5f90c989d77dcec23dc`,
and the input SHA-256 is
`ce55e37ed74f5b34773ce83597e5d61a83d0d0792d9cbb95fe0fc898ed09a1ee`.
All independently recomputed hashes equal the run header.

## Pre-timing machine shape and attribution

The scalar sources all spell the frozen two-stage candidate-and-verify
algorithm and contain no search/comparison library call. Whitefoot raw LLVM
has seven OP-4 trap sites. Apple Clang removes four, including all three in the
top-level line scanner, but retains three inside `wf_find_scalar`: the initial
needle access, candidate haystack access, and tail haystack access.
`wf_find_scalar` remains a final helper with three assembly call sites. The
same Clang fully inlines the equivalent C helper. Rustc also inlines the safe
Rust scalar search but retains two bounds-panic calls. No scalar assembly
contains a backend-introduced `memcmp` or `bcmp` call.

These differences are plausible explanations for a small C/Whitefoot delta,
but this experiment does not isolate call overhead, retained-check cost,
missed inlining, or another lowering effect. The primary difference is below
the material band, and the safe-Rust diagnostic is close to Whitefoot despite
its own retained panic paths. It would be an overclaim to assign the roughly
5% result specifically to checks or to open proof work from it.

The pinned `memmem` executable has the preregistered indirect `Finder`
dispatch, `searcher_kind_neon`, AArch64 `cmeq.16b` packed-pair candidate
filtering, and full-match verification. This mechanism is qualitatively
different from all three scalar controls and matches the large descriptive
ratio. The experiment identifies algorithm choice as a much larger observed
performance lever than the unresolved scalar compiler delta, while preserving
the formal-classification prerequisite above.

## Noise and sensitivity

Absolute time drifted during the sequential run, so the causal statistics use
within-round ratios and bootstrap complete four-round Williams blocks. Median
elapsed time by process position ranged from 2.7330 to 2.7822 s for Whitefoot,
2.5929 to 2.6693 s for C, 2.6483 to 2.7287 s for naive Rust, and 0.3693 to
0.3773 s for `memmem`.

Removing any one order class left the C/Whitefoot median in
[0.941035, 0.959948], the cross-toolchain median in
[0.986246, 0.994414], and the algorithm ratio in [0.136104, 0.137344]. An
additional non-preregistered leave-one-time-block sensitivity check left the
C/Whitefoot median in [0.948832, 0.959948] and the algorithm ratio in
[0.136104, 0.136783]. These checks do not replace the frozen statistics, but
they show that neither reported direction is created by one process position,
order class, or four-round time block. No temperature or frequency telemetry
supports a more specific explanation of the visible absolute-time drift.

## Decision

Stop after the one frozen run. Record three bounded facts:

1. the active v0.17 language can express the complete runtime-needle literal
   line contract and pass exact correctness checks;
2. this helper-shaped scalar lowering is directionally behind same-Clang C by
   roughly 5%, with retained checks and non-inlining present but no isolated
   material compiler or proof cause; and
3. the pinned SIMD substring algorithm is roughly 7.33x faster than the same
   Rust toolchain's scalar algorithm on this input, as descriptive mechanism
   evidence whose formal promotion prerequisite was not met.

Do not change the language, compiler, proof system, intrinsic set, or runtime
from this result. Future wfgrep work should treat an efficient taught
runtime-literal search pattern or library capability as a high-value candidate
to test in a separately authorized slice, while keeping helper/check lowering
as a smaller measured floor question. It must still establish end-to-end value
and cannot infer a product result from this in-memory experiment.
