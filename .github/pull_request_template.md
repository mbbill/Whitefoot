## Change

Describe the concrete problem and resulting behavior. Explain the choice and
its material limitations; cite specification rules, source cases, designs,
measurements, or decision memory where they support the claim.

## Language and evidence impact

Use this section when changing the specification, conformance evidence, or a
language-design premise. Otherwise omit it.

- What changed, and what selected it? For a specification amendment, include
  the META-5 delta: rules, tokens, spellings, and exceptions.
- Which constitutional premises and derivation rows are affected? Record any
  conflict or re-grounding need. An `existence-only` row still states a need
  and a condition for choosing its form; consider whether this change supplies
  that evidence. Do not describe an existence-only row as an empty premise.
- Which derived cases, generated syntax, implementation, and guidance changed
  with the specification? An outgoing specification is archived byte-for-byte
  under its versioned name; identity is derived by `compiler/build.rs`.
- For conformance changes, explain the normative expectation and why the new
  evidence tests it. A compiler gap does not change the language verdict.

## Validation

Name the revision tested and the checks actually run, including any failure or
unavailable environment. Include the root `make check` stage summary when the
complete gate ran; do not describe a partial run as all-tests green.

Mention relevant example or reference checks for documentation changes and
behavior or cost checks for implementation changes. Explain any deliberately
retired test; never remove or weaken evidence merely to make the gate pass.

## Decisions and review

Link settled decisions and rejected alternatives in their existing
`mcts_mem/` owners when this change settles a question. Update affected
standing guidance at the same time. Useful investigation designs and
measurements remain evidence after implementation; they need not be archived
at landing. A completed task is not evidence for a technical claim.

Keep the description about the current diff. Before requesting approval to
merge into `main`, identify the exact revision and refresh any statements
invalidated by later edits. The complete approval and merge rules are in
[AGENTS.md](../AGENTS.md#branch-and-main-boundary); this template adds no plan,
record, or approval prerequisite. Remove sections that do not apply.
