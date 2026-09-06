# System-capability architecture

Frozen coordination history. This record reports how the BOUND-1
architecture-selection task was carried out; it is not authority.

- **Status:** `DONE` — owner selected the architecture on 2026-08-05
- **Authority (historical):** the 2026-08-04 `ACTIVE` `docs/current-plan.md`
  (BOUND-1 architecture selection), derived from Outline revision 6

## Outcome

The owner selected the dossier architecture: exact typed entry inputs under a
declared program kind; immutable values, shared capabilities, and unique
stateful resources over ordinary `own`/`&`/`&uniq`; exact `external` and
`blocks` effects with conservative source ordering; operation-specific
one-attempt I/O with portable error classes and lossless target paths;
compiler-owned resource contracts with three completion policies; the Route C
system-declaration domain (recorded fallback: prelude extension if the
syntactic conditional-visibility mechanism is declined); and static target
qualification. Raw fd, ambient functions, a unique `Process` object, and
literal WASI source APIs were rejected with recorded reasons.

The dossier passed a structured adversarial review — four rotating hostile
critics, 31 issues raised (8 correctness), all resolved by evidence and
applied, none escalated — before selection.

## Evidence and validation

- Canonical evidence:
  [DOSSIER.md](../../research/investigations/system-capability-architecture/DOSSIER.md)
  and the
  [review decision record](../../research/investigations/system-capability-architecture/decisions.json).
- Investigation evidence landed in `03fc7b2`; the revised dossier, review
  record, outline revision 7, and this closure land in the decision-closure
  integration change.
- Validation: hostile reviews and the adversarial pass report no remaining
  internal-consistency blocker; every §2 current-state claim was verified
  against `spec/kernel-spec-v0.17.md` and compiler sources.

## Follow-ups

- The Current Plan (activated 2026-08-05) carries the v0.18 specification
  batch and first command slice; specification bytes still require the
  exact-approval step of the specification-change workflow.
- Durable design decision recorded in `mcts_mem/` at decision closure.
- The loan/freeze candidate vacated the v0.18 slot and is parked evidence
  under `STORE-1`.
