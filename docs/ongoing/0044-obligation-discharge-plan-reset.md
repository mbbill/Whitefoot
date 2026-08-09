# 0044 — select complete obligation-discharge delivery before wfgrep

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** owner instruction of 2026-08-09: complete every selected
  obligation-discharge feature, including `ensures`, before returning to
  wfgrep; use `spec/kernel-spec.md` as the stable active specification after
  the ENT-5 switchover
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`, branch
  `codex/0044-obligation-discharge-plan`
- **Base revision:** `c986534`
- **Dependency:** task 0042 terminal; task 0043 is carried unchanged and may
  resume after this plan lands

## Goal and method

Replace the stale mixed wfgrep/spec-batch plan with one ACTIVE milestone that
owns the complete dossier §8 sequence. Update Direction Outline status without
copying semantics or measurements out of their canonical homes. Keep one
current bounded step (ENT-5 plus verification recovery) and an explicit ordered
roll-forward: SYS count postconditions, provenance gate, counted range loop,
requires-as-goal, `ensures`, then claim ledger and opt-in `deny-claims`.

Record that wfgrep remains parked until the whole selected sequence is complete
or the owner handles a reproduced blocker. Carry task 0043's exact native-gate
repair under the replacement plan. State that ENT-5 activates the stable
`spec/kernel-spec.md` model and that later activations archive the outgoing
bytes without renaming the active file. Exact specification bytes still require
their normal owner approval.

## Scope, validation, and closure

Expected touch set: `docs/roadmap.md`, `docs/current-plan.md`, this record, and
deletion of `docs/handover.md` after its live sequencing and file-model content
has been absorbed. No specification, compiler, corpus, protected verdict, or
approval-ledger change.

Validate `make repository-invariants`, check that the plan derives from the new
outline revision, and ensure task 0043 is explicitly carried. Stop if any item
is not already selected by PROOF-8/dossier §8 or this owner instruction.

## Progress

- Current: update the outline and replace the Current Plan.
- Next: close this record, rebase task 0043 onto the selected plan, and resume
  its bounded gate repair.

## Closure

Pending.
