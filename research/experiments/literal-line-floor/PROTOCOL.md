# WF-LITERAL-LINE protocol

Status: FROZEN BEFORE COMPARATIVE TIMING

The Git commit containing this status, this complete protocol, and the
experiment apparatus is the freeze identity. Comparative timing is forbidden
before that commit exists. After the freeze, a failed or incomplete attempt is
retained; the workload, schedule, repetitions, statistics, and claim rules are
not changed in response to a result.

Authority: separate owner-approved bounded research on 2026-08-05, registered
as `docs/ongoing/0003-literal-line-floor.md`. This experiment may inform
Direction Outline items `PERF-1`, `FLOOR-1`, and `FLOOR-2`. It authorizes no
numbered-specification, compiler, proof, intrinsic, runtime, system-capability,
storage, regex, threading, or end-to-end `wfgrep` change.

## Question and claim boundary

Can the active v0.17 language and ordinary compiler express one faithful,
in-memory, multi-byte literal line matcher at competitive same-algorithm
machine quality? If it can, how much further performance is available from the
pinned upstream `memchr` substring-search implementation on this exact input?

The four controls separate two questions:

- Whitefoot and C spell the same two-stage candidate-and-verify algorithm and
  use the same Apple Clang backend and C harness. `C-control elapsed /
  Whitefoot elapsed` is the primary compiler-floor attribution.
- Safe naive Rust spells the same algorithm as a cross-toolchain diagnostic.
- Rust using `memchr` 2.8.3 `memmem::Finder` replaces the scalar candidate
  finder in both discovery and matched-line enumeration. `memmem elapsed /
  naive Rust elapsed` is the algorithmic ceiling attribution within one Rust
  toolchain.

The fourth control is a standalone pinned upstream-library ceiling. It is
**not ripgrep**. Ripgrep 15.2.0 at commit
`e89fff89ac9af12e8d4ce9d5fd07beb408ca730f` reaches this dependency through
`regex-automata` 0.4.15's literal strategy, which directly owns a `memchr`
2.8.3 `Finder`, but this experiment omits ripgrep's regex strategy, file
searcher, line buffer, printer, CLI, I/O, and process behavior. No result may
be called grep, ripgrep, end-to-end, or evidence that the flagship 2x claim has
been met.

## Frozen matcher semantics

The Whitefoot kernel boundary is:

    scan(hay: &buffer<u8>, needle: &buffer<u8>) -> u64

Both arguments are opaque runtime buffers. The kernel allocates nothing,
performs no system operation, emits no output, and retains every required
check. No operand is specialized by corpus, needle bytes, or fixture identity.

Bytes are opaque. LF (`0x0a`) is the sole line delimiter; CR (`0x0d`) is
ordinary data. Byte offsets are zero-based and ends are exclusive. Line
numbers are one-based. A delimiter is excluded from the line it terminates,
and an input ending in LF has no phantom trailing line.

An empty needle or a needle containing LF has zero matches. Otherwise search
each line from left to right. Report the leftmost match, resume at its
exclusive end, and therefore report non-overlapping matches. A candidate may
not cross a line boundary.

Each logical match is the six-`u64` tuple, in this exact order:

    (ordinal, start, end, line_start, line_end, line_no)

`ordinal` is zero-based over the complete haystack. `start` and `end` are the
half-open match byte range. `line_start` and `line_end` are the half-open line
range and exclude LF. `line_no` is one-based.

The return value consumes every tuple and the complete work identity. Define,
with all arithmetic modulo `2^64`:

    mix(state, value) = (state XOR value) * 1,099,511,628,211

Start with `state = 14,695,981,039,346,656,037`. For every match in semantic
order, mix its six fields in the order above. Then mix the total match count,
haystack length, and needle length, in that order. The resulting state is the
kernel digest.

Each timed sample invokes the kernel exactly 16 times. Its consumed digest
starts at the same fixed state and, for each zero-based repetition ordinal,
mixes the ordinal and then that invocation's kernel digest. This aggregation
occurs inside the timed region and is checked after it. Sixteen repetitions
of the real input account for exactly 1,073,741,824 logical haystack bytes per
fresh-process sample.

## Frozen source algorithms

Whitefoot, C, and naive safe Rust use the same two-stage source shape. This
preserves the relevant pinned upstream structure: search a large remaining
slice for a confirmed match, locate its line, then enumerate that matched line
for record construction.

