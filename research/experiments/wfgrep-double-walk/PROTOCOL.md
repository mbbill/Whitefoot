# WFGREP-DOUBLE-WALK protocol

Status: FROZEN BEFORE ANY COMPARATIVE TIMING

The Git commit containing this status, the three shape sources under
`shapes/`, `runner.rs`, and the bundle `Makefile` is the freeze identity.
No comparative number, hardware counter, emitted-IR dump, or assembly
listing of any shape was observed before that commit. Pre-freeze
development is disclosed below.

## Authorization

Task `0023-double-walk-optimization-slice` under the `ACTIVE`
`docs/current-plan.md` Work item 1 (PERF-1, one attributed cause). This
bundle attempts catalog-legal source shapes against the divergence that
WFGREP-BASELINE attributed to the algorithm/source-shape layer, on the
ordinary compiler path. It authorizes no language, compiler, specification,
optimizer-fact, or proof change, no bounds-trap elision, and no widening of
the project. The bounds-trap secondary (~18% ceiling, per the baseline) is
out of scope.

## Question

Can a legal Whitefoot source shape close the scalar double-walk compute
divergence of the sequential `wfgrep` — every input byte walked twice by
scalar per-byte loops, 9 instructions/byte for the newline scan plus 13 for
the literal matcher (baseline attribution) — while preserving byte-identical
behavior, all required checks, and the frozen §9.1 cost shape?

No direction is assumed. A credited material improvement, a demonstrated
inability of every attempted legal shape to reach a materially faster form
(a FLOOR/lowering finding with a minimal witness), and a preregistered
precision failure are all successful outcomes.

## Subjects

All subjects build through the ordinary compiler path only —
`whitefootc -o BIN SOURCE.wf` (emitted LLVM through `/usr/bin/clang -O2`),
compiler revision = the freeze commit's tree (base `7240f84` plus this
bundle). No flag, fact, patch, or check change. Source digests (SHA-256):

- **B0 (fresh baseline)**: the current `tests/programs/wfgrep.wf` bytes —
  task 0021's helper-decomposed form —
  `7c7833906e9b8bf512eac3499e30bda50e49ecffd971650a8e15c036be137595`.
  These differ from the bytes WFGREP-BASELINE froze
  (`d5f94c1a…`), so every shape comparison here is against a fresh
  same-protocol baseline of the current bytes, first, making all
  comparisons same-source-lineage on the same day and host.
- **S1 (restructured inner loop: hoisted first byte)**:
  `shapes/s1-hoisted-first-byte.wf`,
  `c2192db88b100c65cf9b00ee1e5304b3e8adbf85657f6772316cd1d0c7ceacef`.
  The double walk is kept. One variable: `line_matches` loads
  `pattern[0]` into a local once per line and the candidate loop compares
  each source byte against that local, entering the byte-verify loop only
  on a first-byte hit (verify starts at offset 1).
- **S2 (fused single-pass scan+match)**:
  `shapes/s2-fused-scan-match.wf`,
  `0216dcaa7c052a78b482728f987d7f639f96975840bbfed23e51d9f40eebca72`.
  `line_matches` is deleted; `main` walks each byte once, testing newline
  and hoisted first-pattern-byte in the same pass, verifying candidates
  in place with two guards that preserve the frozen contract exactly: a
  source newline byte inside a verify window is an unconditional mismatch
  (in-line windows never contain a newline, and any window crossing the
  line terminator fails at it, exactly as the bounded matcher rejects it),
  and a window reaching `available` is a mismatch (the incomplete tail is
  carried and re-walked next chunk, as today). The empty pattern
  initializes each line's hit flag true. A matched line's remaining bytes
  skip the candidate filter.
- **S3 (widened word-at-a-time newline scan)**:
  `shapes/s3-swar-newline-scan.wf`,
  `ec9ba19cc5759b6ac3c684233c1c0a18da35f0c3a9a0ffb86065e83307ed32e4`.
  The double walk is kept. One variable: the newline scan first steps in
  8-byte words — eight indexed loads assembled by `cvt`/`ishl.wrap`/`ior`
  into a `u64`, tested by the exact SWAR zero-byte predicate
  `((w ^ 0x0A…0A) − 0x01…01) & ~(w ^ 0x0A…0A) & 0x80…80` — falling to the
  unchanged byte loop to locate the newline (or for the < 8-byte tail).
  The predicate is exact for presence of a newline in the word, so the
  byte loop always finds the true first terminator.

### Legal shapes considered and not attempted, with reasons

- **Vectorized early-exit scan**: no v0.19 source form can request
  vectorization; WF-SCAN-FLOOR showed early-exit byte-search loops stay
  scalar even for same-Clang C, so there is no legal spelling whose
  ordinary lowering differs — S3 is the one legal widening.
- **Word-at-a-time literal verify**: the verify loop is cold on every
  frozen case once S1/S2's first-byte filter exists (candidates are rare
  by corpus construction and by the general first-byte-mismatch argument);
  it cannot carry the primary statistic. Not a candidate this slice.
