# 0032 — ENT core engine

This is a temporary live coordination record, not execution authority.

- **Status:** `DONE` (2026-08-07: engine dark-landed, 25 tests, gates green; ENT-5 loop ambiguity reported for next semantic batch)
- **Authority:** `ACTIVE` docs/current-plan.md selected slice; batch-1/v0.22 candidates and rulings in governance/
- **Owner / workspace:** exec-0032 / worktree `/Users/bytedance/do_not_scan/wf-0032`, branch `task/0032-ent-core-engine`
- **Base revision:** fb7d16a915f5535cd3f1156ce0f150b5e8eacb32
- **Dependency:** none

## Goal

Build the checker's entailment substrate per active v0.21 ENT-1..6 (and the v0.22 candidate where anchors differ): difference-bound facts over tracked terms with declaration-anchored identity; the flow framework (facts forward on the FN-1 graph, kills on assignment/effect-row writes/scope-exit edges before joins, joins at merges, no-induction loop rule); branch/match fact source; ENT-4 least-fixed-point closure with reflexive bounds and contradiction handling; residual computation for one obligation. New module under semantic/, additive; unit tests per rule. No OP-4 behavior change yet — the engine runs dark (no acceptance impact) behind the existing path. Dependency: none (worktree off main; rebase over 0031 when it lands).

## Progress

- Completed: module `compiler/src/semantic/entailment{,.rs}` (term substrate,
  fact state and ENT-4 closure, ENT-3/ENT-5 flow walker, S1 source with both
  comparison-origin shapes, ENT-6 obligation judging and residual rendering);
  dark wiring in `check_function_signature` retaining per-function obligation
  dispositions on the checked program; 25 unit tests in
  `semantic/tests/entailment.rs` covering each rule family and the F2
  stale-fact/fresh-binding shapes through break/give/propagate edges and
  nested regions. Landed on branch at 152cb73. `make -C compiler check` and
  `make check` green before and after; acceptance unchanged (full suite and
  program corpus pass with the engine running dark over every function).
- Current: reported to lead for review; branch tip 152cb73.
- Next: lead review and integration; fact sources S2/S4..S10 are task 0033;
  OP-4/CLM behavior is task 0034.

## Finding (spec-precision, reported to lead)

ENT-5's no-induction loop rule ("minus every fact having a support member
that any kill event (a)–(d) occurring anywhere inside the loop body ... may
kill") is readable two ways when the body contains a function-exit edge
(`return`, `propagate` Err): read structurally, that edge is a kill event (d)
for every binding-supported fact, so any loop containing an error return
subtracts all pre-loop facts (including future S4 requires facts) at the
head; read path-sensitively, kills on edges that cannot reach the next
iteration head never subtract. The two readings derive different sets on
real shapes (obligation-discharge SIMULATION.md's deflate loops rely on the
permissive reading; the literal words state the structural one). The engine
implements the literal structural reading; no test pins the contested
sub-case, pending an owner/lead ruling for the next candidate batch.

## Validation

Unit tests per rule family (25, all green); `make -C compiler check` green;
repository `make check` green at branch tip.
