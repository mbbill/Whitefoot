# Current Plan

Status: PROPOSED — N1 completed on 2026-08-03; owner approval is required
before N2 begins.

Derived from: [Direction Outline revision 2](roadmap.md), item `CAND-1`

This proposal authorizes nothing until the owner accepts it.

## Milestone

Recommend one bounded first external validation milestone, or conclude that no
candidate is ready, from the completed N1 shortlist.

## Why now

N1 found two honest but materially different finalists:

- yyjson provides the broadest medium-scale language pressure, with the highest
  storage and number-conversion risk;
- LZ4 best balances systems recognition and current reach, with a stateful
  streaming-boundary risk.

QOI was rejected as the first flagship despite its lower implementation risk:
its public-attention signal is weaker and its reference decoder does not supply
the independent malformed-input oracle required by the proposed safety claim.

BLAKE3 and CMSIS-DSP have strong public identities but are parked because the
smallest attention-worthy Whitefoot milestones depend on absent parallel or
target/boundary capability. Choosing a branded micro-kernel from either would
not validate the project people recognize.

## Proposed current step

### [ ] N2 — Recommend one bounded milestone, or none

- **Why:** the evidence is now compact enough to make one explicit tradeoff;
  more broad candidate generation would postpone rather than improve the
  decision.
- **Do:** compare yyjson and LZ4 against the owner's public-attention goal and
  the project gates. Recommend exactly one smallest authentic milestone, or
  recommend that none is ready. State the public claim, pinned scope,
  Whitefoot/upstream boundary, correctness oracle, performance comparator only
  if performance is claimed, first zero-change port, expected first blocker,
  strict outline dependencies, exclusions, and stop condition.
- **Verify:** have an independent reviewer try to refute the recommendation for
  weak public meaning, borrowed branding, hidden infrastructure, a project-
  specific language special case, a weak oracle, or a first slice that cannot
  run before multiple unrelated changes. Use only current compiler evidence and
  pinned primary upstream sources; mark inference explicitly.
- **Accept:** the owner can approve, revise, park, or reject one self-contained
  execution proposal without reopening the repository-wide candidate search.
  Approval of that later proposal, not completion of N2, selects the project.
- **Stop:** stop after one reviewable recommendation packet. Do not port source,
  install dependencies, benchmark, change the compiler, or draft language
  semantics.

## Evidence

- [N1 external candidate shortlist](../research/notes/headline-artifact-shortlist.md)
- [Direction Outline revision 2](roadmap.md)
- [Current compiler boundary](../compiler/README.md)

## Not authorized

- No candidate project is selected.
- No source port, oracle harness, benchmark, dependency installation, or
  external integration may begin.
- No compiler, specification, conformance, project-law, or MCTS design change
  may be made from this proposal.
- BLAKE3, CMSIS-DSP, simdutf, zlib-class, ML-stack, and storage micro-kernel
  work remains parked.

## Parallel research

None. N2 is a bounded decision synthesis, not a new research program.

## Completion

If the owner activates N2, produce and hostile-review one recommendation, then
replace this file with the resulting `PROPOSED` project execution plan. Only a
subsequent owner decision changes that plan to `ACTIVE` and authorizes the
first project slice.