1. Reject the empty or LF-containing needle as zero matches under the digest
   rule above.
2. From `search_cursor` to the haystack end, advance to a byte equal to
   `needle[0]`. If enough haystack remains, compare every remaining needle byte
   in source order. On failure resume one byte after the candidate; on success
   return that confirmed match. Because the needle contains no LF, a confirmed
   match cannot cross a line delimiter.
3. Advance a monotone line cursor from the preceding searched-line boundary to
   the confirmed match, counting LF bytes to recover `line_start` and the
   one-based line number. Scan forward once to establish `line_end`.
4. Rescan exactly `[line_start, line_end)` with the same candidate finder,
   consuming every leftmost non-overlapping six-field match tuple. Resume at a
   match end after success and one byte after the candidate after failure.
5. Continue global discovery immediately after the matched line's LF, if
   present. Stop when discovery finds no remaining match. Unmatched lines
   produce no record and a trailing LF creates no phantom line.

Every Whitefoot index has a direct controlling length guard. The source may
not use slices, an output buffer, a `Finder` object, an intrinsic, an optimizer
fact, a proof, a pattern-specific helper, or an experiment-specific compiler
path. C and naive Rust may not call `memchr`, `memmem`, a string-search
intrinsic, or an equivalent library routine.

The `memmem` control keeps the same two stages, line-location work, match
semantics, tuple digest, and repetition aggregation. It constructs one
`memchr::memmem::Finder` from the opaque runtime needle before all warmups and
timing, uses it first on the complete remaining haystack for discovery, and
then reuses it to enumerate the matched line. This matters: restricting every
call to a short line would frequently select `memchr`'s short-haystack
Rabin--Karp fallback and would not exercise the audited large-slice packed-pair
path. Finder construction is not silently charged to the naive controls.

## Frozen inputs

### Timed real input

The sole timed haystack is the first 67,108,864 bytes of RG-BASE's complete
OpenSubtitles v2016 Russian `ru.txt`:

- full file size: 1,714,880,274 bytes;
- full file SHA-256:
  `08c2d7399372afe859238e25cb414e5fadbe5a416a8e69418787305b1e79296f`;
- prefix length: 67,108,864 bytes;
- prefix SHA-256:
  `ce55e37ed74f5b34773ce83597e5d61a83d0d0792d9cbb95fe0fc898ed09a1ee`.

The runtime needle is the 23-byte UTF-8 encoding of `Шерлок Холмс`, SHA-256
`192672866949818d8c8ea7089c9e622801bd763489f0314c004a459c616cc9b1`.
The independent oracle finds 74 non-overlapping matches, 1,324,231 LF bytes,
20,172,286 occurrences of the first needle byte (`0xd0`), and final match
start offset 63,538,171. These are frozen diagnostic invariants in addition to
the complete tuple-derived digest.

The runner verifies the full-file identity before extracting or accepting the
prefix. Every process loads and hashes the exact prefix and constructs the
runtime needle before warmup and timing. File open, allocation, input copy,
hashing, needle construction, and oracle work are outside the kernel timer.

### Adversarial correctness input

The deterministic adversarial haystack is exactly 8,388,608 bytes. Initialize
every byte to `a`, then replace the last byte of every 4,096-byte block
(offsets 4,095, 8,191, and so on) with LF. The needle is 31 `a` bytes followed
by one `b`. Plant its only two matches at start offsets 1,024 and 4,195,328 by
replacing each corresponding byte at `start + 31` with `b`.

This input forces a high first-byte candidate rate and long failed tail
verification while retaining two known, line-contained matches. It is used
only for correctness and code-shape validation. It is never timed and may not
be promoted after observing the real-input result.

## Correctness gate

Before any comparative timing, an independent host oracle scans every
line-contained start position, materializes the complete vector of six-field
tuples, compares that vector with explicit expected vectors on focused cases,
and only then folds it with the frozen mixer. It must not reuse candidate-first
search, the Whitefoot implementation, or `memmem`.

All four controls must equal the oracle digest on the real prefix, the
adversarial input, and focused cases covering:

- empty haystack, empty needle, and needle longer than a line;
- needle containing LF and a would-be match crossing LF;
- match at haystack, line, and final-byte boundaries;
- consecutive empty lines, final LF, and no final LF;
- CR, NUL, and non-ASCII bytes as ordinary data;
- multiple matches on one line and on different lines; and
- overlapping candidates that establish leftmost non-overlap behavior.

