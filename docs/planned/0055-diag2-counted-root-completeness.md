# 0055 — DIAG-2 counted-root completeness

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-12, `Trust prerequisite — bounded existing-DIAG-2 repair`. It is not
yet claimed; claiming fills in status, owner, workspace, and base revision and
moves this number unchanged to `docs/ongoing/`.

- **Authority:** active v0.27 DIAG-2, ENT-3 S11, ENT-4, ENT-5, and the Current
  Plan's requirement to retain every S11 derivation whether or not later
  queried

## Goal

Complete the DIAG-2 root set on top of task 0054's landed derivation arena by
retaining every normative S11 relation for every counted statement, including
unused facts, with exact endpoint source identity, counted snapshot ordering,
continuing kills, body-entry roots, and deterministic occurrence coverage.

## Direction and invariants

- Populate roots during the existing analyzer walk, not from later queries or
  a second checked-tree inventory. Each exact `for_stmt` occurrence has one
  `CountedDerivationSet`, in deterministic statement-walk order.
- Each set retains the two capture equalities, binder initialization equality,
  and two true-header bounds in normative S11 order. This is exactly five
  semantic roots and eight directed atomic-bound roots per occurrence.
- All eight atomic roots enter the existing sole `DerivationLedger` root
  channel. `DerivationLedger::finish()` must retain and remap every referenced
  parent together with the published set; no second retention list or
  pre-remap ID may survive.
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
2. Add `CountedDerivationSet { counted_node_path,
   lower_capture_eq_endpoint, upper_capture_eq_endpoint,
   binder_eq_lower_capture, lower_capture_le_binder,
   binder_lt_upper_capture }`. Each equality holds both directed atomic roots.
   Populate it during existing S11 preheader and body-entry operations, add all
   eight atomic parents through the sole ledger root channel, and remap those
   IDs during the existing finalization, not in a post-pass.
3. Record snapshot/materialization parents for every materialized consequence
   used by continuing state, then apply the existing kill summary to both facts
   and parents. Retain the normative S11 roots observationally even when their
   facts later die or no query consumes them.
4. Add hostile tests for zero-trip, reversed, singleton, maximum edge, mutable
   endpoint sources, both endpoint writes, binder hidden update, early return,
   matching/enclosing break, nested ranges, join, ordinary-loop near miss, and
   unused S11 roots.
5. Maintain encountered-counted and completed-root counters in that same walk;
   assert completeness while building `FunctionEntailment`. Root relations
   must equal the final normalized relations; identity mismatch is an internal
   compiler defect, never a category-only fallback.
6. Cross-check the three real SHA-256 counted loops and all synthetic counted
   statements: each occurrence appears exactly once, every required relation
   and directed equality component has one valid root, and no ordinary loop
   appears. Run focused and complete gates.

## Scope and expected touch set

Only task 0054's six implementation/test files:
`compiler/src/semantic/entailment.rs`, `state.rs`, `term.rs`, `flow.rs`,
`flow/sources.rs`, and
`compiler/src/semantic/tests/entailment.rs`, plus this lifecycle record. No
README, research, specification, protected corpus, consumer source, lowering,
backend, provenance, generated, plan, roadmap, approval, or MCTS bytes. A need
for another file stops for lead review.

## Dependencies and integration order

Task 0054 is terminal at implementation commit
`0e9a206188d8cc37ec3bb248889e42109122246a`; this task may be claimed only
after the 0054 `DONE` record lands, and that terminal closure revision is its
base. Task 0056 follows this task. Stage 8b waits for task 0056 plus positive
task 0053.

## Validation

- Every counted statement has one source-ordered root group with all five S11
  relations and exactly eight directed atomic roots; unused, zero-trip,
  break/return/propagate, and nested occurrences remain present and distinct.
- Parent replay proves each relation without rerunning closure.
- Snapshot-before-kill ordering is directly tested; a normal query-derived
  consequence cannot survive the same kill.
- Endpoint writes/projections/dereferences, normal binder update, maximum-u64
  endpoints, generic endpoints, and contradictory-predecessor joins preserve
  only the facts allowed by current kills and snapshot materialization;
  ordinary loops still gain no induction.
- Mutation controls that delete or duplicate a root, change its `NodePath` or
  relation, corrupt a parent/snapshot marker, or retain a killed parent fail in
  the test-only structural checker.
- `tests/programs/sha256_abc.wf` has three occurrence groups, exactly 15
  semantic S11 roots and 24 atomic roots, plus exact roots for its existing
  nine accepted bounds obligations. UTF-8, the four raw-DEFLATE sources, and
  wfgrep have complete bounds/call/counted roots without changed outcomes.
- Repeated compilation produces a byte-identical normalized ledger; all
  runtime and emitted no-trap evidence is unchanged.
- Focused checks and both complete repository gates pass.

## Stop condition

Stop if complete roots require reconstructing flow after the pass, treating
ordinary query closure as live, exporting binder/capture facts, adding loop
induction, a general ProofFlow/CFG, serialization, full closed-state snapshots,
claim/O3 witnesses, broad identity remodeling, or changing counted
control/runtime behavior; or if any checked counted occurrence cannot be
enumerated exactly once. If the SHA-256 3/15/24 inventory does not match the
actual checked model, stop with a reproduction rather than changing semantics
or the expected count.

## Done-when

All accepted subscripts, discharged call goals, and all S11 facts now have
complete retained roots on one canonical derivation channel, with task 0056
remaining only for cost and terminal evidence.
