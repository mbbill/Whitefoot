# Optimizer-first language research corpus

Purpose: collect sourced evidence before deciding what an optimizer-first programming language should look like.

Important: this corpus intentionally does **not** make final language-design decisions yet.

## Seed artifacts

- `notes/user-directives.md` — fixed design constraints from the project owner (D0: AI-written/performance-first; D1: memory/thread-safety bugs impossible).
- `sources/deep-research/2026-06-30-wzt36pg88.output.json` — immutable raw workflow artifact copied from the temporary task output.
- `sources/deep-research/2026-06-30-wzt36pg88.manifest.json` — provenance/checksum manifest.
- `sources/source-registry.jsonl` — auditable source registry.
- `notes/workflow-confirmed-findings.jsonl` — atomic evidence cards F001-F009.
- `notes/phase2-numeric-findings.jsonl` — phase-2 numeric/LLVM undefinedness evidence cards.
- `notes/phase2-arrays-findings.jsonl` — phase-2 arrays/layout/vectorization evidence cards from direct source extraction.
- `notes/phase2-dispatch-findings.jsonl` — phase-2 dispatch/generics/runtime evidence cards from direct source extraction.
- `notes/phase2-concurrency-findings.jsonl` — phase-2 concurrency/memory-model evidence cards from direct source extraction.
- `synthesis/phase2-concurrency-findings-index.md` — human-readable phase-2 concurrency index.
- `notes/phase2-jit-findings.jsonl` — phase-2 dynamic JIT/deoptimization evidence cards from direct source extraction.
- `synthesis/phase2-jit-findings-index.md` — human-readable phase-2 JIT index.
- `synthesis/phase2-dispatch-findings-index.md` — human-readable phase-2 dispatch/generics/runtime index.
- `synthesis/phase2-arrays-findings-index.md` — human-readable phase-2 arrays/layout/vectorization index.
- `synthesis/phase2-numeric-findings-index.md` — human-readable phase-2 numeric findings index.
- `synthesis/workflow-confirmed-findings-index.md` — human-readable finding index.
- `matrices/feature-evidence-coverage.csv` — coverage/gap matrix.
- `notes/refuted-and-unverified.jsonl` — negative/unsettled workflow states.
- `notes/card-verification-2026-07-02.jsonl` — adversarial verification of the 10 load-bearing provisional cards (9 confirmed, C004 confirmed with amended sources).
- `notes/calibration-audit-v0.json` / `notes/checker-prototype-report.json` — first build-phase gate results (spec pricing; D1a checker feasibility).
- `debates/spec-critique-round1-raw.json` — 4-critic adversarial review of kernel-spec-v0 (63 findings; drove v0.1).
- `notes/missing-research-backlog.jsonl` — follow-up research tracks.
- `implementation/` — scaffolds for semantic contracts, LLVM/MLIR mappings, pass consumers, lowering sketches, and validation.
- `debates/` — agendas for the next multi-agent discussion phase.
- `debates/round1-index.md` — round-1 debate verdicts across 8 design questions (with per-topic files and full journal).
- `debates/round2-index.md` — round-2 AI-writer feature-essentialism verdicts (10 features + minimal-core synthesis).
- `debates/round3-index.md` — round-3 safety-envelope verdicts (unsafe hatch, check policy, FFI attenuation, proof burden + layered envelope synthesis).
- `debates/round4-index.md` — round-4 v2 teachability verdicts under corrected D2 (spec budget, familiarity, compiler-as-teacher + spec-architecture synthesis).

## Evidence discipline

- Treat workflow-confirmed findings as source-grounded evidence, not recommendations.
- Preserve scope limits next to every claim.
- Do not infer importance from what the first workflow happened to cover.
- Promote features through decision gates before design debate.
