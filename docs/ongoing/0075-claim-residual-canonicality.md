# 0075 — Claim residual canonicality

Status: IMPLEMENTATION AND MIGRATION COMPLETE ON BRANCH. Canonical
exact-revision `make check` is reported in the merge handoff without a
post-gate edit to this record. This record reports how the work was carried
out. It is coordination evidence, not permission to work and not an additional
merge condition.

Workspace: `codex/claim-residual-plan` in
`/Users/bytedance/do_not_scan/whitefoot-claim-residual-plan`. The locality
implementation began from `e10b1eae`; the branch now incorporates `main` at
`0728d7c6`. Registered: 2026-08-21; updated: 2026-08-22.

The branch stable specification is now ACTIVE v0.34, SHA-256
`cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03`.
The immutable outgoing v0.33 archive has SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.

The complete live branch and `main` boundary is the four rules in
[`docs/WORKFLOW.md`](../WORKFLOW.md). In particular, branch changes did not
need intermediate approval. The exact revision entering `main` needs owner
approval and a green canonical `make check`; because this branch changes the
specification and conformance evidence, the approved content is recorded in
`governance/APPROVALS.md` as part of that merge.

## Capability delivered

The compiler now admits a written claim only as a retained runtime boundary
for a current-function-local, checker-unknown, individually load-bearing proof
residual. Assertion, intentional-abort, runtime-test-oracle,
checker-redundant, refuted, and unused-theorem uses are rejected or expressed
with ordinary control and typed outcomes instead.

Every accepted claim still:

- contributes its written `traps` effect;
- lowers through the ordinary claim instruction;
- preserves its source identity and diagnostic data;
- executes on every dynamic reach in facts-on and facts-off builds; and
- cannot be removed by optimization, review, or an optional solver.

## Claim residual implementation

The existing residual implementation forms deterministic direct,
support-canonical, and fully structural images for each predicate. The
support-canonical image alone owns the contribution basis. Admission rejects
vacuous, redundant, refuted, inconsistent, overlapping, malformed, and
non-load-bearing candidates before publication. Each contribution component
and the whole occurrence must be necessary for at least one eligible
operation, FN-8 requirement, or complete FN-9 postcondition under the fixed
otherwise-valid claim set.

The compiler retains the exact five-field derivation shape, reconstruction
evidence, contribution identity, and terminal consumer evidence in the checked
claim ledger. Counterfactual proof identities stay private to their temporary
analysis and never become lowering authority.

## Local-authority correction

The handoff review exposed a soundness hole in treating facts about returned
values as if a caller could reassert them with a claim. The correction makes
claim authority explicitly current-function-local:

- every user-call or system-call result component starts as
  `BoundaryResult`;
- that class propagates through value, control, holder, and storage flow;
- copying, projection, mutation, branch selection, dereferenced field access,
  verified `ensures`, and S12 publication do not declassify it;
- PRV-internal classification does not make a call result claim-local; and
- a caller consumes a machine-verified boundary relation directly rather than
  restating or strengthening it through a claim.

Parameters and values genuinely created in the current function remain local.
Ordinary branches may establish facts about boundary values, but a claim
component influenced by boundary-result control is still non-local. CLM-1
reports the least non-local component with its stable boundary witness and
support carrier before the occurrence can enter lifecycle, residuality,
ClaimLedger, or lowering.

The protected corpus now contains direct user-result and system-result
negative cases, plus value, control, holder, storage, projection, aggregate,
match, loop, mutation, and PRV/S12 laundering falsifiers. Same-function
remainder residuals provide positive controls.

## Performance shape and measurements

`ClaimAuthorityAnalysis` is a forward analysis of one concrete function
inventory. It is constructed once for functions that contain claims and
skipped entirely for claim-free functions. CLM-1 queries its frozen reaching
states; complete/U/B views and component or occurrence masks do not rebuild
it. PRV-1 is also frozen once from phase A because claims change neither value
nor storage flow. S12 uses retained ancestry rather than a fresh program scan.

