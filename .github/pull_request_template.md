Say what this changes and why, then answer every line below. A line that does
not apply is answered "n/a" with the reason, never left blank or deleted: the
value of this list is that a skipped step has to be written down as skipped.

## Derivation — language changes only

The constitution governs what the language is: syntax, semantics, and what a
program may be admitted for. It does not govern how the project is run. A
change to CI, a gate, the repository layout, or the compiler's internals is
not a constitutional question and skips this section entirely — say "not a
language change" and move on. Answer it when this changes `spec/`, the
grammar, or what the checker accepts.

- Constitutional premise this serves (P0, W1, W2, W3, T1–T3, R1–R6), and the
  derivation from it in one or two sentences:
- If the derivation could not be completed, say so plainly rather than
  omitting it. An underived language change is allowed; a silent one is not,
  and `spec/derivation/derivation-ledger.md` is where the rule's status says
  which one this is.
- If this contradicts the constitution, the constitution is what changes. Name
  the amendment here. No ruling, review, or preference outranks it.

Everything else — what to build next, what to defer, what earns a file — is
decided against the project goal and priority order in `CLAUDE.md`, not
against the constitution.

## Specification

- [ ] `spec/kernel-spec.md` unchanged, **or**: new digest recorded, activation
      chain extended, and `compiler/src/spec_identity.rs` regenerated with
      `whitefoot-spec --emit-identity` rather than edited by hand.
- [ ] Everything derived from the specification moved with it in this change:
      conformance cases and verdicts, generated syntax data, tests, docs.
- [ ] The specification carries no commentary about its own versions.
- [ ] `governance/APPROVALS.md` carries the records the four rules require.

## Written at landing

`mcts_mem/` records the current state, so it is written when the change lands,
not before.

- [ ] `mcts_mem/<area>.md` updated with what was decided and why, including the
      designs that were built and rejected.
- [ ] `docs/roadmap.md`: the affected item's `Current` and `Missing / next` are
      in the present tense. No per-version paragraph anywhere in the file.
- [ ] The investigation under `research/investigations/` is archived if the
      capability it studied has landed.

## Citations

- [ ] Nothing here cites a finished task as evidence. Support comes from the
      specification, a conformance case, a measured result under
      `research/experiments/`, a design under `research/investigations/`, or a
      decision in `mcts_mem/`.
- [ ] Nothing cites `research/` as a description of the current implementation.
      It is pre-implementation study and goes stale by design.

## Gate

Report `make check` stage by stage. A single "green" line hides a stage that
never ran: `check` stops at the first failure, so an early failure silently
gates every stage after it.

| stage | result |
| --- | --- |
| | |

- [ ] Any stage that could not run here is named, with the reason and where it
      was reproduced against a build from before this change.
- [ ] No test was deleted, disabled, narrowed, or unwired to reach green. A
      deliberately retired test carries its technical explanation in this
      change.
