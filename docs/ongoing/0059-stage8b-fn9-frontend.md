# 0059 — Stage 8b FN-9 frontend and semantic surface

- **Status:** `IN PROGRESS` (claimed 2026-08-14)
- **Owner:** Codex executor
- **Workspace:**
  `/Users/bytedance/do_not_scan/whitefoot-0059-stage8b-fn9-frontend`, branch
  `codex/0059-stage8b-fn9-frontend`
- **Base revision:** held candidate H0
  `7a293861ddb0aad0cd8d494ee8f4bfa5f6cbba29`
- **Claim coordination revision:**
  `b79cf48185fd87e204097ae872e8ad7256913730`

- **Authority:** the ACTIVE Current Plan, Workstream 8b `verified
  normal-return postconditions`, under Direction Outline revision 38 item
  `PROOF-8`
- **Frozen candidate premise:** held commit
  `7a293861ddb0aad0cd8d494ee8f4bfa5f6cbba29`; specification SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`;
  outgoing v0.27 archive SHA-256
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`;
  derivation-ledger SHA-256
  `5207e1d46cfa4d03dcedb4fcd80a8fa07c30510417a2da2d6573a13dad10fc4b`

## Goal

Install the frozen FN-9 grammar, parser, canonical form, resolution identity,
checked semantic surface, and v0.28 compiler identity on top of the held
candidate. Make the native grammar verifier recognize exactly 73 productions,
90 decisions, and 97 terminal predicates. Leave every semantically valid
postcondition explicitly unsupported until task 0060 installs callee proof;
never misreport that temporary implementation boundary as invalid source.

## Direction and method

1. Claim from the exact 0058 lifecycle commit, but create the executor branch
   from held candidate `7a293861…`, not from active-v0.27 main. Recompute all
   three candidate hashes before editing. Candidate specification, archive,
   and derivation-ledger bytes are read-only inputs.
2. Implement the frozen `ensures` terminal and productions, generated grammar
   data, parser nodes, and canonical formatter. Preserve
   `requires -> ensures -> body`, the exact plain/selected-`Ok` selector
   surface, and every excluded or malformed form and diagnostic owner.
3. Extend the existing resolution pass with one private selector record in
   `ResolvedSyntaxUnit`: candidate origin, selector-owned in-block uses, and
   the first later-local collision origin. It is excluded from ordinary
   declaration and deferred-use inventories. Resolution may link exact-scope
   uses provisionally; FN-9 admission activates them only as the template
   datum. Preserve any delayed entry `ResolutionIssue` through one narrow
   semantic-outcome/driver forwarding path so its original rule, location,
   payload, and resolution stage survive after selector admission. Do not
   create a `BindingId`, storage, a second general lookup path, or an ad hoc
   checker name scan.
4. Reuse and factor the existing pure, total, nontrapping ANF and alpha-
   expansion machinery in `semantic/check/requires.rs`. Add the frozen
   RelationTemplate and checked postcondition metadata without changing
   GoalTemplate or admitting arithmetic-expression terms. Concrete generic
   identity, FN-3/FN-4 exclusion, zero-selected hard rejection, selector/local
   freshness, and deterministic residuals must match the candidate.
5. Update `compiler/src/spec.rs` to the exact v0.28 identity, the native spec
   tool's rule count from 131 to 132, and exactly the three v0.27 target-
   qualification guards to v0.28. These are held identity changes, not a
   backend, lowering, runtime, system ABI, or gate-wiring change.
6. Commit one lead-reviewable H1 on top of H0. Record its full SHA in this
   task before task 0060 may claim; keep this task live in `WAITING` until the
   later approved atomic activation.

## Scope and expected touch set

- Syntax and tools: `compiler/src/syntax/terminal.rs`,
  `compiler/src/bin/grammar_tables/{model.rs,ebnf.rs,main.rs}`,
  `compiler/src/syntax/grammar/generated.rs`,
  `compiler/src/syntax/grammar.rs`, `compiler/src/bin/grammar.rs`, parser,
  grammar, and canonical-format tests under the existing syntax homes.
- Resolution: `compiler/src/resolution/{mod.rs,scopes.rs,engine.rs,tests.rs}`
  and the existing `engine/{admission.rs,roles.rs,inventory.rs,lookup.rs}`.
- Semantic surface: new cohesive `semantic/postcondition.rs` and
  `semantic/check/ensures.rs`, plus the minimum existing semantic model,
  checker, requires, generics, contracts, and test modules that own the frozen
  metadata and admission. `compiler/src/driver.rs` may change only to forward
  a delayed original `ResolutionIssue` selected after FN-9 admission; it owns
  no lookup or semantic reinterpretation.
- Identity: `compiler/src/spec.rs`, `compiler/src/bin/spec.rs`, and exactly
  three version guards in `compiler/src/backend/qualification.rs`.

No entailment proof, S12 publication, receiver, delivery, real-source,
protected-conformance, approval-ledger, active-plan, MCTS, lowering, runtime,
ABI, runner, adapter, Makefile, or gate-wiring implementation belongs here.

## Dependencies and integration order

H0 is the sole implementation base. H1 remains held and must not enter the
active-v0.27 integration branch. Task 0060 may claim only after this record
names a lead-reviewed H1 commit and its exact validation. Later held commits
stack strictly H2 onto H1 through H6 onto H5.

## Validation

- Native grammar generation, parser, canonical round-trip, resolution,
  selector-collision, generic, contract, and semantic-surface tests cover all
  admitted and excluded forms with deterministic owner and location.
- Archive-to-candidate native grammar verification is genuinely green at
  `73 productions / 90 decisions / 97 terminal predicates` with the candidate
  frontend embedded; archive-to-archive remains the v0.27 control.
- No-`ensures` programs retain their prior frontend and checked behavior.
- Before editing, H0's compiler gate is expected to stop first at the
  candidate spec versus embedded-v0.27 identity/frontend mismatch. H0's
  repository gate instead reaches archive integrity first and stops on the
  unrecorded v0.28/stable-archive activation state before its nested compiler
  gate. Those are expected pre-H1 observations, not H1 verdicts.
- After H1, `cargo fmt`, relevant clippy/check targets, focused suites, and all
  subgates independent of activation identity pass. The grammar/frontend/spec
  identity mismatch is gone. The compiler gate may then stop only at the
  missing v0.28 approval/activation-chain premise; the repository gate may
  stop at its earlier archive-integrity expression of that same governance
  premise. Neither is called green, and any other after-H1 failure is a
  blocker.

## Stop condition and done-when

Stop on any candidate-byte drift, grammar ambiguity, unresolved resolution
choice, need for ordinary storage or a second lookup authority, duplicated
requires machinery, GoalTemplate expansion, semantic proof or publication in
this handoff, path outside the named conditional test owners and the single
resolution-outcome forwarding path above, or need for a
backend/lowering/runtime/ABI/gate change beyond the three version guards.

The handoff is implementation-complete when reviewed H1 implements exactly
the frozen surface and identity, focused validation is green, and task 0060's
premise names H1. It remains `WAITING`, not terminal, until the eventual exact
approved activation atomically lands the held stack.

## Progress

Claimed from lifecycle revision `b79cf48185fd87e204097ae872e8ad7256913730`.
The lead autonomously refined the implementation touch set after the resolver
correctly deferred entry issues but the semantic outcome lacked a route that
could preserve their original resolution rule and stage. That bounded driver
forwarding is part of the approved frontend handoff, not a new lookup path or
semantic direction. Next: complete and review H1 at exact H0.
