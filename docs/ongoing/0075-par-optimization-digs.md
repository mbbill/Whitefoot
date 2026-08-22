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
- **Dig 9 — checker: the band/derived-index discharge asymmetry.** Queued as
  Dig 5 and **renumbered to 9 on dispatch** by lead decision, 2026-08-22, so
  the number could not collide with a concurrently dispatched session; the
  scope is unchanged and this entry supersedes the Dig 5 label in place. A
  `band` claim proving two bounds discharges in straight-line code and fails
  against a derived loop index where two separate claims succeed (recorded
  in the round-3 debate with compiling probes). Fixing the fact-propagation
  gap widens the claim-free set — more eligibility, zero spec bytes.
  **Corrected on execution:** the loop is not the trigger. The same failure
  reproduces in straight-line code as soon as the index is a let-bound
  derived value, and swapping the conjunct order does not move which
  conjunct is lost — so the asymmetry is over derived terms, not over
  loops or operand position.
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

- **Dig 8 — the adjacency window (F3).** Added by lead direction after Dig 7,
  under this batch's chartering direction. One builtin statement written
  between two calls makes the pair silently unjudged, so the same operation
  wrapped in a pure function keeps a parallel chain while the builtin version
  keeps none: two programs with byte-identical output differ 1.41x in wall
  time. Permission must follow grammar and semantic rule, never accidental
  statement adjacency. Widen the judgment from an adjacent pair to a window
  (s1, the statements between, s2), with every condition quantified over the
  interposed statements and every form the analysis cannot account for denying
  **with a report** rather than ending the enumeration. Soundness first: the
  rule is the intersection of what the two realizable schedules admit, not
  what the current backend alone would survive. Acceptance: the builtin
  version and the wrapped version get the same verdict, the dossier's
  counterexamples are denied under the right condition, the oracle is
  unchanged where no widening applies, and outputs stay byte-identical.

- **Dig 10 — compile-time attribution and fix.** Added by owner direction,
  quoted verbatim (2026-08-22): "一个44k的程序编译46秒。这已经是非常极端的慢
  了。优化性能要提上日程了。不然以后测试没法搞了" — a 44 KB program takes 46
  seconds to compile, which is extreme, so compile performance joins the
  agenda because it gates all future testing. Dig 7 recorded the symptom and
  correctly declined to own it. Attribute the cost before fixing anything:
  control for Defender real-time scanning, split the phases, and fit the
  scaling law. Then fix the attributed dominant cost with normal collections,
  leaving every emitted byte identical; if the only effective fix moves
  emitted bytes, stop and report options. Acceptance: an attribution table
  with the named hot function, `wfgrep` at or under 5 s in both modes or an
  irreducible-remainder report, byte-identical output everywhere, and the
  gate green with its wall time reported before and after.

## Approval classes

