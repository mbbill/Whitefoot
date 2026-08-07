# 0032 — ENT core engine

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` docs/current-plan.md selected slice; batch-1/v0.22 candidates and rulings in governance/
- **Owner / workspace:** exec-0032 / worktree `/Users/bytedance/do_not_scan/wf-0032`, branch `task/0032-ent-core-engine`
- **Base revision:** fb7d16a915f5535cd3f1156ce0f150b5e8eacb32
- **Dependency:** none

## Goal

Build the checker's entailment substrate per active v0.21 ENT-1..6 (and the v0.22 candidate where anchors differ): difference-bound facts over tracked terms with declaration-anchored identity; the flow framework (facts forward on the FN-1 graph, kills on assignment/effect-row writes/scope-exit edges before joins, joins at merges, no-induction loop rule); branch/match fact source; ENT-4 least-fixed-point closure with reflexive bounds and contradiction handling; residual computation for one obligation. New module under semantic/, additive; unit tests per rule. No OP-4 behavior change yet — the engine runs dark (no acceptance impact) behind the existing path. Dependency: none (worktree off main; rebase over 0031 when it lands).
