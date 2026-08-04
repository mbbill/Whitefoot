# Current Plan

Status: ACTIVE — owner selected step `N1` on 2026-08-03. This plan authorizes
candidate research only; it authorizes no port, compiler, or specification
work.

Derived from: [Direction Outline revision 1](roadmap.md), item `CAND-1`

## Milestone

Prepare an evidence-backed owner decision on Whitefoot's first external
validation project.

## Why now

Whitefoot has a broad implemented compiler surface, several historical port
studies, and many promising language directions, but no selected project that
decides which capability matters next. Continuing the former phase checklist
would implement features before a real project demonstrates their value.

The first decision is therefore project selection, not a compiler feature or a
new specification version.

## Current step

### [ ] N1 — Build a small candidate shortlist

- **Why:** the outline contains several plausible project classes, but their
  public interest, integration cost, and ability to test Whitefoot have not
  been compared on current evidence.
- **Do:** select three to five concrete external projects or bounded components.
  Pin upstream, version, and license; state the externally legible claim; name
  the smallest authentic milestone; map it to one or two primary outline items
  and every strict dependency; and list prerequisites that the current compiler
  does not supply.
- **Evidence home:** replace the dated brainstorm in
  [`research/notes/headline-artifact-shortlist.md`](../research/notes/headline-artifact-shortlist.md)
  in place with the current comparison; Git retains the historical version.
- **Verify:** use primary project sources for scope and interfaces, current
  compiler evidence for implemented capability, and canonical RESULTS records
  for historical measurements. Mark every inference and do not promote a
  historical compiler result into a current claim.
- **Accept:** each candidate can be selected, parked, or rejected from one
  compact comparison without additional repository archaeology.
- **Stop:** stop after one reviewable comparison of three to five candidates,
  or earlier if fewer than three survive the pin, license, boundary, oracle,
  and achievable-first-milestone tests. Do not begin a port or benchmark.

## Next proposal — not authorized

### N2 — Recommend one next-stage proposal

- **Why:** a shortlist is useful only if it produces a falsifiable next move.
- **Do:** recommend one bounded milestone, or recommend that none is ready.
  State why it should run now, its authenticity boundary, correctness oracle,
  performance comparator only if performance is claimed, first zero-change
  port, expected blockers, stop condition, and explicitly excluded work.
- **Verify:** challenge the recommendation for project-specific language
  pressure, ecosystem prerequisites larger than the milestone, weak oracle,
  uninteresting public claim, and dependence on unimplemented infrastructure.
- **Accept:** the owner can approve, revise, park, or reject the proposal. Only
  owner approval changes this file to an active execution plan.

Completing `N1` does not authorize `N2`. After the shortlist and its hostile
review are complete, replace this file with a `PROPOSED` N2 decision packet.

## Not authorized

- No candidate project is selected.
- No language or specification change is open.
- No candidate source port, benchmark, dependency installation, or external
  integration work is authorized.
- No optimizer fact family, proof checker, parallel runtime, storage mechanism,
  FFI, backend, or deployment artifact may be implemented from this plan.
- Existing repository program witnesses and historical experiments may be read
  as evidence; they are not project authority.

## Parallel research

None. Candidate comparison is the active step itself, not a parallel lane.

## Completion

When `N1` passes review, update `CAND-1` and any candidate disposition supported
by the evidence, increment the Direction Outline revision, and replace this
file in place with a `PROPOSED` N2 packet. A shortlist recommendation is not
project selection; only a later owner decision may activate a project
milestone.
