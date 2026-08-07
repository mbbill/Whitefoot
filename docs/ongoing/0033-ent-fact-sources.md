# 0033 — ENT remaining fact sources

This is a temporary live coordination record, not execution authority.

- **Status:** `ONGOING` (claimed 2026-08-07)
- **Authority:** `ACTIVE` docs/current-plan.md selected slice; batch-1/v0.22 candidates and rulings in governance/
- **Owner / workspace:** exec-0033 / worktree `/Users/bytedance/do_not_scan/wf-0033`, branch `task/0033-ent-fact-sources`
- **Base revision:** 482609d39f82d4170dff297334d6cee601876256
- **Dependency:** 0032 (terminal, `docs/done/0032-ent-core-engine.md`)

## Goal

Extend 0032's engine with the remaining ruled sources: FN-8 requires substitution, check/claim facts, buffer_new/array_new length equality, const-array element ranges, literal/constant propagation with constant-offset arithmetic, S10 boundary count bounds (QUAL trust class). Unit tests per source incl. kill discipline. Still dark. Dependency: 0032 terminal.
