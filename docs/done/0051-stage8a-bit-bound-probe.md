# 0051 — Stage 8a bit-bound proof probe

- **Status:** `DONE` (2026-08-13)
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12, Workstream 8a
  `Local facts` and `Caller audit`, under Direction Outline item `PROOF-8`
- **Execution revision:**
  `36410174dfac97d76b6f30cf26e8bfd0c10aab5a`
- **Landed production/compiler commit:** none; every experiment byte was removed
- **Research evidence:**
  `research/investigations/obligation-discharge/ACCEPTANCE.md`, section
  `Stage 8a bit-bound proof refresh after DIAG-2 root retention`; installed
  section SHA-256
  `0e1c9336b2b15d9a7c2d84d067514019ae8c5878b0b05183ba3f2c6be18cfc65`

## Outcome

The removable production-path experiment established that two bounded
unsigned sources suffice for the real `read_bits` normal-result goal
`value < mask_high`: unsigned `iand` result bounds and a checked
constant-one wrapping-left-shift nonzero fact. The proof uses the existing S7
wrapping offset and ENT-4 closure. It needs no third fact family, arithmetic
term, Boolean decomposition, induction, fixed point, solver, recognizer, or
writer-visible assertion.

The later real `state.hold` write kills only its separately supported
relation; the `value <= mask < mask_high` route remains live. Signed and
operation near misses gain no fact, and error outcomes publish no
normal-result witness. Neither temporary source is installed, and no
compiler, program, specification, conformance, generated, MCTS, accepted-set,
lowering, or runtime byte changed.

The former protected-evidence classification introduced in task-local records
was incorrect: research results own measurements but are not protected
compliance evidence. This closure installs the section as ordinary research
documentation and makes no `governance/APPROVALS.md` change.

## Evidence and validation

- The unchanged real-body witness retained one `Unproved` goal and no root.
  The replacement grouped matrix discharged 192/192 goals with exact roots
  and byte-identical repeated summaries. The terminated monolithic fixture and
  the superseded over-strong oracle are recorded but count as neither passes
  nor semantic failures.
- All 18 near misses remained `Unproved`; the support/kill matrix retained
  exactly 15 discharged and 7 unproved goals; the real `read_bits` normal path
  discharged its one goal. A paired source-admission control retained one
  discharged and one unproved result.
- `SourceDistinct` is eligibility evidence for admitting the S7
  `SourceBound`; it is correctly absent from the retained query root.
- The restored focused suite passed 112/112. The final compiler gate passed
  718/718 library tests and 30/30 real programs. The repository gate passed
  all 28 recorded-specification identities, conformance structure 23/23,
  coverage 131/131, nested compiler 718/718, and programs 30/30.
- Independent review found no P0, P1, or P2 finding in the final section or
  restored execution state.

## Remaining dependency

This is local feasibility evidence, not an installed fact source or a complete
caller proof. With tasks 0051 and 0052 terminal-positive, task 0053 may now
enumerate the exact 14/20 caller map. The independent DIAG-2 trust prerequisite
is also terminal-positive; Stage 8b now awaits task 0053 and then its own exact
specification and protected-conformance approval.
