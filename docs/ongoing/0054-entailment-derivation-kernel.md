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
- Each existing deterministic function walk allocates a dense `FlowEventId`
  for proof-producing events. Where a checked node already has a `NodePath`,
  the event also retains it; joins and arms use the local event ID plus a
  predecessor ordinal. Do not add paths to every checked statement or build a
  second flow graph.
- Nodes are limited to exact source/implicit facts, L0 transitivity,
  requested-bound subsumption, equality's two directed roots, disequality
  strengthening, signed opaque sources and exact L0 projections,
  contradiction, predecessor-complete joins, and the counted materialization
  marker. Parent IDs are acyclic and topologically earlier.
- Ordinary closure consequences remain query-local. Only current joins and
  the counted S11 preheader snapshot may materialize a derived relation into a
  live state. Kills remove both fact and live parent; they never leave an
  endpoint-only derived fact behind.
- Canonical choice first prefers the stronger bound. Equal bounds prefer the
  smaller proof depth (`1 + max(parent depth)`), then the lexicographically
  smaller node-kind and parent/event tuple. Closure and join emission sort
  normalized fact/goal keys, including distinct and opaque facts; no hash-map
  iteration order becomes observable.
- The ledger records only nodes needed by mandatory roots or an existing
  join/counted materialization. It does not retain every closed state, claim
  support, or an O3 absence witness.

## Method

1. Reproduce the active DIAG-2 gap and run the pre-change compiler gate.
2. Add one private `DerivationLedger` per `FunctionEntailment`, with dense
   `DerivationId(u32)`, nodes, query roots, and function-local event identity.
   Extend live bound, distinct, opaque, and contradiction bookkeeping with the
   exact producing node while preserving every public semantic result. Clones
   share IDs; kills remove both the fact and its live handle.
3. Teach the existing single `close` computation to carry canonical parents
   beside each best bound and sign. Never call a second closure to reconstruct
   a proof. Exact arithmetic uses the same safe representational policy as the
   accepted bound computation.
4. Teach establishment, kills, joins, empty/all-derivable states, ordinary-loop
   continuing kills, and counted snapshot materialization to carry or remove
   parent identities at the same event where the fact changes. A join node
   names every reaching predecessor in ordinal order, using either that
   predecessor's fact proof or its contradiction proof.
5. At each accepted subscript, retain exact bounds `NodePath`, conjunct zero,
   normalized requested bound, and one root. At each discharged ordinary-call
   requirement, retain exact call `NodePath`, concrete callee, final-check
   occurrence, substituted typed goal, and one root. Unproved/refuted outcomes
   gain no positive root. Preserve current contradiction and call-evidence
   precedence: derivable/contradiction, then positive opaque, then exact L0
   projection; retain the existing `CallGoalEvidence` category unchanged.
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

- `compiler/src/semantic/entailment.rs`, `state.rs`, `term.rs`, `flow.rs`, and
  `flow/sources.rs`.
- `compiler/src/semantic/model.rs` only for checked-program retention wiring;
  focused ordinary tests under `compiler/src/semantic/tests/`.
- No specification, protected conformance, real-program, generated, roadmap,
  Current Plan, compiler README, approval, research, provenance, lowering, or
  MCTS change. If another implementation file is required, stop for lead
  review rather than inventing a module.

## Dependencies and integration order

- Plan activation `c2c4092` was the starting premise. During hostile join
  coverage this task reproduced the pre-existing ENT-4/ENT-5 defect tracked by
  task 0057. That repair landed as
  `a6f11672eda883a37f147ce78400e2504f1c4031`; this task must now
  refresh/rebase onto it and extend its canonical strict-bound-to-disequality
  and join parents without changing the repaired semantics.
- The preserved implementation work remains isolated and uncommitted during
  that refresh. It may otherwise overlap Stage 8a tasks 0051 and 0052.
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
- A test-only structural walker validates node kind, parent type, relation
  arithmetic, and acyclicity without re-running entailment. Direct/implicit,
  transitivity/subsumption, both equality directions, two-parent
  disequality-strengthening, integer/type/array-length implicit facts,
  opaque/projection evidence, both contradiction kinds, generic substitution,
  recursion, empty/all-contradictory joins, and ordinary-loop non-induction
  each have focused coverage.
- Set commit, callee write, consume, and scope exit invalidate the same parent
  as the same live fact; element writes preserve fixed-length premises.
- Joins enumerate every reaching predecessor, using contradiction evidence for
  a contradictory-neutral edge; empty and all-contradictory inputs follow
  active ENT-4/ENT-5 exactly.
- Concrete generic instances never share local IDs; recursive calls produce
  finite local acyclic roots.
- Repeating the same fixture at least 20 times produces byte-identical
  normalized root/node dumps.
- Acceptance, dispositions, residuals, diagnostics, lowering, runtime output,
  effects, and required checks are byte-for-byte unchanged.
- `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make -C compiler check`
  and `env TMPDIR=/Users/bytedance/do_not_scan/whitefoot-cargo-tmp make check`
  pass.

The retained representation exposes exact root-class counts, unique node
count, parent-edge count, maximum reachable depth, and ledger-owned retained
bytes (arena capacities plus nested join-predecessor capacities). It excludes
transient `FactState`/`ClosedState` scratch and adds no CLI or persistent
measurement script; task 0056 records the frozen-program measurements.

## Stop condition

Stop with the smallest missing case if exact roots require a second closure or
semantic walk, a serialized/portable identity, a shadow verifier, ProofFlow,
lowering capability, full-state retention, root-local proof copying, claim/O3
witnesses, guessed parent, cyclic proof, or incomplete join/kill accounting;
if exact joins cannot be represented by function-local events and predecessor
ordinals; if the representation changes acceptance, diagnostics, or runtime
behavior; or if it cannot retain exact call substitution and contradiction
compactly.

## Progress and closure

- **Completed:** active plan, Stage-1 research evidence, live design node and
  rejected verifier/replay alternatives consulted; valid clean pre-gate;
  private shared-DAG implementation and a passing exact-root structural
  regression within the closed touch set.
- **Current:** refresh/rebase the preserved six-file implementation onto 0057
  commit `a6f1167`, resolving the repaired closure only by adding exact
  derivation parents for its now-correct semantic result.
- **Next:** rerun the opposite-orientation join witness, the structural root
  walker, every hostile proof case, and the complete gates.

Close only through lead review by moving this record to `docs/done/` with the
landed commit and validation. The DIAG-2 prerequisite remains incomplete until
tasks 0055 and 0056 also close.
