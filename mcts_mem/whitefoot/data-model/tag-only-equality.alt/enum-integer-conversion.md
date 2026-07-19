- The language exposes a conversion from a tag-only enum discriminant to an integer.
- Enum equality and inequality convert both operands and apply the integer comparison family.

## Moves

- 2026-07-19 replaced by [[tag-only-equality]]: direct nominal tag equality preserves representation freedom, while enum-to-integer conversion would expose tag representation, invite integer operations and ordering, and add a broader conversion proof surface (sourced)
