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
- Report arena nodes/edges, logical retained bytes, maximum and distribution
  of proof depth, wall compile time, and peak RSS per frozen unit. Label any
  platform-dependent quantity and preserve raw commands/results under the
  canonical acceptance record.
- Root completeness is checked from the checked program: accepted subscript
  occurrences, discharged call occurrences, and counted-statement S11 groups
  each map exactly once to valid roots. Negative outcomes do not gain positive
  roots.
- No persistent profiler framework, serialization, portable IDs, cache,
  replay, CLI artifact, ProofFlow, shadow verifier, or lowering capability is
  created for measurement.

## Method

1. Claim only after tasks 0054 and 0055 are terminal. Refresh both the result
   and detached baseline worktrees and verify exact source/spec identities.
2. Add only focused ordinary tests needed to walk every retained root and
   compare complete occurrence inventories. Mutation controls delete/rebind a
   root, parent, join predecessor, kill, substitution, and S11 entry and must
   fail the audit deterministically.
3. Use temporary bounded instrumentation to print node/edge/byte/depth counts;
   remove it unless the same private method is directly needed by permanent
   tests. Measure release compilation wall time and peak RSS with repeated
   baseline/result runs and record method, sample count, and limitations.
4. Re-run acceptance/disposition/diagnostic snapshots, the complete raw-
   DEFLATE and wfgrep oracles, facts-on/off controls, focused semantic tests,
   and the complete repository gate.
5. Record exact installed results in
   `research/investigations/obligation-discharge/ACCEPTANCE.md` and update
   `compiler/README.md` only as needed. Close the DIAG-2 prerequisite; do not
   advance Stage 8b unless task 0053 is also terminal positive.

## Scope and expected touch set

- Focused ordinary semantic tests and minimal private audit helpers in the
  existing entailment modules.
- `compiler/README.md`, the existing acceptance record, and this task record.
- Scratch profiles and detached baseline under `/Users/bytedance/do_not_scan`.
- No spec, protected conformance, real consumer, lowering/backend authority,
  generated, roadmap, plan, approval, or MCTS change unless a genuinely new
  durable decision is separately identified and handled through its skill.

## Dependencies and integration order

Tasks 0054 and 0055 must be terminal. This task may integrate after Stage 8a
evidence, but it must refresh onto every landed entailment change. Stage 8b is
claimable only when both this task and task 0053 are terminal positive results.

## Validation

- Exact occurrence/root cardinality and parent replay pass for UTF-8,
  SHA-256, raw-DEFLATE, wfgrep, generics, recursion, joins, contradictions,
  kills, and synthetic unused counted facts.
- Each mutation control fails at the intended audit boundary.
- Baseline/result acceptance, claim lifecycle, call dispositions, residuals,
  diagnostics, runtime output, effects, cleanup, and required checks match.
- Node/edge/byte/depth, release-time, and peak-RSS results are exact and
  reproducible enough to support the Current Plan's bounded-cost judgment.
- `make -C compiler check`, `make check`, and the independent conformance run
  are green or carry only the exact unchanged OWN-3 unsupported boundary.

## Stop condition

Stop if root completeness cannot be audited without a second closure or
semantic walk; cost or representation requires serialization, portable
identity, cache/replay, ProofFlow, a shadow verifier, or lowering authority;
any frozen acceptance/runtime behavior changes; or the retained representation
is not compact enough to satisfy the Current Plan without material expansion.

## Done-when

The active compiler retains the complete DIAG-2 derivation set with bounded
measured cost, all hostile and complete gates pass, canonical evidence is
installed, and the trust prerequisite is terminal for the Stage 8b decision.
