# Batch 0075 — optimization digs against the paired-layout oracle

Branch: `par/proof-derived-parallelism` (continues after batch 0074, which
remains closed and pending owner merge; this batch's commits stack after it
and enter the same merge review).
Authority: owner chartering direction, 2026-08-21, verbatim:

> 挺好的,这样我们就可以用这个测试来对照优化了。它展示了不少当前可以优化
> 的地方,一个个的挖吧

The referenced test is the paired browser-layout benchmark (18 rounds x 144
cells, bit-identical across languages; final tables of 2026-08-21). It
becomes the standing oracle: every dig measures against it before and after,
same protocol (interleaved rotation, min-of-N, sub-20% differences reported
unresolved, byte comparison on every run).

## Gap-hunt input (2026-08-21, reordered the queue)

The realistic-shape gap hunt (findings landed at
`research/investigations/proof-derived-parallelism/gap-hunt-findings.md` by
Dig 0) reported: correctness never broke across every probe, worker count,
and malformed setting — but found F1 CRITICAL BUG (`--par` shrinks max
recursion depth ~4x via an unconditional per-activation hand-out frame,
pool off included; death is a bare SIGSEGV), F2 CRITICAL perf GAP (no grain
control; fine folds to 48.6x slower; kernel-time dominated with named
runtime micro-causes), F3 HIGH (one builtin between the calls makes the
pair silently unjudged; the same op wrapped in a pure function keeps a
1.65x chain — 1.41x between byte-identical programs), F6 HIGH design limit
(dynamic-size allocation needs a buffer_fits claim, so realistic build
phases can never actualize), F4 MEDIUM (ledger reports pairs, never runs
nor backend narrowing), F5 MEDIUM (allocation lanes ~3x per-hand-out cost),
F7 LOW (path arguments reported as inputN.wf). The queue below is
re-sequenced accordingly.

## The dig queue (value-ordered; one at a time)

- **Dig 0 — the oracle and the findings become durable.** The paired
  benchmark (WF sources, Rust twin, harness, tables) moves from scratch
  space into `research/investigations/proof-derived-parallelism/bench/`
  with a one-line runner; the gap-hunt findings land beside it; measured
  baselines recorded. Evidence stops being mortal.
- **Dig 1 — F1: the hand-out frame must stop taxing every activation.**
  The `--par` build must match the sequential build's recursion-depth
  ceiling (within ~10% on the min_stack sweep) when no lane is granted;
  today it loses ~4x because the frame alloca and its spills land in every
  activation unconditionally. Acceptance: depth-sweep parity, oracle
  numbers not regressed, all existing tests green.