- Spec bytes: none planned (scheduler policy and codegen are implementation
  liberty under the CANDIDATE [PAR-1] rule, which states permission, not
  policy). If any dig turns out to need rule text, it stops and records the
  boundary. Dig 8 examined this and needed none; it did find that the rule as
  written is *over*-permissive, and recorded that boundary rather than editing
  the file. See its log entry.
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
- Dig 8 (done; F3, the finding that one builtin between two calls makes the
  pair silently unjudged, is fixed at its named cause; the oracle's compiled
  code is byte-identical before and after; and one spec-soundness boundary is
  recorded rather than crossed).
  **What the judgment used to do, and why it lost the pair.** The permission
  judgment walked each block collecting *consecutive* call statements into a
  group, and any statement of another form ended the group. Two calls with one
  ordinary statement between them therefore never formed a pair at all: no
  verdict, no ledger line, nothing to explain the missing parallelism. In
  `probes/p1a.wf` the two recursive `layout` calls sit at lines 116 and 118
  with `let gap = fmul.strict(child_inh, 1.5_f64);` at 117, and the whole fold
  ran sequentially because of that one line.
  **What landed: the judged unit is now a window.** A window is the ordered
  pair (s1, s2) of call statements together with every statement of the block
  written between them, and every condition is quantified over those
  interposed statements as well as the two calls. The rule is derived as the
  **intersection of the two schedules a conforming implementation may pick**,
  hand s1 to a lane and run the rest on the calling thread, or run s1 and hand
  s2 out with its operands evaluated at the hand-out point, because permission
  may not be stated in terms of a schedule. That derivation is what produces
  the two new dataflow clauses (nothing between them may read what s1 defines,
  because under the first schedule that value does not exist until the join;
  and s2 may not read what they define, because under the second its operands
  were evaluated before they ran) and the footprint clauses. It also produces
  **one deliberate asymmetry**: an interposed write is judged against s2's
  caller-side operand reads but *not* against s1's, because under the first
  schedule s1's operands are read before the fork and under the second s1 has
  already completed. The two mistakes are not symmetric: **adding** the missing
  obligation on s1's side is merely over-conservative and costs a few grants
  (it is what would refuse the shape the grant fixture pins), while
  **dropping** the obligation on s2's side admits a real race, because that is
  the side where a hand-out genuinely hoists the operand read above the
  interposed write. It is stated in the module doc in those terms and pinned
  from both sides: a denial fixture for s2's side and a grant fixture for
  s1's, so either error fails a test rather than passing silently.
  **Fail-closed, and reported.** Every interposed form is classified by an
  exhaustive match, so a statement form nobody classified cannot contribute an
  empty footprint and quietly widen permission. A `let`, `set`, or `replace`
  gets a footprint: the place it assigns plus the places its consumed `own`
  operands name count as writes, and what its operands read counts as reads.
  A form that can leave the block without reaching s2 denies under **condition
  4, the no-skipping-exit condition**; that covers `return`, `give`, `break`,
  a `propagate` whose error edge returns from the function, and a `claim`,
  whose trap edge to the diagnostic sink is an exit like any other. The
  `claim` case closes a hole the eligibility check structurally cannot see: it
  walks *callees* looking for reachable claims, so a `claim` the writer put in
  the caller's own block between the two calls was invisible to it, and the
  window would have read permitted and eligible. Every remaining form (a
  match, a loop, a region, an expression statement) denies under **condition
  2, the disjoint-footprints condition**, as an unresolved footprint, on the
  rule's own principle that an element whose place is not resolved overlaps
  every place. Whether those forms stay refused is a later question: computing
  a real footprint for a `match` or a loop body is possible and would widen the
  admitted set again, and nothing here forecloses it. The point is that these
  now **deny with a line** instead of silently ending the enumeration, which is
  the disclosure half of F3 and the half a verdict-only test suite is blind to.
  **Eligibility delta, measured over all 17 probes and all 12 oracle sources
  by running both compilers.** `p1a` is the target and it moves: eligible pairs
  1 to 2, and the new one is exactly `pair(layout, layout) eligible` with
  `run(layout, layout) 2 members` at line 116. `zero_elig` goes from reporting
  **nothing at all** to one eligible pair and chain. Every oracle source gains
  one reported (denied) pair and two chain lines and **no** eligibility, which
  is why the oracle cannot move. Across all 29 sources no previously reported
  pair changed its verdict; the additions are new denials for windows that were
  silent, plus the chain lines described below.
  **The F3 headline closes. Interleaved min-of-9, all eleven cells in one
  rotation, machine checked quiet first.** Every cell of every program
  published the same `2c4d496258ec3e06`, `p1b`'s included, so the claim that
  these two programs emit the same bytes is measured here rather than assumed
  from their sources. `p1a` sequential 0.7677 s. `p1a` `--par` **before**:
  0.7685 / 0.7573 / 0.7619 / 0.7601 s at 1/2/4/8 workers, flat, no speedup at
  any worker count, which is what a program whose fold was never judged looks
  like. **After**: 0.7658 / 0.5132 / 0.3984 / 0.3749 s. That is **1.91x at four
  workers and 2.03x at eight against the same program compiled before this
  commit**, and 1.93x/2.05x against its own sequential build. The dossier
  predicted "1.00x to about 1.96x at W=4" and the measurement lands inside it.
  Spreads (max over min within the rotation) are 5.2% to 7.2% on every
  multi-worker cell, so those readings are tight; the one-worker and sequential
  cells ran 42% to 64%, which is machine noise on the slowest cells and is why
  only the minima are read.
  Against `p1b`, the wrapped-function twin that kept its chain all along,
  `p1a` now reads 0.3984 against 0.4049 at four workers and 0.3749 against
  0.3808 at eight: **parity, from a gap that was 1.9x between two programs
  whose output bytes are identical.** The 1.6% edge is in the predicted
  direction (`p1b` pays a second hand-out for a trivial `scale_up`) but is
  inside the band and is **not** claimed as a win; the four-worker `p1b` cell
  in particular spread 81.6%, so the eight-worker comparison, whose `p1b` cell
  spread 1.2%, is the one to read. The opt-in tax did not move: one worker
  0.7685 to 0.7658, 0.4%.
  **Is the shape realistic?** `p1a` is the paired browser-layout fold this
  batch adopted as its oracle, not a probe built to display the bug, and the
  twelve oracle sources are the same `layout` shape. They escaped the defect
  only because their generator happens to emit the two child calls adjacent,
  while `p1a` writes one arithmetic line between them, which is the accident
  the whole finding is about.
  **What is untouched on the oracle, stated precisely, because the oracle did
  change in one respect.** Its *diagnostic* output changed: every one of the
  twelve sources gains one denied line and two chain lines, as the delta above
  says. Its *compiled code* did not, and that is the load-bearing claim: the 12
  oracle `--par` modules the compiler emits are **byte-identical** before and
  after, and all **24 oracle binaries rebuilt with the new compiler are
  byte-identical** to the ones sitting in `bench/bin` from Dig 7. Identical
  machine code cannot time differently or print differently, so no rotation
  could have found a regression there; one was run anyway as an end-to-end
  check: `./rerun.zsh 1`, 144 cells, every run byte-identical within and across
  both languages, mean cell spread 0%, `skew_d16_w192` publishing
  `420229e929506cdd`, the same bytes Digs 3 and 7 recorded. The **default**
  compilation is untouched too: 0 of 28 emitted modules differ across every
  probe and oracle source. (Linked executables *do* differ byte for byte when
  the output filename differs, which briefly looked like a defect; the control
  is that one compiler compiling one source to two different output names also
  produces two different binaries, and the 24 same-path rebuilds above are
  identical. The module comparison is the real evidence.)
  **No protected evidence is involved, and the file-level check says so.** The
  oracle is research measurement under `research/investigations/`, not
  conformance or compliance evidence, and none of it was edited: the recorded
  snapshot in `bench/baseline/` is unmodified, and the regenerated ledger,
  binary, and output directories are all gitignored. `git status` at commit
  time shows exactly six modified files, five compiler sources and this record,
  and no additions anywhere. The changed diagnostic lines live in the
  compiler's developer channel, which participates in no verdict and no
  mandatory record.
  **Code size, the cost of widening the clone set.** Dig 7's second world
  covers every function from which a hand-out is reachable, and that set is
  derived from the judgment, so admitting `layout`'s pair widens it: `p1a`'s
  clone set goes from `{build}` to `{build, layout}`, hand-out sites 1 to 2,
  thunks 1 to 2. The derivation needed no change because it is computed from
  the emitted call graph rather than cached. Price on `p1a`: machine code
  (`__text`) 12 560 to 13 272 bytes, **+5.7%**; linked file 52 848 to 52 928,
  +0.15%. `p1b`, whose judgment did not change, is unchanged in both, which is
  the control. Both figures sit inside Dig 7's recorded 7.0% to 14.2% range for
  what the second world costs.
  **Ledger truthfulness (F4), the contained half.** The ledger now emits a
  `chain` line per eligible run beside its pairs, because three permitted pairs
  and one three-member run read identically as pairs and are completely
  different work. `p1a` now shows `run(layout, layout) 2 members through line
  118` where before it showed nothing. **The uncontained half is recorded as a
  follow-up rather than attempted**: what the *backend* keeps is narrower than
  what the judgment permits (one call definition per site, all members in one
  block, no addressed binding but the last), and that narrowing happens in
  lowering, after the checker has rendered this ledger, so reporting it needs
  the ledger moved or the narrowing surfaced back. A `chain` line therefore
  states what the judgment permits, not what the emitter actualizes, and its
  doc comment says so.
  **SPEC BOUNDARY, examined and recorded, no spec byte touched.** The
  chartering rule for this batch is that a dig needing rule text stops. This
  dig does not need any, and the reason is worth the owner's attention. The
  CANDIDATE rule says permission holds for the ordered pair where "s1
  **precedes** s2 in one block", not where it immediately precedes it, and the
  word "adjacent" appears nowhere in the rule (it appears twice in the whole
  specification, both about terminal leaves in the trivia rules). So the
  specification already permitted this widening and the compiler was, and after
  this commit still is, **strictly narrower** than it. Every window this
  compiler now permits satisfies every condition the rule lists; the new
  clauses only remove permission. **But the rule as written is
  over-permissive, and that is a real defect independent of this dig.** Its
  conditions quantify over "the two calls" only, and the sentence is a
  biconditional ("exactly when all of the following hold"), so a window whose
  interposed statement writes the storage s2's callee reads satisfies every
  listed condition and therefore *has* permission, while the rule's own next
  paragraph promises that a permitted overlap produces exactly the source-order
  observables, which that overlap does not. The rule grants permission and then
  makes a false guarantee about it. The smallest witness is three statements:
  read a cell through one call, write that cell with a plain `set`, read it
  again through a second call. The two *calls* touch different storage on the
  first read and the same storage on the second, every listed condition holds,
  so the rule grants permission, and yet overlapping them lets the second call
  read the value from before the `set`. That exact program is now one of the
  denial fixtures in the compiler
  (`an_interposed_write_into_the_second_callees_read_is_denied_by_condition_two`),
  which is why nothing is at risk today: the compiler refuses what the rule
  would allow.
  Closing it in the rule needs roughly three sentences quantifying the
  footprint and dataflow conditions over the interposed statements, plus
  widening the exit clause from "no edge out of s1" to "no edge out of s1 or of
  any statement between them". **Recorded for the owner, not written**: it
  changes the language rule, nothing this dig ships needs it, and the compiler
  is fail-closed against it. **Recommendation, since a paragraph in a batch
  record is a poor place for a soundness defect to live**: it should become its
  own item with the specification-change workflow, not a note inside a
  performance batch. It is not urgent (no accepted program can exploit it,
  because no implementation in the tree takes the permission the rule
  over-grants) but it is exactly the class of thing that gets forgotten.
  **Tests: eleven added, five existing ones strengthened, none weakened.** Nine
  window fixtures in the judgment suite: the F3 grant and its chain; the
  interposed write into s2's callee read, over s1's callee write, and under
  s2's operand read (the last is the obligation no pair rule has, and its
  callees carry no relevant row at all, so it is the schedule and nothing else
  that refuses it); the two dataflow clauses; the interposed `propagate` and
  the interposed `claim`; and the interposed `match`, which asserts the denial
  is **reported** rather than merely absent. One grant fixture pins the
  asymmetry that must not be tidied away. One backend fixture,
  `a_fold_whose_calls_are_separated_by_a_builtin_hands_out_and_agrees`, is the
  differential the checker change made reachable: the same fold with a builtin
  between its two recursive calls still hands work out, and its bytes match its
  own default compilation (which contains no hand-out at all) at every worker
  count, so a moved read or a misplaced join cannot hide by being present on
  both sides. It is produced by editing the existing adjacent fixture in one
  place rather than by copying it, so the two sources cannot drift. Five
  existing cases gained assertions on the new fields (which window statement a
  denial cites) rather than ignoring them; two ledger cases were updated for
  real text changes (a conflict line now names both sides, since it may now
  cite an interposed statement) and one gained the chain-line assertion. No
  test was deleted, relaxed, or given a weaker expectation.
  **Cost, found and paid.** The widening enlarged the candidate group from
  "consecutive call statements" to "every call in the block", and the chain
  search judges every ordered pair, so classifying the window inside the
  judgment made it quadratic in calls times linear in statements. Each block is
  now classified and projected **once**, and whether a claim is reachable from
  each function is answered by one reverse walk of the call graph at startup
  instead of a whole-program search per judged pair. With that in place there
  is no measurable cost: the gate's own suite runs **167.80 s after against
  167.92 s before**, and compiling the largest real program in the tree
  (`tests/programs/wfgrep.wf`, 1 269 lines), interleaved min-of-6, reads
  44.81 s before and 45.70 s after, 1.02x inside a 12% spread. (An earlier gate
  run read 176 s, and the programs suite 206 s against 154 s; both were
  measured while this executor was running builds and timings beside them, and
  the quiet re-run above is the reading to keep. Recorded because the inflated
  numbers were nearly written down as a cost.)
  **What `--par` costs at compile time, measured at lead request and handed
  forward rather than dug into.** Interleaved min-of-6 on the same 1 269-line
  program, one compiler, both modes compiled all the way to a linked binary:
  **default 45.43 s, `--par` 46.15 s, a ratio of 1.016x**, the `--par` column
  spreading 6.2% and the default column 21.5% on one outlier round.
  **Load caveat, and it applies to every wall-clock number in this entry**:
  this machine was not quiet. Microsoft Defender was observed at 320% CPU, the
  worktree is not excluded from its real-time scanning, and a peer session's
  Whitefoot processes were running beside these compiles. The *ratio* survives
  that, since both columns take the same interference and the minimum of six
  rounds is the statistic least exposed to it, but the **absolute** seconds are
  informational and should be re-measured by the chartered compile-time dig
  under proper controls. Nothing about this dig's acceptance rests on them: the
  load-independent evidence is the byte identity recorded above, which
  contention cannot fake. So Dig 7's
  second world, which really is emitted for this program (`wfgrep` carries
  three eligible pairs, three hand-out sites, three thunks and three sequential
  clones), costs **essentially nothing in compile wall time**, far below the
  1.3x that would have made it a finding. What it costs is *size*: linked file
  51 776 bytes default against 71 032 with `--par`, machine code 16 528 against
  24 440, which is +37% and +48%. That pair of figures is **not** comparable to
  Dig 7's recorded 7.0% to 14.2%, and conflating them would overstate the
  clone: Dig 7 measured the clone's own incremental cost by comparing `--par`
  before its change against `--par` after, while this compares the default
  build against `--par` and therefore also counts the runtime, the thunks and
  the hand-out edges. The number worth carrying out of this is the one neither
  column explains: **a 1 269-line program takes about 46 seconds to compile in
  either mode**, so whatever dominates that is in the path both modes share and
  not in anything `--par` adds. That is a standing compile-speed problem, it is
  not this dig's, and nothing measured here causes it.
  **Deviation, recorded.** Roughly an hour was lost to a "regression" that was
  an instrument error: `cargo test` uses the default unoptimized profile while
  the gate uses `--profile gate`, and the suite's real-program tests run about
  16x slower there, which `compiler/Makefile` states plainly and this executor
  had not read. Two hypotheses were built and one fix written before a profile
  of the actually-slow test showed 100% of samples inside the entailment engine
  and none in the permission judgment, and before stashing the change showed
  the identical timeout at the parent commit. The precomputation described
  above was kept because it removes genuinely redundant work that this
  widening introduced, not because it fixed the phantom.
  **Incidental finding, reported, not fixed.** A `let` selecting `propagate` is
  admitted as a *candidate*, so it can be s2 of a permitted pair, while the
  rule's first condition admits only a `let` selecting an ordinary right-hand
  side. This is pre-existing, harmless today (the lowering refuses such a
  member outright, and both schedules are sound for it), and keeping it a
  candidate is what makes the propagate-as-s1 denial reportable rather than
  silent. Narrowing it needs a decision about which condition would cite it,
  which is not an executor's to take inside an F3 dig.
  **Verification.** `make -C compiler check` exit 0 before (1019 lib + 52
  program tests) and after (1030 lib + 52), with the two ledger cases and the
  new window and backend cases green. Approval classes touched: **no spec
  bytes** (the boundary above is recorded, not crossed), no conformance or
  compliance evidence, no new repository root entry. `make check` still exits
  at the conformance step on the pre-existing `PAR-1` coverage blocker Dig 3
  recorded (135/136); nothing here touches it.
