# 0046 — revise and remeasure the held provenance rule

- **Status:** `IN PROGRESS`
- **Authority:** the ACTIVE stage-5a-R plan derived from Direction Outline
  revision 24; the owner's 2026-08-09 authorization to make independently
  reviewed specification revisions while completing the selected direction.
- **Owner / workspace:** Codex lead /
  `/Users/bytedance/do_not_scan/wf-0045-final-activation`, branch
  `codex/0045-ent5-activation`.
- **Base revision:** `5998b87`; task 0041 is terminal.

## Goal

Repair only the direct place-read dependency and PRV-2 diagnostic-relation
defects exposed by task 0041, then remeasure the same frozen programs. Produce
reviewable design evidence; do not change or activate the language.

## Direction and invariants

- Preserve the two-class, flow-insensitive, search-free least-fixed-point
  model and the subject-position policy. This is not a noninterference system.
- A selected stored value explicitly depends on its root and subscript offsets;
  ordinary control flow and write-address implicit flow remain outside the
  classifier and are named as such.
- PRV-2 must retain each protected leaf as its concrete [FN-2] instantiation
  plus exact [ENT-6] obligation occurrence, without an unbounded path domain or
  iteration-order diagnostic.
- O3 `requires` remains open and blocks activation. No project-, function-, or
  corpus-shaped rule is permitted.

## Method

1. Write the minimal place-read join and finite parameter/leaf-obligation
   relation, including ordinary and recursive call composition.
2. After relation convergence, reconstruct one argument event's witness with a
   terminating search over `(function, parameter, leaf)` states: minimize call
   boundaries, then order ties by complete call/argument node paths, leaf node
   path, and concrete identity. Join that callee witness to the caller's PRV-1
   predecessor chain; no path is part of the fixed-point domain.
3. Rewalk all 33 obligations and 23 claims, the five earlier boundary-bearing
   programs, and hostile direct/implicit address-dependency controls.
4. Update the held design record, `PROBE-TAINT.md`, outline, and relevant design
   memory with the reviewed result. Disposable probes leave no tracked tool.

## Progress

- **Completed:** two independent source walks reproduced task 0041. Design
  comparison selected place-read offset joining over whole-root write taint and
  path/flow sensitivity; a metadata audit derived the finite PRV-2 relation.
- **Current:** register the bounded review and turn those findings into exact
  held-design rules and frozen evidence.
- **Next:** independent rule/table review, then terminal closure or a reproduced
  blocker.

## Scope and expected touch set

- `governance/spec-evolution/provenance-gate-candidate.md`
- `research/investigations/obligation-discharge/PROBE-TAINT.md`
- `mcts_mem/whitefoot/checks-and-proofs/obligation-discharge.md`
- `docs/roadmap.md`, `docs/current-plan.md`, and this lifecycle record

No active specification, compiler, generated datum, protected conformance
case, approval ledger, or active-spec identity is in scope.

## Dependencies and integration order

Task 0041 and activation `f4c7e60` are fixed premises. This task is the only
current execution slice. Stage 6 may be proposed only after this task closes
positively; stage 7 remains a hard dependency of any provenance activation.

## Validation

- Preserve the frozen source digests and exact 33-obligation/23-claim
  denominator; report the full classification and diagnostic projection.
- Require canonical 3/3 and exactly one newly external frozen obligation
  subject: `destination_in_symbols`, yielding 19/33 external subjects, six
  unasserted-state discharges, and 13 rejected obligations under 11 claims.
  Require no new gate among the earlier 15 boundary-program claims.
- Separately enumerate all changed binding/root and signature classifications.
  Check the new `lengths` dependency of `build_huffman_table`'s result, the
  selected-offset lineage in `decode_table_symbol`'s result, and the absence of
  unsupported write-column expansion. Check all three columns through ordinary
  and recursive call fixed points.
- Add hostile controls for direct offset reads, nested offsets, external versus
  internal roots, external-index table selection, branch-laundered literals,
  `len`, and the retained write-address implicit-flow limit. Any propagation
  beyond the stated direct-dataflow boundary stops the review. The hostile
  write-address case must show that guarded `set a[external_i] = 1` leaves
  `a[0]` and its supporting claim internal/legal even when an environment
  choice makes the claim fire; the branch-laundering control likewise keeps an
  internally computed literal internal.
- Check finite least-fixed-point and deterministic witness behavior for direct,
  multiple-leaf, call, recursive, and mutually recursive PRV-2 cases.
- Run `npx mcts-mem lint`, `make repository-invariants`, `git diff --check`, and
  the complete repository gate before closure.

## Stop condition

Stop if closing either defect requires general control taint, path/flow-sensitive
storage, an unbounded diagnostic domain, or favorable corpus filtering. Record
the smallest reproducer and return the blocker rather than weakening the gate.

## Closure

On success, put the durable rule and measurement in their canonical evidence,
update the outline, move this record to `docs/done/`, and replace the Current
Plan with the next owner-selected slice. The held design remains unactivated
until stage 7 closes O3 and a later exact-byte workflow selects it.
