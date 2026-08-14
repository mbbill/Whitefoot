# 0052 — Stage 8a counted append proof probe

- **Status:** `DONE` (2026-08-13)
- **Authority:** the `ACTIVE` Current Plan selected 2026-08-12, Workstream 8a
  `Local facts` and `Caller audit`, under Direction Outline item `PROOF-8`
- **Execution revision:**
  `36410174dfac97d76b6f30cf26e8bfd0c10aab5a`
- **Landed production/compiler commit:** none; every experiment byte was removed
- **Research evidence:**
  `research/investigations/obligation-discharge/ACCEPTANCE.md`, section
  `Stage 8a counted-append proof refresh after DIAG-2 root retention`;
  installed section SHA-256
  `42736be221f5bb60ba30cac07b04f8f4e5b3e17ac88b85baf48dd57d0be90e1a`

## Outcome

The removable production-path experiment established that the existing
counted range can express a truthful, behavior-equivalent `append_slice` on
the exact admitted domain `filled <= capacity`, while both normal return
shapes derive `result <= capacity`. The early return uses S11's
`at < capacity`; exhausted execution returns `capacity` directly. No
induction, fixed point, variable-subtraction relation, post-loop binder
equality, claim, or fallback check is needed.

The current invalid-domain behavior remains outside the claimed equivalence
boundary: capacity 3, filled 4, and empty text return 4 while leaving the
destination unchanged. Every temporary source, executable, compiler edit, and
test byte was removed. No production compiler, real program, specification,
conformance, generated, MCTS, accepted-set, lowering, or runtime byte changed.

The former protected-evidence classification introduced in task-local records
was incorrect: research results own measurements but are not protected
compliance evidence. This closure installs the section as ordinary research
documentation and makes no `governance/APPROVALS.md` change.

## Evidence and validation

- The ordinary-loop result bound remained FN-8 `Unproved`; the counted
  combined, early-only, and exhaustion-only sources were proof-positive.
- The no-requirement and `at +wrap 1` controls kept their mathematically valid
  proofs while failing separate behavioral equivalence witnesses.
  `at +wrap 2` and an independent result remained FN-8 `Unproved`; a
  post-loop binder use remained TYPE-5 `InvisibleUse`.
- All three behavioral controls executed successfully, and all 2,430 admitted
  differential cases matched the current result and every destination byte.
  Unchanged wfgrep passed 9/9 and raw-DEFLATE passed 3/3.
- The final compiler gate passed 718/718 library tests and 30/30 real
  programs. The repository gate passed all 28 recorded-specification
  identities, conformance structure 23/23, coverage 131/131, nested compiler
  718/718, and programs 30/30.
- Independent review found no P0, P1, or P2 finding in the final section or
  restored execution state.

## Remaining dependency

This is local admitted-domain evidence and installs no production mechanism.
With tasks 0051 and 0052 terminal-positive, task 0053 may now enumerate the
exact 14/20 caller map. The independent DIAG-2 trust prerequisite is also
terminal-positive; Stage 8b now awaits task 0053 and then its own exact
specification and protected-conformance approval.
