# 0060 — Stage 8b callee postcondition proof and summaries

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-14, Workstream 8b. It is not yet claimed; claiming supplies the live
fields and may occur only after task 0059 records its reviewed held commit.

- **Authority:** the ACTIVE Current Plan, Workstream 8b `verified
  normal-return postconditions`, under Direction Outline revision 38 item
  `PROOF-8`
- **Frozen candidate:** commit `7a293861ddb0aad0cd8d494ee8f4bfa5f6cbba29`,
  specification SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`
- **Execution premise:** the exact lead-reviewed H1 commit recorded by task
  0059; claim and integrate only onto H1

## Goal

Implement the two measured S7 sources and prove every FN-9 selected normal
exit in complete, unasserted, and S4-blinded views. Produce finite,
concrete-instance callee summaries and exact local DIAG-2 roots in one shared
view-tagged flow and event stream, without yet using any user-postcondition
summary as a caller fact.

## Direction and method

1. Replace the two post-hoc U/B entailment rewalk authorities with one
   structural function flow and one function-local derivation DAG carrying
   complete/U/B views; retain the independent provenance dependency walk.
   Make the PRV-1 component result available for task 0061's final strata, but
   do not run a PRV verdict as a pre-transfer call gate or publish any S12 fact
   in this handoff.
2. Add only the frozen unsigned `iand` bound and unsigned
   `ishl.wrap(one,count)` nonzero sources. The one operand is a direct checked
   typed literal or earlier named constant with mathematical value one;
   local bindings, const generics, signed or wrong operations, reversed
   shapes, and support kills establish nothing. Distinguished zero retains
   only constant-bound and the exact shift-source zero-disequality roles.
3. Enumerate every structurally selected normal exit for each concrete
   instance. Reject a zero-selected selector at FN-9. Query the exact
   instantiated RelationTemplate after ordinary cleanup, consume, write,
   scope, and join effects. Complete must discharge; U/B dispositions are
   retained separately and never borrow a root from another view.
4. Build `PostconditionExit` and `PostconditionAggregate` roots in source
   order. Keep callee proof local: no foreign derivation ID becomes a local
   parent. Same-SCC postcondition summaries are unavailable, components are
   processed in deterministic callee-before-caller order, and a component
   publishes atomically only after its independent clauses succeed. There is
   no least/greatest fixed point, iteration-to-stability, or writer clause as
   axiom.
5. Keep every S12/call-result route explicitly unavailable in H2. Functions
   whose clause needs a user-postcondition fact safely remain unsupported at
   this handoff; task 0061 alone connects verified earlier-SCC summaries.
6. Commit reviewed H2 on H1, record its full SHA here, and leave this task
   `WAITING`; only then may 0061 claim.

## Scope and expected touch set

Use the existing semantic ownership homes:
`compiler/src/semantic/{entailment.rs,provenance.rs,check.rs,model.rs,postcondition.rs}`,
`compiler/src/semantic/entailment/{flow.rs,flow/sources.rs,state.rs}`, and
focused `semantic/tests/{entailment.rs,derivation.rs,provenance.rs,postconditions.rs}`.
Reuse H1's checked metadata. Do not change candidate spec/archive/ledger bytes,
call publication, receivers, delivery, real programs, protected evidence,
lowering, runtime, ABI, backend identity, or gate wiring.

## Validation

- Positive, negative, near-miss, support-kill, early-return, Err, divergence,
  contradiction, generic, direct/two-hop-independent, self, mutual, seedless,
  and command-entry controls pin every complete/U/B disposition.
- Assertion/claim-only proof is complete-only; S4-only proof is C/U but not B;
  no view crosses into another. Entry-image writes/consumes permanently remove
  eligibility.
- Read-bit and both append candidate bodies prove their expected local
  summaries without a user-call summary dependency. The append invalid return
  is C/U discharged through the S4 contradiction and exactly B refuted.
- Root validation proves parent completeness, deterministic ordering, pruning,
  and one DAG/event authority. Focused fmt, clippy, and cargo tests pass; any
  full gate has only the unchanged preapproval activation-chain stop.

## Stop condition and done-when

Stop on candidate drift, a fourth semantic walk, post-hoc proof reconstruction,
same-SCC summary use, any summary fixed point, expression terms, a third fact
source, view laundering, imported foreign derivation IDs, call publication in
H2, or a required change outside the named semantic/test owners.

The handoff is implementation-complete when reviewed H2 proves and records the
frozen local summaries and roots with all S12 routes unavailable, and 0061's
premise names H2. It remains `WAITING` until atomic activation.
