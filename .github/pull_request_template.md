Say what this changes and why.

**Answered at: `<short sha>`** — leave this as `draft` until you request
review.

The list below is answered once, when review is requested, against the head
commit at that moment; name that commit above. Do not fill it in while the pull
request is a draft: answers written against a head that then moves describe a
revision nobody is being asked to review, which is worse than no answers,
because they read as current.

Push after answering and the answers are stale by construction. Re-answer them
and move the commit above, or say which lines still hold. A reviewer comparing
that commit against the head can see at a glance whether they are reading the
pull request or its history.

A line that does not apply is answered "n/a" with the reason, never left blank
or deleted: the value of this list is that a skipped step has to be written
down as skipped.

## Derivation

Two classes, and they are held to different standards. The constitution
governs what the language is — syntax, semantics, and what a program may be
admitted for. It does not govern how the project is run.

### A. Language change — derivation is required and the conflict check must be complete

Answer this when the change touches `spec/`, the grammar, or what the checker
accepts. Skip to B otherwise.

Completeness is required here and nowhere else, because the specification is
the object whose internal consistency *is* the product. Two rules that
contradict each other fail no test: they make some program's acceptance
arbitrary while everything still looks green. A compiler defect fails a test.
So the sweep is demanded exactly where testing cannot substitute for it.

- Constitutional premise this serves (P0, W1, W2, W3, T1–T3, R1–R6), and the
  derivation from it in one or two sentences:
- If this contradicts the constitution, the constitution is what changes. Name
  the amendment here. No ruling, review, or preference outranks it. The owner
  may choose among options that all derive; the owner may not choose one that
  contradicts.
- [ ] Walked the rule table in `spec/derivation/derivation-ledger.md` and found
      no conflict. This is a bounded set, so "did you read every row" has an
      answer — give it. A conflict found means either this change is wrong or
      that rule needs re-deriving; settle which one here, not later.
- [ ] Rows whose status is `existence-only` are swept too, and the ones this
      change bears on are named by ID. Such a row is not empty. The *need* for
      the rule is derived there; only its *form* is minimality-selected, and
      the row states the evidence that would settle the form — "remains
      existence-only until wider source compares proof length, diagnostic
      quality, and missing expressivity" is a representative one. So two
      questions are asked at such a row, not one: does this change contradict
      it, and does this change's evidence meet the promotion condition it
      names? Skipping the second is how a measurement gets paid for and then
      thrown away.

### B. Everything else — derivation is required, completeness is not

CI, gates, repository layout, the compiler's internals, what to build next,
what to defer, what earns a file.

- Why this, in one or two sentences. Cite the constitution if it genuinely
  applies; otherwise decide against the project goal and priority order in
  `CLAUDE.md` and say so.
- No ledger sweep. An implementation mistake here is a defect that tests
  catch, not a contradiction that hides.

## Specification

- [ ] `spec/kernel-spec.md` unchanged, **or**: the outgoing bytes archived as
      `spec/kernel-spec-vN.md`, and the new version number is free — if another
      branch took it first, retitle and rebuild. The identity needs no action:
      `build.rs` derives it from the bytes.
- [ ] Everything derived from the specification moved with it in this change:
      conformance cases and verdicts, generated syntax data, tests, docs.
- [ ] The specification carries no commentary about its own versions.
- [ ] The delta declaration and selection ground [META-5] requires are stated
      in this pull request, above.

## Written at landing

`mcts_mem/` records the current state, so it is written when the change lands,
not before.

- [ ] `mcts_mem/<area>.md` updated with what was decided and why, including the
      designs that were built and rejected.
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

Report `make check` stage by stage, on the commit named at the top. A single
"green" line hides a stage that never ran: `check` stops at the first failure,
so an early failure silently gates every stage after it.

| stage | result |
| --- | --- |
| | |

- [ ] Any stage that could not run here is named, with the reason and where it
      was reproduced against a build from before this change.
- [ ] No test was deleted, disabled, narrowed, or unwired to reach green. A
      deliberately retired test carries its technical explanation in this
      change.
