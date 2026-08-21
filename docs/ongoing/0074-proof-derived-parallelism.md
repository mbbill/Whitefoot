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

## Outcome

(Filled at closure: landed commits, verification results, measurements,
audit dispositions.)
