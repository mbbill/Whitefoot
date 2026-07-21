# Successor specification hostile review

Date: 2026-07-21

Verdict: **GO for owner-approval presentation**

Reviewed candidate:

- path: `grammar-verifier/proposal/kernel-spec-successor-candidate.md`
- byte count: `98044`
- SHA-256: `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`

This verdict is bound to those exact bytes. Any candidate edit invalidates it.
It is not approval, guarded installation, a protected-surface migration, parser
authorization, semantic-kernel authorization, or a release claim.

## Reviewed boundaries

The hostile pass checked the complete changed-rule surface against v0.8, the
adopted compiler architecture, and the owner-selected A-01 visibility ruling.
The changed rules are `CONST-1`, `CONST-2`, `DIAG-1`, `EX-1`, `FN-1`, `FN-4`,
`FN-8`, `FORM-2`, `FORM-3`, `FORM-4`, `FORM-5`, `FORM-7`, `GIVE-1`, `GRAM-1`,
`GRAM-2`, `GRAM-3`, `GRAM-4`, `GRAM-7`, `PRE-1`, `PROG-1`, and `TYPE-6`; new
rule `PROG-2` is additive.

The review specifically attacked:

- installation suitability of the exact bytes as the active numbered v0.9
  document, including its status and authority wording;
- raw byte scanning, malformed UTF-8, quote-aware STRING failures, comment
  attribution, maximal numeric and operation-name formation, context-free
  terminal membership, fixed-atom expansion, and the sole `[0-9]+` pattern
  atom;
- the terminal partition and the complete grammar's conceptual strong-LL(2)
  decision families, including place versus call, atom versus nested call or
  construction, ordinary versus try versus value-match lets, statement versus
  value matches, generic arguments, modes, and nullable lists;
- the single compilation-unit `program` root, source-local item forests,
  non-crossing record boundaries, logical-path validity, source identity,
  source order, empty records, and declaration order;
- structural FORM-2 rendering, unlisted transparent wrapper productions,
  canonical prelude and example bytes, trivia-gap selection, deepest-common-
  ancestor ownership, and source-local diagnostic coordinates;
- deterministic pre-tree attribution, expected-terminal construction,
  dotted-call and forbidden-nesting windows, mandatory-name traversal through
  nullable prefixes, structural-choice stopping points, and total fallback;
- the total visibility table: all top-level function signatures are visible
  throughout the closed unit, while all other declarations retain lexical
  declaration-before-use;
- value-match delivery through `give`, return, an escaping break, and nested
  statement matches, including the nested-value-match, may-trap, and loop near
  misses;
- the early FN-8 structural subset check and its precedence over child semantic
  errors; and
- FN-4's closed source-acceptance calculus, exact unsigned and signed
  saturating-addition rows, one base derivation per law/conformance pair,
  same-kernel replay, and the prohibition on deriving optimizer authority from
  that base record. Optional law optimization remains behind its own separately
  approved, independently verified fact family.

Within the successor/frontend delta reviewed above, no remaining contradiction,
grammar-semantic ambiguity, soundness leak, hidden optimizer-authority path, or
installation-blocking proposal wording was found in the frozen candidate. This
is not a review of every unchanged v0.8 rule or every separately recorded
residual discrepancy.

## Residual nonblocking debts

- The complete delta, protected-surface census, source manifests, evidence,
  ledger updates, and approval packet must all bind the exact hash above and
  pass their required gates. This review does not pre-approve them.
- The candidate closes the canonical frontend entrance; it does not silently
  resolve the separately recorded semantic, artifact-schema, target, backend,
  or release gates. Those remain later owner decisions under `THE-PLAN.md`.
- FN-4 defines mandatory acceptance evidence but does not authorize an optional
  optimizer fact schema or consumer consequence. Each such authority-increasing
  family still requires its own hostile review and owner approval.
- Implementations and independent evidence must preserve source-EBNF provenance
  and child-index identity exactly. A parser-stack approximation or a second
  diagnostic grammar would not satisfy DIAG-1.

Subject to those separate gates, the exact candidate is ready to present to the
owner for approval.
