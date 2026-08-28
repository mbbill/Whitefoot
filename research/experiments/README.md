# Experiments Index

Current experiment bundles are self-contained: sources, a Makefile or run
script, and a RESULTS.md with measured numbers and honest caveats.
Binaries/corpora are regenerable and gitignored. Some retained historical
bundles below still name the retired democ toolchain; their RESULTS and source
evidence remain useful, but their old runner is not replayable from HEAD and is
not a current compiler gate. Historical chronology and decisions are indexed
by `../../archive/governance/decision-log.md`; current design decisions live in
`../../mcts_mem/`, current direction status in `../../docs/roadmap.md`, current
execution proposal and status in `../../docs/current-plan.md`, and
implementation detail in `../../compiler/README.md`. The current plan records
high-level sequencing; plans do not grant or withhold branch permission.

## Current flagship experiment evidence

- `wfgrep-baseline/` — the PERF-1 zero-change baseline of the frozen
  sequential wfgrep against the pinned system `grep -h -F`, preregistered
  with null-comparison precision gates per the RG-BASE lesson. Measured:
  0.647/0.656 (large/no-match scan), 0.605 (many small files), 1.105
  (match-dense win), smaller process floor. Attributed: the dominant
  many-files loss to the host's per-open cost for unsigned binaries (C
  control; not a Whitefoot layer); the compute loss to the scalar
  double-walk shape, with the literal matcher above the newline scan and
  the retained per-byte traps a bounded ~18%-ceiling secondary term.
- `ripgrep/` — RG-BASE preregistration for the owner-selected 2x ripgrep
  flagship. It freezes the Apple M4 target, pinned official/native ripgrep
  comparators, two real source trees, one large-text corpus, nine equal-weight
  end-to-end cases, correctness oracles, statistics, and the future 2x rule
  before comparative timing.

## Completed current-compiler bounded research

- `differential-fuzz/` — the mechanical source of programs nobody wrote, for the
  one property [PAR-1], [PAR-2], and [PAR-3] all state: under a permitted
  overlap the observables equal the source-order ones, and whether an overlap
  happened is not observable. A seeded generator writes accepted command
  programs that do real I/O and control flow from the [GRAM-4]/[GRAM-5] fence
  under a typing and ownership environment; the oracle compiles each three ways,
  establishes that the program agrees with itself, and then requires the
  overlapping builds to publish the same stdout, stderr, and exit status across
  `WF_WORKERS` x `WF_IO_HELPERS`, some of them with stdout on a FIFO whose reader
  is delayed. First campaign, 2026-08-28: 2004 accepted programs, 78 156
  executions, 1255 permitted [PAR-1] pairs, 678 permitted [PAR-2] loops, 857
  permitted [PAR-3] stages, zero divergences, zero unstable programs. The two
  findings were a harness defect (argument zero reaching a program's digest,
  fixed) and a spec-conformant [CLM-1] rejection recorded for the owner. Not a
  gate and not reachable from `make check`; report and reasoning in
  [`docs/done/0097-differential-fuzz.md`](../../docs/done/0097-differential-fuzz.md).
- `blind-writer/` — the standing corpus of what unguided writers write, one
  dated directory per trial. The 2026-08-28 trial handed a senior systems
  programmer with no prior Whitefoot exposure the spec, `docs/patterns.md`, the
  gate binary and `tests/programs/`, and asked for five ordinary I/O utilities.
  All five compile and are correct against their Unix references with zero
  `claim` statements; all five, and every worked I/O example in the repository,
  compile to code byte-identical to `--no-overlap`, against a hand-widened
  comparator 1.78x faster on this host and 2.17x/2.90x faster on the committed
  quiet-host medians. Fourteen defects with dispositions in
  [`docs/done/0098-blind-writer.md`](../../docs/done/0098-blind-writer.md). It
  is removed when the language stops changing.
- `io-completion-bench/` — the program-level answer to whether the unified-state
  completion I/O model reaches native performance on whole programs, which
  until 2026-08-27 had only C-level component evidence. Three lines per
  workload, all publishing the same checked bytes: the best hand-written
  native shape, the Whitefoot program built `--no-overlap`, and the same
  source built the way it ships. On a many-independent-files workload the
  shipped build is 2.05x its own sequential build on macOS and 2.41x on Linux,
  and lands within 3.4 percent of a hand-written io_uring pipeline running at
  the same queue depth the source can ask for. The distance to a deeper native
  shape is source width, not protocol cost: overlap groups are runs of
  consecutive calls in one basic block, so the natural one-file-per-iteration
  loop overlaps nothing. Table in
  [`io-model/RESULTS.md`](../investigations/io-model/RESULTS.md).
- `buffer-initialization-cost/` — the dossier §9.1 initialization-cost row,
  whose control §9.1 requires to be an *uninitialized* native read loop. A
  Whitefoot drain over a language-initialized reused buffer measures at
  practical parity with the uninitialized C control (1.0014 [0.9982, 1.0083]),
  and the same-source `calloc`/`malloc` ablation is likewise parity. The
  decisive figure is direct: initializing one 4096-byte page costs 28.76 ns,
  which is 612x below 1% of this program's 1.76 ms empty-input process floor,
  so no input size makes it material. Dossier §11's stop condition did not
  fire.
