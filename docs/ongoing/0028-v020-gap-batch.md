# 0028 — v0.20 gap batch (OWN-6 disposition; TYPE-7/OWN-1 ordering)

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 3 and the
  owner's 2026-08-06 batch authorization; exact byte approval still
  required before activation
- **Owner / workspace:** executor drafts, lead reviews, owner approves
- **Base revision:** (executor fills at claim)

## Goal

Draft the v0.20 candidate resolving the two recorded gaps: (1) the OWN-6
disposition of reborrow forms outside call-argument position — either a
minimal sound definition grounded in the existing OWN-4/returned-borrow
machinery or an explicit rejection with a recorded deferral; consult
`mcts_mem/whitefoot/ownership*` and the three affected protected cases,
and state what happens to each case's verdict under the chosen disposition
(protected changes ride the activation approval); (2) a deterministic
ordering rule for simultaneously established post-resolution rejections
(the TYPE-7/OWN-1 instance), consistent with DIAG-1's existing
determinism law. Hostile-review the draft before presenting; verifier;
exact-approval packet with SHA and delta accounting.

## Validation, stop, and closure

Grammar-preserving verifier green; META-5 header complete (delta AND
selection ground); the choice between definition and deferral is presented
to the owner with both costs, not made silently. Close at activation.