- **P7 branchless i1 dataflow scan**: the scan must yield the newline
  *position*, not a count; the P7 recurrence has no position output
  without an early exit, which returns it to the S3/byte-loop shapes.
- **Buffer/chunk-size retuning, I/O restructuring**: outside the
  attributed cause (the read path measured 13% of `large` wall and the
  chunk size is a frozen §9.1 property).

### Ablation logic (same-source causal ablation, preregistered)

S1 changes only the matcher's candidate loop; S3 changes only the scan;
S2 replaces both walks with one. Comparing each against B0, and S2 against
S1's result, separates the three mechanisms: candidate-restart removal
(S1), scan widening (S3), and walk fusion beyond candidate filtering
(S2 − S1). Credit for any composite mechanism cites these single-variable
measurements, not the composite alone.

## Expected code-shape consequences and falsifiers

The baseline's static model (9 + 13 = 22 instructions/byte, matching its
measured 22.3 on `nomatch`) is the reference. B0's helper decomposition
touched no per-byte loop, so B0 is expected within ±1 instruction/byte of
22.3; every shape expectation below is stated against B0's *measured*
value. The code-shape phase runs before any timing and records, per
subject, the optimized LLVM and assembly of the same module the timed
binary is built from.

- **S1** — expected consequence: the optimized candidate fast path loses
  the per-candidate pattern reload (the baseline's hot `+1584` shape); the
  candidate-advance loop keeps exactly one memory load (the source byte)
  compared against a register; `nomatch` instructions/byte drops by ≥ 2.
  Expected wall ratio vs B0 on `large`/`nomatch` around 1.10–1.25.
  Falsifier: the candidate loop still loads a pattern byte per candidate,
  or the instructions/byte drop is < 2, or the 95% interval fails to lie
  wholly above 1.10 on both `large` and `nomatch`.
- **S2** — expected consequence: the optimized `main` contains one
  per-byte fast-path loop (one source-byte load, a newline compare and a
  register first-byte compare) and no `wf_line_matches` definition or call
  anywhere in the module; `nomatch` instructions/byte drops to ≤ 14
  (static model ≈ 11). Expected wall ratio vs B0 on `large`/`nomatch`
  ≥ 1.4. Falsifier: two separate per-byte walks persist in the optimized
  `main`, or `nomatch` instructions/byte stays > 16, or the 95% interval
  fails to lie wholly above 1.10 on both `large` and `nomatch`.
- **S3** — preregistered prediction: the widening does NOT materialize as
  a wide load. Each of the eight loads sits behind its own trap-guarded
  bounds branch against a runtime buffer length; the trap call's side
  effect pins evaluation order, and `-O2` load-combining does not fuse
  across trap-guarded branches, so the optimized scan is expected to keep
  ≥ 8 separate byte loads per word step and the raw assembly ≥ 40
  instructions per 8-byte word (≈ 7/byte gross, against 9 scalar), with
  the matcher's 13 unchanged — expected classification parity or
  directional, not material. Falsifier of the obstruction claim (the good
  surprise): the optimized scan performs one 8-byte load per word step
  and `nomatch` instructions/byte drops by ≥ 4. Either outcome closes the
  "widened comparison" branch: confirmation of the prediction is the
  minimal witness that guarded byte indexing blocks word formation under
  ordinary lowering.
- **S4 (conditional composite, S2+S3)** — constructed and run only if
  both S2's and S3's expected deltas confirm; its source would replace
  S2's pre-candidate walk with S3's word step. If constructed, it is a
  labeled post-freeze artifact under this rule, with the same bands and
  the falsifier that it must beat S2's ratio to credit the widening term.

Predicted concentration, falsifiable: the improvement appears on
`large`/`nomatch` (compute-bound), is smaller on `dense` (output path
share; the fused walk also pays a per-byte hit-flag branch there), and is
small on `many` (the baseline attributed its divergence to the host
per-open identity cost, not a Whitefoot layer; only the ~66 ms compute
share can move).

## Frozen environment, corpus, and work identity

Inherited unchanged from WFGREP-BASELINE (`../wfgrep-baseline/`):

- Corpus, generation seeds, and rules: identical `runner.rs` generation
  code; every corpus digest, per-case output SHA-256, and exit code is
  verified against the same pinned `../wfgrep-baseline/MANIFEST.txt`
  before any timed phase. The five cases (`large`, `nomatch`, `dense`,
  `many`, `floor`) and their roles are unchanged; `floor` remains a
  diagnostic, never classified.
- Equivalent work: identical pattern, file list, order, working
  directory, cleared environment (`LC_ALL=C` only); timed stdout to
  `/dev/null`, stderr required empty, exit codes required equal to pins.
- Comparator for the fresh-baseline and confirmation phases:
  `/usr/bin/grep` as `grep -h -F` (same pinned binary as the baseline;
  its identity is re-recorded in the verify phase).
- Noise controls: explicit whole-file warming before every round; three
  discarded invocations of each side per case before its first round; 30
  rounds per case with strict order alternation; position medians
  recorded; `pmset` power source and Low Power Mode recorded at phase
  boundaries (AC is the house default and holds at authoring); no CPU
  pinning on macOS, disclosed not simulated.
- Statistic: within-round ratio; median of 30; deterministic 10,000-
  resample bootstrap over rounds, seed 20260806, 95% percentile interval.
- Samples are never deleted, extended, or rerun after a result is
  observed. Run id `wfgrep-double-walk-1`; all phases append to
  `raw/wfgrep-double-walk-1.jsonl`.

## Phases, ratios, and interpretation

Phase order:

```text
build -> verify -> code-shape -> null-before -> bench-grep ->
bench-shape s1 -> bench-shape s2 -> bench-shape s3 [-> bench-shape s4] ->
null-after -> counters [-> confirm BEST] -> RESULTS.md
```

- **null-before / null-after**: B0 against itself under the full harness,
  every case. Gate per classified case: both null intervals must have
  relative half-width ≤ 2%; the demonstrated precision `w` is the larger
  half-width, with any resolved position bias folded in as in the
  baseline. A comparative interval with relative half-width > 5% is
  precision-inconclusive.
- **bench-grep (fresh baseline)**: ratio = grep elapsed / B0 elapsed,
  below 1.0 means B0 slower — the baseline's exact orientation and bands
  (parity `[0.95, 1.05]`, material loss below `0.90`, material win above
  `1.10`, widened by `w` under the degradation rule).
- **bench-shape sN**: ratio = B0 elapsed / sN elapsed, above 1.0 means
  the shape is faster. Bands, fixed in advance, per classified case:
  **material improvement** — the whole interval lies above `1.10 × (1+w)`
  when `w` exceeds 2%, else above `1.10`; **material regression** — the
  whole interval below `0.90`; **practical parity** — the whole interval
  within `[0.95, 1.05]`; **directional** — the interval excludes 1.0 but
  clears no material threshold; otherwise **inconclusive**.
- **counters**: `/usr/bin/time -l`, 5 repetitions per binary per case,
  medians, descriptive; instructions/byte computed against the case's
  corpus bytes (`large`/`nomatch` 268,435,456; `dense` 134,217,728;
  `many` 67,108,864).
- **confirm** (conditional): if at least one shape classifies material
  improvement on both `large` and `nomatch`, the best such shape (largest
  `nomatch` ratio median) is paired directly against grep — ratio = grep
  elapsed / shape elapsed, the baseline orientation and bands — for the
  updated product comparison. Run once, after counters.

## Credit and outcome rules, fixed in advance

A shape is a **credited win** only if all of: the verify phase shows
byte-identical pinned outputs and exit codes on every case; the nine-case
oracle and the ten §9.1 cost-shape gates run green with the shape's bytes
substituted at `tests/programs/wfgrep.wf` (pre-freeze development runs
already satisfied this; they are rerun at closure for the landed shape);
its preregistered expected code-shape consequence is present in the
retained optimized artifacts; its ratio classifies material improvement on
both `large` and `nomatch`; and no material work difference remains
unexplained after the ablation logic above. A credited win lands the
winning bytes as `tests/programs/wfgrep.wf` with the §9.1 gate constants
re-derived from the new source (per task 0016's re-derivation rule —
counts move only by source derivation, never by relaxation), and the
outcome feeds the plan's rerun-baseline deliverable.

If NO attempted shape reaches material improvement, the recorded result is
the demonstrated obstruction: which legal shapes were tried, what each
measured, and the minimal witness (the retained code-shape artifacts
naming why each expected delta did or did not appear). That negative is a
full success of the slice and feeds the FLOOR/lowering decision.

If the null gates fail, the recorded result of the attempt is the
precision failure itself.

## Pre-freeze development, disclosed

Before this freeze, the three shape sources were developed to
correctness: each was compiled by `whitefootc`, and each ran green — with
its bytes substituted at `tests/programs/wfgrep.wf` — against the
nine-case oracle (`compiler/tests/programs/wfgrep.rs`) and the ten §9.1
gates (`compiler/src/backend/tests/cost_shape.rs`); `tests/programs/`
was restored to the current bytes afterward. Two syntax/resolution
rejections were fixed during that development (a `check` keyword
collision and a TYPE-6 shadowing collision, both renames). No timing, no
counters, and no inspection of any subject's emitted or optimized IR or
assembly occurred before the freeze. The baseline bundle's corpus and
manifest are reused, so no new corpus was generated for this experiment.

## Out of scope

No bounds-trap change or PROOF-1 work, no optimizer fact, no compiler or
language change, no new workload after a result is seen, no comparator
retuning, no removal of any sample, and no claim beyond this host and
this comparator. The `dense` win/loss structure against grep's
architecture is reported as context only.
