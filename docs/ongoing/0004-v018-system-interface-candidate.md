# v0.18 system-interface candidate

This is a temporary live coordination record, not execution authority. Move
this same numbered record to `docs/done/` at terminal disposition.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` [`docs/current-plan.md`](../current-plan.md) Work
  item 1, derived from Outline revision 7 (`BOUND-1`, `CAND-8`)
- **Owner / workspace:** lead agent / `main` integration workspace
- **Base revision:** `527a39a`

## Goal

Produce `governance/spec-evolution/kernel-spec-v0.18-candidate.md`: v0.17 plus
exactly the selected architecture's first-command-slice deltas, ready for
grammar verification and the owner's exact byte approval.

## Direction and invariants

- The semantic delta equals the dossier §11/§11.1 inventory with the Route C
  declaration home — nothing extra rides along, nothing listed is dropped.
- Released specifications stay untouched; the candidate is non-authoritative
  until exact approval and atomic activation.
- The unlabelled `fn main` entry remains admissible; existing conformance
  verdicts are not weakened.

## Method

Copy v0.17 as the drafting base, then apply the inventory rule by rule:
entry form and program kind; the seven opaque types, operation set, and
outcome inventory; `external`/`blocks` with the EFF-1/EFF-2/FN-3/STOR-3
extensions scoped to new resource families; the Route C declaration domain
(TYPE-6 three rows, OP-1, PROG-1, DIAG-1 rank and origin kind, syntactic
program-kind visibility); portable `IoError` classes; host strings and paths
with the command-lifetime backing guarantee in target qualification; and
first-slice conformance expectations. Verify with the native grammar
verifier; consult [[system-interface]] design memory throughout.

## Progress

- **Done:** task registered; candidate seeded; five delta packages drafted in
  parallel and integrated serially (25 new rules, 13 modified, new §16/§17,
  v0.18 status header); hostile integration review (17 findings: 2 blocking,
  11 required, 4 editorial) fully applied — reflow to one-line paragraphs,
  fenced inventory, SYS-3/FN-7 trigger unification, missed second-fragment
  replacements, count/preorder disambiguation, version-label sweep, DIAG-1
  qualification-failure entry, six META-4 dedups, rank-renumber header
  sentence. 119 rule definitions, no duplicates, no orphan citations. The
  first-slice conformance catalog and Work-item-2 planned-task pre-drafts are
  complete in the scratch directory.
- **Done (verifier):** task 0005 landed (`a9c6e1a`); the native verifier
  reports the staged candidate contract verified (64/74/75) and the active
  spec unchanged (62/72/72); both gates green on main.
- **Current:** exact-approval packet presented to the owner (candidate
  SHA-256 `307a758e41366531c71dc8736bddc466054dbeba37f6e6db13f0859787711a28`).
- **Next:** on approval — record in `governance/APPROVALS.md`, activate
  atomically with every derived artifact, close this record, and register the
  Work-item-2 planned tasks.

## Scope and expected touch set

- Primary: `governance/spec-evolution/kernel-spec-v0.18-candidate.md`.
- Read-only inputs: `spec/kernel-spec-v0.17.md`, the architecture dossier and
  decision record, `mcts_mem/whitefoot/system-interface*`.
- Excluded write scope: `spec/`, compiler source, conformance verdicts,
  `wfgrep` source; those belong to activation and later planned tasks.

## Dependencies and integration order

- **Prerequisites:** none; the architecture decision closed in `0001`.
- Every implementation task planned under Work item 2 depends on this task's
  activation and lands after it.

## Validation, stop, and closure

- **Validate:** native grammar verifier passes on the candidate; the delta
  inventory cross-checks against dossier §11/§11.1 item by item; owner exact
  approval recorded in `governance/APPROVALS.md` before activation.
- **Stop:** any needed delta outside the dossier inventory stops this task
  for owner review rather than widening the batch.
- **Close:** after atomic activation with every derived artifact, move this
  record to `docs/done/` in the activation change and decompose Work item 2
  into `docs/planned/`.
