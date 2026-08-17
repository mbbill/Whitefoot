# 0070 — Specified-language gap closure and take/replace resolution

Owner: lead. Workspace: `caps-batch` branch. Base: main `d5e1897` (v0.30).
Registered: 2026-08-17, under the ACTIVE Current Plan (all four
workstreams). 0069 was allocated and reverted unlanded; its number stays
burned.

## Authority

The ACTIVE `docs/current-plan.md` (owner direction 2026-08-17). Protected
surfaces produced here — the v0.31 candidate bytes, conformance case
family, manifest status flips — are marked candidates awaiting the owner's
exact-byte morning approval; nothing activates on this branch.

## Scope

W1 gap closure (13 pending + own3), W2 take/replace + collections design,
candidate, and core implementation, W3 stretch wfgrep traversal, W4 audit
and morning packet, per the plan. Executors work in isolated worktrees on
file-disjoint briefs; the lead integrates and verifies.

## Out of scope

Everything the plan excludes; any activation; any unmarked protected
change; merging to main.


## Morning review (2026-08-18)

Branch `caps-batch`, ~40 commits over main `d5e1897`. Full `make check`
exit 0 at the tip — including the conformance structural and coverage
legs (134/134). The conformance adapter is fully green for the first
time in project history: Pass=460 Skip=1 Fail=0 (pre-batch: 432 pass,
1 fail, 13 pending). lib suite 832 → 902+ tests, every workstream shipping both-direction
regressions with negative controls.

### What landed, by workstream

- **W1 (14 specified-but-unimplemented gaps):** 11 capabilities proven
  implemented (several pre-existed and needed only promotion), FN-6
  polymorphic-recursion rejection implemented, arena execution and
  STOR-4 confinement implemented, own3 outlives-store now runs. Three
  pending cases were themselves defective (missing main; GRAM-10 binder;
  missing effect row) — repaired as protected candidates. One manifest
  verdict contradicted the spec (OP-3 vs OP-1) — corrected. One item
  honestly stopped on proportionality (nested slice types need
  CheckedType interning across 22 files; scoped item).
- **W2 (§5 take/replace):** atomic-replacement semantics (no hole at any
  program point) as new rule SET-2, implemented end to end;
  `growable_vec.wf` real consumer; affine buffer elements explicit
  unsupported with ledger row; the generic-vector capability gap
  recorded (generics+regions combination).
- **W5:** overflow obligation family (OP-2) with the trap surface
  dissolved exactly where proof reaches; reborrow extension
  (signature-determined provenance, grandchild composition) plus a
  switch-independent v0.30 defect fix (borrow-returning call typing);
  O11 signed decomposition honoring all four recorded findings; const
  arithmetic and struct consts; #35 position-conditional repair;
  non-ASCII diagnostics; escaped display; Stage-2 extraction locks
  (one caught a real spec-table/compiler disagreement at merge);
  ratchet REFUTED by measurement (dense content, -161B defensible;
  DOSSIER correction owed) with 4 proven cuts applied.
- **Conformance candidate:** 16 new v0.31 cases, 12 verified
  promotions, 14-case arithmetic migration split by subject (which
  exposed real latent case defects — recursion guards admitting
  infinite recursion), O11 positive rewrite with visible removal.

### The v0.31 candidate

`Status: CANDIDATE v0.31 supersedes v0.30 5ed21019…`, digest
`6657c3420bef8678be2e43bdf941d6db0095a4ccfece46c557beba26d09b07f4`,
134 rules, grammar 74/96/99 two-path verified, five delta documents
applied with byte-verified anchors, all integration switches live —
the branch implements its candidate. Zero activation artifacts: no
chain line, no archive, candidate status intact (lead-verified).

### Owner batch approval = the merge

Everything above is prepared; per the no-blocking ruling the single
approval act is merging caps-batch to main plus the activation commit
(archive v0.30, flip status to ACTIVE v0.31, chain line, identity).
Decisions folded into the same review: DELTA-DIAG1 (+386B, buys
machine-extractable rows; not applied), adapter #[ignore] lift (+~200s
to every gate; reason text corrected, wiring untouched), enum-const
families stay DEFERRED (recorded reasons), unverifiable prior plan
approvals (task #44), DOSSIER ratchet-target correction.

### Honest incompletes

W3 wfgrep traversal slice and the byte-string program did not land
(W2's night went to the candidate + SET-2 + vec); the generic vector
waits on the recorded generics+regions gap. These open the next batch.
Three fable agents died at the session limit mid-flight; all work was
salvaged from their worktrees and completed by opus successors.
