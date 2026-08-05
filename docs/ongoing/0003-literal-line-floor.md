# 0003 — WF-LITERAL-LINE

Status: IN PROGRESS — frozen apparatus, timing not yet observed

Authority: separate owner-approved bounded research on 2026-08-05. This task
may inform `PERF-1`, `FLOOR-1`, `FLOOR-2`, and a later concrete `STORE-1` or
proof proposal only if its exact measured pressure appears. It authorizes no
specification, compiler, proof, intrinsic, runtime, system-capability, regex,
thread, or end-to-end `wfgrep` implementation change.

Owner / workspace: Codex task `codex/literal-line-floor` in
`/Users/bytedance/.codex/worktrees/73c2/Whitefoot`

Base revision: `b9475f2`

## Goal

Determine whether the active v0.17 language and ordinary compiler can express
a faithful multi-byte literal line matcher at competitive machine quality.
The slice must search runtime haystack and needle bytes, verify complete
matches, reconstruct line boundaries and line numbers, and consume an exact
match-record digest. Separate same-algorithm lowering quality from the
algorithmic ceiling of the pinned upstream literal-search machinery.

## Direction and invariants

- Use the ordinary current compiler path and retain every required check.
- Bind every timed result to exact match offsets, line boundaries, line numbers,
  and a consumed digest checked by an independent oracle.
- Compare equivalent Whitefoot, C, and safe-Rust algorithms for compiler-floor
  attribution; label pinned `memchr`/ripgrep literal machinery as an algorithmic
  ceiling rather than equivalent work.
- Use exact real RG-BASE corpus identities plus a deterministic overlapping or
  high-candidate adversarial control; freeze inputs and claims before timing.
- Inspect raw and optimized LLVM plus final assembly before assigning a cause.
- A single-file or in-memory result cannot be called grep, ripgrep, end-to-end,
  or evidence for the 2x flagship claim.

## Method

Audit the pinned ripgrep literal path and exact offline dependencies, then
select the smallest authentic line-matching contract. Preregister corpus
regions or generated inputs, needles, work accounting, correctness oracle,
algorithm variants, build identities, code-shape observables, paired schedule,
statistics, materiality bands, and stop rules. Implement one self-contained
experiment bundle under `research/experiments/`, run correctness and code-shape
inspection before timing, and classify the first material gap among algorithm,
source shape, required check, compiler lowering, LLVM recovery, target code,
or noise.

## Progress

- Completed: owner authorization, numbering, independent-scope review against
  0001, exact ripgrep/dependency audit, active-language audit, hostile protocol
  review, implementation, correctness gate, and pre-timing code-shape review.
- Current: freeze the complete apparatus and pre-timing observations in one
  clean commit. No comparative timing has been observed.
- Next: execute the single create-once run, classify it under the frozen rules,
  retain the evidence, update admitted design-memory facts, and stop.

## Scope and expected touch set

Expected paths:

- `research/experiments/literal-line-floor/`
- `research/experiments/README.md` after a completed result
- relevant existing MCTS-Mem nodes only if completed evidence passes their
  admission test
- this task record, moved without renumbering to `docs/done/` at disposition

No numbered specification, compiler source, conformance expectation,
`docs/current-plan.md`, system-capability dossier, runtime, file API, output
API, regex parser or automaton, directory walker, ignore engine, thread/runtime
work, or `wfgrep` product source is in scope.

## Dependencies and integration order

This task is semantically independent of
`docs/ongoing/0001-system-capability-architecture.md`: it performs no system
operation and changes no entry, effect, resource, target, or ABI model. Either
may integrate first. Before integration, refresh main, reread 0001, rebase, and
preserve its authoritative changes.

Any observed need for a new source rule, compiler lowering, intrinsic, proof,
storage capability, or pattern card is a finding returned to the owner. It is
not implementation authority inside this task.

## Validation, stop, and closure

- Validate with independent oracles, identical-work controls, pinned ceiling
  identity, hostile overlap/edge cases, optimized-code inspection, frozen
  paired statistics, order/noise review, experiment-local checks, MCTS lint
  after any tree edit, and `make check` before integration.
- Stop after the first faithful slice is classified as practical parity,
  material same-algorithm loss, material algorithmic-ceiling gap, or
  precision-inconclusive, and its first material cause is attributed.
- Do not widen after seeing results into regex, Unicode semantics, traversal,
  I/O, formatting, output publication, or parallel execution.
- At closure, retain positive, negative, or inconclusive evidence in the
  experiment bundle, update only admitted design memory, and move this same
  numbered record to `docs/done/` with landed commits and claim limits.
