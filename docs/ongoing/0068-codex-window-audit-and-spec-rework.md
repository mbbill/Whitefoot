# 0068 — Codex-window audit repairs and specification representation rework

Owner: lead. Workspace: `spec-rework` branch. Base: `9b6b678` (main, v0.29).
Registered: 2026-08-16, during the owner's overnight delegation of 2026-08-15.

## Authority

Owner conversation 2026-08-15: audit the Codex window `bd6dea4..HEAD` and
repair its findings; research the specification representation problem
(AI-context fit, iteration safety, process cost, conciseness) and implement
the selected design on a branch; direct commits on the branch are authorized,
while landing on main, any activation, and every protected-surface change
remain owner-approval items for the morning review. No `ACTIVE-SPEC:` line,
no archive, and no digest-chain append happens on this branch.

## Scope

1. Repairs for confirmed audit findings (workflow `wf_a7ad69d8-a03`): the
   root-entry law restoration, the two mcts_mem silent divergences, the
   deleted evidence playbook, O11's roadmap entry, the stale provenance
   design-record header, the baked Makefile tally.
2. Specification prose repairs: stale v0.18 self-references, version-tagged
   amendment prose inside rules, Status/Prior changelog eviction.
3. The representation rework selected by the 2026-08-16 research decision
   record (see `research/investigations/spec-representation/`), implemented
   as candidate commits following the freeze-candidate pattern; the spec
   identity gate is expectedly red on this branch until an owner-approved
   activation.
4. A morning review document for the owner enumerating every commit, its
   approval class, and its verification.

## Out of scope

Any semantic language change; any conformance verdict change; activation of
any candidate; merging to main.
