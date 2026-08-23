# Batch 0078 — loop-shaped permission

Branch: `par/loop-permission`, stacked on the frozen
`par/proof-derived-parallelism` tip `27e02b1f` (batches 0074-0077, closed,
awaiting owner merge; nothing on that branch moves). This batch's work
enters a separate, later merge review so the two phases stay independently
trackable.

Authority: owner chartering direction, 2026-08-23, verbatim:

> 强迫循环写成递归感觉违背了默认形态就是最优的原则,所以我觉得我们应该认真
> 的把循环问题也解决了。不过如果这件事有blocker的话可能需要好好研究一下。
> 目前收官完成的分支可以放着不动,然后在这个顶上再继续开个新分支开始循环的
> 研究吗?这样我们可以轻松的追踪这两个阶段。

Consequence adopted: the counted-loop ledger hint (batch 0077) is a bridge,
not the end state — the loop form itself must receive permission. This is
the plan's W4 "indexed-loop permission (Tier A)" made current.

Second chartering direction, owner, 2026-08-23, verbatim, redirecting the
claim-free eligibility condition that batches 0074 and 0078-A both carried:

> 我感觉这和claim的初衷有一点点区别。claim是帮助checker证明程序,如果checker
> 完备,那么任何一个claim都不会trap。如果checker不完备但是每一个claim都能人工
> 证明成立,那么这个程序也不可能trap(所有trap都是死代码)。唯一可能trap的情况
> 就是checker不完备(显然)+claim有错。也就是说一个*正确*的wf程序是不可能trap
> 的。只要trap了,说明程序本身是错误的,某一个claim是非法的。这时候我认为程序
> 设计不应该为错误的程序垫背。claim目前是唯一一个地方我们把信任交给非作者的,
> 比如AI写完了以后人类验收。所以我觉得应该假定claim是正确的,在这种情况下因为
> 要为了一个错误程序垫背而不给一个正确程序进行合理优化,我觉得这个取舍是不合
> 理的。

Consequence adopted: the schedule-unobservability guarantee of [PAR-1] and
[PAR-2] becomes **conditional on contract compliance**, mirroring [SCOPE-3]'s
conditional form. A false executed claim is already the sole writer-reachable
language runtime contract violation [SCOPE-4], so an execution containing one
is *erroneous*; for an erroneous execution the guarantee narrows to one
well-formed [DIAG-3] record of *a* claim whose predicate evaluated false, with
memory safety, abort-without-unwind, and the absence of external effects from
overlapped regions unchanged, and with *which* such claim it names left to the
schedule. Correct executions keep the guarantee whole. The claim-free
eligibility gate is deleted from both judgments.

## Known blockers at charter time (the research program's targets)

1. **Spec**: [PAR-1] judges statement pairs; iterations of one statement
   need new rule text — permission quantified over an index range, stated
   without naming any schedule. A further amendment to the CANDIDATE v0.34.
2. **Checker**: disjointness quantified over the symbolic index (iteration
   i's footprint vs iteration j's for all i != j), in tiers: pure/no shared
   writable; index-disjoint buffer writes (derived-index territory adjacent
   to Dig 9 and F6); reductions.
