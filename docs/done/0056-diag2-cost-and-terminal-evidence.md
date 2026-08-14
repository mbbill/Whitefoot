# 0056 — DIAG-2 cost and terminal evidence

- **Status:** `DONE` (2026-08-13)
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12,
  `Trust prerequisite — bounded existing-DIAG-2 repair`
- **Implementation commits:**
  `0e9a206188d8cc37ec3bb248889e42109122246a` and
  `491446af053bfe8db95941e6093b30f4ff9cfb7a`
- **Measurement contract correction:**
  `48a92a94a3d9cf49dc5ff998741eddd3f8a8aea2`
- **Research evidence:**
  `research/investigations/obligation-discharge/ACCEPTANCE.md`, section
  `DIAG-2 exact-derivation retention and bounded-cost confirmation`;
  installed section SHA-256
  `7d3c6827d92c45a2916298b6d1c347fec7726117afc50293015fc5452844cc5e`

## Outcome

The independent audit confirmed complete mandatory DIAG-2 roots for frozen
UTF-8, SHA-256, four-source raw-DEFLATE, and wfgrep programs. One shared
function-local derivation DAG retains accepted bounds obligations, discharged
call goals, and every counted-range S11 atomic root. All retained nodes are
root-reachable; event pruning leaves no orphan event and establishes `E <= S`.

Across the frozen units the retained representation contains 114 roots, 374
unique nodes, 355 parent edges, 137 events, and 706 capacity-charged path
component slots, totaling 35,792 ledger-owned bytes. The audited storage bound
is `O(S + P + R + C)`; no root-local DAG copy, full program-point state,
query-triggered reclosure, or second authority was introduced.

An exclusive 64-run release session preserved every exit status, diagnostic,
advisory, source identity, and program hash. Median candidate wall-time deltas
were +63.29% for UTF-8, +34.62% for SHA-256, +60.51% for DEFLATE, and +43.43%
for wfgrep. Peak-RSS medians changed by +6.97%, -0.11%, -0.18%, and +41.47%
respectively. The Current Plan defines no numeric threshold, so these are
reported costs rather than an invented pass/fail budget.

The former task-local classification of this research section as protected
equivalent-compliance evidence was invalid and is superseded by this closure.
The section is ordinary research documentation; no approval-ledger,
specification, protected-conformance, gate-wiring, compiler-production, or
real-consumer byte changed.

## Evidence and validation

- The formal session completed 64/64 invocations with 97 clean competition
  snapshots. Its 484-entry SHA-256 manifest, self-hash, run ordering, statuses,
  diagnostics, binaries, and statistics passed independent reconstruction.
- The corrected storage audit passed 1/1 across all four frozen units, with
  exact root/node/edge/event/path/byte reconstruction and `E <= S`.
- Focused entailment passed 112/112. The compiler gate passed 718/718 core
  tests and 30/30 real programs. The repository gate passed 23/23 independent
  runner tests, coverage 131/131, core 718/718, and programs 30/30.
- The read-only native adapter remained `Pass=409 Fail=1 Skip=13`; the sole
  failure was the recorded OWN-3 A3 unsupported boundary.
- Temporary instrumentation was removed, and baseline, candidate, and task
  worktrees ended clean with every frozen identity restored.

## Remaining dependency

The existing-DIAG-2 trust prerequisite is terminal-positive. Stage 8b remains
blocked only on terminal-positive task 0053 and then requires its separate
exact specification and protected-conformance approval.
