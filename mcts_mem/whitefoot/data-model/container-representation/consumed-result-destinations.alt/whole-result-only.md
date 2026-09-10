- An ordinary synchronous call's whole owned result may reuse one consumed,
  same-typed aggregate binding when the callee snapshots inputs before body
  writes, complete CFG liveness kills the old contents, and the backing is not
  exposed.
- Ambiguous inputs, ordered multi-results and overlap/completion schedules
  retain separate storage. This bounded ABI argument supplements checked
  ownership; it does not follow from an own mode alone.

## Facts

- 2026-09-10 (8bc22df5) measurement: the heap caller uses separate 144-byte input
  and 152-byte multi-result allocations. A control changes only its complete
  allocation and field addresses, preserving the callee and transfer instruction;
  LLVM then removes fifteen additional post-pop transfers.
  [Baseline and discriminating control](../../../../../research/experiments/container-representation/families/RESULTS.md#multi-result-destination-experiment). (code)

## Moves

- 2026-09-10 (e3924d7f) replaced by [[consumed-result-destinations]]: Whole-result-only reuse could not eliminate the measured post-pop transfers; placement within the complete result allocation preserves the input snapshot and field-lifetime obligations while removing those transfers. (sourced)
