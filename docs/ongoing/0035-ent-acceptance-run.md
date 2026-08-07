# 0035 — ENT acceptance run

This is a temporary live coordination record, not execution authority.

- **Status:** `ONGOING` (claimed 2026-08-07)
- **Authority:** `ACTIVE` docs/current-plan.md selected slice; batch-1/v0.22 candidates and rulings in governance/
- **Owner / workspace:** exec-0035 / /Users/bytedance/do_not_scan/wf-0035 (branch task/0035-ent-acceptance-run)
- **Base revision:** 9fa3d6d
- **Dependency:** 0034

## Goal

The preregistered acceptance criterion: the real checker reproduces SIMULATION.md's proven/claim/branch buckets on utf8parse, the deflate-dynamic unit, and sha256_abc, with one-line residuals; add conformance cases covering CLM-1/2 and each ENT rule (positive and negative); full gates green. Divergence from the simulation stops the task with evidence — the buckets are the falsifier, not a target to tune toward. Dependency: 0034 terminal.
