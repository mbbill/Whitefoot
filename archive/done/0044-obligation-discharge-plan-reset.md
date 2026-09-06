# 0044 — select complete obligation-discharge delivery before wfgrep

This is frozen coordination history, not execution authority.

- **Status:** `DONE` (2026-08-09)
- **Authority:** owner instruction of 2026-08-09 to complete the selected
  obligation-discharge direction, including `ensures`, before returning to
  wfgrep, and to adopt `spec/kernel-spec.md` as the stable active filename
- **Outcome:** Direction Outline revision 20 now records the complete selected
  boundary and parks the credited wfgrep checkpoint. The replacement ACTIVE
  plan authorizes only the independently reviewable ENT-5/verification slice;
  later capabilities are an explicit dependency map and must enter the rolling
  plan one slice at a time.

## Landed work

- `0197a13` corrected the current landscape: dossier items 1–4 are shipped;
  VERIFY-2 is the separately invoked `Pass=389 Fail=1 Skip=13` adapter; and
  CAND-8 retains one credited system-grep win without claiming the ripgrep 2x
  objective.
- The dependency map separates provenance measurement from activation, requires
  the single-atomic `requires` goal to close the O3 bypass, makes the real
  `ensures` proof prerequisites explicit, and splits deterministic claim-ledger
  tooling from the later transitive `deny-claims` marker.
- Planned task 0041 now depends on terminal ENT-5 acceptance and revalidation of
  the already-shipped SYS-8/ENT-3 S10 facts. The obsolete handover was deleted
  after its valid file-model and sequencing content reached canonical owners.
- Branch-local drafting, implementation, rehearsal, and lead review are
  explicitly distinct from owner approval of complete specification bytes;
  neither `ACTIVE-SPEC:` nor activation may precede that approval.

## Validation and remaining boundary

`git diff --check` and `make repository-invariants` exited 0. Three independent
reviews found no remaining P1/P2 issue after correction. Task 0043 remains the
carried verification repair and must refresh onto `0197a13`; after it closes,
the ACTIVE plan continues with the bounded ENT-5/stable-file candidate.