- `literal-line-floor/` — the active v0.17 language expresses an exact
  runtime-needle literal line matcher, but its helper-shaped scalar lowering is
  directionally about 5% behind same-Clang C: C/Whitefoot is 0.9535
  [0.9223, 0.9609], below the preregistered material-loss threshold. Pinned
  Rust `memmem` is descriptively 7.33x the same-toolchain scalar control with
  the expected NEON packed-pair mechanism, but strict primary parity was not
  met, so this is not promoted to a language, ripgrep, end-to-end, or 2x claim.
- `wfgrep-scan-floor/` — the active v0.17 language and ordinary compiler match
  same-Clang C on two safe single-buffer scanner shapes. The width-16
  Boolean-dataflow full pass measures 0.9993x C [0.9969, 1.0023], and four
  scalar early exits measure 1.0008x C [0.9981, 1.0088]. LLVM removes the
  guard-dominated bounds traps in both. This validates a narrow language floor,
  not an algorithmic, end-to-end, or 2x-ripgrep claim.

## Historical fact-channel benchmarks

These results used the now-archived democ implementation. They remain measured
evidence rather than current-compiler benchmark requirements. Historical
benchmark runners are not active compiler gates; maintained unit tests that
still execute against HEAD are included by the root `make check` target.

- `effect-attrs-channel/` — channel 2: effect rows -> LLVM fn attributes.
  O(n)->O(1) at opaque boundaries; ties fat LTO at per-file build cost.
- `scoped-alias-channel/` — channel 1: ownership provenance -> alias.scope.
  Short-trip wins, 17x code size vs Rust's guard-versioned loops; parity at
  long trips (Rust recovers via runtime checks).
- `checked-law-channel/` — channel 3: FN-4 discharged laws license
  reassociation. 3.3x over the obvious fold; refutes false laws compile-time.
- `frequency-study/` — completed one-time directional scan of popular Rust
  sources/applications; points the next real port at relational bounds proofs.

## Paused expressiveness evidence

- `data-layout-owning-sequence/` — historical E0.1 research and rejected
  isolated-prototype evidence. D11 leaves all work paused before bounded G0-Core
  and a later exact dense-family Lock A; neither is currently selected. The
  evidence separates fixed SoA/AoS
  layout from initialized-prefix ownership and growth, protects the compiler's
  current SoA as the zero-tax baseline, and forbids feature-flagged dual
  semantics.

## Port studies (real programs; historical D9-era evidence)
- `zlib-core-kernels/` — deferred RFC 1951 kernel handoff. Ordinary scalar
  lowering is not competitive for short-period match overlap and trails the
  pinned all-literal Huffman projection; bounds-check elision alone does not
  close either gap. Two unchanged-source stage-0 prototypes recover isolated
  performance through periodic expansion and a guarded six-symbol bit window.
  The directory preserves corrected raw results, compiler patches, LLVM and
  ARM64 snapshots, candidate writer patterns, proof obligations, and production
  pickup gates. These are feasibility results, not complete proofs or a
  whole-inflate claim.
- `default-floor/` — historical D9a protocol: a fixed low-tier model's first
  correctness-green Whitefoot artifact versus an exact unmodified shipped Rust
  library. Two separately preregistered results are complete: Terra Whitefoot beats
  `percent-encoding` 2.3.2 `percent_decode` by 1.653x [1.631, 1.667] and
  one-shot `utf8parse` 0.2.2 by 1.098x [1.085, 1.145]. Neither result is a
  proof-elision win, and current W1 does not use a model score as a gate; see
  the aggregate claim boundary in
  `default-floor/RESULTS.md` and the two target-specific reports beneath it.
- `port-study/binary-trees/` — floor-raising result: the slow shape is
  unrepresentable; ~11% checked-semantics tax vs identical-shape Rust.
- `port-study/wc/` — full-counts 0.27s vs GNU 0.48 / uutils-Rust 0.56 on a
  426MB corpus (regenerate: see RESULTS); -l honest gap vs memchr/bytecount.
- `port-study/wc-chunk-summary/` — ordered-monoid parallel wc. NEGATIVE
  result for channel attribution (Rust expresses the same algebra); reached
  C/Rust parity after the OWN-1 Bool-copy amendment (220->134ms).
- `port-study/base64/` — first const-array consumer; 1.6x GNU/uutils,
  ~parity BSD (table-width algorithm gap); PROOF-1 discharges 15/27 bounds
  sites and improves the kernel 2.50 -> 2.93 GB/s, with PROOF-2 debt isolated.

## Preserved code-generation fixtures

- `codegen-vs-rust-c/` — the splitmix scalar-backend-parity evidence. Its old
  democ runner is dormant and has no current compiler adapter. Making it a
  maintained test would require a Rust harness that binds the fixture to the
  current compiler.

## Earlier corpus-era studies
Moved to `../../archive/experiments/` (scatter residual, guarded-plan
measurements). Their durable conclusions and current dispositions are
summarized in `../archive-promotion-audit.md`; the old protocols grant no
current Direction Outline or Current Plan authority.
