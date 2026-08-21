# Batch 0074 — proof-derived parallelism v1 (permission, ledger, actualization)

Branch: `par/proof-derived-parallelism` (worktree, from main 4f01bab6).
Authority: owner chartering direction, 2026-08-21 (recorded verbatim in
`research/investigations/proof-derived-parallelism/DESIGN.md` §0), under the
merge-boundary process landed in 93aedd79. The plan revision rides this
branch as PROPOSED and activates at merge.

## Scope

Implement DESIGN.md §2 "In" exactly: permission judgment P (four
conditions + claim-free eligibility), non-normative `--par-ledger`,
default-off runtime actualization (`WF_WORKERS`), spec CANDIDATE v0.34 with
the single [PAR-1] rule, compiler tests (grants, per-condition denials,
codegen, in-crate determinism repeat), demo workload + measured RESULTS,
durable landing of deciding research probes, current-plan (PROPOSED) and
roadmap PAR updates.

Exclusions (deferred with triggers, DESIGN.md §2 "Out"): `pal` marker;
Tier A/B loop machinery; I/O concurrency lane (sequenced first at plan
level, own packet); claim-bearing actualization / arbitration; heartbeat
policy; reduce-clause regrouping.

## Approval classes this batch will touch

- `spec/kernel-spec.md` bytes: YES — CANDIDATE v0.34 on the branch;
  activation is the approved merge's activation commit. Packet carries
  SHA-256, exact diff, impact inventory, grammar-verifier output.
- Protected conformance/compliance evidence: NO changes. Zero corpus
  cases added or modified (the rule changes no acceptance and no verdict;
  rationale in DESIGN.md §6). Gates and wiring untouched.
- Repository root: no new entries.
- Rulings requested at merge (not blocking branch work): (a) permission
  attribution is not "undeclared parallelism" under the recorded
  auto-parallelism constraint (it changes no acceptance; declaration-shaped
  surface, the `pal` marker, is the named next packet); (b) runtime worker
  threads are TCB implementation below the language (no construct exists to
  carry a row; DESIGN.md §5).

## Executor log

(One line per stage at completion; blockers recorded here honestly with
reproduction, never worked around.)

- E1 (permission judgment P): landed `semantic/permission.rs` (four
  conditions, claim-free eligibility, per-function pair/chain table on
  `CheckedProgramData`) plus `semantic/places.rs`, which lifts the [OWN-7]
  overlap relation, the holder/place prepass, and the [EFF-2] actual-side
  projection out of `entailment/flow.rs` so P reuses them instead of copying
  them; `expression_children` moved to `model.rs` for the same reason. Nine
  in-crate tests: three grants, one chain-stops control, four per-condition
  denials, one not-actualizable. `make -C compiler check` green (995 lib
  tests, +9, no other count moved). Three deviations from DESIGN §3, all
  fail-closed widenings, none narrowing the deliverable: (a) an
  `allocates(arena 'r)` row contributes the caller region as a written
  footprint element under condition 2, because two overlapped calls would
  both mutate one allocation list — untestable today, the arena runtime is
  an unsupported capability before a checked program exists; (b) an actual
  whose caller place P cannot resolve while its row projects an access denies
  the pair under condition 2; (c) P resolves a direct slice value's source
  place on its own side only, leaving [ENT-5] kill behavior byte-identical.

- E2 (permission ledger): landed `semantic/permission_ledger.rs`, which renders
  the permission table as the DESIGN section 4 developer lines, plus the
  `TreeView::source_line`/`path_spelling` citations it needs, a
  `permission_ledger` field on `CheckedProgramData`, the public
  `compile_with_permission_ledger` entry point (one pipeline: every other entry
  point is a projection of it), and `whitefootc --par-ledger`, which prints the
  lines on stdout and never on the mandatory record channel. The condition
  number in each denial line comes from `Denial::condition`, so the reported
  condition cannot drift from the judging one. Two decisions worth the audit's
  attention: byte-identical lines are collapsed, because the table is dense by
  `FunctionId` and one generic monomorphized twice would otherwise report one
  source site twice; and `--par-ledger` with `--emit-llvm` and no `-o` is
  refused rather than interleaving two streams into one stdout. Six tests:
  three driver-level ledger tests covering an eligible line, a
  not-actualizable line with its claim count, a denial line for each of
  conditions 1 to 4, and the empty-ledger/unchanged-module control; three
  `whitefootc` option tests. `make -C compiler check` green.

## Outcome

(Filled at closure: landed commits, verification results, measurements,
audit dispositions.)