Focused cases compare the oracle's complete tuple vector to literal expected
tuples before any digest comparison. The real-prefix gate additionally checks
the frozen match, LF, first-byte-candidate, and final-start invariants. The
16-repetition aggregate digest is derived independently from the oracle digest
and checked on every timed process.

A crash, trap, panic, nonzero exit, stderr output, malformed record, input or
executable identity mismatch, tuple/digest mismatch, repetition mismatch, or
logical-byte mismatch invalidates the attempt. Correctness failure overrides
all timing.

## Compiler, library, and harness boundary

Whitefoot passes through the ordinary `whitefootc --emit-llvm` path. The
experiment may use the same narrowly verified transport rewrite as
WF-SCAN-FLOOR: expose exactly the target function's linkage and rename the
generated but unused host `main`, rejecting zero or multiple matches. It may
not change the target body, instruction, check, attribute, type, target triple,
or data layout. This is an experiment ABI boundary, not a compiler capability.

The primary Whitefoot/C controls share:

- one C harness and `CLOCK_MONOTONIC_RAW` timer;
- one opaque cross-object kernel call per repetition;
- Apple Clang 21.0.0 at `-O2`, without LTO or target-specific CPU flags; and
- the same allocated input bytes, runtime needle, warmups, aggregation, and
  correctness check.

The two Rust controls use rustc 1.97.1 with LLVM 22.1.6, `opt-level=2`,
`panic=abort`, one codegen unit, no LTO, and no target CPU flag. Their kernels
are behind a cross-crate optimization boundary and use `Instant`; the timer and
toolchain difference makes naive-Rust/Whitefoot secondary evidence. The
naive-Rust/memmem comparison shares both. The naive Rust implementation itself
uses no `unsafe`; the pinned `memchr` dependency is an upstream library whose
internals are outside that claim.

The offline dependency identity comes from the pinned ripgrep 15.2.0
`Cargo.lock`: `regex-automata` 0.4.15 has crates.io checksum
`1f388202e4b80542a0921078cc23b6333bcf1409c1e3f86404cae4766a6131db`,
and `memchr` 2.8.3 has checksum
`cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98`.
The ceiling crate pins exactly `memchr = 2.8.3` and builds from the frozen lock
with default features under `--offline`. Substituting another crate version,
lockfile, feature set, or locally modified implementation invalidates the
ceiling comparison.

The target is the RG-BASE Mac16,12 Apple M4 host: macOS 26.5.2 build 25F84,
16 GiB RAM. Accepted timing requires AC power and Low Power Mode off before and
after the block. This is target- and toolchain-specific evidence.

## Code-shape gate before timing

The frozen apparatus reproducibly emits raw Whitefoot LLVM, optimized
Whitefoot LLVM, and final assembly for all four controls under the excluded
work root. `CODE_SHAPE.md` records the inspection completed before timing and
is hash-bound into the run header. Review records, for each relevant loop:

- global-discovery, monotone line-location, line-end, matched-line candidate,
  tail-verification, and match-resume control;
- target loads, calls, allocations, copies, and runtime needle-length use;
- every raw and optimized Whitefoot trap/check edge;
- whether loop guards discharge redundant checks or a check remains;
- vector width and candidate-filter shape, if any;
- full-needle verification after each surviving candidate; and
- whether tuple and 16-repetition digest consumption remain live.

Whitefoot, C, and naive Rust sources must still implement the frozen direct
algorithm and may not explicitly call a search or comparison library.
Backend-introduced `memcmp`/`bcmp` or equivalent substitution is retained as a
lowering/LLVM-recovery finding; an asymmetric substitution must be reported and
attributed rather than silently called the same final algorithm. Corpus or
needle specialization, constant folding, changed semantic work, unexpected
allocation/copy, or setup inside the timer invalidates that control. The
pinned 23-byte `Finder` path is expected to show the audited 128-bit NEON
packed-pair candidate filter followed by full-match comparison. If that path
is absent, the run stops for review before timing; the name `memmem` alone is
not mechanism evidence.

A timing ratio receives no causal interpretation unless its preregistered
machine-code difference is present. Code-shape inspection may fail or stop the
experiment; it may not motivate a favorable replacement input or algorithm.

## Measurement schedule

