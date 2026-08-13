# 0055 — DIAG-2 counted-root completeness

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-12, `Trust prerequisite — bounded existing-DIAG-2 repair`. It is not
yet claimed; claiming fills in status, owner, workspace, and base revision and
moves this number unchanged to `docs/ongoing/`.

- **Authority:** active v0.27 DIAG-2, ENT-3 S11, ENT-4, and the Current Plan's
  requirement to retain every S11 derivation whether or not later queried

## Goal

Complete the DIAG-2 root set on top of task 0054's landed derivation arena by
retaining every normative S11 relation for every counted statement, including
unused facts, with exact endpoint source identity, counted snapshot ordering,
continuing kills, body-entry roots, and deterministic occurrence coverage.

## Direction and invariants

- Enumerate counted statements from the checked function, not from later
  queries. Each exact `for_stmt` occurrence has one complete root group.
- Retain the two capture equalities, binder initialization equality, and the
  two true-header bounds in the normative S11 order. Equality roots expose
  both directed bounds, yielding the complete atomic bound inventory rather
  than a Boolean flag.
- The dedicated counted-preheader snapshot occurs after capture establishment
  and closure, before continuing-kill subtraction. No other new
  materialization boundary is admitted.
- Roots name the concrete range occurrence, binder, capture side, endpoint
  term, proof point, relation, and parent. Binder/captures remain private and
  never escape into the continuation.
- This task changes no source acceptance, lowering, runtime behavior, fact
  family, or loop semantics.

## Method

1. Refresh onto terminal task 0054 and rerun its focused root verifier.
2. Add one retained counted-root group to `FunctionEntailment` in deterministic
   checked-tree order. Populate it during the existing S11 preheader and body
   entry operations, not in a post-pass.
3. Record snapshot/materialization parents for every materialized consequence
   used by continuing state, then apply the existing kill summary to both facts
   and parents. Retain the normative S11 roots observationally even when their
   facts later die or no query consumes them.
4. Add hostile tests for zero-trip, reversed, singleton, maximum edge, mutable
   endpoint sources, both endpoint writes, binder hidden update, early return,
   matching/enclosing break, nested ranges, join, ordinary-loop near miss, and
   unused S11 roots.
5. Cross-check the three real SHA-256 counted loops and all synthetic counted
   statements: each occurrence appears exactly once, every required relation
   and directed equality component has one valid root, and no ordinary loop
   appears.
6. Run focused and complete gates and update compiler documentation only for
   the exact retained capability.

## Scope and expected touch set

Task 0054's entailment/derivation modules, counted flow/sources, focused
semantic tests, and `compiler/README.md`. No specification, protected corpus,
consumer source, lowering, backend, generated, plan, roadmap, approval, or MCTS
bytes.

## Dependencies and integration order

Task 0054 must be terminal and its commit is the exact base. Task 0056 follows
this task. Stage 8b waits for task 0056 plus positive task 0053.

## Validation

- Every counted statement has one source-ordered root group with all five S11
  relations and all directed bound components; unused facts remain present.
- Parent replay proves each relation without rerunning closure.
- Snapshot-before-kill ordering is directly tested; a normal query-derived
  consequence cannot survive the same kill.
- Three real SHA counted loops are complete, deterministic, and still
  discharge 9/9 without claims; all runtime and emitted no-trap evidence is
  unchanged.
- Focused checks and both complete repository gates pass.

## Stop condition

Stop if complete roots require reconstructing flow after the pass, treating
ordinary query closure as live, exporting binder/capture facts, adding loop
induction, or changing counted control/runtime behavior; or if any checked
counted occurrence cannot be enumerated exactly once.

## Done-when

All accepted subscripts, discharged call goals, and all S11 facts now have
complete retained roots on one canonical derivation channel, with task 0056
remaining only for cost and terminal evidence.
