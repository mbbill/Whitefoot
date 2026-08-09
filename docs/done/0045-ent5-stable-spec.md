# 0045 — correct ENT-5 and switch to the stable active specification

This is frozen coordination history, not execution authority.

- **Status:** `DONE` (2026-08-09)
- **Authority:** the ACTIVE obligation-discharge plan derived from Direction
  Outline revision 21, the exact-approved v0.24 ENT-5 delta, the adopted
  stable-filename proposal, and the owner's approval of the complete digest,
  named protected changes, and S10 evidence disposition
- **Outcome:** v0.24 is atomically active at `spec/kernel-spec.md`; ENT-5 now
  summarizes only kills that can continue to a later head of the same loop;
  installed verification and the frozen acceptance rerun reproduce the reviewed
  result without regression

## Landed work

- `f4c7e60` is one linear activation commit from integration base `19b02ea`.
  It installs exact digest
  `53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`,
  records the owner's exact approval and the sixteenth chained activation, and
  updates every compiler, grammar, conformance, derivation, workflow, and live
  identity in the same tree transition.
- The structured reverse reachability analysis distinguishes return,
  propagated-error, current/enclosing break, nested-loop continuation,
  else-free continuation, and local `give` destinations. Eight focused ENT-5
  controls and the approved positive/negative corpus witnesses exercise the
  normal checker path.
- The one-time file-model switch reuses immutable
  `spec/kernel-spec-v0.23.md`. It creates no `kernel-spec-v0.24.md` and no
  `ARCHIVE-SPEC: v0.23`; a future activation archives v0.24 only when replacing
  it.

## Canonical evidence and validation

The specification identity and approval live in `spec/kernel-spec.md` and
`governance/APPROVALS.md`; derivation lives in
`spec/derivation/derivation-ledger.md`; the installed measurements live in
`research/investigations/obligation-discharge/ACCEPTANCE.md`; Direction Outline
revision 22 owns current status.

- Post-activation `make check` exited 0: 25 recorded specification identities,
  128/128 rule coverage, 584 compiler-library tests, 30 real-program tests, and
  `whitefoot-spec` v0.24 with 128 rules and 16 unbroken activations. Native
  grammar remains 69 productions, 84 decisions, and 93 terminal predicates.
- The separately invoked adapter reports `Pass=390 Fail=1 Skip=13`. Its only
  runnable divergence remains `own3-pos-outlives-store`, an existing unsupported
  regions/borrows boundary rather than an ENT-5 or corpus change.
- The installed frozen rerun records UTF-8 `22/33`, SHA-256 `0/9`, deflate
  `11/29`, and dynamic deflate `11/24` as claim-independent, with no
  proven-site regression and the same five redundancy advisories.
- The real boundary path produces `taken <= room`; focused actual-index
  consumers cover all four S10 producer families plus invalidation. Per the
  approved disposition, no end-to-end consumer is claimed for the driver.

## Follow-up

Planned task 0041's dependency is satisfied but it remains unclaimed. The
revision-22 ACTIVE plan carries it as the evidence-only stage-5a provenance
measurement; its claim must be a separate lifecycle commit.
