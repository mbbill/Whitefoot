- The integer equality and inequality family accepts integers, tag-only enums, and the Boolean enum.
- The explicit type argument selects numeric or nominal comparison under the same integer-prefixed spellings.

## Facts

- 2026-07-19 (489237a9) pitfall: stage 0 and the compiler source admitted non-integer integer-equality calls, but the investigation treated that discrepancy only as migration-cost evidence, never as authority for the language rule. (sourced)

## Moves

- 2026-07-19 replaced by [[tag-only-equality]]: the dedicated enum-domain family preserves truthful one-domain naming, while widening integer comparison would make the integer prefix cover unrelated numeric and nominal domains and make existing invalid source the rule's design center (sourced)
