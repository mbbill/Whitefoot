# 0064 — Stage 8b protected corpus and combined exact packet

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-14, Workstream 8b. It is not yet claimed; claim only from the reviewed
held H5 recorded by task 0063.

- **Authority:** the ACTIVE Current Plan, Workstream 8b `verified
  normal-return postconditions`, its exact specification/protected-compliance
  approval boundary, and Direction Outline revision 38 item `PROOF-8`
- **Frozen candidate:** commit `7a293861ddb0aad0cd8d494ee8f4bfa5f6cbba29`,
  specification SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`
- **Execution premise:** the exact lead-reviewed H5 commit recorded by task
  0063; claim and integrate only onto H5

## Goal

Add the smallest protected conformance matrix needed to pin the exact frozen
Stage 8b semantics, prove that every pre-existing protected case and manifest
row is byte-identical and disposition-identical, and form one combined exact
specification plus protected-conformance owner packet. Then hard-wait without
editing the approval ledger or starting activation.

## Protected matrix and method

1. Add cases only under `tests/conformance/cases/` and append corresponding
   `tests/conformance/manifest.jsonl` rows. Do not modify, delete, rename, or
   reorder an existing case or row. Freeze exact new filenames, row count,
   declared verdict/rule/status, source hashes, and manifest hashes before the
   packet review.
2. Add exactly these fourteen case IDs and manifest oracles. Every row has
   `status: runnable`; the listed `rules` array and `expect` object are exact:
   - `fn9-pos-plain-direct-result`: rules `[FN-9, ENT-3, ENT-6]`, run exit 0;
     plain result plus fresh direct ordinary-let publication.
   - `fn9-pos-ok-selected-receiver`: rules `[FN-9, ENT-3, ENT-5, ENT-6]`, run
     exit 0; selected `Ok` summary plus immediate bare payload-to-outer
     receiver.
   - `fn9-neg-no-selected-normal-exit`: rules `[FN-9]`, reject FN-9 at the
     selector with residual `no selected normal exit`.
   - `fn9-neg-unproved-selected-return`: rules `[FN-9, ENT-4]`, reject FN-9 at
     the first selected unproved return.
   - `fn9-neg-entry-image-kill`: rules `[FN-9, ENT-5]`, reject FN-9 after the
     first overlapping formal-image kill.
   - `fn9-neg-same-scc-summary`: rules `[FN-9, ENT-6]`, reject FN-9 where proof
     needs same-SCC S12; include an independently provable recursive control
     in the same source.
   - `ent3-pos-stage8b-bit-sources`: rules `[ENT-3, FN-8]`, accept; unsigned
     direct `iand` and literal/earlier-named-constant-one shift sources.
   - `ent3-neg-stage8b-local-one`: rules `[ENT-3, FN-8]`, reject FN-8 because a
     local binding equal to one creates no shift source for the exact call
     requirement.
   - `fn9-pos-direct-set-receiver`: rules `[FN-9, ENT-3, ENT-5, ENT-6]`,
     accept; same-binding direct result with the overwritten formal absent from
     the relation.
   - `fn9-neg-named-outcome-no-publication`: rules `[FN-9, ENT-3, FN-8]`,
     reject FN-8 because naming then matching the whole outcome carries no S12
     fact to the exact later requirement.
   - `ent5-pos-value-if-delivery-join`: rules `[ENT-5, FN-8]`, run exit 0;
     forward delivery and weakest-bound `value_if` join.
   - `ent5-neg-value-match-no-delivery`: rules `[ENT-5, FN-8]`, reject FN-8
     because a byte-similar `value_match` carries no relation to the exact
     later requirement.
   - `prv2-neg-complete-only-postcondition`: rules
     `[FN-9, ENT-3, PRV-1, PRV-2, FN-8, OP-4, SYS-2]`, reject PRV-2 when a
     complete-only fact would launder an external protected leaf.
   - `prv2-pos-postcondition-b-summary`: rules
     `[FN-9, ENT-3, PRV-1, PRV-2, FN-8, OP-4, SYS-2]`, accept; a B-summary
     control succeeds without assertion or S4 laundering.
   Internal derivation parent shapes and metrics remain ordinary Rust tests,
   not protected-corpus internals.
3. Differential all 423 existing cases by source and manifest bytes and by
   actual verdict, owning rule, and status—not merely by failure set. Preserve
   30 coverage annotations and the adapter baseline
   `Pass=409 Fail=1 Skip=13`, whose sole failure remains OWN-3 unsupported,
   before counting additive cases. The exact expected additive result is 437
   cases, `Pass=423 Fail=1 Skip=13`, and rule coverage 132/132; the sole
   failure remains OWN-3.
4. Run the native grammar verifier against exact previous/candidate bytes and
   every applicable focused compiler, real-program, protected, adapter,
   formatting, and lint check that does not require an installed activation
   chain. Do not temporarily add a fake approval or alter a gate to obtain
   green output.
5. If invoked, the full compiler/repository gates must stop only at the exact
   missing v0.28 approval/activation-chain premise. Record that expected
   governance stop; do not claim either full gate green before activation.
6. Freeze H6 on H5 and independently audit the full H0–H6 diff, exact
   specification/archive/ledger/compiler/source/protected hashes, generated
   identities, impact inventory, verifier results, old/new corpus arithmetic,
   diagnostics, oracles, limitations, and atomic activation touch set.
7. Present that unchanged combined packet to the owner and set this task
   `WAITING`. Do not allocate an activation task, edit
   `governance/APPROVALS.md`, or continue in the same turn. Any changed
   specification or protected byte invalidates the packet and returns to this
   step.

## Scope and integration order

The only new implementation bytes in this handoff are additive protected case
files and appended manifest rows. Read-only scratch packet/audit material stays
below `/Users/bytedance/do_not_scan`. H6 remains on the isolated held stack;
none of H0–H6 enters active-v0.27 main independently.

After explicit owner approval of the exact unchanged packet, the lead may
allocate the next task number for one atomic activation. That later task alone
may write the real approval entry and ACTIVE-SPEC chain, install the outgoing
archive, and materialize the exact final H6 tree as one coherent main change
without making H0–H6 intermediate commits main ancestors. It may then update
the top derivation-ledger counts to 83/49/132, update compiler documentation,
roadmap, plan and task lifecycle, apply and lint the required MCTS paired
decision update, and run the first genuine full compiler/repository gates.
Approval of this plan or H0 alone is not that authority.

## Validation, stop, and done-when

The packet must be independently reproducible from exact commits and report
every source/manifest hash, verdict/rule/status differential, grammar and rule
count, real-program oracle, protected total, adapter total, expected
preactivation gate stop, and activation impact. It must explain accepted-set
narrowing/widening and unchanged runtime, effects, errors, cleanup, lowering,
ABI, runner, adapter, and gate wiring.

Stop on candidate or predecessor drift, any old protected-byte or disposition
change, need to alter runner/adapter/gate wiring, a second unsupported boundary,
behavior/root mismatch, incomplete impact, nonreproducible hash, or any
unwritten semantic choice. This task reaches its preapproval done condition
only when H6 and the exact packet are frozen and independently reviewed; it
then remains `WAITING` at the owner hard wait. Terminal disposition occurs only
with the later approved atomic activation or an honest stopped outcome.
