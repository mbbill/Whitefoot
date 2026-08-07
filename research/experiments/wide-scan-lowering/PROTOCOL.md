# WIDE-SCAN-LOWERING protocol

Status: FROZEN BEFORE ANY MEASUREMENT AND BEFORE ANY IMPLEMENTATION

The Git commit that carries this file is the freeze identity. At freeze
time, no line of the candidate transform exists, no emitted or optimized
IR of any candidate build has been observed, and no comparative number,
hardware counter, or assembly listing has been taken for this question.
Every expected-code-shape statement below is a design prediction, not an
observation.

## Authorization

Task `0026-check-aware-wide-scan` under the `ACTIVE` `docs/current-plan.md`
Work item 1 (PERF-1/FLOOR: check-aware wide-scan lowering slice), base
revision `23bb7b06f2769611bf0859b6a3be5bf16e442384`. This bundle authorizes
one compiler-lowering investigation on the ordinary path. It authorizes no
specification change, no change to `tests/programs/wfgrep.wf` bytes, no
bounds-trap elision or weakening of any required check, no optimizer-fact
channel, and no work beyond the routes below.

## Question

Can this compiler lower the fused trap-carrying byte walk (the landed S2
shape of `tests/programs/wfgrep.wf`, the `@walk` loop) to a wide stride —
W bytes examined per fast step — while preserving every required check
observably? Preservation is observational: on a hostile input the trap
must still fire at the exact byte it fires at today, with byte-identical
DIAG-3 record on stderr and abort; on every accepted input, outputs, exit
codes, and effect order are byte-identical; acceptance of programs is
unchanged.

Per the task, three routes are evaluated in order of preference, and an
honest negative on any of them is a successful outcome.

## Route (a) — pure emission shape LLVM auto-vectorizes: preregistered as closed, with witnesses

Evidence already on record, cited rather than re-measured:

1. WF-SCAN-FLOOR (frozen `a965cb4`, run 2026-08-05, this host, same
   `/usr/bin/clang`): the early-exit byte-search kernel stays scalar even
   for the same-Clang **C** control at `-O2` — the toolchain performs no
   early-exit (uncountable-exit) loop vectorization on any spelling,
   independent of Whitefoot's checks.
2. WFGREP-DOUBLE-WALK S3 witness: eight guarded byte loads never fuse
   into a wide load; each trap-guarded bounds branch owns a distinct
   noreturn side effect that pins evaluation order (`s3.opt.ll`, no
   `load i64` anywhere).
3. The fused walk is additionally a multi-exit loop with a nested verify
   loop and loop-carried `hit`/`terminator` state — outside LLVM's
   vectorizable class regardless of 1 and 2.

Preregistered conclusion: no reachable emission of the *same per-byte
loop semantics* auto-vectorizes on this toolchain; route (a) is closed
by witnesses 1-3. Reopening falsifier: if the base build's `code-shape`
phase (below) shows a vector fast path in the optimized `@walk` of the
unmodified compiler, route (a) reopens and route (b) is unnecessary.

## Route (b) — the candidate: a compiler-derived wide probe on the single lowering path

