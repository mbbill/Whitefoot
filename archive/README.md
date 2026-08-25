# Archive

Superseded or shelved material, kept for the record. Nothing here gates
anything; no tool or test reads from this tree.

Machine-specific locations in retained records are redacted with descriptive
tokens such as `<historical-repository-root>`, `<local-home>`, and
`<local-workdir>`. The tokens preserve provenance roles without retaining a
developer's absolute directory.

- `DECISION_SPRINT.md`, `ROADMAP.md` — pre-consolidation plans, superseded by
  `/THE-PLAN.md` (2026-07-10). Kept because gates-log entries cite them.
- `HANDOVER-2026-07-17.md` and `compiler/PLAN-2026-07-17.md` — the last
  competing handover and compiler roadmap, retired when the owner made
  `/THE-PLAN.md` the sole execution plan.
- `research/validation-harness-plan.md` and
  `research/systems-performance-coverage-FOLLOW-UPS-2026-07-17.md` — retired
  policy and follow-up registers whose live requirements moved into
  `/THE-PLAN.md`.
- `tools/verify_performance_research_status.py` — the old duplicated-status
  verifier, replaced by `/tools/verify_project_state.py`.
- `research/` — the evidence-first research era: multi-agent debates
  (`debates/`), source papers (`sources/`), feature matrices, synthesis
  notes. This produced the corpus that CONSTITUTION.md and the spec derive
  from; the derivation ledger cites into it.
- `experiments/` — corpus-era measurement studies (noalias collapse vs
  Rust/C, region-effect scatter residual, guarded-plan parallelism). Their
  conclusions are absorbed into the corpus notes and THE-PLAN's evidence
  ledger; the active successors live in `/experiments/`.
- `m3/` — the model-tier authorship harness, shelved per D5 (models improve
  faster than the weak-writer test depreciates) but SHELF-READY: the
  requires-accounting design (section 13.4) names it for the authorship
  experiments, and `trial.py` runs against any model CLI unchanged.
- `research/minimal-systems-capability/` — the superseded B-Strata /
  Candidate B/C / G0-Core capability-research era (2026-07-14..15), suspended by
  D15. Historical evidence and falsifiers for the active
  systems-performance-coverage design; its standalone verifier tools are under
  `verifier-tools/` (inert — not run). ~179MB.
- `research/capability-floor/` — the general-purpose data-structure capability
  floor research (2026-07-13) that preceded and fed the D15 pass.
- `toolchains/self-hosting-2026-07-20/` — the retired Whitefoot wfc, Python
  democ, and tape-era inventory. Their exact Git
  identities and replay instructions are recorded inside. The active compiler
  starts fresh in Rust and imports nothing from this archive.