- Dig 10 (attribution landed; fix in progress): **the 46 seconds are ours,
  and they are one cubic loop.** Recorded before the fix so the measurement
  stands on its own.
  **Step 0, Defender: negative, and controlled rather than assumed.** The
  worktree is outside the `do_not_scan` exclusion and Defender had been seen
  at 320% CPU, so nothing was believed until the exposure was varied. Full
  `wfgrep` compiles: source and output in the worktree with the default
  `TMPDIR`, 45.686 s; output and `TMPDIR` moved into the excluded directory,
  46.102 s; source, output and `TMPDIR` all moved there, 45.671 s. Three
  configurations, one number. Real-time scanning accounts for none of it and
  the fix is algorithmic, not operational.
  **The cost is inside our compiler, not the host toolchain.** `--emit-llvm`,
  which stops before `clang`, takes 45.074 s; the full compile takes 45.754 s;
  `clang -O2` run alone on the emitted module takes 0.267 s. So **98.5% of the
  wall time is whitefootc's own frontend, and the backend is 0.6%**. The
  emitted module is 9 935 lines / 414 727 bytes, which is nothing for LLVM.
  (Note for the next reader: `--par-ledger` does *not* isolate the frontend —
  it runs the same full pipeline including the link. `--emit-llvm` is the
  instrument.)
  **Program size does not predict compile time; terms per function do.**
  Frontend seconds against source bytes across `tests/programs/`:
  `raw_deflate_vectors.wf` 31 884 B → 0.008 s, `dir_walk.wf` 18 010 B →
  4.198 s, `utf8parse.wf` 9 511 B → 5.515 s, `wfgrep.wf` 44 367 B → 45.961 s.
  Four programs, a 4x byte range and a 5 700x time range: bytes are the wrong
  variable. Two synthetic families at matched call counts separate the right
  one — `wide` puts N calls in one function, `narrow` spreads the same N over
  N/4 four-call functions. `wide` measures 0.036 s / 0.096 s / 0.224 s /
  0.793 s / 1.963 s / 3.925 s at 40/60/80/120/160/200 calls, a clean
  **exponent of 3.13** (80→160 and 120→200 both fit it); `narrow` measures
  0.009 s at 40 and 0.014 s at 80, essentially linear. **The cost is cubic in
  the terms of a single function and linear in the number of functions**,
  which is why `wfgrep` — one 435-line `walk` — costs 46 s while a 31 KB
  program of small functions costs 8 ms.
  **The named hot function.** `sample` on a `wfgrep` compile puts effectively
  every stack in `semantic::entailment::state`. The dominant leaves are
  SipHash over `(TermId, TermId)` keys, then `insert_closed_candidate`,
  `close_with_excluded_term`, and `DerivationLedger::depends_on_postcondition_call`.
  The loop is the [ENT-4] closure at
  `compiler/src/semantic/entailment/state.rs:2429`, whose transitivity fixed
  point at `state.rs:2526-2556` is a Floyd–Warshall over difference bounds
  held in a `HashMap` keyed by term pairs:

      for middle in &ids { for left in &ids {
          bounds.get(&(*left, *middle));          // T^2 probes
          for right in &ids { bounds.get(&(*middle, *right)); ... } } }

  The `(middle, right)` probe is invariant in `left` and is re-hashed for
  every `left`, so a hub term — `ZERO`, which nearly every bound mentions —
  is probed T times per predecessor instead of once. `bounds` and
  `bound_proofs` are two maps under the same key, so each logical read hashes
  twice.
  **Regression answer: none. This is a standing defect on `main`.** The branch
  never touched `state.rs` (`git diff` over the file across all nine branch
  commits is empty; the branch's only entailment change *removed* 375 lines
  from `flow.rs`). Built at the fork point 4f01bab6 in a detached worktree with
  the same release profile, `wfgrep` takes **46.347 s** against **45.074 s** at
  branch HEAD 974d5513 — the branch is marginally faster, not slower. There is
  no guilty commit to bisect.
  **Gate baseline.** `make -C compiler check` green at 974d5513 in **348.44 s**
  wall (1030 lib + 52 program tests). The suite compiles `wfgrep` afresh in
  each of its 12 `programs::wfgrep` tests with no shared build, and those tests
  each exceed 60 s, so the gate carries this defect twelve times over. That is
  the concrete sense in which the cost "gates all future testing".