3. **Reduction law**: an accumulator recombines only under exactly
   associative operations (the hint's enumerated integer/boolean set);
   float accumulators never. Whether this is spec text or judgment-internal
   is a design decision.
4. **Exit edges**: `break`/early exit is the loop analog of condition 4 —
   v1 scope likely full-range counted `for` only.
5. **Lowering**: a permitted loop actualizes through the existing runtime
   (claim/publish/join/release, deques, two worlds) via an internal index
   split; chunking keys on runtime state, never constants.
6. **Traps/claims**: eligibility = transitively claim-free (v1 doctrine
   unchanged); the trap-order question dissolves for claim-free loops.
7. **Recorded hazards to consult**: [OWN-9] granularity and the c2-F4
   aliasing case (plan W4), the round-3 debate corpus, and the relevant
   mcts_mem node with its rejected alternatives before the design lands.

## Approval classes

Spec bytes: a CANDIDATE amendment is expected (branch-autonomous; full
packet at merge). Protected conformance: coverage for any new rule id will
be prepared and flagged. No new repository root entries.

Batch C adds no protected conformance or compliance change of its own: no
conformance case or verdict is added, modified, deleted, or renamed, and no
gate, collection path, invocation wiring, or gate-integrity test is touched.
It **does** supersede **both** of the phase-1 merge packet's "Required before
merge" items (`docs/ongoing/0074-proof-derived-parallelism.md`):

- **item 1, the [PAR-1] conformance annotation bytes.** Not a reason-text
  revision — the whole line changes. 0074 line 114 carries
  `"covered_by": "compiler-parallel-tests"` in 576 bytes; the line prepared
  below carries `"covered_by": "compiler-permission-judgment"` in 1 207 bytes
  with wholly different `reason` text. Applying 0074's item 1 as written
  installs the wrong protected line.
- **item 2, the [PAR-1] amendment recipe**, replaced by the [PAR-1] v2 recipe
  below.

Both are changes the owner applies once, at whichever merge lands [PAR-1], and
both are flagged below.

**Added paths, all inside existing directories, none at the repository root:**
`compiler/src/backend/tests/trap_latch.rs`;
`mcts_mem/whitefoot/parallelism/permission-judgment.alt/claim-free-eligibility.md`;
`research/investigations/proof-derived-parallelism/debate/d1-defense.md`; and
the four `mcts_mem` nodes the loop ruling and the redirect's rejected rival
needed — `parallelism/loop-permission.md`, its
`loop-permission.alt/{iteration-leaf-split,par1-amendment}.md`, and
`permission-judgment.alt/elision-rank-join-arbitration.md`. An earlier version
of this paragraph named one.

No batch of this record edits `spec/kernel-spec.md`. The candidate text, its
insertion points, its candidate SHA-256, and the native grammar-verifier
result are prepared below as merge-time application recipes, following the
batch 0074 pattern; the file's in-tree bytes and recorded digest are
unchanged, so the landed-archive gate stays green on the branch. **Batch C's
recipes supersede batch A's** — the section below carries the v2 text for both
rules and batch A's is not to be applied. **The
protected conformance coverage annotation [PAR-2] needs is not landed and is
not prepared here: it is prepared with batch B**, alongside [PAR-1]'s, so the
two annotations reach the owner in one protected-class audit rather than two.
Until then the repository coverage gate reports the same 135/136 it reported
before this batch, because the rule count is unchanged in tree. `make check`
therefore still exits nonzero at its `conformance` target and green everywhere
else; that red is [PAR-1]'s missing annotation and predates this batch, which
was checked rather than assumed — the coverage runner reports the identical
`135/136 ... 1 uncovered` with this batch's changes stashed and unstashed.
`make -C compiler check` is green before and after.

## Defects found in already-presented material

The adversarial probe of the loop surface found two defects in batch 0077's
landing, which is in the pending phase-1 merge packet. Both are corrected on
this branch and the packet must present the corrections with the material
they correct.

- **D-2, `--par` emitted invalid LLVM.** Any actualized overlap group sitting
  in a phi-predecessor block produced a module the host assembler rejects.
  It reached 2 of the 22 corpus units.
- **D-1, the counted-loop ledger hint was unsound at `give`.** Advice that
  can change a program's published bytes; developer-channel only, no accepted
  program or verdict moves.

## Executor log

(One line per completed unit.)

- D-2 fixed: the emitting world's overlap set is one stored slice, so a
  sequential clone can no longer label a phi predecessor with a `par.done`
  block it never emits. `--par` now compiles and links all 22 corpus units
  (2 failed before) and publishes the default build's bytes on every unit the
  lowering changes — **8** when this line was written, **9** at the repaired
  tip, the newcomer being `prefix_expression`, which the claim-gate deletion in
  C1 made eligible. The `--par` compile test is widened from `par_layout.wf` to
  the whole corpus and fails without the fix.
- D-1 fixed: `give` refuses the counted-loop hint, split out of the
  `DropExpression` arm and placed with `return` and `propagate`. The
  dossier's give-bearing loop publishes 27 where the advised split publishes
  80; it now draws no line, while the same loop without the give still does.
- D-3 fixed: the hint named the boolean combines `and, or, xor`, which are
  not spellings the language has; it now names `band, bor, bxor` as [OP-1]
  does. No case covered the boolean row, which is why the wrong spellings
  shipped; one now pins all three.
- Research and design closed: five dossiers (spec rule, checker mechanism,
  lowering, adversarial soundness, prior art) plus a value falsifier; the
  lead's synthesis is `research/investigations/proof-derived-parallelism/
  loop/DESIGN.md` with its promoted probes and the falsifier table. Ruling:
  v1 loop permission is the REDUCTION (full-range counted `for`, claim-free
  exit-free body, one accumulator under a normatively enumerated
  exactly-associative set); the map is deferred as legal-but-worthless at
  today's place granularity, with a named re-entry condition. The
  parallelism decision record entered `mcts_mem` (dda51964) before the
  design landed, per the tree discipline.
- A1, the loop permission judgment landed: `semantic/loop_permission.rs`
  judges every counted `for` against the four loop conditions and reports
  `PermittedEligible`, `PermittedNotActualizable`, or a denial naming the
  condition. It supersedes `semantic/loop_hint.rs` in place rather than
  standing beside it: one body walk now produces both the verdict and the
  split advice, so the two can no longer disagree about what a loop carries.
  The judgment shares the window judgment's [EFF-2] projection through a new
  `CallProjection`, so neither grows a private copy of the footprint or of the
  [OWN-7] overlap relation. It consults no entailment state.
  **Zero emitted bytes, verified rather than asserted.** All 38 corpus and
  bench sources emit byte-identical modules before and after, in the default
  and the `--par` world both; and over the sweep of every source that checks
  standalone, every `PAR` pair and chain line is byte-identical with the new
  loop lines filtered out, so no pair verdict moved among them. (The population
  was recorded here as "703 sources", which was the count of non-archive `.wf`
  files at `13ffab4c` rather than the count of sources the sweep actually
  judged; the census section below names both.)
- A2, the [PAR-2] candidate prepared: full rule text, insertion point,
  candidate SHA-256, and the native grammar verifier's grammar-preserving
  result, recorded above as the merge-time recipe. `spec/kernel-spec.md` is
  untouched in tree.
- A3, the attack battery landed as tests: `semantic/tests/loop_permission.rs`
  carries **34** cases at the repaired tip (33 when A3 landed at `13ffab4c`;
  C1 replaced its two `..._not_actualizable` cases with three), each denial
  asserting the condition that judged it, plus the ledger-line assertions in
  `driver.rs`. **Every attack the lead reproduced from the batch's adversarial
  dossiers is a case** — the dossiers themselves live in the lead's scratch and
  are not in the repository, so that is a statement about what was reproduced
  and not a property a reader can check. Every admitted combine has a grant.
- A conservatism in already-presented material, fixed by the supersession
  rather than by a separate correction: batch 0077's hint refused every loop
  containing a `give`, including one whose `give` delivers into a `value_if`
  the loop body itself opens and therefore leaves nothing. That is the
  under-advising direction, not the unsound one D-1 named, so it moved no
  published byte; the judgment now counts the value initializers the body
  opens and tells the two `give`s apart, with a test pinning each direction.
- One over-refusal caught before it shipped: the first draft asked whether any
  holder in the function reached the accumulator, which is flow-insensitive
  and denied a sound reduction whose result is borrowed *after* the loop. The
  question is unnecessary — a borrow formed inside the body is itself a
  counted read, and a borrow formed outside it makes the direct write an
  [OWN-5] borrow conflict — so the check was removed and the argument that
  the read count is complete is recorded in the module doc, with a test
  pinning both directions.
- B1, the split lowering landed: a permitted counted loop becomes one new
  `IrOperation::LoopSplit` and two synthesized `IrFunction`s — a *chunk*,
  which is the loop itself seeded by its first parameter, and a *splitter*,
  whose two recursive halves are an ordinary overlap group. Nothing about the
  hand-out changed: the claim, the frame, the thunk, the deque, the join, and
  the two-world selection are the ones a permitted pair already used. The
  overlapped world renders the site as one runtime allowance query plus a call
  to the splitter; the sequential world renders it as a call to the chunk's
  clone, which is the loop. The splitter's `IrOverlap` is produced by this
  lowering and is **not** a second permission judgment — it actualizes the
  loop's own permission, and each world resolves its phi labels against its
  own overlap set, which is the `eabefcc8` lesson honoured by construction.
- B2, the runtime allowance landed: one new entry point
  `wf__par_split_budget(span, weight)` in `par_runtime.c`, asked **once per
  loop entry** and never per iteration, answering from a lane count settled
  once per process, this thread's own deque occupancy, and the caller's static
  weight estimate. Its two policy constants sit beside `WF_PAR_SPIN_ROUNDS`
  and are named for the packet: `WF_PAR_SPLIT_OVERSUBSCRIBE` = 16 and
  `WF_PAR_SPLIT_WORK_PER_CHUNK` = 1200000. Both are measured, and the
  measurements are written into the file beside them.
- B3, the compile-time declines landed. A lane frame is 256 bytes, so a split
  whose frame is wider would be refused every lane forever and sequentialize
  with no report; the lowering measures the frame at emission and declines
  with a ledger line naming the width. The bound is restated once in the IR as
  `LANE_FRAME_BYTES` and pinned to the runtime's `WF_PAR_FRAME_BYTES` by
  `the_compile_time_frame_bound_is_the_runtimes`. The lowering's own ledger
  lines (`PAR split`) ride the judgment's under `--par --par-ledger`, so an
  actualization is as reported as a permission.
- B4, three findings from this batch's own adversarial review of its diff,
  all fixed before the commit landed:
  - **A silent loss of already-granted parallelism.** A chunk was built with
    no overlap group at all, so a permitted [PAR-1] pair *inside* a loop body
    vanished the moment the loop was split — correct, but sequential, and
    reported nowhere. The whole permission table now travels into the chunk,
    whose body is the loop's own statements, so a pair inside them is
    actualized exactly as it was before the split existed. Reproduced on a
    loop carrying both: the chunk claims a lane where it previously claimed
    none, and the program publishes the same answer at every worker count.
  - **A missing operand check.** `emit_loop_split` checked only that the
    endpoints were `u64`, where the sibling hand-out path checks every
    argument against the callee's declared parameters. One lowering builds
    both lists, so nothing could reach it today; the check is there so a later
    lowering defect stops at the emitter instead of reaching the assembler.
  - **Two tests that could not have failed.** No fixture captured anything —
    every one had an empty capture list — and every fixture folded under
    `+wrap`, so a swapped capture or a wrong identity element was invisible to
    `cargo test`. One new fixture closes both: three enclosing values used
    asymmetrically, folded under `ixor`, compared against the default
    compilation of the same source with the grant count read.
- **A defect found and fixed inside this batch's own work.** The first weight
  estimator read every `break` as a back edge — the builder creates a loop's
  exit block *before* its body, so a break jumps backwards in block index —
  and a body with three breaks weighed 65 536 times what it costs. The
  grid loop's weight read 860 510 instead of 305. Back edges are now tested by
  their definition (delete the target; ask whether the jumping block is still
  reachable from the entry) rather than approximated by block order. This
  moved no accepted program and no published byte, only the allowance, and it
  is exactly why the constant below was calibrated after the fix and not
  before.

- C1, the claim-free actualizability gate deleted from both judgments. The
  window judgment's `claim_closure` reverse walk, its `reaches_claim`
  precomputation, its `ClaimWitness`, and the `PermittedNotActualizable`
  verdict class are gone, and with them the whole call-graph machinery
  `permission.rs` carried: nothing in the file needs `direct_callees` any
  more, so `Program` is now the functions and their signatures. The loop
  judgment loses its `roots` and `claims` collection the same way, and a
  `claim_stmt` in a loop body is now an ordinary statement whose predicate is
  read like any other expression. **Every other condition is untouched**, and
  the one place a claim still refuses is unchanged and now pinned by a case
  that says so: a claim written *between* the two calls of a window is an
  exit-bearing interposed form and still denies under condition 4.
- C2, the trap latch. `wf_trap` — the one path a false claim reaches — takes a
  process-wide latch by `cmpxchg` before its first byte. The winner writes its
  complete [DIAG-3] record and aborts; a thread that loses parks on a volatile
  load, and the winner's abort takes it down with the process. The cost off the
  trap path is zero: the latch is one private global that nothing outside
  `wf_trap` reads or writes.
- C2 corrected after the batch audit (`5d73ae59`). As landed at `f6c55a9d` the
  latch went into `wf_trap` on the sole condition that the module carries a
  `claim`, with no `--par` predicate anywhere on that path — so it changed the
  bytes of 21 default compilations and built a parked lane into a world where a
  lone thread can never race anyone for the record. It is now conditional on
  the module handing a call out to a worker lane, which is exactly when the
  module can have more than one thread inside it; every other module emits the
  pre-latch trap writer it emitted before this path existed. The evidence is
  under "What the corpus does" below.
- C3, the tests. `backend/tests/trap_latch.rs` carries the erroneous-execution
  guarantee: forty runs of a two-false-claim race at `WF_WORKERS=4`, each
  producing exactly one record whose [DIAG-3] shape is parsed rather than
  substring-matched; the sequential schedule's byte-identical record at
  `WF_WORKERS` 0 and 1; a single-false-claim program byte-identical at five
  worker settings; and **the control that gives the other three teeth** —
  the same emitted module with the latch's branch forced, which writes two
  records. Measured detection for the control was 200 of 200 runs.
- C3 corrected after the batch audit (`f4188977`). The control counted a run as
  caught when the record channel held anything other than one well-formed
  record — which an **empty** channel also satisfies, so a later change that
  made the defeated build die before writing would have kept the control green
  while proving nothing. A caught run now has to carry exactly two records,
  each parsed as a [DIAG-3] line naming one of the two claims.
- C3, the payoff pinned rather than asserted:
  `the_claim_bearing_fold_is_granted_lanes_and_publishes_the_same_bytes`
  reads the runtime's own grant counter for `par_layout.wf`, because every
  other case in that file passes just as well against a runtime that refused
  every lane.
- C4, the [PAR-1] v2 and [PAR-2] v2 recipes, below, superseding both the
  phase-1 packet's [PAR-1] recipe and batch A's [PAR-2] recipe.
- **A test that could not have failed, found and replaced.**
  `only_the_claim_free_fold_is_handed_out` asserted that the claim-bearing
  fold's body did not contain `"wf_par"` — with one underscore. Every runtime
  symbol reserves `wf__par_`, so no emitted module can contain that string and
  the assertion was vacuous from the day it was written: the case's stated
  subject, that a claim-bearing fold names no part of the runtime, was never
  checked. Its replacement checks `wf__par_` and puts the negative control on
  `@wf_measure_band`, the callee that actually carries the claim and sits in
  no permitted pair.
- **The doctrine this redirect restores was already on file.** DESIGN.md §0
  records the owner's ruling of 2026-08-21 — "如果程序的trap只可能由审计失败
  的程序产生……我们应该不需要考虑这种情况" — and DESIGN.md §1 then derived
  claim-free eligibility from it, which is the opposite conclusion. Both §1
  bullets are struck through in place with the withdrawal dated, rather than
  rewritten, so the dossier still reads as the contract it was.

## What the redirect actualized, measured

**Machine was not quiet.** Apple M4, 10 cores; other agent sessions were
running throughout. One-minute load average 5.1 before the timing pass and 3.4
after. Every ratio below is subject to the protocol's 0.83x-1.20x unresolved
band, and each was reproduced in at least two independent interleaved
rotations. `par_layout.wf` compiled by the branch-tip compiler and by the
pre-redirect compiler at `ddf1d139`, both `cargo build --release`, both linked
by `/usr/bin/clang -O2` with the same embedded runtime.

**Verdicts that moved, over every `.wf` source that checks standalone.** The
repository holds 776 `.wf` files; **312** of them check on their own and are
the judged population, and both compilers were run over all 776 with
`--par --par-ledger` so nothing that could produce a line was excluded by hand.
**Eleven pairs moved from `not-actualizable` to `eligible`**, each gaining the
`chain` line its two members now compose into; 319 ledger lines became 330.
**No denial moved, no verdict narrowed, and no loop verdict moved anywhere** —
the loop half of the redirect widens nothing in today's corpus, because no
counted loop the repository contains was refused for a claim. Newly eligible in
`tests/programs`: `par_layout.wf` (`layout_banded`), `prefix_expression.wf`
(`evaluate`), `recursive_tree.wf` (`count`). One conformance case,
`x-borrowed-pool-tree-run.wf` (`checksum`, four claim sites). Seven research
probes, listed in `probes/README.md`. The two multi-file `raw_deflate` units
are in neither sweep; their six distinct lines are listed under "What the
judgment reaches today" and are all denials.

*Measured before the rebase onto `main`, and kept as measured.* The two
compilers this paragraph compares are the branch tip and `ddf1d139`, both
pre-rebase, so 776/312/319/330 are that tree's numbers and the eleven-pair
delta is that comparison's result. It is the evidence for the redirect and
stands as taken. The rebased tree's own census is 765/270/161 with 19 loop
verdicts — restated with what moved under "The corpus-wide census" below — and
it does not contradict anything here: no denial moves in either, and the
population shrank only because v0.34 retired the claims of seventeen probes.

**Those seven probes' disposition, which `probes/README.md` deferred to this
record: all seven are kept**, and the disposition is now written in that file
rather than here, under its own standing rule that a probe is deleted when the
decision or finding it settles is superseded. What the redirect superseded in
each case is the `not-actualizable` verdict line the probe's paragraph quoted,
not the finding the probe settles — except `p7_dyn.wf`, whose conclusion (that
a parameter-sized buffer's shape is permanently out of reach) is reversed, and
which is kept because it is the only carrier of both the shape and the
reversal.

**Grants, read from the runtime's own counter.** `par_layout.wf`, functions
that hand out: **2 to 3**.

| WF_WORKERS | 1 | 2 | 4 | 8 | absent |
|---|---|---|---|---|---|
| grants before | 0 | 1 052 | 6 128 | 13 709 | 14 694 |
| grants after | 0 | 2 772 | **12 617** | **28 251** | 29 270 |

Published bytes are identical in all ten runs. `dd3b6c59c5c38307` is the
**SHA-256 of `par_layout.wf`'s standard output**, truncated to sixteen
characters, not a value the program publishes — the program publishes
`420a993efa7437a1 41fa962893d45299` with no trailing newline, and
`sha256` of exactly those bytes begins `dd3b6c59c5c38307`. Naming the hash
function matters here: without it the digest cannot be checked from this record
alone, which is the whole point of quoting one.

**Wall clock**, minimum of 18 interleaved rounds, no cell twice in a row:

| WF_WORKERS | before (s) | after (s) | after/before |
|---|---|---|---|
| 1 | 0.8076 | 0.9595 | ~~**1.19 (regression)**~~ withdrawn, see below |
| 2 | 0.5595 | 0.4640 | 0.83 |
| 4 | 0.5845 | 0.3291 | **0.56** |
| 8 | 0.5442 | 0.2380 | **0.44** |
| absent (shipped default) | 0.5607 | 0.2278 | **0.41** |

One published-byte digest across every cell and every round.

### WITHDRAWN 2026-08-23: the 1.19x sequential-world flag was load contamination

**This section flagged a 1.19x regression in the `--par` build at
`WF_WORKERS=1`. It does not reproduce, and the flag is withdrawn.** The row is
struck in the table above rather than deleted, because the number was presented
to the owner and the correction belongs where the claim was made.

Nine independent rotations now bear on that one cell — four by the batch
audit's finder, three by its refuter, and two run for this repair — and none of
them finds it:

| rotation | rounds | load | `--par` W1 after/before (min) | (median) |
|---|---|---|---|---|
| finder 1 (driver link) | 15 | 2.68 | 1.000 | 0.997 |
| finder 2 (driver link) | 30 | 2.59 | 0.996 | 1.001 |
| finder 3 (manual clang) | 24 | 1.71 | 0.997 | 1.002 |
| finder 4 (manual clang) | 20 | 1.95 | 1.004 | 0.998 |
| refuter 1 | ~20 | 9.6-11.7 | 0.921 | — |
| refuter 2 | ~20 | 9.7-104.8 | 0.891 | — |
| refuter 3 | 22 | 5.88-5.74 | 1.010 | 1.000 |
| repair 1 | 20 | 4.06-4.44 | 1.002 | 1.001 |
| repair 2 | 24 | ~4.0 | 0.989 | 0.988 |

The two repair rotations are `tests/programs/par_layout.wf` compiled and linked
by the repaired tip and by the pre-redirect compiler at `ddf1d139`, four cells
interleaved with the order alternating every round and no cell run twice in a
row. The same harness reproduces the win (`--par` at the shipped default,
after/before 0.435 for the finder against the record's 0.41) and both of the
record's 1.00x controls, and does not reproduce the regression.

**What the record's own cells say about themselves.** They were taken at a
one-minute load average of 5.1 before the timing pass and 3.4 after, and the
flagged row's "after" cell (0.9595 s) sits 19% above its own "before" (0.8076
s) — where the eighteen cells that agree across rotations agree to under 1%.
The refuter's minimum for the tip at `WF_WORKERS=1` on a loaded machine is
0.805-0.810 s, which is essentially the record's *before* value. Load
contamination of that one cell is the reading the measurements support.

**The noise floor, measured rather than assumed.** After the repair below, the
tip and the pre-redirect compiler emit **byte-identical** default modules for
`par_layout.wf` (`cmp` on the two `.ll` files, and both publish SHA-256
`0e28f3d229be2e69…`). The same harness still reports 0.990-0.997 between the
two binaries built from those identical bytes. That is this measurement's
resolution: about ±1%, an order of magnitude below the withdrawn 19%.

**The property `e82c113f` established stands.** Asking for `--par` costs a
program nothing when it gets no lane: across all nine rotations the `--par`
build at `WF_WORKERS=1` is within 1.1% of the default build, before the
redirect and after it. The earlier sentence in this record saying the redirect
dents that property is withdrawn with the flag.

What remains true and is not withdrawn is the *mechanism* the flag was
attributed to: `layout_banded` is now on a path to a handed-out call, so the
two-worlds lowering gives it a sequential clone and the module carries the fold
twice (the tip emits `@wf__par_seq_layout_banded` and 1 708 IR lines against
1 622). That is a real change to the module; it is not a measurable cost.

## Carried for another record: the L1 depth flag

Not this batch's work, landed by this batch's commits because this branch is
where the file is editable. Batch 0077's L1 item flagged the shipped-default
change with a no-pessimization check over twelve configurations that measures
**time only**; the **depth** dimension was omitted. The lead measured it and
the paragraph is now in `docs/ongoing/0077-night-par-ceiling.md` at the L1
item: an adversarial spine shape loses about 3x of its recursion ceiling at the
default (of which 1.5x is structural to the overlapped lowering and the rest is
an LLVM interprocedural-constant-propagation accident, per Dig 1), a realistic
tree shape is at or above sequential but not yet deterministically so, the
`WF_WORKERS=0/1` opt-out restores the full ceiling, and the bare-SIGSEGV
presentation predates the change in every build.

**The frozen phase-1 branch is not corrected by this.** `par/proof-derived-parallelism`
at `27e02b1f` still carries the incomplete flag, and the phase-1 merge packet
is drawn from there, so the owner reviewing that packet sees the time-only
check unless the correction is carried across.

## The alternative this redirect rejected, for the design tree

The lead records the re-decision; this is the executor's statement of what was
weighed, so the tree entry is not written from memory.

**Rejected: elision-rank join arbitration.** The alternative to a conditional
guarantee is to keep the unconditional one and pay for it — arbitrate, at the
join, which of several trapping lanes owns the report, by a rank derived from
each lane's position in the source order the elision defines, so the record an
overlapped execution writes is the record the sequential execution would have
written. It is on file from batch 0074 — the EFF-4 two-half ruling and Lemma C
(join arbitration = sequential composition) of the round-3 debate's defense —
and was deferred there rather than refused.

**That citation named a file that was not in the repository, and now is.** It
pointed at `debate/d1-defense.md`, which existed only in the lead's scratch;
`research/investigations/proof-derived-parallelism/probes/README.md` states the
governing doctrine itself — "a claim whose evidence has evaporated is an
assertion" — so relying on the file while leaving it outside was the defect.
It is promoted verbatim to
`research/investigations/proof-derived-parallelism/debate/d1-defense.md`, with
a provenance header and with three machine-local paths in its reproduction
appendix replaced by placeholders. The second citation, at `../DESIGN.md:70`,
is corrected in the same change — it also still called the arbitration
deferred, which this record refuses.

It is refused now, on the ground the owner's direction states: every byte of
that machinery exists to make a *defective* program's report reproducible, and
it is paid for by every *correct* program, which cannot reach it. The cost is
not only the arbitration — it is a coordinator, a rank the lowering has to
compute and carry, parked lanes with a wakeup protocol, and a join that can no
longer be a plain wait-for-all. Against that, the latch is one global and one
`cmpxchg` on a path a correct program never executes, and the reproduction the
arbitration would have bought is available for free at `WF_WORKERS=1`, which
is deterministic and which a defective program's owner can always run.

The narrower rejected variant, **write every record and let them interleave**,
is refused for a different reason: [DIAG-3] fixes exact record bytes, and two
concurrent writers produce neither one record nor two clean ones. The measured
control in `trap_latch.rs` shows what it produces instead — two records in 200
of 200 runs on this machine, which is the good case; a partial interleave is
the bad one.

## What the judgment reaches today

Every counted loop the repository contains, judged. `tests/programs` holds 12
counted `for` loops and **the rule permits none of them**:

| loop | verdict | why |
|---|---|---|
| `byte_string.wf:62` `@copy` | denied 2 | element write into an enclosing buffer |
| `byte_string.wf:80` `@append` | denied 2 | an expression statement (`bs_push(...)`) in the body |
| `byte_string.wf:95` `@concat` | denied 2 | an expression statement in the body |
| `dir_walk.wf:42` `@copy` | denied 2 | element write |
| `growable_vec.wf:20` `@copy` | denied 2 | element write |
| `growable_vec.wf:37` `@append` | denied 2 | an expression statement |
| `growable_vec.wf:49` `@seed` | denied 2 | an expression statement |
| `raw_deflate_boundary.wf:28` `@append` | denied 2 | element write |
| `sha256_abc.wf:54` `@copy_block` | denied 2 | element write |
| `sha256_abc.wf:58` `@extend_schedule` | denied 2 | element write |
| `sha256_abc.wf:82` `@compression_rounds` | denied 1 | `set h = g;` reduces nothing |
| `wfgrep.wf:132` `@append` | denied 2 | element write |

**The corpus-wide census, re-derived after the rebase onto `main`, with its
population named.** The repository holds **765** `.wf` files, of which **270**
check standalone — the rest are negative conformance cases that are meant not
to check, 71 files under `archive/`, and multi-file fragments that need their
unit. Sweeping those 270 with `--par --par-ledger` gives **161 `PAR` lines**,
of which **19 are loop verdicts: 5 permitted, 14 denied**, and **0** are
`not-actualizable`.

The 5 permitted are two loops of the conformance case
`ent3-pos-s11-counted-range-run.wf` and three across three probes this batch
promoted — `m1_pair_in_for` (1), `p4_split_equiv` (1), and
`r2_grid_loop_d21_w256` (1) — every one of them a `+wrap` reduction.

*What the rebase moved, and what it did not.* The pre-rebase figures were 776
files, 312 checking standalone, 330 `PAR` lines and 21 loop verdicts (7
permitted). Denials are unmoved at **14**, and `not-actualizable` is still
**0**. The standalone-checking population fell because v0.34 admits a claim
only as a local non-derivable residual, so the seventeen claim-bearing probes
of `probes/` — retired rather than rewritten, see that directory's README —
stop checking, and `r1_mandelbrot_for` is one of them. Its two permitted loops
are the whole of the drop from 7 to 5; no loop the judgment still sees changed
its verdict.

An earlier version of this paragraph said "18 loop verdicts: 4 permitted, 14
denied" over a "703-source sweep". Both figures were correct at `13ffab4c` and
were not re-derived afterwards: 703 is 690 tracked non-archive `.wf` plus the
13 untracked generated `bench/wf` sources, which is that commit's non-archive
population; the same population at this tip is 705. The three loops that appear
between them are `r1_mandelbrot_for`'s two and `r2_grid_loop_d21_w256`'s one,
both promoted by later commits of this same batch — the paragraph two below
already credits `r1_mandelbrot_for`'s two, which is the contradiction that
should have caught it. Neither 703 nor 776 was ever the judged population; 312
is.

**Multi-file units are in no per-file sweep, and are listed here instead.** The
corpus has two, both `raw_deflate` variants, and neither appears above because
neither `raw_deflate_boundary.wf` nor `raw_deflate_vectors.wf` checks alone.
Compiled as units they add **6 distinct `PAR` lines, every one a denial**:

- the boundary unit gives 3 — the loop denial at `raw_deflate_boundary.wf:28`
  (condition 2, the element write the table above carries) and two pair
  denials in `raw_deflate_dynamic_decode.wf` at `:104` and `:250`;
- the vectors unit gives 5 — the same two pair denials, plus three more in
  `raw_deflate_vectors.wf` at `:357`, `:385`, and `:400`, all condition 2 on an
  interposed region.

So "no pair verdict moved anywhere" and "no loop verdict moved anywhere" are
claims about the 312 standalone-checking sources. All eight lines the two units
produce are denials, so nothing is likely hiding there, but the claim is scoped
rather than left to read as whole-repository.

The bench sources say the same thing from the other side: not one of the 13
carries a counted loop at all, because every one of them is the hand-written
recursion the owner objected to.

The other half of the picture is `r1_mandelbrot_for.wf`, promoted beside
DESIGN.md: `tests/programs/mandelbrot_grid.wf` with its two hand-counted
`loop`s written as counted `for`s. It exits 0 — its
`ieq(escaped_points, 2437_u32)` claim holds exactly as the original's does —
and **both** of its loops are permitted under `+wrap`. So the rule does reach
a program the project wrote, as soon as that program is written in the form
the language calls the default one. That is the owner's charter, answered on
real code rather than on a fixture.

Say this to the owner without dressing it: **the reduction rule fires on zero
loops of the real corpus as it stands**, which is the same number batch 0077 measured for
the hint and for the same reason — every counted loop the project has written
is a copy into a buffer, a push through a trapping callee, or a sequential
recurrence. The justification for the rule is the owner's principle that the
default form must be the optimal form, the `grid` family's measured 6.5x, and
programs not yet written. It is not corpus payoff, and a 0-of-12 number that
went unstated would be the kind of silence this ledger exists to end.

The table partitions as **7 element-write (the deferred map), 4 expression
statements, and 1 non-reduction** — twelve, and the counts an earlier version
of this paragraph gave did not add up to it. None of the three groups is in
this batch's scope. The four expression statements are refused by the window
judgment too and for the same unresolved [STOR-3] release; admitting them would
move `byte_string` and `growable_vec` from "refused for a reason about
`bs_push`" to "refused for a reason about the buffer", which is more honest but
no more permitted. And two of the map loops (`raw_deflate_boundary`, `wfgrep`)
also carry a `return`, so they need the exit condition relaxed as well as the
place work.

## The measurement: the default form reaches the hand-written form

The charter's whole claim in one table. `r2_grid_loop_d21_w256.wf`, promoted
beside DESIGN.md, is the bench family's `grid_d21_w256` with its recursive
`tile` written as the counted `for` a writer reaches for; everything else in
the two programs is the same text. The twin is the hand-written recursive
split the owner objected to having to write. Both were compiled by the
branch-tip `whitefootc`, linked by `/usr/bin/clang -O2`, and run under the
bench harness's interleaved rotation at the oracle protocol: minimum of 18
rounds, no cell run twice in a row.

**Machine was not quiet.** Apple M4, 10 cores; other agent sessions were
running on it throughout. One-minute load average 4.04 before the pass and
4.29 after. That is what the interleaved minimum-of-N is for, and the
protocol's 0.83x-1.20x unresolved band applies to every ratio below; a second
independent pass of 15 rounds, taken earlier at load 2.74, reproduces every
cell within four percent.

| cell | min (s) | loop / twin | loop / rayon |
|---|---|---|---|
| `wf_loop_seq` | 0.5079 | 0.98 (u) | — |
| `wf_loop_par/1` | 0.5061 | 0.98 (u) | — |
| `wf_loop_par/2` | 0.2525 | 0.95 (u) | 0.95 (u) |
| `wf_loop_par/4` | 0.1360 | 0.98 (u) | 0.99 (u) |
| `wf_loop_par/8` | 0.0889 | 0.99 (u) | 0.95 (u) |
| `wf_loop_par/default` | **0.0777** | **1.00 (u)** | **0.95 (u)** |
| `wf_twin_seq` | 0.5178 | | |
| `wf_twin_par/1` | 0.5176 | | |
| `wf_twin_par/2` | 0.2662 | | |
| `wf_twin_par/4` | 0.1383 | | |
| `wf_twin_par/8` | 0.0898 | | |
| `wf_twin_par/default` | 0.0776 | | |
| `rs_seq` | 0.4841 | | |
| `rs_rayon/2` | 0.2644 | | |
| `rs_rayon/4` | 0.1379 | | |
| `rs_rayon/8` | 0.0935 | | |
| `rs_rayon/default` | 0.0819 | | |

**The loop form reaches the hand-split form's numbers.** Every one of the six
loop-versus-twin ratios is inside the unresolved band, and the two cells that
matter most — the shipped default and eight lanes — are 1.00x and 0.99x. The
same is true against `rayon`. The loop form scales 6.54x from its own
sequential build (0.5079 to 0.0777), and the `--par` opt-in costs it nothing
at one lane (0.5061 against 0.5079), because that run takes the sequential
world where the split does not exist.

**Byte comparison.** One sequence — the sixteen hexadecimal characters
`000000000033517d` that the program itself publishes on standard output — from:
the loop form's default compilation; the loop form's `--par` build at
`WF_WORKERS` 0, 1, 2, 3, 4, 5, 8, 10, 16 and unset; the hand-split twin both
ways at all of the same; and the Rust twin sequential and under rayon at 2, 4,
8, and its own default pool. Every run of the timed rotation above produced
standard output whose **SHA-256 begins `a6522da3cd244c2c`** — that is a hash of
the published bytes, not a second published value, and it is exactly
`sha256("000000000033517d\n")` truncated to sixteen characters — all seventeen
cells. `r1_mandelbrot_for.wf` likewise exits 0 — its
`ieq(escaped_points, 2437_u32)` claim holds — at all ten worker settings and
in its default compilation.

### The residual, attributed

The first pass, taken at `WF_PAR_SPLIT_OVERSUBSCRIBE` = 4, put the loop form
consistently 11% behind the twin at the shipped default: inside the
unresolved band, but in the same direction twice. Two candidate causes: the
one extra `narrow` per point that the loop form pays and the twin does not, or
the chunk count. The discriminating experiment is to move only the chunk count
and see whether the gap moves with it — the conversion is unaffected by it.
Minimum of 40 interleaved rounds at the shipped default:

| chunks per lane | 4 | 16 | 64 | twin |
|---|---|---|---|---|
| min (s) | 0.0822 | 0.0754 | 0.0732 | 0.0763 |

Monotone in the constant, and at 16 the loop form already matches the twin. So
the residual was grain, not the loop form: at 4 chunks per lane this
4-performance-6-efficiency machine leaves the slow cores straggling with
nothing for the fast ones to steal. The constant moved to 16, which is where
the table flattens; 64 buys a further three percent and cuts the range finer
than any measurement asks for.

### The over-split hazard, closed by measurement rather than by argument

Splitting a range that is not worth splitting is a real regression, so the
work term of the allowance was calibrated directly rather than assumed. A
counted loop of estimated weight 150 was swept over its width at a fixed total
amount of work, and the split compared against the same program's sequential
build at the shipped default:

| width | 2 000 | 8 000 | 32 000 | 128 000 | 512 000 |
|---|---|---|---|---|---|
| split (probe allowance) | 0.0831 | 0.0269 | 0.0116 | 0.0084 | 0.0072 |
| plain | 0.0205 | 0.0196 | 0.0201 | 0.0201 | 0.0210 |
| | 4.1x loss | 1.37x loss | 1.73x win | 2.4x win | 2.9x win |

The crossing is near a width of 16 000, which sets
`WF_PAR_SPLIT_WORK_PER_CHUNK` = 1 200 000 instruction-equivalents. Re-measured
with the shipped constants, the allowance now refuses exactly where splitting
loses and takes it where it wins:

| width | 2 000 | 8 000 | 32 000 | 128 000 | 512 000 |
|---|---|---|---|---|---|
| shipped | 0.0198 | 0.0197 | 0.0138 | 0.0077 | 0.0071 |
| plain | 0.0203 | 0.0192 | 0.0192 | 0.0192 | 0.0201 |
| | parity | parity | 1.39x win | 2.5x win | 2.8x win |

### The recursion depth is the allowance, not the range

The splitter descends exactly `budget` levels and `budget` is the base-two
logarithm of the chunk count, which the allowance caps at
`WF_PAR_MAX_LANES * WF_PAR_SPLIT_OVERSUBSCRIBE` = 1024. **The bound is ten
frames, for any range up to 2^64 iterations.** That is a theorem about the
emitted shape rather than a policy, and it is strictly better than the pair
lowering's, whose depth follows the writer's own recursion.
`a_split_loop_costs_a_bounded_stack` runs a split loop under a 512 KB stack
limit at eight lanes.

### What the corpus does, and what this batch emitted everywhere else

Re-run at the repaired tip: `tests/programs` holds **twelve** counted `for`
loops, **none permitted and therefore none split**. The whole-corpus `--par`
compile-and-publish test is unchanged by this batch because it has nothing to
change. The measurement above is on the promoted probes, which is where the
shape exists.

The earlier version of this paragraph said eleven, and the method is why.
`raw_deflate_boundary.wf` cannot be compiled on its own — standalone it fails
with `[TYPE-5] UnresolvedUse { spelling: "InflateError" }` — so a sweep that
compiles one file at a time silently drops its loop and reports eleven. Judged
with the other three files of its unit it produces exactly the row the table
below carries. Every count in this record that is derived from a per-file sweep
carries the same blind spot, and each such population is now named where it is
used.

**Zero emitted bytes outside the shape, verified rather than asserted.** Every
standalone source of `tests/programs`, `bench/wf`, and `loop/probes` was
emitted by the batch-B compiler at `ddf1d139` and by the batch-A compiler at
`0314c01e`, in the default compilation and under `--par`, and the modules
compared byte for byte: **74 identical, 4 differing.** The four are `--par`
only and are exactly the four programs carrying a loop [PAR-2] permits —
`m1_pair_in_for`, `p4_split_equiv`, `r1_mandelbrot_for`,
`r2_grid_loop_d21_w256`. **No default compilation of anything moved one byte**,
which is the property the whole `OverlapLowering::Off` path exists to have.

**That sentence was true when written, was falsified inside this batch, and is
true again.** `f6c55a9d` emitted the trap latch on the sole condition that a
module carries a `claim`, with no `--par` predicate on the path, so it changed
**21 of the 39 default compilations** in the population above — every module of
the corpus that carries a claim gained `@.wf_trap.latch`, a `cmpxchg`, and a
`park:` block. The batch audit found it and the record did not: this paragraph
was never revisited after batch C landed. The repair makes the latch
conditional on the module actually handing a call out to a worker lane, which
is false for every default build and for a `--par` build that actualizes
nothing. At the repaired tip, against the pre-latch compiler at `ddf1d139` over
the same 39 sources in both worlds (78 comparisons): **all 39 default
compilations byte-identical**, 10 `--par` modules differing, and those 10 are
exactly the modules that both carry a claim and hand out — the only 10 that
name `@.wf_trap.latch` anywhere.

### One gate flake, chased rather than waved through

The first full run of `make -C compiler check` after the review fixes failed
one case — `the_runtime_replaces_the_modules_weak_refusal`, on "the runtime
granted no lane". That run was concurrent with this batch's own timed
measurement pass, at a one-minute load average of 7.77 on a ten-core machine.
The case asserts that a pool actually took work, and a saturated machine lets
the offering thread run every one of its own offers at the join, which is the
runtime's documented common case and grants nothing.

It was not accepted as a flake on the strength of that story. The case passes
in isolation three times over; its fixture contains no counted `for` at all, so
`split_counted_range` is never reached for it; the corpus program that
exercises the same path, `par_layout.wf`, is byte-identical to `0314c01e` in
both worlds by the comparison above; and `sequential_clone_set`'s new predicate
reduces to the old one exactly for a program with no synthesized function. The
gate is green on a quiet machine. **The load sensitivity is a real property of
that case and predates this batch**; it is recorded here rather than fixed,
because tightening it belongs with whoever owns the parallel test harness.

## FLAGGED: the protected conformance coverage annotations, prepared not landed

**Protected class touched at merge, not on this branch.** `make check`'s
`conformance` target reports `135/136 rules covered ... 1 uncovered`, and the
one uncovered rule is `PAR-1`. That red predates batch 0078 — batch A verified
it with its own changes stashed and unstashed — and [PAR-2] will make it two
the moment the rule activates. Both annotations are prepared here as exact
bytes and **neither is added to `tests/conformance/manifest.jsonl` on this
branch**, so the file's protected bytes are untouched and the audit at merge
has one before/after rather than two.

Exact before: `tests/conformance/manifest.jsonl` is **540 lines**, of which 32
are annotations. The annotations are not a trailing block — they sit at lines
150-158, 177-186, and 412-424, interleaved with case rows that start at line 4
and run to 540. Line 424 is the **last annotation**, the `GATE-2` one; line 425
is the case `run-sysexit-code-0` and 116 case rows follow it. (This census was
re-derived after the rebase onto `main`: the pre-rebase figures were 536 lines,
30 annotations at 148-156, 175-182 and 408-420, and the same case at line 421.
Main's v0.34 added two annotation rows and two cases; the block structure, the
row that follows the last annotation, and the 116 rows after it are unmoved.) The file contains
no `PAR-1` and no `PAR-2` row of any kind
(`grep -c '"rule": "PAR-' tests/conformance/manifest.jsonl` is 0). An earlier
version of this paragraph said the file "ends at line 420", which is wrong
about both the end of the file and the block structure; the runner is
order-independent and both trials below were run by appending at end of file.

Exact after: two lines appended at end of file, in this order, each one line of
JSON. **The [PAR-1] line supersedes the phase-1 packet's line entirely** — a
different `covered_by` and wholly different `reason` text, not a reason-text
revision — and its new text covers the erroneous-execution clause the [PAR-1]
v2 recipe adds. Nothing about the manifest's in-tree bytes changed, because
neither line is landed:

```
{"rule": "PAR-1", "covered_by": "compiler-permission-judgment", "reason": "A permission rule with nothing in a program to accept or reject: it grants an implementation the room to overlap two sibling calls and forbids nothing a writer can write. Every accepted program is accepted identically whether or not the permission is taken, and the compiler's own actualization tests establish the one observable consequence — that a taken overlap publishes the bytes the sequential schedule publishes — by running one emitted module at every worker count and against the lowering that overlaps nothing. The rule's erroneous-execution clause is covered by the same tests and for the same reason: a false executed claim is a contract violation, so the clause governs only defective programs, and the compiler's cases run one such program many times at several worker counts, parse the single mandatory record each run produces, and carry a control that defeats the trap latch to show that the single record is a mechanism and not an accident. A conformance case could only re-run a program and observe no difference, which is a statement about the implementation's schedule rather than about a source verdict."}
{"rule": "PAR-2", "covered_by": "compiler-permission-judgment", "reason": "The counted-loop half of the same permission, and covered for the same reason: it grants an implementation the room to overlap the iterations of a counted for and to choose the combination tree over the enumerated exactly-associative operations, and forbids nothing. No program's acceptance moves and no verdict moves. The one observable consequence — that a regrouped fold publishes the bytes the sequential fold publishes — is established by the compiler's own cases, which run one emitted module at ten worker settings, against the lowering that splits nothing, and against a two-sidedness table over every admitted combine at every width it carries."}
```

Both were applied to a scratch copy and run, rather than reasoned about:

- **[PAR-1]'s line alone, against today's 136-rule specification, is
  `136/136 rules covered (116 by case [+115/-55], 31 by annotation); 0
  uncovered`.** That is what the line does to the gate, and it is recorded so
  the owner knows what applying it produces — **it is not a form to apply on
  its own.**
- **[PAR-2]'s line is *rejected* today** — `invalid conformance manifest:
  annotation PAR-2: unknown rule` — because the rule is not yet in the active
  specification. It is therefore not an independent decision: it belongs to
  the same activation change that applies the rule text, and applying it
  earlier would break the gate rather than extend it.

**Neither annotation may be applied without its rule text, and this is the
one place in this record where that could be misread.** An earlier version of
the first bullet called the [PAR-1] line "the form to apply if the owner takes
the annotations without the rule", and said it turns `make check` green end to
end. It does turn the gate green — and that is precisely the hazard. The
prepared reason attests coverage of "the rule's erroneous-execution clause",
and no such clause exists in the [PAR-1] block the specification carries today:
in-tree `spec/kernel-spec.md:1984` still reads *"No function reachable from
either callee through the ordinary call graph contains a `claim_stmt`
[CLM-1]."*, and `:1988` still reads *"That identity holds in every execution,
not in a typical execution or in some execution."* Applying the annotation
alone therefore produces a green gate attesting coverage of rule text that is
not in the specification.

It is worse than a stale attestation. **`f6c55a9d` made the branch compiler
contradict the in-tree v1 rule**: it grants and actualizes exactly the
claim-bearing pairs line 1984 forbids — the ledger reports
`pair(layout_banded, layout_banded) eligible` and `@wf_layout_banded` emits
`@wf__par_claim`, `@wf__par_thunk_`, and `@wf__par_join`, where the same
function at `ddf1d139` emitted none of them. And `backend/tests/trap_latch.rs`
accepts either claim's name at `WF_WORKERS=4` while asserting the sequential
schedule names `left_total_is_zero`, which is a behavior v1 `[PAR-1]` forbids
under `:1987`-`:1988`. Nothing in the gate can see any of this: the coverage
runner checks only that the rule id is known and that `covered_by` and `reason`
are non-empty (`tests/conformance/runner.py:344-351`), and never compares
reason text to spec bytes.

**So the pairing in the candidate table below is the whole of the guidance.**
The [PAR-1] annotation line is applied only together with the [PAR-1] v2 rule
text, in the same change; the [PAR-2] line only with [PAR-2] v2. There is no
annotation-without-rule path, and the gate going green is not evidence that
there is one.

The manifest was restored byte for byte after each trial; nothing of this
reached the commit.

No conformance *case* is added, modified, deleted, or renamed by this batch,
and no case verdict moves. The delta is exactly the two annotation lines
above.

## The merge-time application recipes: [PAR-1] v2 and [PAR-2] v2

**These two recipes supersede two earlier ones and are the only ones to
apply.** They replace the [PAR-1] amendment recipe carried in the phase-1
merge packet (`docs/ongoing/0074-proof-derived-parallelism.md`, "Required
before merge" item 2), and batch A's [PAR-2] recipe recorded earlier in this
file. Nothing below is applied to `spec/kernel-spec.md` on this branch; the
file's in-tree bytes and recorded digest are unchanged at
`73d647c8945ad3d51eea3ed030714b433d6171e0d36b0869dd91366238cbd8f5`, 3,412
lines, 429,059 bytes, so the landed-archive gate stays green here.

**Re-derived after the rebase onto `main`.** Both digests below were recomputed
against the rebased candidate, which is main's activated v0.34 body plus this
branch's [PAR-1] section under a v0.35 CANDIDATE header superseding v0.34 at
`cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03`. Every line
number quoted below is the rebased file's. The pre-rebase digests
(`81cbe968…`, `cec3e25e…`) named a candidate whose body was v0.33's and do not
apply to anything on this branch any more.

Both candidates were produced by applying the exact edits below to scratch
copies and hashing the result — reproducible from this record and nothing
else. The owner applies **one** of them:

| candidate | apply when | SHA-256 | size |
|---|---|---|---|
| [PAR-1] v2 alone | [PAR-1] activates without [PAR-2] (the phase-1 merge) | `15165306e224f13795a1049cc55e1bf78985d71e8c3f1b6432b894f527bf9049` | 430,808 bytes, 3,418 lines, 136 rules |
| [PAR-1] v2 + [PAR-2] v2 | both rules activate in one change | `f99bb580eec570c62ee6df414aa324039d3b1e55b0cd72ec04b033d73e43bcfd` | 437,169 bytes, 3,443 lines, 137 rules |

The second is exactly the first with the [PAR-2] edits applied on top, so the
two are a chain and not two alternatives to reconcile.

**Grammar verification.** Neither adds a production, token, or spelling. The
native verifier, which reuses the compiler's own lexer and parser, confirms it
on both candidates:

```
$ whitefoot-grammar spec/kernel-spec.md <candidate>
grammar-preserving candidate verified by the active compiler: 74 productions,
93 decisions, 105 terminal predicates
```

identical to the installed inventory in both runs.

### [PAR-1] v2 — six edits

**Edit 1 — the caller-side operand-read half of the footprint.** This is the
phase-1 packet's first amendment sentence, carried forward unchanged because
it is still unlanded and still needed. Insert it immediately *before* the
sentence beginning "A footprint element whose caller place the implementation
does not resolve" (the file's line 2005):

```
Evaluating a statement's own argument expressions is part of that statement and therefore part of the overlap, so each call's written footprint also overlaps no place the other statement's argument expressions read; taking the address of a place is not reading it, and both directions are required because which statement's argument evaluation an overlap moves is the implementation's choice.
```

**Edit 2 — the unresolved-element sentence** (the file's line 2005) becomes:

```
A footprint element whose caller place the implementation does not resolve overlaps every place, and so does a place read by an argument expression whose caller place the implementation does not resolve, so an unresolved element denies permission rather than granting it.
```

**Edit 3 — delete the eligibility condition.** Remove the file's line 2008
entirely, leaving no blank line behind:

```
No function reachable from either callee through the ordinary call graph contains a `claim_stmt` [CLM-1].
```

This is the edit the 2026-08-23 direction charters, and it replaces the
phase-1 packet's third amendment sentence, which appended to this condition
rather than removing it. That sentence is withdrawn with the condition it
qualified.

Deleting rather than restating it also removes a duplication the rebase made
visible. [CLM-3] already forms `MayClaims(K)`, the transitive closure of
retained claim occurrences over a component and its strictly outgoing callees,
and v0.34 retains every accepted claim — so this condition was exactly
"`MayClaims` of the callee's component is empty", a second claim-reachability
analysis that would have had to agree with [CLM-3] forever. The v0.35 text
simply has no such condition: one less clause, and no second analysis to keep
in step.

**Edit 4 — the observable identity becomes conditional.** Replace the file's
line 2012:

```
That identity holds in every execution, not in a typical execution or in some execution.
```

with these seven lines:

```
That identity is conditional on contract compliance, exactly as [SCOPE-3]'s freedom from undefined behavior is conditional on its trusted computing base.
For an execution in which no executed `claim` is false it holds in every execution, not in a typical execution or in some execution.
An execution in which some executed `claim` is false is erroneous: the program has violated the sole writer-reachable language runtime contract [SCOPE-4], and this rule then requires exactly the following of that execution.
The process writes exactly one complete [DIAG-3] record, naming one `claim` whose predicate evaluated false, and then aborts the whole process without unwinding and without language cleanup [TRAP-1].
No second record, and no partial or interleaved record, is written.
Which such `claim` that record names may depend on the schedule, and is the only thing this specification permits a schedule to select.
Nothing else narrows for an erroneous execution: it has no undefined behavior [SCOPE-3], no overlapped pair reaches one place except as the disjointness condition above admits, and no statement of a permitted overlap produces an external effect at all, because neither callee's row may carry `external` [EFF-1].
```

**Edit 5 — the META-5 delta declaration** (the file's line 6). Its final
sentence becomes:

> The one added rule states when an implementation may overlap the execution
> of two statements and requires every observable of a permitted overlap to be
> the source-order execution's, conditional on contract compliance in the sense
> [SCOPE-4] fixes; it adds no construct, changes no accepted program, changes
> no verdict, and removes no required check.

**Edit 6 — the selection ground** (the file's line 7). The clause naming the
recorded evidence becomes:

> whose measured lane-budget results are recorded in
> `research/investigations/proof-derived-parallelism/`, under the owner's
> chartering direction of 2026-08-21 and the PROPOSED Current Plan derived
> from it, and whose eligibility condition on claim-free call closures was
> withdrawn under the owner's chartering direction of 2026-08-23 in favour of
> the conditional guarantee this rule now states.

### [PAR-2] v2 — three edits, applied on top of [PAR-1] v2

Batch A's recipe with two changes, both consequences of the same direction:
its claim-free condition sentence is **deleted**, and its identity sentence is
replaced by one that shares [PAR-1]'s clause by reference rather than
restating it. Everything else is byte-identical to batch A's text.

**Edit 1 — the rule block.** Insert the following, preceded by one blank line,
immediately after [PAR-1]'s closing sentence ("This rule binds neither [CAP-1]
predicate, because its disjointness condition admits ...", the last line of
section 13 before the blank line preceding `## 14. Gated family`).

```
[PAR-2] An implementation may execute two iterations of one `for_stmt` body with overlapping execution, and may recombine that loop's accumulator across them, only when the permission this rule defines holds for that counted loop.
Permission holds for a `for_stmt` L exactly when all of the following hold, writing B for L's body and forming every written, read, and operand-read footprint of a statement of B exactly as [PAR-1] forms one.
A footprint of B writes at most one place rooted in a binding declared outside L; that binding is L's accumulator, and every occurrence of it in B is one operand of one `set` statement whose target is that whole binding and whose right-hand side is one operation applied to that operand and to a second operand reaching the accumulator nowhere.
That operation is one operation fixed for the accumulator across the whole of B, and is exactly one of `+wrap`, `*wrap`, `iand`, `ior`, `ixor`, `imin`, `imax`, `band`, `bor`, and `bxor` [OP-1].
Every place a footprint of B writes is either that accumulator's whole place or is rooted in a binding B itself introduces, so no two iterations write one place except through that accumulator.
A footprint element whose caller place the implementation does not resolve overlaps every place, so an unresolved element denies permission rather than granting it.
No effect row of a call in B contains `external` or `blocks`, and no statement of B evaluates a system operation [EFF-1, SYS-2].
Every normal continuation of every statement of B reaches L's compiler-owned binder update, so no statement of B is a `return_stmt`, a `give_stmt`, a `break_stmt` naming L or a loop enclosing L, or a `let_stmt` selecting `propagate_let_rhs` [FN-1, GIVE-1, ERR-3].

Under a permitted overlap every observable is the observable the same program produces by executing L's iterations in index order: the value of every binding and place, the trap-or-normal outcome, the exact [DIAG-3] record bytes, and the external-effect order [EFF-5] requires.
Write a0 for the accumulator's value on the true header edge entering the first executed iteration, and t0 through tm for the values the second operand of its writes evaluates to, in the order those writes execute across L's iterations taken in index order.
Source order computes the accumulator's value at L's continuation as the left-nested application of that operation to a0 then t0 through tm where its writes place the accumulator in the first operand position, and as the right-nested application to t0 through tm then a0 where they place it in the second.
An implementation may instead apply that operation over any binary tree whose leaves are exactly a0 and t0 through tm, each occurring once and in that same left-to-right order.
Every admitted operation is a total function on the complete value set of its type, carries no domain obligation, and is associative on that set — `+wrap` and `*wrap` are the ring operations of the integers modulo two to the width, `iand`, `ior`, and `ixor` are the meet, join, and group operations of the bit vector, `imin` and `imax` are the meet and join of that type's total order, and `band`, `bor`, and `bxor` are the two-element cases of the same three — so every such tree denotes one value of that type and the accumulator's value at L's continuation is that one value in every execution.
No further operation is admitted: `+`, `+defined`, and `+checked` each attach a domain obligation or a `Result` route to every application, `+sat` is not associative, and no float operation of [OP-1] is associative, so recombining a `fadd.strict` or `fmul.strict` fold could change published bytes.
This rule uses associativity alone: it never reorders leaves, requires no commutativity, and names no identity element, so a range of iterations that writes the accumulator not at all contributes no leaf and is combined with nothing.
That identity is conditional on contract compliance exactly as [PAR-1]'s is, and an erroneous execution of L — one in which some executed `claim` is false — receives exactly the guarantee [PAR-1] states for one, with the `claim` the single [DIAG-3] record names selected from among those whose predicates evaluated false.
Both endpoint atoms are still evaluated exactly once each in [FN-1]'s order before any iteration begins, and the binder still takes each value of the half-open range exactly once; this rule relaxes only the order in which iterations execute and the shape of the accumulator's combination, never the set of iterations, the values the binder takes, or either endpoint evaluation.
The number of workers, the identity of the host thread that executes an iteration, the schedule, how the index range is divided, and whether any overlap or recombination was performed at all are not observable, and no rule of this specification is stated in terms of them.
An implementation that overlaps nothing therefore conforms: this permission is never an obligation, and no program depends on it being taken.
When an execution of one iteration does not reach its continuation, the overlapped execution produces exactly the observables the index-order execution produces before that point and produces none after it.
Exhaustion of the execution resources an implementation spends on overlapping is a resource condition under [SCOPE-3] and is not an observable of this rule.
Permission over the iterations of a `for_stmt` written inside B is exactly this rule applied to that loop; no rule of this specification joins two index ranges into one iteration space.
This rule binds neither [CAP-1] predicate, because its conditions admit concurrent access only to places no permitted overlap writes and to one accumulator whose every write it recombines under one associative total operation.
```

**Edit 2 — the META-5 delta declaration** (the file's line 6) becomes:

> META-5 delta declaration: numbered rules +2/-0 ([PAR-1], [PAR-2]; 137
> remain); grammar productions +0/-0 (74 remain); unique fixed lowercase
> grammar atoms net +0; writer operation spellings +0/-0; runtime-trap
> families +0/-0; entry forms +0/-0; contract block forms +0/-0; system
> operations +0 and declaration records +0; exception clauses +0/-0. The two
> added rules state when an implementation may overlap the execution of two
> statements of one block, and when it may overlap two iterations of one
> counted loop and recombine that loop's accumulator across them; each
> requires every observable of a permitted overlap to be the source-order
> execution's, conditional on contract compliance in the sense [SCOPE-4]
> fixes, and neither adds a construct, changes an accepted program, changes a
> verdict, or removes a required check.

**Edit 3 — the selection ground** (the file's line 7) gains one sentence at
its end:

> [PAR-2] is selected on the same ground by the loop-shaped permission
> investigation of batch 0078, whose value falsifier, probed byte-identity of
> a regrouped wrap-family fold, and corpus census are recorded in
> `research/investigations/proof-derived-parallelism/loop/`, under the owner's
> chartering direction of 2026-08-23; it states the admitted combination set
> normatively because a conforming implementation chooses the combination
> tree.

### Impact inventory

`[PAR-1]`'s extent moves from lines 2000 to 2019 and 3,269 bytes to lines
2000 to 2025 and 4,801 bytes, in both candidates. In the combined candidate
`[PAR-2]` occupies lines 2027 to 2050 and 5,781 bytes. Line-initial rule
definitions are 136 for the [PAR-1] v2 candidate and 137 for the combined one,
so `RULE_COUNT` moves only when [PAR-2] activates.

**Bracketed rule-token occurrence counts do move**, which the phase-1 packet's
[PAR-1] recipe could report as unchanged and this one cannot. [PAR-1] v2 alone:
`CLM-1` 14 to 13 (the deleted condition's citation), `DIAG-3` 11 to 12,
`EFF-1` 5 to 6, `SCOPE-3` 7 to 9, `SCOPE-4` 5 to 7, `TRAP-1` 2 to 3. No rule
loses its last reference, and no rule becomes unreferenced. Adding [PAR-2] on
top moves `CAP-1` 4 to 5, `DIAG-3` 12 to 14, `EFF-5` 4 to 5, `FN-1` 24 to 25,
`OP-1` 11 to 13, `SCOPE-3` 9 to 10, `PAR-1` 2 to 5, and `PAR-2` 0 to 3. Every
count in this paragraph was recomputed against the rebased file; only `CLM-1`'s
starting value moved, because main's v0.34 body cites it twice more than v0.33's
did.

**The counting convention, stated because the numbers do not reproduce without
it:** every count above is of the single-token citation `[X]` only, and the
specification also carries 47 multi-rule citations of the form `[A, B]` that
these figures do not count. Under the all-citation convention the combined
candidate additionally moves `EFF-1` 6 to 7, `FN-1` 29 to 31, `SYS-2` 26 to 27,
`GIVE-1` 19 to 20, and `ERR-3` 6 to 7 — [PAR-2] v2's text adds `[EFF-1, SYS-2]`
and `[FN-1, GIVE-1, ERR-3]`. "No rule loses its last reference" holds under
both conventions.

The [PAR-1] v2 recipe adds no non-ASCII byte. **The combined candidate does:**
the [PAR-2] v2 block adds four U+2014 em dashes, two on each of the added lines
quoted at "the loop-shaped permission" and "an execution of L". The
specification already carries non-ASCII on 100 lines and no gate forbids it, so
this is a correction to this record rather than to either candidate.

### Derived material the activation change must carry

For either candidate: `compiler/src/spec_identity.rs` is regenerated rather
than hand-edited (`whitefoot-spec --emit-identity`), taking `SPEC_SHA256_HEX`
to the digest above, and the transcribed digest literal in
`compiler/src/spec.rs` moves with it. The combined candidate additionally
moves `RULE_COUNT` to 137, gains a second existence-only derivation-ledger row
for [PAR-2], and needs the [PAR-2] coverage annotation prepared under "FLAGGED"
above. The ledger's totals line reads `84 derived · 52 existence-only · 0
underived` today, where 84 + 52 = 136 is the branch's rule count. One added
existence-only row makes it **`84 derived · 53 existence-only · 0 underived`**
across 137 rules. An earlier version of this paragraph wrote `85 derived · 52
existence-only`: that sums to 137, but it increments the derived column, which
would make [PAR-2] a derived rule and contradict the same sentence that calls
its row existence-only. An owner applying it verbatim writes a wrong totals
line. The
conformance corpus delta is zero cases either way: neither rule changes an
accepted program or a verdict.

### Where the rules are deliberately wider than the verifier

Unchanged from batch A for [PAR-2]: the rule states the accumulator condition
at the algebraic boundary while the implementation keeps the stricter test that
the accumulator is read exactly once in the body, so a loop combining one
accumulator under two branches is refused by this compiler and admitted by the
rule. [PAR-1] v2 adds a second instance of the same direction: the rule now
requires only one well-formed [DIAG-3] record from an erroneous execution,
while this implementation additionally makes the sequential schedule
deterministic, so a defective program has a reproduction path the rule does not
demand. Both directions cost nothing — an implementation never has to take the
room a rule leaves it — and both avoid a further [META-5] amendment when the
implementation is widened.

## Outcome

(Filled at closure.)