Residuality itself remains program-wide: each contribution mask, plus the
whole-occurrence mask for a multi-component claim, reruns every function
inventory and the program-level PRV-2/PRV-3 and bridge scans. That scaling path
already existed at the `e10b1eae` baseline; this change reduces its cost by
freezing PRV-1 rather than introducing the rerun.

Warm same-machine probes recorded during this batch were:

- `utf8parse`: about 0.70 s versus about 0.93 s at the compared baseline,
  approximately 25% faster;
- optimized `wfgrep`: 28.52/26.79 s versus 32.68/29.98 s, approximately
  10–13% faster; and
- claim-locality analysis: about 226 microseconds, once per applicable
  function inventory.

These bounded measurements establish the cost shape observed on the selected
programs, not a universal compiler-speed guarantee. They do not close the
inherited high-claim-count risk from per-mask program reruns.

## Source and evidence migration

Real programs, inline semantic fixtures, backend fixtures, and protected
conformance were migrated by semantic purpose:

- deliberately false and possible-failure conditions became ordinary control,
  typed `Result` values, or nonzero exit statuses;
- runtime observations became explicit value and variant checks;
- closed-call assumptions became verified boundaries or same-function
  residuals;
- checker-redundant claims were removed;
- effect rows and reverse callers were updated with the source changes; and
- surviving claims retain exact five-field derivations and authentic
  proof-required consumers.

`x-base64-buffer-transform-run` now uses typed `IndexError` propagation and a
same-function remainder residual. The borrowed-pool integration uses typed
`PoolError` propagation and explicit observations rather than claims as error
oracles. Protected rejection migrations preserve the intended first rule and
diagnostic payload instead of weakening verdicts.

At `99df5579`, the protected corpus has 501 cases: the native adapter reports
500 passes and one intentional skip. Its source-level claim census is 95
statements in 74 `.wf` files, counted by
`^[[:space:]]*claim `. A raw-text `claim` count is not evidence because case
documentation and derivation strings intentionally discuss claims.

## Branch commits for the locality closure

- `b1fb46ed` — current-function claim authority, specification delta,
  no-claim fast path, frozen PRV-1 reuse, and hostile tests.
- `95b1e18e` — dereferenced-field authority preservation.
- `fe9b0d25` — semantic and real-program migration.
- `bd39352e` — backend fixture migration.
- `8c72200c` — first protected runtime-oracle migration.
- `a31e5517` — protected accepted-claim locality migration.
- `bbb71792` — downstream protected diagnostic preservation.
- `e637b6a9` — remaining protected runtime-oracle migration.
- `000eb415` — protected residual-case closure.
- `99df5579` — locality negatives and synchronized conformance descriptions.

Earlier commits in this branch preserve the residual-canonicality design and
implementation history. Only the current state reported above describes this
branch.

## Verification state

Completed evidence includes focused locality tests, all migrated semantic
modules, the changed real-program tests, focused backend tests, optimized
compile-and-run checks for migrated protected programs, conformance structure
and coverage checks, and the 500-pass/one-skip native adapter result.

Canonical root `make check` has not yet been run successfully on the final
exact revision. This record deliberately makes no full-green claim until that
command completes. If later edits change the branch tree, the exact-revision
test result must apply to the changed tree.

## Remaining work

1. Finish independent review of the exact branch diff, including locality,
   accepted-set, diagnostic, frozen-PRV, and cost-shape risks.
2. Finish the exact specification and conformance before/after boundary
   required by rule 4 in the merge revision; the branch ACTIVE identity and
   outgoing v0.33 archive are recorded above.
3. Run canonical root `make check` on that exact revision and correct any real
   failure without deleting, disabling, narrowing, or unwiring a test.
4. Present the exact green revision for owner approval to merge into `main`.

No other document, artifact, or coordination practice adds another
precondition.