- Dig 10 (fix landed, first increment): **the closure now runs dense, and
  `wfgrep` compiles 4.6x faster with every emitted byte unchanged.** Two
  changes, both inside the [ENT-4] closure the attribution above convicted,
  and neither touches what the judgment decides.
  **The bound relation is dense, so it is now held dense.** Instrumenting the
  fixed point settled what the profile could only suggest: on `wfgrep` the
  inner loop runs 1.444e9 times and **all 1.444e9 of those probes find the
  edge already present**. The relation saturates, and it was being kept in a
  `HashMap<(TermId, TermId), i128>` beside a second `HashMap` of proofs under
  the same key — so the innermost step hashed a term pair twice to read
  something it always held. `TermId` is already a dense ordinal, so the
  working relation is now one `Vec` indexed by `left * width + right` with the
  bound and its proof in a single cell, rebuilt into the published maps once
  per closure: linear, against a fixed point that is cubic. **45.074 s →
  15.604 s.**
  **The derivation node is built only when it can still win.** With the
  hashing gone the profile's second entry was `DerivationNode` construction
  and drop: the loop built a node for every one of those 1.444e9 candidates,
  and nearly all of them were then rejected on the two numbers alone. A route
  strictly weaker than the bound already held is rejected whatever its
  derivation, so that test now runs first; the node is still built for every
  candidate that can win, the equal-bound case `candidate_better` has to
  inspect included. **15.604 s → 10.710 s.**
  **Results, min of the runs stated.** `wfgrep` frontend 45.007 s → 9.782 s
  (**4.60x**); full compile including the link 10.267 s default and 10.118 s
  `--par`. The rest of `tests/programs/` moves with it: `utf8parse` 5.445 →
  1.068 s (5.10x), `sha256_abc` 0.568 → 0.125 s (4.55x), `dir_walk` 4.082 →
  1.076 s (3.80x), `percent_decode` 3.19x, `grayscale_pixels` 2.89x. Nothing
  regressed: the three programs that read below 1.00x in the min-of-2 table
  are all ~6 ms compiles and read 0.95x-1.07x at min-of-9, which is noise.
  **Byte identity, the whole point.** All 25 programs in both modes — 50
  (program, mode) pairs — produce byte-identical emitted modules, identical
  stderr (so the [CLM-2] advisories are unmoved), and identical exit codes,
  against a binary built at the immediately preceding commit 00b9e686. The
  judgment's results are untouched and only the machinery is faster, which is
  what this dig was allowed to change.
  **Gate.** `make -C compiler check` green **before 348.44 s, after 146.85 s
  (2.37x, 201.6 s saved)**, at the same 1030 lib + 52 program tests — nothing
  was deleted, skipped or weakened. The program suite alone falls 172.24 s →
  34.21 s (5.03x), which is the twelve fresh `wfgrep` compiles getting cheaper.
  **Second increment: the closed state carries the matrix.** `ClosedState` was
  still publishing two pair-keyed maps, so every one of the 3185 closures
  rebuilt its whole relation into two `HashMap`s only for the next query to
  hash it back. It now carries the matrix itself, one field where there were
  two, and the pair-keyed form is built at the single place that still needs
  it — the `FactState` in `materialize_closure_at`, the rare S11 path. Two
  sorts go with it: cell order *is* ascending `(left, right)`, because the
  index is `left * width + right`, so the walk arrives already ordered.
  `wfgrep` 9.782 s → 9.393 s, which is only 4% — the average closure is far
  narrower than the 94-term maximum, so the rebuild was smaller than the
  profile's rank suggested. The gate is where it shows: **146.85 s → 109.04 s**,
  because the suite is mostly small analyses where the rebuild was a larger
  share.
  **Where this landed, end to end.** `wfgrep` frontend **45.299 s → 9.054 s
  (5.00x)**; full compile including the link **9.612 s default, 9.267 s
  `--par`**. Across `tests/programs/`: `utf8parse` 5.388 → 0.984 s (5.47x),
  `sha256_abc` 4.99x, `dir_walk` 4.087 → 0.974 s (4.20x), `percent_decode`
  3.45x, `grayscale_pixels` 3.26x; nothing regressed. **`make -C compiler
  check` green, 348.44 s → 109.04 s (3.20x, 239 s saved)**, at the unchanged
  1030 lib + 52 program tests. All 50 (program, mode) pairs stay byte-identical
  in module, stderr and exit code against the pre-fix commit.
  **The scaling probe, re-fitted.** The `wide` family that gave exponent 3.13
  before now reads 0.029 / 0.064 / 0.121 / 0.221 s at 80/120/160/200 calls in
  one function — **exponent 2.06 over 80→160 and 2.43 over 120→200, and 17.8x
  faster at 200 calls than the 3.925 s it took before**. Stated honestly: the
  fixed point is still a Floyd-Warshall and still cubic in the terms of a
  function. What collapsed is its constant, by enough that over this range the
  lower-order work now shows through instead. The practical consequence is the
  one the owner's directive was about — the wall that made a 44 KB program cost
  46 s has moved a long way out, and it moves further the larger the function.
  **Third increment, on lead direction: the two named levers, one refuted and
  one landed.** Both were pursued to a number rather than argued about.
  **Lever (b) — narrow the hot stream — is REFUTED.** The premise was that the
  inner loop is bandwidth-bound on its 48-byte `Option<(i128, DerivationId)>`
  cell when the rejection test reads only the 16-byte bound. Confirmed the cell
  really is 48 bytes, then tried it two ways. Presence, bound and proof in three
  parallel columns: **11.074 s, 18% *worse* than the 9.393 s it had to beat** —
  three streams and three bounds-checks cost more than the traffic saved. One
  `Vec<Option<i128>>` for the hot test with proofs alongside, 32 bytes and a
  single stream: **9.411 s, indistinguishable from 9.393 s**. Cutting the hot
  stream by a third moved nothing, so the loop is not bound on that traffic at
  all and the premise was simply wrong. Both variants were byte-identical, so
  this is a refutation on performance, not on correctness. Reverted; nothing of
  lever (b) is in the tree. Worth recording separately: the sentinel encoding
  that would have made this cheaper is **unavailable**, because `i128::MIN` and
  `i128::MAX` are genuinely reachable — a negative cycle lowers a bound until
  `saturating_add` pins it, which is exactly how this fixed point terminates on
  one. A reserved bound value would have been a latent defect.
  **Lever (a) — skip a `middle` whose row and column are unwritten — is
  LANDED.** A term is marked when a bound is written into its row or column, and
  a `middle` with neither mark is skipped. Sound because routes through an
  unmarked `middle` carry the same two bounds *and the same two proofs*, so
  `via` is unchanged while the target can only have fallen: a route refused for
  being weaker is refused again; a route that tied and lost to the held proof
  loses to whatever replaced it, since `candidate_better` orders on
  `(depth, structural tie)` and that order is strict; and a route that won last
  time now meets its own proof, which cannot beat itself. Marking on the
  **write** rather than on a changed bound is what makes this hold — a candidate
  that keeps the bound and only replaces the proof still marks.
  **9.393 s → 8.417 s on a quiet machine (1.116x)**, byte-identical.
  **Why it is 11% and not the ~28% predicted.** Instrumented: **30.5% of
  middle-sweeps are skipped** (114 516 of 375 616), so the count estimate was
  right and the *time* estimate was wrong. Sweep cost is heavily skewed, and the
  skipped sweeps are the cheap tail — a `middle` is skipped precisely when
  nothing wrote to it, which correlates with having few predecessors, and such a
  sweep already bailed out of its `left` loop immediately. The expensive hubs
  keep being written and so keep being swept. Round count is unchanged at 8826;
  this makes rounds cheaper, it does not remove them.
  **Measurement note, honest.** The final absolutes were taken while the machine
  was loaded by corporate agents this session does not control (`CorpLinkExtension`
  ~60%, `epsext` ~58%), which inflates every absolute. The load-independent
  evidence is interleaved min-of-5, old and new binaries alternating so both meet
  the same conditions: **10.620 s → 9.319 s, 1.140x** on the frontend, and
  1.101x / 1.185x on the full compile. That agrees with the quiet-machine 1.116x,
  so the gain is real and the absolutes are the noisy part.
  **End state: the 5 s acceptance is NOT met, and this is the honest end.**
  `wfgrep` full compile ~9.1 s against a 5 s target. Both named levers are now
  resolved — (b) refuted by measurement, (a) landed for ~11% — so the stop rule
  is satisfied and no third lever was invented. What remains is attributed and
  still reducible, but not by anything cheap: the 1.444e9 triples are genuine
  Floyd-Warshall work over 3185 closures, and the two ways to cut them further
  are to stop recomputing closures from scratch as the flow walks (deep, in
  `flow.rs`, not this dig's scope) or semi-naive propagation, which stays
  **rejected**: it reorders `intern_for` and so reassigns `DerivationId`s, and
  no dig should move emitted bytes on a hunch about what is observable.
  **Approval classes touched:** no spec bytes, no conformance or compliance
  evidence, no gate wiring, no new repository root entry.

- Dig 9 (done; the gap is closed at its named cause and the checker is
  strictly smarter, never more permissive): **the loss is in expansion, not
  in the band.** Attribution first, because the queued description was wrong
  in a way that would have sent the fix to the wrong place. `band(a_ok,
  b_ok)` over two plain parameters discharges both bounds; make either
  operand a let-bound derived value and that operand's bound is lost while
  the other still discharges. Swapping the conjunct order does not move
  which one is lost, and the loop is not required — the minimal
  reproduction is straight-line. So the trigger is a **derived term**, and
  the mechanism is this: a claim or guard establishes two goals, the named
  binding and its expansion (`goal_origin_set`, `flow.rs:3519`), and only
  the expanded one had anything to decompose, because the named one is a
  bare `Datum(Place)` and `collect_decomposition_members` only walked
  Boolean *operations*. `expand_goal_expression` (`flow.rs:3425`) replaces
  every still-valid ordinary-let leaf by its origin, so the conjunct that
  read `next < room` became `at +wrap 1 < len(deref(input))` — and
  `goal_operand` returns `None` for an arithmetic root (`flow.rs:3897`), so
  that conjunct's projection was `None` and
  `establish_boolean_decomposition` skipped it with `continue`. Expansion
  is lossy: it can turn a projectable comparison into an unprojectable one.
  The equivalent single-bound claim never expands — `scrutinee_relation`
  reads `state.origins`, which holds the relation over the binding's own
  place term — which is exactly why the pair discharged where the
  conjunction did not.
  **The fix follows the same route the pair takes.** Decomposition now reads
  through an unprojected `own Bool` leaf that carries a still-valid
  ordinary-let Boolean origin (`collect_decomposition_members`,
  `flow.rs:3590`), so the members of a `band` written over comparison
  bindings are those bindings; a member that is a bare comparison binding
  takes its relation from `state.origins` (`member_binding_relation`,
  `flow.rs:3705`). That is the same map, the same terms, and the same
  validity discipline the single-bound claim already
  depends on — `origins` and `goal_origins` are invalidated together at every
  kill, scope exit and join, and `origins` additionally dies when either of
  the relation's terms dies. No claim shape is special-cased and no source
  pattern is matched: the rule is over the goal's grammar.
  **Not more permissive.** The negative twins all still reject, at HEAD and
  after: a third index the band never bounded (`c < len(deref(input))`
  survives), `bor` instead of `band` (a disjunction bounds neither side), the
  **else** edge of `if both` (`-band` is genuinely disjunctive), and
  `bnot(band(..))`. The runtime check is untouched and is what makes the
  widening sound: a band claim that is false at run time still traps —
  `{"rule_id":"CLM-1",...}`, exit 134 — so the admitted subscript never
  executes out of range. Three tests pin this in
  `semantic/tests/boolean_composition.rs`: the conjoined and separate forms
  are asserted to discharge the *same* two obligations, the guard form is
  asserted to admit the true edge and **not** the false edge, and the
  uncovered-index and disjunction cases are asserted to keep their
  obligations undischarged.
  **Eligibility delta, measured over 697 sources** (`tests/programs`,
  `tests/conformance/cases`, `tests/codegen`, and all of `research/`),
  before-binary against after-binary: **4 verdicts changed, all
  REJECT -> ACCEPT, all of them this dig's own probes**; 276 programs
  accepted by both produce **byte-identical LLVM IR**, and 417 rejected by
  both keep the identical rule. Nothing moved in the DENY direction. The
  ledger changed on exactly two files, both newly compilable:
  `d2_band_window_guard.wf` gains `pair(window, window) eligible` plus a
  2-member chain, and its twin `d2_band_window_claim.wf` gains
  `pair(window, window) not-actualizable: 1 claim site via window`. That
  pair is the campaign point in two files: the same two adjacent reads, the
  same output, and only the claim-free branch-guarded form is eligible —
  and before this fix **neither form compiled at all**, so a writer who
  wanted the eligible shape had no way to write it.
  **Cost: none measurable.** This is a compile-time change and the emitted
  bytes are identical, so only analysis time could move. On the wfgrep
  frontend, 12 interleaved before/after pairs of user CPU give a **mean
  paired delta of -0.086 s** (min 9.71 s before / 9.80 s after, median
  11.55 / 11.47) against a per-round spread of +/-3 s — below this machine's
  noise floor. `dir_walk`, which has no Boolean composition at all, reads
  +1.1% at both min and median, which is the cost of the added
  `goal_origins` lookup on ordinary guards; the lookup returns without
  retaining the origin unless the origin is a Boolean root, which is what
  keeps it at that size. **Machine not quiet:** another session held a core
  at ~99% throughout, so every absolute here is inflated and only the
  interleaved paired deltas are load-independent.
  **Oracle rotation.** All 12 `bench/wf` sources emit **byte-identical IR**
  before and after, so the rotation compares the same program with itself.
  Run anyway on `skew_d16_w192`, min of 5 interleaved: seq 1.83 -> 1.61,
  par1 1.81 -> 1.67, par2 0.95 -> 0.86, par4 0.41 -> 0.43, par8 0.32 -> 0.35
  (ratios 0.88-1.09). That scatter **is** the noise floor rather than a
  result, since the binaries differ only by the linker's UUID — verified by
  building the same source twice with the same compiler and getting
  different bytes, which is why IR, not the linked file, is the identity
  criterion here. Output is identical at seq and at 1, 2, 4 and 8 workers.
  **One deliberate behavioural change beyond the fix, flagged.** The O11
  decomposition inventory now records the binding-rooted parent as well as
  the expanded one, so `boolean_decompositions` doubles on existing shapes
  (1 -> 2, 2 -> 4). Four assertions in
  `semantic/tests/boolean_composition.rs` counted those entries and were
  updated. They were **not** weakened to go green: every existing member and
  projection assertion is retained unchanged, and each updated test gained
  assertions on the new entry's members. bxor still records nothing on
  either sign or either parent shape, which the new count pins.
  **Gate.** `make -C compiler check` **green** (196.7 s wall, on the
  contended machine) at **1033 lib + 52 program tests** — 1030 lib before,
  the 3 added here, none deleted, none skipped, none ignored. Full
  `make check` exits 2 at the known 135/136 coverage line and nowhere else.
  **Approval classes touched:** no spec bytes, no conformance or compliance
  evidence, no gate wiring, no new repository root entry. Four probes
  promoted into the existing `probes/` home with README entries.

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
