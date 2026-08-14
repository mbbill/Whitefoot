# 0062 — Stage 8b value-if-only fact delivery

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-14, Workstream 8b. It is not yet claimed; claim only from the reviewed
held H3 recorded by task 0061.

- **Authority:** the ACTIVE Current Plan, Workstream 8b `verified
  normal-return postconditions`, under Direction Outline revision 38 item
  `PROOF-8`
- **Frozen candidate:** commit `7a293861ddb0aad0cd8d494ee8f4bfa5f6cbba29`,
  specification SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`
- **Execution premise:** the exact lead-reviewed H3 commit recorded by task
  0061; claim and integrate only onto H3

## Goal

Implement bounded typed L0 relation delivery for `value_if` only. Preserve a
machine-visible distinction from byte-similar `value_match`, which must deliver
zero relations, and extend the one existing DIAG-2 DAG without introducing
assignment equality or another flow authority.

## Direction and method

1. Carry `ValueInitializerKind::{ValueIf, ValueMatch}` from the actual checked
   production through the checked model. Entailment reads this discriminator;
   it must not rediscover the source form from shape or text.
2. The carrier must be a non-consuming bare tracked own-fragment binding of
   the exact receiver type, rooted at a body `let`, `for` binder, parameter, or
   match binder, with no suffix. A literal, named const, const generic, Z,
   counted capture, requires local, projection, subscript, computed value,
   construction, call, borrow, move, wrong mode, or wrong type carries no
   relation. For each reaching `value_if` give edge and each eligible typed L0
   bound or disequality containing that carrier, evaluate the value, replace
   every occurrence of the delivered atom with the receiving binding, then
   apply ordinary scope/event kills to every other support. Facts not
   containing the carrier and opaque signed goals are not candidates. Carry
   the surviving relation and exact parent on that edge.
3. Feed all reaching edges to the ordinary L0 join and retain only the joined
   weakest/largest-constant relation on the receiver plus identical surviving
   support. Run complete, U, and B independently; no relation or edge evidence
   crosses views. Missing, non-atom, consuming, ill-typed, killed, or
   non-common edge evidence establishes nothing.
4. Emit exact `PostconditionGive` and `PostconditionDeliveryJoin` nodes with
   every reaching positive edge parent in deterministic order. Kill facts and
   their parents together; do not reconstruct roots after the flow.
5. Add every eligible delivery fact and root to H3's one optimistic semantic
   batch alongside S12 before the PRV strata finalize. PRV-1 freezes first;
   PRV-2/PRV-3 then converge over that shared batch. Any rejection event
   discards all S12 facts, delivery facts, their roots, and the checked program
   together; no delivery fact has a separate publication verdict or pass.
6. An otherwise byte-similar `value_match` produces no delivery image under
   every source shape. Keep its downstream OP-4/FN-8 failure visible. Commit
   reviewed H4 on H3, record it, and leave this task `WAITING` before 0063.

## Scope and expected touch set

Touch the minimum existing checked-control model, entailment flow/state, and
derivation tests required for the discriminator and per-edge relation image.
Expected primary owners are `compiler/src/semantic/check/control.rs`, the
checked statement model, `semantic/entailment/{flow.rs,state.rs}`, and focused
entailment/derivation tests. The existing provenance batch owner may be touched
only to include delivery roots in H3's already-frozen atomic candidate set; it
must not add a PRV class, pass, or verdict. Preserve existing lowering
behavior. Do not edit candidate spec/archive/ledger, call publications,
receivers, real sources, protected evidence, lowering, runtime, ABI, backend,
or gate wiring.

## Validation

- Positive `value_if` relations include unequal branch bounds that join to the
  exact common weakest bound and the frozen A10 form.
- Byte-similar `value_match`, statement `if` plus `set`, non-atom/consuming
  delivery, missing reaching relation, wrong type, killed support, nested
  initializer, unequal nonjoinable support, and cross-view parent attempts all
  establish zero delivery relations.
- Assertions count the checked discriminator, edge ordering, delivered-atom
  substitution direction, support kills, root parents, deterministic join,
  and no general equality. A failed PRV batch leaves no S12, delivery, orphan
  root, or checked program; success finalizes both fact families once. Focused
  fmt, clippy, and cargo validation passes; any full gate has only the expected
  preapproval chain stop.

## Stop condition and done-when

Stop on candidate drift, a missing production discriminator, any positive
`value_match` delivery, inverse or unrelated substitution, a new join rule,
cross-view evidence, assignment equality, a second flow/derivation authority,
or a required change to lowering/runtime/ABI/gates.

The handoff is implementation-complete when reviewed H4 provides only the
frozen `value_if` delivery and the exact `value_match` negative, and task
0063's premise names H4. It remains `WAITING` until atomic activation.
