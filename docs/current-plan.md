# Current Plan

Status: PROPOSED — awaiting owner selection; no compiler or specification work
is authorized by this file.

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

## Proposed work

### [ ] N1 — Build a small candidate shortlist

- **Why:** the outline contains several plausible project classes, but their
  public interest, integration cost, and ability to test Whitefoot have not
  been compared on current evidence.
- **Do:** select three to five concrete external projects or bounded components.
  Pin upstream, version, and license; state the externally legible claim; name
  the smallest authentic milestone; map it to one or two primary outline items
  and every strict dependency; and list prerequisites that the current compiler
  does not supply.
- **Verify:** use primary project sources for scope and interfaces, current
  compiler evidence for implemented capability, and canonical RESULTS records
  for historical measurements. Mark every inference and do not promote a
  historical compiler result into a current claim.
- **Accept:** each candidate can be selected, parked, or rejected from one
  compact comparison without additional repository archaeology.

### [ ] N2 — Recommend one next-stage proposal

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

## Not authorized now

- No candidate project is selected.
- No language or specification change is open.
- No optimizer fact family, proof checker, parallel runtime, storage mechanism,
  FFI, backend, or deployment artifact may be implemented from this proposal.
- Existing repository program witnesses and historical experiments may be read
  as evidence; they are not project authority.

## Parallel research

Independent bounded investigations may be proposed for unresolved outline
items. Each must name one question, the evidence that would change the outline,
and a stop condition. A probe runs only after this file is `ACTIVE` and lists
it, or after separate owner approval. Research may update facts and next gates
after review; it does not silently authorize implementation or a specification
change.

## Completion

When the owner decides the proposal, update the affected direction items in
`docs/roadmap.md` and increment its revision. If approved, replace this file in
place with the `ACTIVE` rolling plan; if parked or rejected, record `NO ACTIVE
PLAN`. Git retains this proposal; do not create a second plan or dated copy.
