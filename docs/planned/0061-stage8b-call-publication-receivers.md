# 0061 — Stage 8b call publication, receivers, and provenance atomicity

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-14, Workstream 8b. It is not yet claimed; claim only from the reviewed
held H2 recorded by task 0060.

- **Authority:** the ACTIVE Current Plan, Workstream 8b `verified
  normal-return postconditions`, under Direction Outline revision 38 item
  `PROOF-8`
- **Frozen candidate:** commit `7a293861ddb0aad0cd8d494ee8f4bfa5f6cbba29`,
  specification SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`
- **Execution premise:** the exact lead-reviewed H2 commit recorded by task
  0060; claim and integrate only onto H2

## Goal

Connect verified earlier-SCC summaries to ordinary calls and implement the
candidate's closed publication routes: fresh direct plain result, direct
selected-`Ok` match payload, direct same-binding call-result receiver, and the
narrow immediate selected-payload-to-outer receiver. Integrate those facts
with PRV-2/PRV-3 as one failure-atomic batch and extend the same DIAG-2 DAG.

## Direction and method

1. Form `A0` entirely in the pre-transfer state: resolution/concrete
   substitution, argument and borrow checks, actual-expression obligations,
   pre-transfer formal images, and complete FN-8 proof. FN-8 failure creates
   no candidate. For every A0-successful call, failure-atomic semantic scratch
   then applies consume/borrow transfer, callee-effect kills, target
   commit/kill, result substitution, and eligible fact establishment. Only
   after the complete optimistic batch exists do PRV-1 component pairs freeze
   and PRV-2/PRV-3 converge over that batch. A rejection event discards the
   whole batch and checked program; no PRV no-event premise participates in
   candidate formation and no individual fact is committed or retracted.
2. Instantiate only verified earlier-SCC summaries. Complete receives C. U/B
   use the frozen B-first formula: when Bq is discharged, retain the B aggregate
   parent and no same-view Gv parent; only when Bq is false may Uq plus exact
   Gv publish. Gv contains every actual-expression obligation and, when one
   exists, the instantiated FN-8 requirement discharged in that same view;
   the `PostconditionCall` root retains those exact parents. Every route also
   retains the A0 complete actual-obligation and FN-8 parents. C-only evidence
   never enters U/B.
3. Treat substituted formals as pre-call entry images. Any consume, effect
   write, target overlap, alias, projection overlap, holder/region/scope exit,
   or other kill suppresses the affected relation. A post-call receiving
   binding is never conflated with the overwritten pre-call actual. Element
   writes preserve the separately supported destination length; root or holder
   writes do not. Establishment eligibility is relation-local: a non-ENT-2
   actual suppresses only a relation that references it, while an unreferenced
   non-ENT-2 actual does not suppress another relation. Ephemeral FN-8 datums
   never satisfy persistent summary substitution.
4. Implement only the frozen result destinations. A plain fragment result may
   establish only on the fresh binding of a direct ordinary-let user call. A
   selected `Ok` payload may establish when the ordinary call is the direct
   scrutinee of either a `match_stmt` or a `value_match`, and only at entry to
   its exact direct selected arm. Named, pending, stored, aliased, or propagated
   whole outcomes create no token or deferred fact; discarded, nested,
   projected, computed, and wrong-type substitutions publish nothing.
5. For a direct same-binding call result, require a previously live bare own
   fragment target of the exact result type and exactly one direct,
   non-consuming bare occurrence of that target as an actual. The relation
   must omit the formal supplied by that overwritten actual, and every other
   referenced support must remain live and disjoint under ordinary overlap.
   Run all ordinary transfer/effect/target kills first and substitute only the
   normal result with the post-write receiver. Projected, consuming, repeated,
   overlapping, distinct, or non-call shapes establish nothing. This is not
   general `set` equality.
6. For the selected-payload receiver, require the immediate bare,
   non-consuming `set outer = payload` of the same fragment type. Evaluate the
   RHS, commit and kill the target, then substitute only payload occurrences
   with the post-write outer binding if every other support survives. The
   receiver cannot be a call actual or non-result relation support and cannot
   overlap substituted support. Establish no equality or unrelated fact.
7. Stage all optimistic S12 and receiver facts in one candidate batch before
   finalizing the PRV strata. Freeze PRV-1, converge PRV-2/PRV-3 on that batch,
   and either publish the identical complete batch and checked program or
   discard both. There is no retracting fixed point or second semantic pass.
   Commit reviewed H3 on H2, record it, and leave this task `WAITING` before
   0062 claims.

## Scope and expected touch set

Use the H2 semantic/postcondition, entailment flow/source/state, provenance,
model, checker, and derivation-test owners. Touch the ordinary user-call
checker only if its existing `goal_arguments` cannot encode the exact frozen
pre-transfer formal images; that discovery requires lead review and must not
create a parallel image authority. Do not edit candidate spec/archive/ledger,
delivery, real sources, protected corpus, lowering, runtime, ABI, backend, or
gate wiring.

## Validation

- Direct, two-hop, generic, complete-only, U-not-B, earlier-SCC, self/mutual/
  seedless, and command-entry controls pin the publication formula and PRV
  owner. Bq/Uq simultaneous success has an exact B-first root-parent test.
- The twenty append-shaped direct receivers establish the relation only from
  exact result substitution after kills. Filled/pre-target identity,
  destination length, alias, projected target, nested RHS, discard, killed
  support, and rejected-call near misses are explicit.
- The fourteen selected payload shapes pass a general-shape oracle. Both a
  direct `match_stmt` and direct `value_match` selected arm receive their S12
  payload relation; only 0062's give-delivery rule excludes `value_match`.
  Wrong or unselected arm, named outcome, computed/projected/consuming RHS,
  intervening event, call-actual receiver, overlap, extra write, wrong type,
  and missing S12 establish zero relations.
- DIAG roots cover call, direct result, direct match, direct receiver, and
  selected receiver with exact same-view parents; no orphan root survives a
  failed batch. Focused validation passes, with only the expected preapproval
  activation-chain stop for any full gate.

## Stop condition and done-when

Stop on candidate drift, named-outcome metadata, general assignment/RHS
transfer, pre/post identity conflation, ambiguous aliasing, wrong-view
publication, PRV laundering, a negative fixed point, a second proof authority,
or a required path outside the named semantic/test owners.

The handoff is implementation-complete when reviewed H3 implements only the
four frozen routes and failure-atomic provenance with complete roots, and
0062's premise names H3. It remains `WAITING` until atomic activation.
