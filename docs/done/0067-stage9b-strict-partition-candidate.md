# 0067 — Stage 9b strict no-claim partition

- **Status:** `DONE` (2026-08-15)
- **Owner:** lead agent `/root`
- **Workspace / branch:** `/Users/bytedance/code/Whitefoot`, `codex/0067`
- **Base revision:**
  `61b91f095b7b4942b5fa570eb62929a603a93b7c`
- **Authority:** the then-`ACTIVE` Current Plan, Workstream 9b under Direction
  Outline `PROOF-8`, plus the exact v0.29 specification and protected-evidence
  owner approval recorded in `governance/APPROVALS.md`

## Outcome

The exact candidate frozen by commit
`4e4707317206a103cdb29d2f1d076d8f9807a90f` and approved in commit
`137ef4d87a5a91f7088d04e677b8c87dbd127401` is installed as active v0.29.
The outgoing v0.28 bytes are archived unchanged. Held frontend checkpoint
`ec95b7026e84b9d001cd9912b3f34ee9b2511d19` defined the reviewed
`deny_claims` declaration surface and generated data; held semantic checkpoint
`a927f2ca15471de22bbc878355fd7376d74f660b` defined CLM-3's finite
direct/import may-claim SCC summaries, existing-U OP-4/FN-8 judgments,
marked-entry pre-wrapper check, deterministic diagnostic ownership, and
failure-atomic checked publication through the one ordinary compiler path.
Those bytes are landed by the single atomic main integration containing this
record; the held commits are review identities, not main ancestors.

The protected change is exactly nine additive runnable cases and nine manifest
rows: 446 cases, 433 runnable plus 13 pending, 30 unchanged annotations, and
133/133 rule coverage. The installed adapter is
`Pass=432 Fail=1 Skip=13` in `202.22s`; `make conformance-run` exits `2` only
because `own3-pos-outlives-store` still expects `Run(0)` and reaches
`Unsupported(RegionsAndBorrows)`. No other protected case fails. Wfgrep changes
only by the twelve-byte `deny_claims ` prefix on `report_failure`, producing SHA-256
`fb2f3b44160a947d7adca9fc9b5af851b446a7bcfc179ede4f8c689b21033904`.
No older protected case or row, ordinary unmarked acceptance, runtime body,
lowering, output/error/cleanup/status behavior, facts-off behavior, or
Stage 9a ledger authority changed.

## Canonical evidence and validation

Canonical installed status and measurements are in
`research/investigations/obligation-discharge/ACCEPTANCE.md`; the current
landscape and terminal sequence are in `docs/roadmap.md` and
`docs/current-plan.md`; exact byte approval and activation identity are in
`governance/APPROVALS.md`. The approved derivation ledger remains
SHA-256 `7f2b277c3bafa8d9448f4b16b9ba0066b26668beb804cc31ee05d5c655b22806`.
Its v0.28-authority and v0.29-candidate sentences are intentionally retained
candidate-time historical prose, not present authority. The active
specification is sole language authority; the activation chain records its
installed identity, and the roadmap records terminal `PROOF-8` status.

- Focused strict semantics passed `17/17`; the non-heavy semantic selection
  passed `481/481`.
- The separately selected heavy entailment test
  `frozen_real_sources_retain_complete_entailment_roots_without_counted_false_positives`
  passed `1/1` in `711.37s`.
- The separately selected heavy provenance test
  `canonical_deflate_retains_one_subject_bridge_and_three_unasserted_calls`
  passed `1/1` in `207.28s`.
- Those selections cover the original 483 semantic tests, but were not one
  `483/483` run. The `711.37s` owning test combines multiple real sources,
  production checking, test-only `validate_derivations`, and additional
  assertions, so it has no separable per-source, wfgrep, semantic-checker, or
  validator timing attribution and is not a performance result.
- The exact wfgrep preactivation integration test ran `139.63s` and then
  failed only at the expected still-v0.28 target-qualification `CommandEntry`
  mapping pin. It was neither an activation-era passing wfgrep gate nor a
  Stage 9b semantic failure.
- The final repository-root `make check`, run with `TMPDIR` under
  `/Users/bytedance/do_not_scan`, exited `0`: specification append-only checks,
  archive identities `30`, runner `23/23`, coverage `133/133`, compiler library
  `833/833` in `679.39s`, grammar
  `9/9`, generated tables `1/1`, migration `36/36`, specification `10/10`,
  canonical corpus `3/3`, real programs `32/32` in `2170.33s`, rustdoc with
  warnings denied, and active v0.29 at the exact installed SHA with 133 rules
  and 21 activation links all passed. It printed both compiler and repository
  green markers; the conformance target retained its one deliberately ignored
  known OWN-3 integration test.

PROOF-8 is terminal. CAND-8 is unparked, but this task and the completed plan
grant no authority for its next slice; that undertaking requires a new
owner-approved high-level plan. No separate activation task was registered.
After exact approval, this same 0067 record remained the shared integration
record and already named the frontend, semantic, and atomic activation
sequence. This terminal record reports that history rather than inventing a
retroactive task.
