# Current Plan — claim residual canonicality

Status: IMPLEMENTED AND MIGRATED on `codex/claim-residual-plan`. Canonical
exact-revision verification is reported in the merge handoff rather than by a
post-gate edit to this file. This document records technical sequencing and
remaining work. It neither grants nor withholds permission to work on the
branch and creates no approval or merge condition beyond the four rules in
[`docs/WORKFLOW.md`](WORKFLOW.md).

Aligned with Direction Outline revision 49 and `main` at
`0728d7c66e45832d132ae499066a0da8c6865896`. Active language authority: v0.34
at `spec/kernel-spec.md`, SHA-256
`cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03`.
Main remains on v0.33 until this exact branch revision is merged.

## Objective

Make every accepted claim a retained runtime boundary for a genuine proof
residual: universally true at its source point, unknown to the normative
checker before the claim, total and observational to evaluate, supported by a
five-field derivation record, and individually necessary for at least one
proof-required terminal consumer.

A claim is not an assertion, abort, conditional, or test oracle. If a
predicate can legitimately be false, source represents that possibility with
ordinary control flow, a typed result, return, or exit status. Every accepted
claim keeps its `traps` effect, source identity, ordinary lowering path, and
runtime execution in every build mode.

## Implemented semantic boundary

The checker admits a claim only after these machine judgments succeed:

1. The predicate is total, deterministic, non-consuming, and observational,
   and its `because` string has nonempty `premises`, `derivation`,
   `conclusion`, `checker gap`, and `consumers` fields.
2. The direct, support-canonical, and fully structural predicate images are
   formed deterministically. Contributions come only from the
   support-canonical image; reconstruction first rebuilds that image and then
   materializes the direct image.
3. The pre-claim state is non-vacuous and proves neither the predicate nor its
   negation. A redundant, refuted, inconsistent, overlapping, malformed, or
   otherwise non-residual occurrence is a source error.
4. Each contribution component uses only the current function's own value and
   control authority. A user-call or system-call result is `BoundaryResult`
   authority, directly and through value, control, holder, or storage flow.
   It cannot become claim-local through copying, projection, mutation,
   branching, a verified `ensures`, S12 publication, or an internal PRV class.
5. Removing each component and the whole occurrence from the otherwise fixed
   eligible set makes at least one non-contradictory, non-explosive
   proof-required operation, FN-8 requirement, or complete FN-9 postcondition
   stop discharging through that exact claim event.

`claim True()` is redundant and `claim False()` is refuted. Optional solvers,
offline review, optimizer facts, and test identity never participate in
ordinary source acceptance and never remove an accepted claim.

## Locality and performance shape

Claim-authority propagation is forward and function-local. One
`ClaimAuthorityAnalysis` is constructed for a concrete function inventory,
and functions with no claims take the no-analysis fast path. The frozen
reaching authority is queried by CLM-1 before lifecycle and residuality work.
It is not rebuilt for complete/U/B views or for component and occurrence
masks.

PRV-1 is likewise frozen once from the phase-A inventory because claims alter
no value or storage flow. This does not make residuality itself function-local:
each contribution mask, plus the whole-occurrence mask for a multi-component
claim, currently reruns every function inventory and then performs the
program-level PRV-2/PRV-3 and bridge scans with frozen PRV-1. Its cost therefore
scales with the number of claim masks times the program inventory. That mask
path already existed at the `e10b1eae` baseline; this change reduces its work by
freezing PRV-1 rather than introducing the rerun. S12 ancestry checks do use
retained derivation ancestry rather than rescanning source.

Two representative warm same-machine probes did not show a compiler slowdown
from the locality correction:

- `utf8parse` compilation was about 0.70 s versus about 0.93 s at the compared
  baseline, approximately 25% faster;
- optimized `wfgrep` compilation was 28.52/26.79 s versus 32.68/29.98 s,
  approximately 10–13% faster; and
- the locality analysis itself was about 226 microseconds and ran once per
  applicable function inventory.

These are bounded development measurements, not a general performance
guarantee. They show no material regression on the measured migrated corpus,
but they do not close the high-claim-count scaling risk from the per-mask
program reruns above.

## Completed branch slices

- `b1fb46ed` implements current-function claim authority, the no-claim fast
  path, once-per-inventory analysis, frozen PRV-1 reuse, and the corresponding
  specification and ordinary hostile tests.
- `95b1e18e` preserves authority types through dereferenced field access.
- `fe9b0d25` and `bd39352e` migrate semantic modules, real programs, and
  backend fixtures away from cross-function or oracle-style claims.
- `8c72200c`, `a31e5517`, `bbb71792`, `e637b6a9`, and `000eb415` migrate the
  protected runtime, accepted, rejection-diagnostic, system-host, and residual
  cases while preserving their intended values, exits, effects, ownership,
  and first diagnostics.
- `99df5579` adds user-result and system-result locality negatives and
  synchronizes the conformance manifest.

The current protected source census is 95 claim statements in 74 `.wf` files,
counted with `^[[:space:]]*claim `. Raw word counts are intentionally not used:
they include prose and string literals that discuss claims. The native
conformance adapter currently reports 500 passes and one intentional skip.

## Remaining sequence

1. Review the final branch diff for semantic, accepted-set, diagnostic,
   provenance, and performance regressions.
2. Preserve the measured no-regression result and record the inherited
   per-mask whole-program residuality risk; re-run representative compile-cost
   probes if later code changes touch that path.
3. Bring the live roadmap and batch record to the same implemented state.
4. Freeze the exact ACTIVE branch revision and finish the specification and
   conformance before/after content required by rule 4. ACTIVE v0.34 has digest
   `cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03`;
   the immutable outgoing v0.33 archive has digest
   `fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.
5. Commit these final bytes, then run canonical root `make check` on that exact
   revision. No document should describe the revision as
   fully green before it completes.
6. Present that exact tested revision for the single owner approval required
   to merge it into `main`.

The live documents and supporting evidence help coordinate and evaluate the
work. None is an independent permission gate or merge precondition.

## Acceptance criteria

- User-call and system-call result authority cannot be laundered into an
  accepted claim by value, control, holder, storage, S12, or PRV flow.
- A same-function residual remains expressible and load-bearing without any
  source-name, function-name, project, corpus, or test special case.
- Every retained claim is source-valid, has an exact five-field derivation,
  has at least one authentic proof-required terminal consumer, keeps `traps`,
  and executes normally at runtime.
- Expected failures and runtime observations use ordinary control or typed
  outcomes rather than deliberately false claims.
- Facts-off and facts-on compilation have the same acceptance and claim
  execution behavior.
- Locality runs once per applicable function inventory, not once per proof or
  mask, and PRV-1 is not recomputed by claim counterfactuals.
- Residual masks do not impose a material compile-time regression on the
  selected real-program workloads; the current program-wide rerun per mask is
  a known scaling risk until removed or bounded with evidence.
- The exact revision proposed for `main` passes canonical `make check`.

## Explicit non-goals

No SMT solver, proof-certificate language, serialized proof artifact, claim
elision, intentional-abort claim, compatibility mode, generalized
whole-compiler resource framework, unrelated `wfgrep` optimization, parallel
runtime, FFI, export, dynamic dispatch, or generic-container project is part
of this plan.
