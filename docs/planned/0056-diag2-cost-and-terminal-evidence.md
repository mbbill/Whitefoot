# 0056 — DIAG-2 cost and terminal evidence

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-12, `Trust prerequisite — bounded existing-DIAG-2 repair`. It is not
yet claimed; claiming fills in status, owner, workspace, and base revision and
moves this number unchanged to `docs/ongoing/`.

- **Authority:** the Current Plan's exact-root completeness, hostile evidence,
  and measured parent-node, byte, proof-depth, compile-time, and peak-memory
  acceptance boundary

## Goal

Independently audit tasks 0054 and 0055 for complete DIAG-2 root coverage and
measure their bounded cost on frozen UTF-8, SHA-256, four-source raw-DEFLATE,
and wfgrep programs. Make the trust prerequisite terminal without adding a
second authority or extending its architecture.

## Direction and invariants

- Measurement uses the same canonical compiler path and exact frozen sources;
  the pre-change reference is the plan-activation baseline at `c2c4092` in a
  detached worktree.
- Report mandatory roots by class, unique arena nodes, parent edges,
  ledger-owned retained bytes, maximum reachable proof depth, wall compile
  time, and peak RSS per frozen unit. Label platform-dependent quantities and
  preserve exact commands and summaries under the canonical acceptance record.
- Root completeness is checked from the checked program: accepted subscript
  occurrences, discharged call occurrences, and counted-statement S11 groups
  each map exactly once to valid roots. Negative outcomes do not gain positive
  roots.
- No persistent profiler framework, serialization, portable IDs, cache,
  replay, CLI artifact, ProofFlow, shadow verifier, or lowering capability is
  created for measurement.

## Method

1. Claim only after task 0055 is terminal, building on task 0054 implementation
   commit `0e9a206188d8cc37ec3bb248889e42109122246a`. Use two clean detached
   worktrees at baseline `c2c4092` and the exact 0055 terminal closure
   revision; verify frozen source/spec digests and record rustc, cargo, OS,
   architecture, and SHAs.
2. Build both release compilers once with `--locked --offline`. For SHA-256,
   UTF-8, four-source raw-DEFLATE, and wfgrep, use the Current Plan's real
   release invocation, one warmup, then seven alternating baseline/candidate
   measurements. `/usr/bin/time -l` records wall/user/sys and maximum RSS;
   report median, min/max, and absolute/percentage deltas while preserving the
   expected status/advisories.
3. Collect candidate mandatory roots by class, unique nodes, parent edges,
   maximum reachable depth, and ledger-owned retained bytes. That byte count is
   the capacity-based heap storage of retained node, event, root, and depth
   buffers, nested join-predecessor buffers, and retained `FlowEvent`
   `NodePath.components`. It excludes `DerivationInventory`, nested
   `TermKind`/`RetainedGoal`/`GoalExpression` allocations, outcome containers,
   the cleared interner, and transient `FactState`/`ClosedState` scratch; it is
   not the complete retained representation or total process-memory footprint.
   Peak RSS separately measures the latter. Temporary instrumentation is
   removed from the final diff; add no CLI or script.
4. Audit the structural bound `O(S + P + R)`, where `S` is unique proof nodes
   created by existing source/closure/materialization, `P` real parent edges,
   and `R` mandatory roots. Root-local DAG copying, full program-point states,
   and query-triggered reclosure fail. There is no owner-approved numeric
   slowdown or RSS budget, so do not invent a percentage threshold.
5. Run acceptance/disposition/diagnostic comparisons, all real-program oracles,
   focused semantic tests, `make -C compiler check`, and `make check`.
6. Prepare the complete append-only
   `research/investigations/obligation-discharge/ACCEPTANCE.md` candidate in
   scratch, preserving the installed file as an exact prefix. Present its full
   SHA-256, diff, byte/line counts, impact boundary, and verification results,
   then stop for explicit owner approval under the protected equivalent-
   compliance workflow. Only the exact approved bytes may be installed and
   recorded in `governance/APPROVALS.md`; any changed candidate byte returns to
   the hard wait. Update `compiler/README.md` only as needed after that
   approval. Close the DIAG-2 prerequisite only after the exact evidence lands;
   do not advance Stage 8b unless task 0053 is also terminal positive.

## Scope and expected touch set

- `research/investigations/obligation-discharge/ACCEPTANCE.md`,
  `governance/APPROVALS.md`, `compiler/README.md`, and this lifecycle record
  only.
- Scratch profiles and detached baseline under `/Users/bytedance/do_not_scan`.
- No production Rust/tests, spec, protected conformance corpus, gate wiring,
  real consumer, lowering/backend authority, generated artifact, new script,
  roadmap, plan, or MCTS change. The exact approved acceptance append and its
  required approval-ledger entry are the only protected/approval bytes in
  scope. A discovered defect stops this task for a separately bounded fix
  rather than being absorbed here.

## Dependencies and integration order

Task 0054's implementation is fixed at
`0e9a206188d8cc37ec3bb248889e42109122246a`. Task 0055 must be terminal, and
its terminal closure revision is this task's candidate base. This task may
integrate after Stage 8a evidence, but it must refresh onto every landed
entailment change. Stage 8b is claimable only when both this task and task 0053
are terminal positive results.

## Validation

- Exact occurrence/root cardinality and parent replay pass for UTF-8,
  SHA-256, raw-DEFLATE, wfgrep, generics, recursion, joins, contradictions,
  kills, and synthetic unused counted facts.
- Each mutation control fails at the intended audit boundary.
- Baseline/result acceptance, claim lifecycle, call dispositions, residuals,
  diagnostics, runtime output, effects, cleanup, and required checks match.
- Node/edge/byte/depth, release-time, and peak-RSS results are exact and
  reproducible enough to support the Current Plan's bounded-cost judgment.
- `make -C compiler check` and `make check` are green; any existing independent
  adapter outside that gate is invoked read-only and no protected bytes or
  wiring change.

## Stop condition

Stop if root completeness cannot be audited without a second closure or
semantic walk; cost or representation requires serialization, portable
identity, cache/replay, ProofFlow, a shadow verifier, or lowering authority;
any source digest or frozen acceptance/diagnostic/runtime behavior changes;
roots are nondeterministic; storage violates `O(S + P + R)`; measurement
OOMs/times out; or a gate fails. A production fix, numeric policy, persistent
benchmark framework, specification change, or protected conformance/gate
change is outside this task and stops for the applicable task or approval.

## Done-when

The active compiler retains the complete DIAG-2 derivation set with bounded
measured cost, all hostile and complete gates pass, the exact owner-approved
canonical evidence and approval record are installed, and the trust
prerequisite is terminal for the Stage 8b decision.