This is the route the plan names ("a compiler-derived per-block
obligation hoist that provably preserves trap identity"). The transform
is specified exactly, before implementation:

### Recognized semantic form (by grammar and semantic rule only)

In `lowering`, at `loop` lowering, a loop body qualifies iff, on the
checked AST (no name, project, corpus, or test identity participates):

- R1 (exit guard): the body's first two statements are
  `let done = ige<u64>(i, bound)` + `match done { True => { break L } False => {} }`,
  or the `ilt` polarity (`True => {}`, `False => { break L }`), where `i`
  and `bound` are bindings declared outside the loop and the `False`/`True`
  empty arm has no statements.
- R2 (probe load): the next statement is `let b = index<E>(buf, i)` where
  `buf` is a buffer binding declared outside the loop, `E` is `u8`, and
  the offset is exactly the binding `i`.
- R3 (increment): the body's last statement is
  `set i = iadd.wrap<u64>(i, 1_u64)`.
- R4 (neutral middle): every statement between R2 and R3 is *neutral*,
  defined recursively:
  - a `let` whose value is a pure, trap-free operation (constant, binding
    read, integer/boolean/enum-equality operation with no trap, numeric
    conversion, reinterpret) is neutral; if it has the exact shape
    `let c = ieq<u8>(b, k)` with `k` a `u8` literal or a `u8` binding
    declared outside the loop, it *registers* needle `k` with binder `c`;
  - a `match` on a registered binder `c` whose `False` arm is all-neutral
    with no fallthrough drops is neutral; its `True` arm is unrestricted;
  - a `match` on any other `Bool` binding whose *both* arms are
    all-neutral with no fallthrough drops is neutral;
  - nothing else is neutral (any `set`, `break`, `check`, call, index,
    store, region, loop, or trapping operation outside a registered
    `True` arm disqualifies the loop).
- R5 (closure): at least one needle registers, at most 4 register, and
  the loop's backedge drops are empty.

Soundness argument, fixed in advance: for a byte value matching no
registered needle, the middle statements reach R3 with no effect — every
registered `match` takes its neutral arm, every other `match` is neutral
on both sides, and neutral statements are pure and trap-free — so the
iteration is observationally `i := i + 1`. All reads of `buf`, `bound`,
`i`, and needle bindings inside one iteration read the same loop-header
values (block-parameter SSA), and a no-op run cannot change any of them
nor any memory. Therefore skipping s consecutive iterations whose bytes
all fail every needle, with `i + W <= min(bound, len(buf))` establishing
that each skipped index is in bounds and below the exit bound, is
observationally identity. Every byte at which *anything* can happen —
any needle hit, the exit at `bound`, and every possible trap, including
the hostile `bound > len` trap at exactly `i = len` — is executed by the
UNCHANGED scalar body with its unchanged per-site DIAG-3 records. No
required check is removed: every executed `index` keeps its guard; the
wide probe reads only bytes proven in bounds by its own internal guard
and itself carries no trap and reports nothing.

### Lowering and emission (fixed W = 16)

At the loop header, before the ordinary body lowering, the builder emits
on the single normal path:

- `skip = buffer.probe.skip(buf, i, bound, needles…)` — one new checked-IR
  operation with internal guard, result `u64`:
  - if `min(bound, len(buf)) < W` or `i > min(bound, len(buf)) − W`: 0;
  - else load `<16 x i8>` at `ptr + i` (align 1), compare each lane
    against each needle splat, OR the masks; if no lane matches: W;
    else the number of leading clean lanes (cttz of the LSB-first mask;
    all four supported triples are little-endian);
- `match igt<u64>(skip, 0)`:
  - True: `i' = iadd.wrap(i, skip)`; jump to the header with `i → i'`,
    all other carried bindings unchanged, no drops;
  - False: fall into the ordinary, byte-identical scalar body.

The scalar body, its trap sites, its record contents, and its emission
are untouched. Recognizer failure at any condition R1-R5 means ordinary
lowering with zero change. Acceptance cannot change: the recognizer runs
inside lowering, after all checking, and can only add the fast path or
do nothing. The compiler has no optimizer-fact channel
(`HOST_OPTIMIZATION_ARGUMENTS` is a fixed `-O2`); "facts-off" is the
only and ordinary mode, and this transform is part of ordinary lowering,
not a fact consumer.

### Expected emitted-code shape (falsifiable, before implementation)

- E1 (raw): the emitted module for `wfgrep.wf` contains exactly one
  `load <16 x i8>` (in `main`'s `@walk` header probe), with exactly two
  vector compares (needles: literal `10` and the hoisted first pattern
  byte) OR'd into one mask; the scalar `@walk` body behind it still
  contains its per-byte trap-guarded `load i8`; the module's set of
  `@.wf_trap.*` records is unchanged from the base build's set (same
  count, same byte contents).
- E2 (optimized): the optimized `main` retains the vector probe as a
  reachable fast loop — one `<16 x i8>` load, two vector `icmp eq`, one
  mask test, one skip add on the fast backedge — and retains the scalar
  per-byte walk with its trap-guarded load as the fallback path. LLVM
  neither deletes the probe nor converts the scalar path's traps.
- E3 (counters, binary-delta leg of the credit rule): `nomatch`
  instructions/byte ≤ 9 (base S2 measured 17.67) and cycles/byte ≤ 2.7
  (base 3.81). The corpus makes this precise: content bytes are
  `a-z`/space, both patterns start with `X`, which occurs only at
  injected needles, so interesting bytes on `large`/`nomatch` are ≈ one
  newline per 60-120-byte line plus one `X` per ~1024 lines — ≥ 70% of
  bytes are skippable in full 16-byte probes.

Falsifiers for route (b), fixed in advance:

- F1 (legality): the recognizer cannot cover the landed `@walk` without
  violating R1-R5 as written, or any R-condition must be weakened to
  admit it. Then route (b) fails as specified; record the exact failing
  statement as the obstruction witness and do not ship a weakened rule.
- F2 (shape): E1 or E2 fails in the retained artifacts (probe absent,
  duplicated, or optimized away; any trap record changed, added, or
  removed). A trap-record delta is an immediate kill, not a tunable.
- F3 (identity): any oracle divergence — wfgrep nine-case oracle, the
  25 verify pins, or the trap-identity oracle below — kills the route
  regardless of speed.
- F4 (speed): the paired wall ratio interval (base-compiler binary /
  wide-compiler binary, same `wfgrep.wf` bytes) fails to classify
  material improvement on both `large` and `nomatch` under the inherited
  bands, or E3's binary-delta leg fails. Then no credit is recorded even
  if some improvement exists.
- F5 (collateral): material regression on `dense` or `many`, or any §9.1
  cost-shape gate failure. A `dense`/`many` regression does not undo a
  `large`/`nomatch` finding but blocks the "landed default" outcome and
  escalates the ship/no-ship decision to the lead with numbers.

### Trap-identity oracle (new, compiler-independent expectations)

A new integration test (`compiler/tests/programs/` harness; expectations
are fixed input/output pairs, independent of compiler internals) runs
recognized-form walk programs whose loop bound deliberately exceeds the
buffer length, plus matched control programs, and asserts:

- T1: violation at the first byte (`len = 0`, `bound > 0`): SIGABRT with
  stderr exactly the walk's `index` site DIAG-3 JSON record.
- T2: violation at an unaligned offset inside a would-be wide stride
  (`len` not a multiple of 16, needle bytes placed after `len`): the
  same exact record; the needles beyond `len` must NOT be reported
  (no output that only a wide over-read could produce).
- T3: violation exactly one past the last byte (`bound = len + 1`), with
  the last in-bounds byte both needle and non-needle: same exact record,
  and pre-trap published effects exactly equal to the scalar reference.
- T4 (equivalence, non-hostile): walk results (found position, hit
  count) equal the scalar reference for needle positions 0..16 within a
  block, at block boundaries, at `len − 1`, absent, and for `bound` both
  `= len` and `< len`, on both the base and candidate compilers.

The exact record bytes are pinned from the BASE compiler's build of each
oracle program first; the candidate must reproduce them byte-identically.

## Route (c) — recorded negative

If F1 fires, or F2/F3 fire and the defect is the mechanism rather than an
implementation bug, the outcome is the precise obstruction statement: the
minimal witness program, the R-condition or emission property that cannot
be satisfied, and the missing mechanism it names (a PROOF-1-class
verified fact family, or a language change), recorded in RESULTS.md. That
closes the task successfully with no benchmark phases run (or with them
run only as diagnostics, clearly labeled).

## Benchmark protocol (only if routes a/b produce a candidate binary)

Everything not stated here is inherited unchanged from WFGREP-DOUBLE-WALK
and WFGREP-BASELINE: corpus generation code and pins
(`../wfgrep-baseline/MANIFEST.txt`), the five cases and their roles
(`floor` diagnostic, never classified), equivalent-work rules, warming,
3 discards, 30 rounds strict alternation, within-round ratio, median of
30, 10,000-resample bootstrap (seed 20260807), 95% percentile intervals,
the null gates (half-width ≤ 2%, degradation rule, > 5% inconclusive),
the bands (parity `[0.95, 1.05]`, material below `0.90` / above `1.10`,
widened by `w`), counters via `/usr/bin/time -l` ×5 medians, and the
sample-integrity rule (nothing deleted, extended, or rerun).

Subjects — the source bytes are IDENTICAL (`tests/programs/wfgrep.wf`,
landed S2); only the compiler differs:

- **B**: built by the base-revision compiler (`23bb7b0`, worktree pinned
  at that commit).
- **WIDE**: built by the candidate compiler (this branch's head at run
  time, recorded in RESULTS.md).

Phases, in order, driven by a bundle Makefile adapted from
`../wfgrep-double-walk/Makefile` and a runner adapted from its
`runner.rs` (adaptation: subjects B/WIDE, phase names; generation,
verification, statistics, and schedule code unchanged):

```text
build -> gen -> verify (both subjects, all 25 subject-case pins) ->
code-shape (raw.ll/opt.ll/.s of both, digests recorded) ->
oracle+gates (nine-case oracle, trap-identity oracle, ten §9.1 gates,
`make -C compiler check`, `make check`, unpiped, exit codes recorded) ->
null-before (B vs B) -> bench-grep (grep vs B, product baseline rerun) ->
bench-wide (B vs WIDE, the primary statistic) -> null-after ->
counters (B, WIDE, grep) -> confirm (grep vs WIDE, product movement) ->
RESULTS.md
```

- **bench-wide**: ratio = B elapsed / WIDE elapsed; above 1.0 means the
  wide lowering is faster. Primary cases `large` and `nomatch`;
  `dense`/`many` are the collateral cases for F5; `floor` diagnostic.
- **confirm**: ratio = grep elapsed / WIDE elapsed, the baseline
  orientation — this is the frozen-baseline rerun the credit rule and
  the plan's "Return and replace" step require.

### Credit rule, fixed in advance

A **credited win** requires ALL of: E1+E2 present in the retained
artifacts; E3's binary-delta leg holds; bench-wide classifies material
improvement on both `large` and `nomatch`; zero output/pin divergence;
the trap-identity oracle, nine-case oracle, and all ten §9.1 gates green
by unpiped exit codes; and the confirm phase moves the product ratio on
`large`/`nomatch` above the 0023 rerun values (0.753/0.762). Anything
less is recorded exactly as what it is (improvement without credit,
parity, regression, or negative), and F5 gates the landed-default
decision separately.

## Out of scope

No bounds-trap elision and no `llvm.assume`; no source change to any
`tests/programs/*.wf`; no per-program, per-function-name, or per-test
dispatch anywhere in the transform; no W tuning after a number is seen
(W = 16 is frozen); no new workload; no comparator retuning; no claim
beyond this host and comparator; no PROOF-1 implementation. If a design
question arises that the checked-IR semantics do not settle, the task
stops and reports rather than deciding it here.

## Post-freeze development discipline, disclosed in advance

Implementation happens after this freeze. Correctness-driven inspection
of emitted and optimized IR (unit fixtures and wfgrep) during
development is allowed and expected — the falsifiers above are already
frozen, so inspection cannot move them. No timing, no counters, and no
comparative measurement of any subject before the benchmark phases run
in protocol order. The environment identity at freeze: Apple M4, macOS
26.5 (Darwin 25.5.0), `/usr/bin/clang` = Apple clang 21.0.0
(clang-2100.1.1.101), target `arm64-apple-darwin25.5.0`, comparator
`/usr/bin/grep` re-pinned at verify. Run id `wide-scan-lowering-1`; all
phases append to `raw/wide-scan-lowering-1.jsonl`.
