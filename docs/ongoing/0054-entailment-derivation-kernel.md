# 0054 — entailment derivation kernel

- **Status:** `IN PROGRESS`
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12, `Trust
  prerequisite — bounded existing-DIAG-2 repair`, derived from Direction
  Outline revision 33 item `PROOF-8` and active v0.27 DIAG-2
- **Owner / workspace:** Codex executor /
  `/Users/bytedance/do_not_scan/whitefoot-0054-entailment-derivations`, branch
  `codex/0054-entailment-derivations`
- **Base revision:**
  `c2c40924b5b7a4ac4fbcb54a3b88b9d025285e7d`

## Goal

Add the smallest lifetime-bound internal derivation arena to the existing
canonical entailment pass. Record canonical parents as facts are established,
closed, strengthened, joined, materialized, or used; retain one exact root for
every accepted subscript and every discharged ordinary-call goal, including
contradictory discharge and concrete goal substitution. Do not rerun closure,
change acceptance, or create another proof authority.

## Direction and invariants

- The existing engine remains the sole completeness and acceptance authority.
  Derivations are retained checked-program evidence only; lowering,
  optimization, and source diagnostics gain no new authority.
- Dense IDs are local to one concrete checked function instance and borrow
  that unit's existing term, goal, occurrence, and `NodePath` identities. No
  serialization, hashes, portable identity, replay schema, ProofFlow, shadow
  verifier, or second semantic walk is admitted.
- A proof point distinguishes source occurrence and phase: before/after
  transfer, set commit, claim S3 establishment, scope exit, join, and counted
  snapshot. `NodePath` alone is insufficient.
- Nodes cover exact source/implicit facts, L0 transitivity, equality's two
  directions, disequality strengthening, signed opaque sources and exact L0
  projections, contradiction, predecessor-complete joins, and the counted
  materialization marker. Parent IDs are acyclic and topologically earlier.
- Ordinary closure consequences remain query-local. Only current joins and
  the counted S11 preheader snapshot may materialize a derived relation into a
  live state. Kills remove both fact and live parent; they never leave an
  endpoint-only derived fact behind.
- Canonical choice follows existing deterministic source/term/goal order and
  prefers the already selected smallest bound. Equal alternatives choose the
  lexicographically earliest parent tuple; no hash-map iteration order becomes
  observable.

## Method

1. Reproduce the active DIAG-2 gap and run the pre-change compiler gate.
2. Add private dense derivation IDs, nodes, query roots, and point/event
   identity under `compiler/src/semantic/entailment/`. Extend live bound,
   distinct, and opaque fact bookkeeping with their exact producing node while
   preserving the public semantic result of every operation.
3. Teach the existing single `close` computation to carry canonical parents
   beside each best bound and sign. Never call a second closure to reconstruct
   a proof. Exact arithmetic uses the same safe representational policy as the
   accepted bound computation.
4. Teach establishment, kills, joins, empty/all-derivable states, ordinary-loop
   continuing kills, and counted snapshot materialization to carry or remove
   parent identities at the same event where the fact changes. A join root
   names every reaching noncontradictory predecessor in canonical order.
5. At each accepted subscript, retain its normalized query and exact root. At
   each discharged ordinary-call requirement, retain the concrete callee,
   final-check occurrence, substituted typed goal, exact positive or
   contradiction root, and existing disposition. Unproved/refuted outcomes
   gain no positive root.
6. Publish the arena and roots inside `FunctionEntailment`; do not duplicate
   terms or goals and do not expose a stable artifact format.
7. Add focused hostile tests for direct and transitive bounds, equality,
   strengthening, opaque and projection grounds, both contradiction kinds,
   source ordering, join predecessor coverage, writes, projected callee
   writes, consume, scope exit, ordinary loops, call actual ordering,
   recursion, concrete generics/const substitution, named constants, borrows,
   and deterministic repeats.
8. Run focused and complete gates and integrate only after lead review.

## Scope and expected touch set

- `compiler/src/semantic/entailment.rs`, `state.rs`, `flow.rs`,
  `flow/sources.rs`, and a cohesive new sibling module only if it owns the
  derivation data/invariants rather than forwarding calls.
- `compiler/src/semantic/model.rs` only for checked-program retention wiring;
  focused ordinary tests under `compiler/src/semantic/tests/`.
- `compiler/README.md` only after the retained capability is accurate.
- No specification, protected conformance, real-program, generated, roadmap,
  Current Plan, approval, or MCTS change.

## Dependencies and integration order

- Plan activation `c2c4092` is the sole premise. It may run in parallel with
  Stage 8a tasks 0051 and 0052.
- Task 0055 depends on this task's landed arena and exact root interfaces.
  Task 0056 depends on 0055. Stage 8b depends on 0053 and 0056 terminal.
- If Stage 8a evidence integrates first, refresh/rebase and rerun all affected
  checks. There is no semantic last-writer-wins resolution.

## Validation

- Every accepted subscript has exactly one exact root at its judgment point;
  every discharged call has concrete substitution plus exactly one positive
  or contradiction root; no failed judgment carries a positive root.
- Rewalking the retained parents from each root reaches only exact source or
  implicit facts and reproduces the normalized query without invoking closure.
- Set commit, callee write, consume, and scope exit invalidate the same parent
  as the same live fact; element writes preserve fixed-length premises.
- Joins enumerate every reaching noncontradictory predecessor; empty and
  contradictory inputs follow active ENT-4/ENT-5 exactly.
- Concrete generic instances never share local IDs; recursive calls produce
  finite local acyclic roots.
- Acceptance, dispositions, residuals, diagnostics, lowering, runtime output,
  effects, and required checks are byte-for-byte unchanged.
- `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make -C compiler check`
  and `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make check`
  pass.

## Stop condition

Stop with the smallest missing case if exact roots require a second closure or
semantic walk, a serialized/portable identity, a shadow verifier, ProofFlow,
lowering capability, guessed parent, cyclic proof, or incomplete join/kill
accounting; if the representation changes acceptance or runtime behavior; or
if it cannot retain exact call substitution and contradiction compactly.

## Progress and closure

- **Completed:** active plan, Stage-1 research evidence, live design node and
  rejected verifier/replay alternatives consulted; task registered.
- **Current:** create and refresh the isolated worktree, add a regression that
  proves current outcomes retain no exact parents, then design the private
  dense arena inside the fixed boundary above.
- **Next:** implement parent-carrying closure and exact subscript/call roots.

Close only through lead review by moving this record to `docs/done/` with the
landed commit and validation. The DIAG-2 prerequisite remains incomplete until
tasks 0055 and 0056 also close.
