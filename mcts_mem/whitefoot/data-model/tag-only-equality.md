- Tag-only equality and inequality form a distinct enum-domain operation family over two values of one exact nominal tag-only enum; the Boolean enum is included.
- The family compares declared-variant identity directly in the type's already-selected discriminant representation and is pure and total after ordinary operand evaluation.
- Integer comparison remains integer-only. Payload comparison, enum ordering, structural or generic equality, and enum/integer conversion are absent; equal representation width never permits comparison across nominal types.
- The family creates no optimizer facts and gives no authority beyond ordinary operand evaluation.

## Facts

- 2026-07-19 (489237a9) measurement: the post-v0.7-slice compiler census found 251 non-integer equality sites in 92 functions across 18 files and 22 tag-only types; the v0.7 projections were 21 per-type mappers with 262 arms and 484 calls, or 6,952 structural pair arms plus 11 Boolean rewrites. (sourced)
- 2026-07-19 measurement: immediately before the approved v0.8 landing, the live compiler census found 255 non-integer equality sites, all `ieq`, in the same 92 functions, 18 files, and 22 tag-only types; four later `AstKind` call-validation comparisons explain the increase from 251, and non-integer `ine` remains absent. (code)
- 2026-07-19 (489237a9) measurement: a three-variant probe lowered direct tag comparison to one raw-IR comparison; the mapper caller contained two helper calls, two stack temporaries, an integer comparison, and a branching helper, while optimized assembly recovered the caller comparison but retained the helper definition. No timing or byte-count claim was made. (sourced)
- 2026-07-19 (489237a9) limitation: the operation family is evidence-selected, while the enum-domain prefix follows the established naming rule without an independent weak-writer naming trial. (sourced)
- 2026-07-21 specification continuity: exact v0.9 retains the v0.8 `eeq`/`ene` nominal tag-only enum domain and its exclusions unchanged; the v0.9 installation changes no conformance verdict or optimizer authority for this family. (sourced)

## Moves

- 2026-07-19 replaced [[widen-integer-comparison]]: the dedicated enum-domain family preserves truthful one-domain naming, while widening integer comparison would make the integer prefix cover unrelated numeric and nominal domains and make existing invalid source the rule's design center (sourced)
- 2026-07-19 replaced [[guarded-source-mapper]]: direct nominal tag equality removes the duplicated per-type ordinal convention and the external injectivity guard that v0.7 exhaustiveness cannot discharge (sourced)
- 2026-07-19 replaced [[structural-pair-match]]: direct nominal tag equality expresses the repeated identity test without the projected 6,952 source-level variant-pair arms required by the checker-closed v0.7 fallback (sourced)
- 2026-07-19 replaced [[enum-integer-conversion]]: direct nominal tag equality preserves representation freedom, while enum-to-integer conversion would expose tag representation, invite integer operations and ordering, and add a broader conversion proof surface (sourced)
