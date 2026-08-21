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
  heartbeat-class promotion in `par_runtime.c` (a lane may accept/offer work
  only after a minimum interval of local work since its last promotion —
  bounds scheduling overhead as a fraction of useful work; no static size
  constant, which measurement already falsified). Success criteria against
  the oracle: no grid cell with workers>=2 slower than WF-sequential
  (today's 0.13x cells die), and best-case moves toward rayon. Work stealing
  is the NEXT dig if heartbeat alone leaves a large gap; not this one.
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