The first and only run id is `wf-literal-line-floor-1`, stored in a create-once
directory under `/Users/bytedance/do_not_scan`. That directory must not exist
before the run and is never deleted or reused. A preflight refusal before a
run header does not consume it. Once a header exists, any failure preserves
the partial evidence and returns to owner review; there is no automatic retry
or second id.

Every sample is a fresh process. It loads and validates inputs, constructs any
`Finder`, and performs three correctness-checked internal warmups outside the
timer. It then times exactly 16 invocations and the fixed repetition digest.
Process startup, file I/O, allocation, hashing, oracle construction, and
warmups are excluded.

Name the variants `A = Whitefoot`, `B = C`, `C = naive Rust`, and
`D = memmem Rust`. One four-round Williams block uses these process orders:

    A B D C
    B C A D
    C D B A
    D A C B

Repeat that complete block exactly eight times, for 32 rounds and 32 samples
per variant. Thus every variant occupies every ordinal position eight times,
and the schedule balances first-order carryover. The block order is fixed in
the committed runner. Samples are never deleted, replaced, or extended after
observing a result.

## Statistics and classification

Compute these within-round elapsed ratios before aggregation:

    compiler_floor = C elapsed / Whitefoot elapsed
    cross_toolchain = naive Rust elapsed / Whitefoot elapsed
    algorithm_ceiling = memmem elapsed / naive Rust elapsed

Below 1.0 means the numerator is faster. The point statistic is the median of
the 32 within-round ratios.

A deterministic 10,000-resample bootstrap with seed `2026080503` resamples
the eight complete four-round Williams blocks with replacement. Every draw
therefore preserves one instance of each order class, all four within-round
pairings, and their shared machine-state correlation. For every statistic,
report the central 95% percentile interval. These are descriptive intervals
for this target and run, not general-population confidence intervals.

Also report per-variant elapsed medians by ordinal process position and, for
each of the four order classes, the ratio after omitting all eight rounds in
that class. These sensitivity reports do not delete samples or replace the
frozen point statistic.

For any ratio with point `m` and interval `[lo, hi]`, relative interval
half-width is `(hi - lo) / (2 * m)`. Above 5% is
precision-inconclusive regardless of a crossed threshold.

The primary compiler-floor classifications are:

- practical parity: the complete interval lies within `[0.95, 1.05]`;
- material Whitefoot loss: the complete interval lies below `0.90`;
- material Whitefoot win: the complete interval lies above `1.10`;
- directional only: the interval excludes 1.0 but clears no material band;
  and
- otherwise inconclusive.

The cross-toolchain ratio is diagnostic and cannot override the same-Clang C
comparison. A **material algorithmic-ceiling gap** is reported only when the
compiler-floor comparison is practical parity and the complete
`memmem / naive Rust` interval lies below `0.90`, with acceptable precision
and the expected `Finder` machine shape. Without compiler-floor parity, the
first material same-algorithm cause takes precedence. Any other memmem ratio is
reported descriptively; it is not promoted to a language or ripgrep claim.

## Attribution, stop, and decision routing

The experiment stops after one frozen real-input run is classified and the
first material difference is located among:

1. semantic or work mismatch;
2. Whitefoot source expressibility or forced source shape;
3. a required retained check;
4. emitted Whitefoot lowering;
5. LLVM recovery or missed recovery;
6. final instruction selection or register allocation;
7. the pinned library algorithm; or
8. toolchain, harness, order effect, or unresolved noise.

Practical same-algorithm parity validates only this two-stage, in-memory,
runtime-needle shape in the active compiler. A material C/Whitefoot loss is a
`FLOOR-1`/`FLOOR-2` or `PERF-1` finding according to the inspected cause. A
retained-check cause may motivate a later `PROOF-1` proposal only after its
exact proposition and consequence receive separate authority. A material
`memmem`/naive-Rust gap after primary parity is an algorithmic opportunity,
not evidence for a Whitefoot intrinsic, proof, compiler change, or language
feature.

No outcome authorizes widening into regex, Unicode character semantics,
filesystem traversal, I/O, formatting, output publication, threading, or a
`wfgrep` product. Do not add another workload, select the adversarial control
for timing, change repetitions, or tune an implementation after seeing the
result. Retain positive, negative, failed, or inconclusive evidence, update
only design-memory facts that pass their admission test, and stop.
