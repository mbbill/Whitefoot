# Batch 0077 — loop-shaped permission

Branch: `par/loop-permission`, stacked on the frozen
`par/proof-derived-parallelism` tip `27e02b1f` (batches 0074-0076, closed,
awaiting owner merge; nothing on that branch moves). This batch's work
enters a separate, later merge review so the two phases stay independently
trackable.

Authority: owner chartering direction, 2026-08-23, verbatim:

> 强迫循环写成递归感觉违背了默认形态就是最优的原则,所以我觉得我们应该认真
> 的把循环问题也解决了。不过如果这件事有blocker的话可能需要好好研究一下。
> 目前收官完成的分支可以放着不动,然后在这个顶上再继续开个新分支开始循环的
> 研究吗?这样我们可以轻松的追踪这两个阶段。

Consequence adopted: the counted-loop ledger hint (batch 0076) is a bridge,
not the end state — the loop form itself must receive permission. This is
the plan's W4 "indexed-loop permission (Tier A)" made current.

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

## Defects found in already-presented material

The adversarial probe of the loop surface found two defects in batch 0076's
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
  (2 failed before) and publishes the default build's bytes on all 8 units
  the lowering changes. The `--par` compile test is widened from
  `par_layout.wf` to the whole corpus and fails without the fix.

## Outcome

(Filled at closure.)
