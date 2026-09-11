- Run boundary operations consume an owning run and return its updated owner;
  removal returns both the remaining run and the element.
- A source contract cannot publish a measure of an exclusive parameter's exit
  state. Source exclusive parameters cannot recursively reach a run or an
  opaque type parameter.
- Owning parameter measures in result contracts retain their immutable entry
  meaning across the call.

## Facts

- 2026-09-10 (26153381) measurement: the retained inline-heap experiment counted 720/864 aggregate-transfer bytes in push/pop and 405504 bytes per sixteen-round trace including caller transfers. A same-place storage intervention removed 87.6% of the matched timing gap but still left about 1.03 microseconds; it was an experimental floor, not a source placement guarantee. [D1 attribution](../../../../../research/experiments/container-representation/families/RESULTS.md#measured-transfer-intervention). (code)
- 2026-09-10 (d45ad27c) pitfall: exclusive source run formals were rejected before the growing owning-map body could be checked; returning a scalar count alone did not relate it to the post-call run measure. [D2 rejected forms](../../../../../research/experiments/container-representation/families/RESULTS.md#limits-encountered-and-exact-rejected-forms). (code)

## Moves

- 2026-09-10 (5942341c) replaced by [[exclusive-run-contracts]]: The owning-only boundary transported inline capacity and could not publish an exclusive post-state; verified two-state contracts retain the caller's storage while exact effect kills prevent stale measures. (sourced)
