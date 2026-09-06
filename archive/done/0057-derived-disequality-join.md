# 0057 — derived disequality closure and join repair

- **Status:** `DONE` (2026-08-13)
- **Authority:** the `ACTIVE` Current Plan's existing-DIAG-2 trust repair and
  the workflow's bounded compiler-defect route, under active v0.27 ENT-4/ENT-5
- **Landed commit:**
  `a6f11672eda883a37f147ce78400e2504f1c4031`

## Outcome

The canonical single `close` fixed point now completes its normalized
disequality set from every strict bound, lets that complete set participate in
ENT-4 strengthening, and publishes it to the existing ENT-5 join. Joins retain
a disequality held by every non-contradictory predecessor even when the grounds
or strict orientations differ, without inventing either ordered strict bound.
The existing live fact support and kill path removes a joined disequality after
an overlapping write.

The repair is confined to `state.rs` plus ordinary entailment regressions. It
adds no second closure, solver, fact family, proof ledger, specification,
protected evidence, lowering, runtime, or real-program change.

## Evidence and validation

- Frozen behavior reported `Unproved` for the minimal opposite-orientation
  join and for the isolated join-then-weak strengthening witness; the landed
  implementation reports `Discharged`. A kept/killed pair verifies both join
  materialization and post-join endpoint invalidation.
- Hostile coverage includes same and opposite strict orientations, explicit
  and mixed grounds, three-way and contradictory-neutral inputs, missing and
  equality inputs, pre- and post-join kills, no invented ordered bound,
  contradiction, empty join, and ordinary-loop non-induction.
- Executor gates on the exact landed commit passed 706/706 library tests,
  30/30 real programs, 131/131 rule coverage, all 19 activation records, and
  both `make -C compiler check` and `make check` green. Lead review independently
  reran the three distinguishing join, strengthening, and kill regressions.
- Active specification SHA-256 remained
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.

## Remaining dependency

Task 0054 must refresh/rebase its preserved derivation-ledger implementation
onto the landed commit and add exact strict-derived-disequality and join parent
records. Tasks 0055 and 0056 remain downstream of 0054.
