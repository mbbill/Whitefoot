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
- **Dig 6 — the fine-grain and 8-worker residual.** Added by lead direction
  after Dig 3 scored the grid against a measured ceiling: 14 cells sat below
  92% of it, concentrated at 8 workers and the fine `w16` grain. Investigate
  the steal path, the sleep/wake bounds, Darwin core placement, and the deque
  bound by measurement; convert what is winnable and attribute the rest with a
  named mechanism. Carries one stretch item, the `fib(38)` opt-in tax.
- **Dig 7 — the `--par` opt-in tax, at the boundary Dig 6 convicted.** Added by
  lead direction after Dig 6 attributed `fib(38)`'s 2.96x pool-off tax to the
  emitter: `emitter/parallel.rs` rejoins the granted and refused edges through a
  phi at `%par.done`, so the callee's result flows into a phi rather than into
  the caller's return, which takes the second recursion out of tail position and
  forecloses LLVM's accumulator tail-recursion elimination. `-flto` recovers
  only 5%, so it is a foreclosed compile-time transform and not a runtime cost.
  Emit the sequential lowering as a second world and select between the two
  **once per process**, from whether a pool was asked for — never per task,
  which is the shape Dig 2 measured killing C6 (0.4905 -> 0.9254 s from two
  contended read-modify-writes per task). Acceptance: the tax within 1.05x of
  the sequential build, the pool-on grid not regressed against Dig 6's N=9
  rotation, byte identity at every worker count, and the code size the second
  world costs measured and recorded.

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
- Dig 3 + Dig 4 (done; **no codegen fix landed, and the reason is measured**).
  The dig was chartered to lift the skew sequential floor. The floor is real
  and reproduces, but it is not a Whitefoot codegen defect and no static
  compiler change can remove it. Everything below is measured at `826cea41`
  with one pinned compiler, entirely outside the worktree.
  **The gap survives the instrument.** The recorded rotation confounds
  implementation with slot position (`run_bench.zsh:39-52` is a fixed
  rotation), so the first move was an isolated sequential-only rotation with
  no 8-thread neighbours, N=15: skew 1.118x / 1.206x / 1.267x at w16/w64/w192
  and every balanced control flat at 0.981x. Real, and shape-specific.
  **It is a pure stall, not instruction selection.** `/usr/bin/time -l`,
  min-of-3: WF retires **fewer** instructions than Rust in all six matched
  cells (0.980x-0.997x), while IPC is at parity on balanced (4.003/4.061,
  4.261/4.273, 3.401/3.401) and collapses on skew only (3.455/3.981,
  3.445/4.234, 2.628/3.399). Statically, all twelve WF binaries contain the
  **identical 136-instruction `_wf_layout`** (`cascade` and `measure_words`
  inline into it), so per-binary codegen instability is dead; `fmin`/`fmax`
  are single instructions; `-O2` already narrows the whole-node load to the
  fields each arm uses; and the `set deref(slot)` write-back is **dead-store
  eliminated entirely**, so Whitefoot does strictly less memory work than the
  Rust twin and still stalls.
  **The cause is traversal order, and it is data-dependent.** Swapping the two
  child calls in the source (bit-identical output, ledger still
  `pair(layout, layout) eligible`) moves skew_d16_w192 from 0.7381 s to
  0.5739 s against Rust's 0.5856 s — a 1.23x loss becomes a 1.02x win. A
  depth/shape sweep at w192 separates the variables: balanced shows **zero**
  order effect at d8/d10/d12/d14 (511 to 32767 nodes, 40 KiB to 2.6 MB) while
  skew shows a constant **1.28-1.29x** penalty at d12/d14/d16/d18/d20. Skew
  d12 is 45 KiB — the whole tree is L1D-resident — and still pays in full,
  which kills every capacity and locality account; balanced d14 at 2.6 MB pays
  nothing. **Mirroring the tree flips the sign exactly**: with the shallow
  child on the left, left-first becomes the fast order (0.5699 s) and
  right-first the slow one (0.7258 s). So the rule is "visit the smaller
  subtree last", which is a property of the heap at run time, not of the
  program text. A static sibling reordering would win on this benchmark and
  lose by the same margin on its mirror image; that is keying codegen to one
  corpus's source shape, and it was refused on that ground rather than landed
  for the number.
  **What is not the cause, each falsified by direct measurement.** Node size:
  the Rust twin padded to 64 B and to WF's exact 72 B (80-byte malloc class,
  peak footprint 1632 KiB against WF's 1600 KiB) is unchanged at 0.5789 s
  against its own 0.5802 s. Allocator: `MallocNanoZone=0` leaves the ratio at
  1.30 against 1.27. Recursion depth: constant across d12-d20. Code form or
  alignment: the mirror test gives opposite results from the *same* binary
  form. Rotation slot, `llvm.minimum` vs `minnum`, whole-node load, whole-node
  store: all above. Two further Rust replicas were built to reproduce the
  pathology and did not: one materialising both child pointers before the
  measure loop (WF's `ldp`), one with `#[repr(C)]` reproducing WF's exact
  offsets (tag 0, children 32/40, w/h 48/56, 72 B, 136 instructions). A third
  combining both did reproduce 0.7609 s — but it is **1.25x slower on the
  balanced shape too** (0.7471 s), a uniform regression from the raw-pointer
  probe losing aliasing information, so it discriminates nothing and is not
  claimed as a reproduction. **The microarchitectural root is therefore named
  only to this precision: an order-dependent stall inside the per-node loop,
  costing 0 instructions, absent when the per-node body is small (w1 0.84x,
  w4 0.92x, rising to w192 1.26x), absent on symmetric trees, and not
  reproducible in a structural replica.** It is honest to report it unlocated
  rather than to name a mechanism the probes did not support.
  **The campaign premise this dig was dispatched under is false, and that is
  the load-bearing result.** The brief assumed `T_P` tracks `T_seq/P`, so
  lifting the skew floor would convert skew parity cells into wins. The
  `--par` build **does not pay the floor at all**: `wf_par` at one worker
  against `wf_seq` is 1.005x-1.016x on balanced but **0.912x / 0.832x /
  0.733x** on skew w16/w64/w192. The sequential pathology is a property of the
  default compilation only, so the parallel cells never carried it and lifting
  it would buy them nothing. Sequential users would gain; the parallel grid
  would not move.
  **Parity-cell attribution, against a measured ceiling rather than an assumed
  one.** Full protocol rotation on the pinned build, N=9, 144 cells, 1296 runs,
  **all exit 0 and every run byte-identical within and across both languages**:
  6 cells WF wins outright, 30 parity, **0 where rayon wins**. The machine
  ceiling was measured directly by running N independent copies of the same
  sequential binary — zero scheduler, pure hardware throughput: **1.90-1.97x
  at 2, 3.69-3.89x at 4, and only 4.74-5.71x at 8**, because 8 threads are 4
  performance plus 4 efficiency cores. Against that, **16 of the 30 parity
  cells sit at or above the ceiling (>=92%) and are unwinnable by any
  scheduler**; 14 retain residual overhead, concentrated at 8 workers and at
  the fine `w16` grain (bal_d8_w16/4 at 69%, bal_d10_w16/8 at 50%,
  bal_d12_w16/8 at 73%). Every skew parallel cell is at 98%-126% of ceiling —
  above 100% because its baseline `wf_seq` is the pathological traversal, which
  is exactly the same finding read from the other side. Measured from the
  `--par` build's own one-worker baseline, skew_d16_w192 at 8 workers scales
  4.34x against a 5.40x ceiling, i.e. 80% — indistinguishable from rayon's 79%
  at the same cell. **There is no scheduler gap left on the coarse cells; the
  remaining honest targets are the fine-grain and 8-worker cells.**
  **Dig 4, the outlining paradox: dissolved, both signs.** The "tax" half was
  F1. `par_layout.wf` recorded 742.6 ms default against 902.5 ms `--par`
  (`RESULTS.md:134-155`, 1.2x); re-measured here at `826cea41`, N=11:
  **0.7486 s against 0.7481 s — 1.00x, no tax**, because Dig 1 removed the
  unconditional per-activation hand-out frame. The "win" half was never a
  property of outlining: `--par` at one worker is neutral on all balanced
  shapes (1.005x-1.016x) and only wins where the sequential build is
  pathological (0.733x on skew w192), by exactly the size of the traversal
  penalty. The E5 cross rules out sibling-order reversal as the whole story —
  `--par` is fast in **both** source orders (0.5813 s and 0.5777 s) while the
  sequential build is fast in only one — so the `--par` lowering removes the
  penalty by some property beyond the reordering; that property is unlocated
  for the same reason as above. `fib(38)` was rebuilt and re-measured because
  the doc comment cited it: the 12.6x-slower four-worker row is now **1.33x**
  and eight workers is **0.91x, faster than the default build**, while the
  opt-in tax on that grain survives at 2.6x.
  **F5 re-check (lead's addition): dissolved by Dig 2.** The probe behind it
  (`p4.wf`) was never promoted to `probes/` and is gone, so it was
  reconstructed faithfully — build moved inside the reps loop, one builtin
  between `layout`'s child calls so the fold stops forking (F3), ledger
  confirming **only** `pair(build, build)` eligible, the same 16.4M hand-outs
  at depth 12. Recorded: 1.50x slower at 4 workers, 2.62x at 8, ~58 ns per
  hand-out. Now, N=9, all cells one sha: seq 0.6923 s, W1 0.7341 s, W2
  0.6766 s, **W4 0.6335 s (0.92x), W8 0.6533 s (0.94x)** — no cell slower than
  sequential and the per-hand-out excess now negative. Recorded as dissolved.
  **What landed.** No codegen change: the default build is byte-identical
  because the compiler's lowering was not touched. Two corrections to derived
  material this dig falsified — the `--par` doc comment in
  `compiler/src/bin/whitefootc.rs`, which asserted a 1.2x fold cost that is
  now 1.00x, and two superseding notes in `RESULTS.md` beside the tables whose
  numbers Digs 1 and 2 had already invalidated. **Deviation, recorded:** the
  brief specified one cohesive commit per landed fix, and this dig lands no
  fix, so it commits documentation only — the exception the batch rules allow,
  taken because the dig's entire deliverable is attribution.
  **Verification.** `make -C compiler check` exit 0 before and after. Rotation
  1296/1296 exit 0 with `compare_outputs.zsh` green in both languages and
  across them. Every source variant built for the dig (order swaps, mirror,
  shape sweep, word sweep, F5 probe, `fib`) published bytes identical to its
  baseline, and the Rust padding, spread-field and early-materialisation
  probes all published `420229e929506cdd` unchanged. Approval classes touched:
  no spec bytes, no conformance or compliance evidence, no new repository root
  entry.
  **BLOCKER FOUND, not introduced here, and it blocks the merge.** The
  compiler gate is green but the repository gate is not: `make check` exits 2
  at the conformance step because coverage is 135/136 and the uncovered rule
  is **`PAR-1`** — the CANDIDATE rule this whole branch exists to implement.
  Confirmed pre-existing by running the same check in a detached worktree at
  the parent commit `826cea41`, which reports the identical 135/136 and the
  same uncovered rule, so no commit in this dig caused it. It is reported
  rather than worked around: closing it means adding conformance coverage for
  `PAR-1`, which is protected evidence and needs an exact before/after audit
  and owner approval at merge, so it is a lead decision and not an executor's
  to take. No branch should request merge presenting a green gate until this
  is either covered or explicitly dispositioned in the packet.
- Dig 6 (done; **no runtime change landed, and the reason is that the residual
  was scored against a yardstick that excludes the one cost this workload is
  made of**). Chartered to convert the 14 residual parity cells — concentrated
  at 8 workers and the fine `w16` grain — into wins, or to attribute them.
  Everything below is measured at `5b933c3a`. Every binary was produced by
  relinking the compiler's own emitted IR against a runtime source with the
  exact line `whitefootc` uses (`whitefootc.rs:88-107` with `driver.rs:26`),
  and the rig was checked by rebuilding all 24 oracle binaries that way and
  comparing them against `bench/bin`: **24 of 24 byte-identical**, so a
  variant differs from HEAD only in the runtime source.
  **The residual reproduces, and its shape is a turnover at the fifth worker,
  not diffuse overhead.** Ceilings re-derived by Dig 3's N-copies method
  (min-of-5, `WF_WORKERS` unset): 1.97-2.01x at 2, 3.85-3.90x at 4, 4.90-5.32x
  at 8. Against them the fine cells sit where Dig 3 left them
  (`bal_d8_w16`/8 23.2%, `bal_d10_w16`/8 47.9%, `bal_d12_w16`/8 70.2%,
  `bal_d8_w16`/4 62.6%), and minima repeat to 0.2% across three independent
  passes. But on those cells **8 workers are slower than 4** (`bal_d8_w16`
  0.2220 -> 0.4651 s, `bal_d10_w16` 0.1761 -> 0.2383 s, `skew_d16_w16` 0.1824
  -> 0.2183 s), and a worker sweep puts the knee exactly at five:
  `bal_d8_w16` reads 0.2218 s at 4 and 0.3456 s at 5, degrading monotonically
  to 8, while coarse `bal_d12_w192` improves all the way (0.1627 -> 0.1220 s).
  This machine is **4 performance and 6 efficiency cores**
  (`hw.perflevel0/1.logicalcpu`, `PROTOCOL.md:5`), so the fifth worker is the
  first that must live on an efficiency core. *(The Dig 3 entry and its commit
  message say "four performance plus four efficiency cores"; it is 4P+6E. The
  ceilings are unaffected — they were measured, not derived from the count.)*
  **The turnover is not Whitefoot's.** At the same cells: rayon goes 0.3578 ->
  0.9370 s from 4 to 8 workers, and its recorded N=9 `bal_d8_w16` t=8 is
  1.125 s, **0.51x of its own sequential build**; `rayoncut`, which forks only
  above depth 5 and so barely schedules, goes 0.2485 -> 0.6132 s. The recorded
  rotation's published bytes match these binaries exactly, so it is evidence
  about the same programs.
  **What the oracle asks the machine for, and why the ceiling is the wrong
  yardstick.** The tree is built once and `layout` is folded `reps` times, so
  each repetition is one fork-join episode of 5.7 us (`bal_d8_w16`) to 854 us
  (`bal_d12_w192`), each ending in a join. Holding the tree and the fork count
  fixed and varying only per-node work — the same 255-node tree at w16, w64,
  w192 — moves the 8-worker score 23.2% -> 45.6% -> 70.7%: **a fixed cost per
  episode, not a cost per fork.** Dig 3's N-copies figure is a fair
  *throughput* bound — timing all eight copies separately shows them finishing
  together, 0.875-0.946 s each, so the cores are shared evenly and there is no
  straggler artifact — but it is measured with **no synchronisation at all**,
  and this workload synchronises tens of thousands of times. Measured directly
  with a scheduler-free probe (P threads, a spinning sense-reversing barrier,
  no deque, no stealing, one unit of work): **the barrier alone costs 0.340 us
  at 4 threads and 0.839 us at 5 — a 2.5x jump — reaching 1.399 us at 8.**
  On a 5.7 us episode that is a quarter of the episode spent meeting, before
  any work is distributed.
  **The achievable reference, and Whitefoot against it.** The honest target is
  what the best available scheduler reaches on the same episode structure: a
  flat parallel loop, dynamic self-scheduling off a shared counter with a chunk
  sized so contention is negligible, no tree to discover. Measured at the
  oracle's own episode lengths, speedup against its own one-thread time:
  6 us — 3.06x at 4, 2.24x at 5, **1.61x at 8**; 23 us — 3.44 / 3.43 / 3.69;
  90 us — 3.51 / 3.86 / 4.71; 850 us — 3.54 / 3.99 / 5.12. **The
  fifth-worker collapse is fully present with no scheduler and disappears as
  episodes lengthen, which is exactly Whitefoot's pattern.** Scored against
  that reference instead of the throughput ceiling, the residual cells read
  `bal_d8_w16`/8 **72%** (1.16x of 1.61x), `bal_d8_w16`/4 79%,
  `bal_d10_w16`/8 **65%**, `bal_d12_w16`/8 **74%**, `bal_d12_w192`/8 **95%** —
  a uniform 65-95% rather than a collapse to 23%. The remaining gap is
  distribution: the reference is handed its work up front, while a tree fold
  must discover the work by recursing and move it by stealing. Among
  schedulers that must do that, Whitefoot is the fastest measured at every one
  of these cells.
  **Every candidate the brief named, each rejected by measurement.**
  *Sleep/wake latency and spin bounds:* **falsified.** Counters compiled into
  the runtime report **10 parks in a whole `bal_d8_w16`/8 run**, 0 join-waits,
  and `/usr/bin/time -l` reports **0.00 s system time** with no voluntary
  context switches at any cell. Nothing sleeps.
  *Deque CAP at fine grain:* **falsified.** `refuse` is **0** in every measured
  cell; outstanding offers are bounded by recursion depth and the deepest tree
  here is 16, against a 64-slot bound.
  *Steal-path volume:* correlates, does **not** cause. The idle scan is
  enormous — 219.6M victim probes for 2.09M steals at `bal_d8_w16`/8, 98.0%
  finding an empty deque — and probe *rate* orders the cells perfectly
  (477 M/s at 23%, 308 at 48%, 274 at 56%, 140 at 70%, 32 at 92%). But a
  backoff that cuts the rate makes it **monotonically worse**: caps of
  0/1/4/16/64/256 read 0.5053/0.4843/0.4855/0.5078/0.5452/0.5747 s. Latency
  to pick work up dominates the traffic that finding it costs, so the
  correlation follows idleness rather than causing it.
  *Victim-selection policy:* **rejected.** Trying the lane a steal last
  succeeded against, before the random scan, loses at every 8-worker cell
  (`bal_d8_w16` 0.4714 vs 0.4637 s, `bal_d10_w16` 0.2450 vs 0.2388 s) and adds
  a systematic stall at 4 workers — one run in nine 2-3x long, across all six
  configs.
  *Draining more than one entry per steal:* **refused as unsound, not merely
  slow.** A batch steal taking the k oldest entries under one CAS on `top`
  **hangs** — empty output under a 20 s timeout, three for three, on a cell
  that runs in 0.12 s. A single CAS on `top` cannot claim a *range* of a
  Chase-Lev deque: the owner pops from `bottom`, the ends overlap whenever the
  deque is short, which at this grain is nearly always, and an entry is claimed
  twice. Making it sound means replacing the deque protocol.
  *Worker QoS and core placement on Darwin:* **exonerated.** Requesting
  `USER_INITIATED`, `USER_INTERACTIVE` or `UTILITY` for the pool's threads
  moves nothing at `bal_d8_w16`/8: 0.4693 s default against 0.4697 / 0.4703 /
  0.4737 s. No QoS class creates a fifth performance core.
  *The grant counter's own global read-modify-write:* **priced and
  exonerated.** It is the one shared-line atomic left on the steal path, and at
  this grain the steal path is hot (2.09M increments in 0.465 s). Moving the
  count onto the lane, isolation only, does not pay for itself: 0.4605 vs
  0.4667 s at `bal_d8_w16`, and *worse* at the other two cells (0.2695 vs
  0.2539, 0.2129 vs 0.1703). The counter stays exactly as it is.
  **The extreme fine grain, `fib(38)`: the tax is a foreclosed compile-time
  transform, and the boundary is the emitter.** Min-of-9, every run publishing
  `09eb377162b2565f`: sequential 0.0790 s, `--par` pool-off 0.2337 s
  (**2.96x**), `--par` at 8 workers 0.0777 s (0.97x of sequential). The cause
  is in the emitted arm64. The **sequential** build turns the second recursion
  into a **loop carrying an accumulator** — LLVM's accumulator tail-recursion
  elimination fires because `fib(n-2)` sits in tail position after an
  associative add — executing 63.2M calls at 1.25 ns each. The **refused
  `--par` edge** makes two real calls with no loop: 126.5M calls at 1.85 ns.
  **2.0x the calls and 1.48x the cost per call.** The claim call is not the
  tax: relinking with `-flto` so the refusal can inline recovers only 5%
  (0.2223 s). The transform is foreclosed because the emitter always rejoins
  the granted and refused edges through a `phi` at `%par.done`
  (`emitter/parallel.rs:227-257`), so the callee's result flows into a phi
  instead of into the caller's return. **A measurement therefore convicts the
  emitter, and per the brief this dig stops and reports rather than changing
  it.** Recorded for whoever takes it: this is the one shape where Dig 2 stage
  2's premise (a) *does* hold — the un-promoted path really does carry a tax
  the sequential program does not — but the fix it implies need not be C6's
  per-task demand signal, whose contended read-modify-writes Dig 2 measured at
  0.4905 -> 0.9254 s. With the pool off `wf__par_claim` refuses for the whole
  process, so selecting a sequential clone is a **once-per-process** decision,
  not a per-task one, and does not touch the rule that killed C6.
  **The grant counter's margin, quantified rather than asserted.** The gate
  case `the_runtime_replaces_the_modules_weak_refusal` is untouched. Its
  program, linked with the runtime and the same observer the test uses, run
  **1000 times at `WF_WORKERS=4`: 0 runs with `grants == 0`, minimum observed
  count 7, maximum 20, mean 13.2.** The assertion cleared its bound by 7 in the
  worst run of a thousand, which is stronger than Dig 2's "0 failures in 1000"
  and is the number the merge packet should carry.
  **The grid at HEAD, full protocol rotation, N=9, 144 cells, 1296 runs, every
  run byte-identical within and across both languages and all exit 0.** Against
  rayon's absolute wall time: **12 cells Whitefoot wins outright, 24 parity,
  and no cell where rayon is faster**; against `rayoncut`, 6 wins, 30 parity,
  none lost. **The win count is boundary-sensitive and should not be read as a
  trend**: 8 of the 36 cells sit within 0.02 of the 0.83 line, which is why
  three passes of the same bytes have reported 14 (Dig 2), 6 (Dig 3) and 12
  (here). The invariant holding across all three is that rayon wins nothing.
  **Verification.** `make -C compiler check` exit 0 before and after. No
  compiler byte changed, so no regression is possible; the probes are recorded
  as the branch's state rather than a before/after. `q4.wf` min-of-7: seq
  0.4565, W=1 0.5004, W=4 0.1946, W=8 0.2126, W=64 0.2517 s, against Dig 2's
  recorded 0.2530 at W=64. `bt.wf`: seq 0.1736, W=1 0.1699, W=8 0.0436, W=64
  0.0541 s, against Dig 2's 0.0440 and 0.0550. `par_layout` byte-identical at
  `WF_WORKERS` unset/1/2/4/8/64/65/0/`abc`, all exit 0, grants
  0/0/258211/625471/2072396/2385458/2400054/0/0, so the anti-false-green
  counter still counts and an unparsable or below-two setting still never
  starts the pool. Approval classes touched: no spec bytes, no conformance or
  compliance evidence, no new repository root entry.
- Dig 7 (done; the opt-in tax is gone, the pool-on world is untouched by
  construction, and one acceptance criterion is **not met for a reason that is
  attributed rather than waved at** — see the placement finding below).
  **What landed is two-version compilation with a once-per-process selection**,
  and the two halves of that phrase are the whole design.
  **The second world, and the set it covers.** A `--par` module now also emits,
  for every function on a path from the entry to a handed-out call, a
  *sequential clone* under `wf__par_seq_<name>` — the ordinary lowering with no
  group actualized, whose calls to other cloned functions name their clones. The
  set is a property of the call graph and the judgment, computed as the
  functions reachable from the entry intersected with the functions from which a
  hand-out is reachable; nothing consults a name, a signature, or a source
  shape. On `fib` it is `{fib, main}` and not `hex_digit` or `spell_hex`; on
  `par_layout` it is `{build, layout, main}` and not the other seven; on the
  test fixture it is `{pair, quad, oct, fold, main}` and not `leaf`, `branch`,
  `mix`, `low_byte`, or `spell` — a function with no hand-out below it has the
  same body in both worlds, so both worlds call the one copy and cloning it
  would be bytes with no reader. The clone is not merely similar to the
  sequential lowering: after restoring its own symbols it is **byte-identical**
  to the default compilation's body for the same function, checked by
  `the_sequential_clone_is_the_sequential_lowering` over all five clones of the
  fixture and reproduced by hand on `fib` and `par_layout`. The accumulator TRE
  duly fires — the clone's arm64 is the loop Dig 6 recorded for the sequential
  build, `sub x0, x19, #1; sub x19, x19, #2; bl _wf__par_seq_fib; add x20, x0,
  x20; cmp x19, #2; b.hs`, one call per two levels with the accumulator in
  `x20`.
  **Where the selection lives, and why there.** In the process bootstrap, once,
  before anything runs: `@main` calls `wf__par_pool_active`, branches, and
  enters one world or the other. The clone set is closed upwards through the
  call graph, so the entry function is in it whenever anything is, and one
  branch there puts the choice outside every loop and every recursion in both
  worlds — neither world ever calls the other, so nothing below that branch
  tests anything again. A per-task signal is the thing Dig 2 measured killing
  C6; this reads one word of the process's own environment, once. The
  overlapped lowering is not touched at all: the par-world `wf_fib` is 35
  instructions before this commit and 35 after, identical instruction for
  instruction with only branch targets shifted, and the same holds for
  `wf_layout` at 136.
  **The runtime query starts nothing, and that is measured.**
  `wf__par_pool_active` answers from `WF_WORKERS` through the same
  `wf__par_requested_lanes` the pool start uses, so the pool is still created
  lazily by the first claim, exactly where it was. The first version asked by
  *starting* the pool, which moves creation ahead of whatever the program does
  before its first hand-out; on `par_layout`, which builds its tree before it
  folds, that cost **0.4663 s against 0.3981 s at `W=4` and 0.4404 s against
  0.3724 s at `W=8`, 17% and 18%**, while the shipped version reads 0.3984 s and
  0.3752 s — parity. **A withdrawn reading, recorded because it nearly became a
  comment**: the first attempt to price the two versions used `WF_WORKERS=64`
  and reported a 14%-25% eager-start cost from one pass; an interleaved pass
  refuted it — at `W=64` a single binary's own spread is 47% to 308% and the
  ordering of the three variants flips between passes — and the reading was
  withdrawn before `par_layout` at `W=4/W=8` measured the same effect properly.
  The four entry-point signatures and the weak fallback text are byte for byte
  what they were; the query is a fifth, separate weak definition.
  **`fib(38)`, the acceptance number. Interleaved min-of-11, every run
  publishing `09eb377162b2565f`.** Sequential 0.0794 s. `--par` with
  `WF_WORKERS` unset: **0.2349 s -> 0.0791 s, a 2.96x tax reduced to 1.00x** of
  the sequential build beside it. Pool on: `W=4` 0.1198 -> 0.1232, `W=8` 0.0837
  -> 0.0838, against within-cell spreads of 2.0%-6.3%.
  **Recursion depth, and an unlooked-for 3x.** `min_stack.wf` bisected under a
  1024 KB stack, first failing depth. Sequential: 65 132 ok / 65 223 fail,
  unchanged by this commit. `--par` pool-off: **21 683 before, 65 132 after —
  the identical bisection bracket as the sequential build, from 33% of its
  ceiling to exact parity.** Dig 1 measured that remaining gap and argued no
  hand-out lowering could close it, which was right: this closes it by not being
  a hand-out lowering. Pool on the measurement is **stochastic and is reported
  as such** — how deep the calling thread descends depends on when a thief takes
  the deep side — reading 118 339 / 169 917 / 118 700 / 170 008 before and
  169 014 / 159 982 / 99 097 / 106 957 after, overlapping ranges, both far above
  the pool-off ceiling, which is the invariant Dig 2 recorded and it holds.
  **The pool-on grid: no cell regressed, full protocol rotation, N=9, 144 cells,
  1296 runs, all exit 0, every run byte-identical within and across both
  languages** (`skew_d16_w192` publishing `420229e929506cdd`, the same bytes
  Dig 3 recorded). Against Dig 6's recorded grid every one of the 36
  `workers>=2` cells is at or better than its old time; the worst move is
  1.00x, far inside the 1.20x band. Wins do not decrease: **13 cells Whitefoot
  wins outright against rayon, 23 parity, and no cell where rayon is faster**,
  against Dig 6's 12/24/0 — and the win count remains boundary-sensitive, so the
  invariant to read is that rayon still wins nothing. The 12 default builds are
  **text-identical** to Dig 6's, so the rotation's sequential column is the same
  code.
  **The opt-in column, and what it cost as well as what it bought.** `--par` at
  one worker over its own sequential build now reads **1.00x-1.01x on all twelve
  configurations**, where Dig 6 read 0.68x-1.02x. The sub-1.00x readings were
  not a win being lost lightly: on the skew shape the outlined lowering was
  **faster** than the sequential build, because the sequential build hits the
  traversal-order pathology Dig 3 documented and could not locate. Measured
  directly and interleaved, `skew_d16_w64` pool-off goes 0.5025 -> 0.6016 s
  (sequential 0.5988) and `skew_d16_w192` 0.6006 -> 0.7465 s (sequential
  0.7450): **the pool-off build gives up a 1.20x-1.24x accidental advantage and
  becomes exactly the sequential build.** That is the design working as
  specified in both directions — a `--par` build that was not activated now
  performs as the default build, neither worse nor better — and it makes Dig 3's
  unlocated sequential stall worth more, since it is no longer masked here.
  **ACCEPTANCE ITEM NOT MET, with a reproduction: `par_layout` pool-off is
  1.19x, and the cause is code placement, not the clone.** The brief required it
  to stay at 1.00x. Measured, min-of-15, spreads 4%-6%: sequential 0.7264 s,
  `--par` pool-off 0.8614 s. The attribution is complete and the clone is
  exonerated. (i) The clone's machine code is *identical* to the sequential
  build's — 136 instructions, differing only in the callee symbol of two `bl`
  instructions. (ii) `/usr/bin/time -l` puts instructions retired equal to
  within 0.04% (9 183.1M against 9 181.6M) while cycles rise 19% and **IPC falls
  from 2.888 to 2.425** — the same work, stalling, which is precisely the
  signature Dig 3 named as "a pure stall, not instruction selection... costing 0
  instructions". (iii) The decisive control: **the identical LLVM module linked
  with its two clang inputs in the other order reads 0.7162 s against 0.8599 s**,
  a 1.20x swing with not one byte of the module changed. So the effect is whole-
  binary placement; adding a second copy shifts every address in a `--par`
  binary and re-rolls this workload's known sensitivity, which landed badly for
  this one program and neutrally for the twelve oracle programs, `fib`, `q4`,
  and `bt` — 15 of 16 measured programs read 1.00x-1.01x. **Reordering the clone
  emission was tried and does fix this program (0.86 -> 0.72 s), and was
  refused**: it is the same coin, the effect moves in both directions
  (`skew_d16_w192` pool-off reads 0.79x of its sequential build under one
  placement), and choosing an emission order on one program's timing is exactly
  the corpus-keyed codegen Dig 3 declined. Handed on rather than tuned away, and
  it is a gift to whoever takes Dig 3's stall: this is the first control that
  isolates the effect to *placement alone* with the executed bytes held
  identical.
  **Code size, the price of the second world.** Default build: unchanged, and
  not approximately — the emitted module is byte-identical and all 12 oracle
  `_seq` binaries are text-identical to Dig 6's. `--par` build: the linked file
  grows 0.52% (`fib`) to 0.61%, and the machine code that is the real cost grows
  **7.0% (`fib`) to 14.2% (`skew_d16_w192`)** in `__text`, the runtime and the
  executable format being most of the file.
  **Anti-false-green evidence.** `par_layout` byte-identical at `WF_WORKERS`
  unset/1/2/4/8/64/65/0/`abc`, all exit 0, with grants 0/0/815/2430/18928/22967/
  22904/0/0 against the pre-commit build's 0/0/820/2444/19009/22519/22470/0/0 in
  the same rig — the counter still counts and an unparsable or below-two setting
  still never starts the pool. The gate case
  `the_runtime_replaces_the_modules_weak_refusal` is untouched, and its margin
  was re-measured **before and after in one pass, 1000 runs each at
  `WF_WORKERS=4`**: both read min 7, first percentile 8, median 13, max 20, mean
  13.4 and 13.2, so Dig 6's recorded min 7 / max 20 / mean 13.2 reproduces
  exactly and does not move. (A first pass measured min 5 for the new build
  alone, on a busier machine; the paired pass is the instrument that settles
  it.) Written claims survive the second world, whose clones duplicate the
  [DIAG-3] records: `d1_two_traps` emits the identical record and exit 134
  sequentially, at `--par` pool-off, and at `--par` `W=4`. Probes, min-of-7, all
  byte-identical: `q4` seq 0.4538 -> 0.4530, pool-off 0.4984 -> 0.4567, `W=4`
  0.1942 -> 0.1933, `W=8` 0.2096 -> 0.2122; `bt` seq 0.1718 -> 0.1686, pool-off
  0.1672 -> 0.1671, `W=8` 0.0435 -> 0.0435. `W=64` is reported for neither: it
  is not resolvable on this machine.
  **Tests: two added, one corrected, each with an injected-fault control.**
  `the_sequential_clone_is_the_sequential_lowering` pins the exact clone set and
  compares every clone body against the default compilation's; it fails when the
  clone keeps its groups and when the set is widened to everything reachable.
  `the_bootstrap_selects_one_world_once` pins one selection per process, the
  weak answer, the four untouched signatures, and the invariant that neither
  world calls the other; it fails when the bootstrap stops selecting.
  `handing_a_call_out_adds_no_stack_slot` now counts over the overlapped world
  alone, because counting both copies against one reference would compare a
  doubled module with a single one, and its doc records that;
  `handing_calls_out_keeps_the_sequential_recursion_depth` records honestly that
  with the pool off it now measures the clone rather than the refused edge, why
  adding a pool would not restore that reach on its fixture, and which case
  holds the property instead. A fourth injected fault — a clone set missing the
  entry — was caught by the runtime cases and *not* by the structural ones, so
  the emitter now derives the bootstrap's branch from the set itself rather than
  from a separate argument, and the module is well-formed by construction.
  **Verification.** `make -C compiler check` exit 0 before and after; all 16
  `backend::tests::parallel` cases green. `make check` still exits at the
  conformance step on the pre-existing `PAR-1` coverage blocker Dig 3 recorded
  (135/136); nothing here touches it. Approval classes touched: no spec bytes,
  no conformance or compliance evidence, no new repository root entry.
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
