- Each non-Boolean tag-only enum defines an ordinary pure source mapper from its variants to integer codes.
- Equality and inequality map both operands and apply the integer comparison family.
- A separately generated and reviewed guard proves that every mapper assigns distinct codes; Boolean sites use the existing Boolean operations.

## Facts

- 2026-07-19 (489237a9) measurement: the projected compiler rewrite required 21 mappers, 262 exhaustive arms, 484 mapper calls, and approximately 1,367 added canonical source lines. (sourced)
- 2026-07-19 (489237a9) pitfall: exhaustiveness proves that every variant has an arm but does not prove that mapper result codes are injective; duplicate codes silently equate distinct variants. (sourced)

## Moves

- 2026-07-19 replaced by [[tag-only-equality]]: direct nominal tag equality removes the duplicated per-type ordinal convention and the external injectivity guard that v0.7 exhaustiveness cannot discharge (sourced)