- **Dig 2 — the scheduler: close the fine-grain catastrophe.** Measured
  state: lane-budget tops out at 2.23x, optimum never past 4 lanes, and at
  16 words/node 8 lanes run up to 7x SLOWER than sequential, while rayon
  reaches 4.46x and even a dumb depth cutoff handles fine grain. Approach:
  **work stealing in `par_runtime.c`** — per-thread deques replace the lane
  scan and the hand-off, an idle thread takes the oldest (coarsest) entry,
  and the join reclaims its own offer instead of blocking on it.
  **Resequenced from heartbeat-class promotion by lead decision,
  2026-08-21:** rate-limiting kills the pathological cells but cannot lift
  the ceiling, because the 2.23x limit is measured at the cell where
  overhead is already amortised 4.6:1 and the skew shape is flat because the
  caller blocks on the half it handed away — neither of those is a rate. And
  a heartbeat that promotes the *oldest* pending fork point has to retain
  fork points, which is a deque, so the chartered dig followed honestly
  arrives here anyway. Success criteria against the oracle: no grid cell with
  workers>=2 slower than WF-sequential (today's 0.13x cells die), and
  absolute wall time at or better than rayon's at the same cell.
- **Dig 3 — the skew sequential gap (1.22x/1.41x).** WF-seq loses to
  Rust-seq only on the skewed deep tree, growing with per-node work; two
  recorded suspects: box_new allocation order/locality under skewed
  construction, and 16-level recursion. Profile both compilers' binaries on
  `skew_d16_w192`, attribute the loss (sample-based + counter-based),
  then fix the cheapest attributed cause. Sequential floor is W1: this digs
  for everyone, not just parallel.
- **Dig 4 — the outlining paradox.** `--par` outlining makes the skew shape
  22-28% FASTER single-threaded (resolved, both passes) while batch 0074
  measured a ~1.2x outlining tax on `par_layout`. Diff the emitted
  IR/assembly, identify the code-layout mechanism, and decide whether an
  independent sequential codegen improvement falls out. Understanding
  first; a fix only if it is cheap and general.
- **Dig 5 — checker: the band/derived-index discharge asymmetry.** A `band`
  claim proving two bounds discharges in straight-line code and fails
  against a derived loop index where two separate claims succeed (recorded
  in the round-3 debate with compiling probes). Fixing the fact-propagation
  gap widens the claim-free set — more eligibility, zero spec bytes.

## Approval classes

- Spec bytes: none planned (scheduler policy and codegen are implementation
  liberty under the CANDIDATE [PAR-1] rule, which states permission, not
  policy). If any dig turns out to need rule text, it stops and records the
  boundary.
- Protected conformance/compliance: no changes.
- Repository root: no new entries.

## Executor log

(One line per dig at completion; blockers recorded honestly with
reproduction, never worked around.)

- Dig 0 (done): the oracle and the gap hunt are durable. `bench/` holds the
  generator, the Rust twin's sources, the five harness scripts plus
  `timeit.zsh`, and `baseline/` (the 2026-08-21 N=18 snapshot, byte-identical
  to the scratch originals); `gap-hunt-findings.md` and nine probes in
  `probes/` land beside it (`bt.wf` promoted on lead review because it is the
  sole durable source for the depth-16 nested hand-out evidence — real 0.41
  user 1.18 sys 0.57; `p6.wf` and the rest verified by re-running the ledger
  here). Machine-local paths are gone — every script derives
  its own root, `build_wf.sh` defaults `WFC` to this worktree's release build
  and honours an override. Verified from the new location by the whole loop:
  24 binaries rebuilt, the Rust twin rebuilt, and a one-round rotation of
  144/144 cells all exit 0 with `compare_outputs.zsh` green in both languages
  and across them, all tables regenerated. 0.28 MB, no binaries; the stray
  `a.out` and a copied `rust/target/` were deleted and both are now ignored.
  Ledger verdicts for the seven compilable probes and byte-identical seq/`--par`
  output for both templates were reproduced here, not taken from the report.
  Two executors wrote this dig concurrently in the shared worktree (bench by
  one, findings and probes by the other); they deconflicted on ownership before
  the commit, and each half was checked against the scratch originals rather
  than trusted.
- Dig 1 (done, criterion partly met and the remainder attributed): F1 is
  fixed at its named cause and the residue is measured, not waved at. The
  hand-out now claims a lane *before* it builds anything, and the frame lives
  in the lane rather than in the calling function: the runtime contract
  becomes `wf__par_claim(bytes) -> frame|NULL`, `wf__par_publish(frame, fn)`,
  `wf__par_join(frame)`, `wf__par_release(frame)` — the release is split out
  of the join because the caller reads its result out of the frame after the
  wait, and a lane handed on at the join could refill it under that read. The
  frame stores and the publish sit inside the granted edge, so a refused
  activation executes a null test and its own ordinary call. The refused edge
  now calls the callee directly instead of running the thunk over a
  caller-owned frame; both edges and the thunk still call the same
  monomorphized function on the same operands, rendered once. A lane's frame
  is bounded (256 bytes); a call whose frame is larger is never granted a
  lane and runs sequentially, which is a schedule every program is already
  correct under. **Depth, bisected first-failing depth with `WF_WORKERS`
  unset, all re-measured here.** `min_stack.wf`: sequential 522 460 ok /
  533 984 fail, unchanged by this commit; `--par` 130 663 before, 173 876
  after — 25.0% of the sequential ceiling to 33.3%. `bt_skew.tmpl` (the
  realistic left-spine fold): sequential 128 750 ok / 131 015 fail; `--par`
  103 827 before, **128 750 after — the same bisection bracket as
  sequential, exact parity**, from a 19.4% loss. Frames (`otool -tV`):
  `wf_spine` 16 seq / 64 before / 48 after; `wf_build_skew` 48 seq / 80
  before / 64 after — the 32-byte alloca is gone in both. **Why `min_stack`
  cannot reach the ~10% criterion, and why no lowering can.** Its remaining
  32 bytes over sequential are one callee-saved GPR pair (the lane handle
  live across the inline member, the arguments live across the claim) and one
  callee-saved FP pair that the *sequential* build avoids only because LLVM's
  interprocedural constant propagation proves `v` is the same constant at
  every level — a fact any thunk destroys, being a second caller whose
  argument comes out of memory. Isolated by experiment: the same shape with
  that constant restored by hand inside the thunk compiles to a 16-byte
  frame. A prototype that outlines the whole overlap group into its own
  function removes the GPR pair and reaches 32 bytes — still 2x — so the
  criterion is unreachable on this probe by any hand-out lowering; that
  option was priced and rejected anyway (it duplicates the group's calls,
  adds a frame on the granted path, and needs a region outliner). The
  probe's sequential ceiling is unusually high for a pathological reason, and
  the realistic probe, which is the one the criterion is about, is at parity.
  **Verification.** `make -C compiler check` exit 0 before and after. Default
  lowering byte-identical: 36 modules emitted by the before and after
  compilers, 0 differ. All 36 `--par` modules verified valid by clang.
  `par_layout.wf` byte-identical to the sequential build at `WF_WORKERS`
  unset/1/2/4/8/64/65/0/`abc`, all exit 0; grants 0/0/801/2480/8358, so the
  anti-false-green counter still counts. Oracle: 24 binaries rebuilt, then
  `./rerun.zsh 1` — 144/144 cells exit 0, `compare_outputs.zsh` green in both
  languages and across them. **The one-round table cannot judge timing and
  was not used to**: WF-seq, whose code this commit does not touch, read
  40-55% above its own baseline min in that pass while Rust-seq landed on
  its min, and a min-of-7 head-to-head of the before/after binaries put
  `bal_d8_w16` seq at 0.5701 vs 0.5610 and `--par` w=1 at 0.5436 vs 0.5622,
  same sha. A second pass at N=5 (720 runs, all exit 0, byte comparison
  green) is the timing evidence: `--par` w=1 — the outlining-tax cell this
  commit touches — is within 4.7% of the N=18 baseline on every one of the
  twelve configs, and no cell collapsed. Machine: `mobileassetd` held ~97%
  of one core during an early depth sweep, which reads exit status only; the
  timed passes ran with nothing above ~14%. **Regression guard**: two cases.
  `handing_a_call_out_adds_no_stack_slot` compares alloca counts between the
  two lowerings of one source, so the mechanism cannot come back silently.
  `handing_calls_out_keeps_the_sequential_recursion_depth` runs both
  lowerings of a deep recursion under a 1024 KB stack this case sets, at
  depth 18 600 — measured to sit between the old lowering's ceiling under
  that limit (16 157) and the new one's (21 598), ~15% clear of each; the old
  lowering exits 139 there and the new one exits 0, so it is not a false
  green. Approval classes touched: none — no spec bytes, no conformance or
  compliance evidence, no new repository root entry.
- Dig 2 stage 1 (C1-C5, done, criteria met): the lane scan and the hand-off
  are gone. `par_runtime.c` is rewritten around a per-thread Chase-Lev deque:
  `wf__par_claim` takes a slot from the calling thread's own free list (no
  atomic, no shared line, no scan), `wf__par_publish` is a buffer store plus
  one release store of `bottom`, an idle thread steals the **oldest** entry —
  on a recursive descent the shallowest fork point, so the largest remaining
  subtree — and `wf__par_join` pops its own offer back and runs it as an
  ordinary call. The four entry-point signatures, the weak fallback text, and
  every emitted IR byte are unchanged; the whole stage-1 diff is that one C
  file. The refusal path is now one thread-local load and a branch, and the
  bound on outstanding offers is the slot count (a stated bound like
  `WF_PAR_FRAME_BYTES`, not a grain knob): a thread out of slots refuses its
  *deepest* fork points and keeps the shallow ones it already offered.
  **Numbers, full protocol rotation, N=9, 144 cells, 1296 runs, every run
  byte-identical within and across both languages.** No cell with workers>=2
  is slower than its own `--par` w=1 column — worst is `bal_d8_w16`/8 at
  1.16x, and the 0.13x cells are dead (`bal_d8_w16`/8: 4.3245 s -> 0.4905 s).
  Best cell `bal_d12_w192`/8: 0.3052 s -> 0.1272 s, 4.71x against rayon's
  4.54x at the same cell. Against rayon's absolute wall time, WF is faster
  outside the 0.83-1.20x band in 14 of 36 cells (`bal_d8_w16`/4 is 0.2233 s
  vs rayon's 0.4865 s, 2.18x) and inside the band — parity — in the other 22.
  **There is no cell where rayon is faster.** Per-fork excess at
  `bal_d12_w192`/4, ((measured - w1/4) / 2 866 500 forks): **48.8 ns -> 4.80
  ns**, against rayon's 5.80 ns measured in the same pass. Probes,
  `timeit.zsh` min-of-7, all byte-identical: `q4.wf` W=64 25.55 s -> 0.2530 s
  against W=1's 0.5026 s, so the 48.6x-slower cell is now 1.99x *faster*;
  `bt.wf` W=8 0.3992 s -> 0.0440 s and W=64 4.0218 s -> 0.0550 s (73x).
  **Dig 1's depth property survives**: `min_stack.wf` bisected under a 1024 KB
  stack, first failing depth 22 000 pool-off — unchanged — and 22 500 at
  W=4/W=8, i.e. the pool-on ceiling is at or *above* the pool-off ceiling,
  because the slot bound stops the reclaim path after the first 64 levels and
  handed-out work runs on 8 MB worker stacks. **The spin bound is measured,
  not chosen**: a park and its wake cost 2 097-2 514 ns here, so a thread that
  looks for work for less than that sleeps to save less than the sleep costs;
  the spin phase is set to a few multiples of that round trip.
  **NAMED RISK, disposition — the gate-integrity test is UNTOUCHED, but its
  guarantee changed character and that belongs in the merge packet.**
  `the_runtime_replaces_the_modules_weak_refusal` asserts `granted > 0`, and
  `wf__par_grants` still counts exactly one thing: a frame executed by a
  thread other than the one that offered it. Counting pushes instead would
  have been the precise false green the counter exists to catch, and was
  refused. But under a hand-off every publish incremented it, so `granted > 0`
  held *structurally*; under a deque a steal must actually happen, so it now
  holds by *timing*. Measured: 0 failures in 1000 direct runs and 0 in 40
  runs of the case under `cargo test`. The sensitivity is real and is
  reported rather than hidden — at an under-set spin bound of 128 rounds the
  same case failed 24% of the time (48/200), which is how the bound came to be
  measured rather than guessed. No byte of the test or its fixture changed.
  **Verification.** `make -C compiler check` exit 0 before and after; all 13
  `backend::tests::parallel` cases and all 4 `programs::parallel` cases green.
  Approval classes touched: no spec bytes, no conformance or compliance
  evidence edited, no new repository root entry.
- Dig 2 stage 2 (C6, two-version sequential-clone compilation: **built,
  measured, and NOT landed**). The chartered stage 2 was to emit a sequential
  clone of every function in an eligible pair's call closure and switch a whole
  subtree onto it when the runtime signals no demand, so an un-promoted fork
  point costs zero rather than cheap. It is not the next work, and the reason
  is measured rather than argued.
  **(a) The premise it rests on is not true here.** C6's argument is that the
  un-promoted path carries an overhead the sequential program does not. After
  stage 1 it does not: `wf_par/1` — the same binary with the pool off — is
  *faster* than `wf_seq` on 9 of the 12 configs (0.68x to 0.99x; 1.02x worst).
  There is no tax for the clone to remove; the outlining that would be undone
  is the same effect Dig 4 is queued to explain.
  **(b) The win is bounded below the instrument.** The whole excess over
  ideal-linear at w4 is 1.05x-1.09x at every `w192` cell and 1.07x-1.21x at
  `w64` — and that excess still contains load imbalance and memory effects, so
  the fork-cost share is smaller again. A *zero*-cost fork point could not
  produce a resolvable gain at the coarse cells, whose ratios all sit inside
  the 0.83-1.20x unresolved band. The cells with real headroom (`w16`, 1.14x
  to 1.56x) are the ones stage 1 already wins by 1.22x-2.29x, so C6 cannot
  change a verdict either way. This bound does not depend on how the demand
  signal is built, which is why it, and not (c), is the load-bearing reason.
  **(c) The signal it needs costs more than it can win — measured, and
  attributed by discriminator.** "No demand" needs to know whether any thread
  is looking for work, which is a word every thread writes as it runs out of
  work and finds more. Built as a shared seeking bitmask, `bal_d8_w16`/8 went
  0.4905 s -> 0.9319 s. Disabling only the refusal and keeping the bookkeeping
  still measured 0.9254 s, so the cost is the two contended read-modify-writes
  per task on one shared line — not the refusal, which was free to within
  noise. That is exactly the "no global RMW on the hot path" rule stage 1
  keeps, broken by the signal C6 requires. Reverted whole; `par_runtime.c` is
  the committed stage-1 file and `emitter/parallel.rs` was never touched, so
  no clone symbol, no fifth runtime entry point, and no `cost_shape` census
  change was introduced.
- **The residual at the parity cells is not the scheduler, and here is what it
  is.** After stage 1 there is no cell where rayon is faster, but 22 of 36 sit
  inside the unresolved band. Two mechanisms, both outside this dig: the
  **sequential floor**, which is Dig 3's territory — WF-seq is 1.18x to 1.47x
  slower than Rust-seq on every `w64`/`w192` and skew config, and the cells
  with the thinnest parallel margin are exactly those (skew_d16_w192: seq
  1.44x, parallel margin 1.08x; skew_d16_w64: 1.47x and 1.26x) while the cells
  where WF-seq is at parity are the ones stage 1 wins outright (bal_d8_w16:
  seq 0.99x, parallel margin 2.29x) — and the **machine ceiling** at the
  coarsest cells, where 8 threads on 4 performance plus 6 efficiency cores put
  both implementations near the same limit (bal_d12_w192/8: WF 4.71x, rayon
  4.54x). The scheduler is already making up a code-generation handicap of up
  to 47%; the next parallel gain is a sequential one.
- **Dig 0 deviation, recorded not hidden.** Dig 0 was specified as one
  cohesive commit and initially landed as two with byte-identical subject
  lines: two sessions were writing through one worktree and one shared git
  index, and one committed the full 37-file set while the other still had a
  one-word correction staged, which landed seconds later carrying a message
  describing the whole dig. On lead direction, with nothing stacked on top and
  the branch unpushed, the pair was squashed — together with the follow-up
  fixes (the ninth probe `bt.wf`, the probe-count corrections, this record) —
  back into the single cohesive commit the dig specified. The general cause —
  two writers with access to one worktree and one unlocked index — is the
  reusable lesson: the batch rule of one live worktree, one writer exists
  precisely to prevent this, and it was restored (single writer confirmed by
  the lead) before the squash.

## Outcome

(Filled at closure: per-dig before/after oracle numbers, landed commits,
verification, audit dispositions.)
