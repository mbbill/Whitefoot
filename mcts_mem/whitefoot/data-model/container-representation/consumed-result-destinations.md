- A synchronous whole owned result may share backing with one consumed,
  same-typed aggregate binding after the callee snapshots all inputs.
- A complete struct result may supply one field's backing to a consumed,
  same-typed aggregate binding and that field's consuming projection. Field
  position is unrestricted; the complete parent determines the allocation
  extent and alignment.
- The parent has only distinct consuming field reads after its call in the
  same dynamic block. The complete input, field and parent storage groups retain
  full CFG interference checking; only the selected call-input read and
  consuming projection admit overlap with that parent.
- A returned child uses an ordinary transfer into its caller-provided result
  storage. It cannot redirect the complete parent into that output buffer.
- Selected field components have one level and remain disjoint from existing
  fresh binding destinations. Exposed storage, ambiguous inputs and deferred
  schedules retain independent backing.

## Facts

- 2026-09-10 (e3924d7f) measurement: the priority trace retains all sixteen pop
  calls while removing fifteen additional 144-byte post-pop transfers. Its
  explicit native stack adjustment changes from 416 to 272 bytes. Both
  independent 320-input sorting checks pass; the pop callee's LLVM and assembly
  remain unchanged. No new timing result is established.
  [Recorded comparison and conditions](../../../../research/experiments/container-representation/families/RESULTS.md#compiler-selected-result-fields). (code)
- 2026-09-10 (e3924d7f) pitfall: consuming one field does not end later sibling
  reads of a multi-result aggregate, and distinct logical slots do not establish
  distinct physical allocations. The native controls retain padded sibling
  values and a smaller returned child; planner controls check the complete
  allocation and unrelated live conflicts.
  [Owned-place controls](../../../../compiler/src/backend/tests/owned_places.rs),
  [storage planner](../../../../compiler/src/backend/storage.rs). (code)

- 2026-09-10 (ad624ab7) measurement: two alternating baseline/current timing
  cohorts retain all 224 samples. The current sixteen-round heap medians remain
  about 2.85 and 2.89 times their same-run C controls, with substantial ranges;
  removal of the caller transfers does not close the same-algorithm cost gap.
  [Samples, conditions and remaining comparison](../../../../research/experiments/container-representation/families/RESULTS.md#compiler-selected-result-fields). (code)

## Moves

- 2026-09-10 (e3924d7f) replaced [[whole-result-only]]: Whole-result-only reuse could not eliminate the measured post-pop transfers; placement within the complete result allocation preserves the input snapshot and field-lifetime obligations while removing those transfers. (sourced)
