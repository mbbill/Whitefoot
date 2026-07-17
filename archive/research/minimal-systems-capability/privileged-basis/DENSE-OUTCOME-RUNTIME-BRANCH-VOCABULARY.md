# Dense Outcome Runtime Branch Vocabulary

Status: frozen D14 research authority pending owner review, 2026-07-15.
This file defines the runtime inputs used by conditional cells in
`DENSE-OUTCOME-ROUTE-AUTHORITY.tsv`. It authorizes no implementation,
language change, optimizer fact, proof elision, or performance claim.

## Rule

Each predicate is either an independent semantic branch of the selected
ordinary-library algorithm or an explicitly marked trace classifier. A true
branch requires the named route; a false branch forbids that route. The paper
model does not evaluate these runtime inputs and grants no positive action or
cost credit to either branch. Optimizer success, writer intent, and
implementation convenience are not predicate inputs.

Counts are logical action counts. Proof-erased focus and a machine-proved
check may emit zero instructions while their semantic obligation remains.

## Frozen inputs and guards

| Predicate | Runtime inputs | True guard |
|---|---|---|
| `PREDICATE::ABORTING_BEHAVIOR_USES_SOURCE_LOAN` | `aborting_behavior_kind` | The aborting behavior is `CLONE`, `CLONE_FROM`, or `CACHED_KEY_EXTRACTION`, each of which receives a source loan. |
| `PREDICATE::CHECK_REACHED_BEFORE_BEHAVIOR_ABORT` | `completed_behavior_prefix`, `next_checked_arithmetic_position` | The completed behavior prefix reaches the next selected size, layout, or capacity check before abort. |
| `PREDICATE::CLONE_EXPANSION_REQUIRES_SOURCE_LOAN` | `new_length`, `old_length` | `new_length` exceeds `old_length` by more than one, so the frozen `max(delta - 1, 0)` clone count is nonzero. |
| `PREDICATE::COMPLETED_OWNER_PRODUCING_RESULTS_BEFORE_ABORT` | `completed_owner_producing_behavior_result_count_before_abort` | At least one behavior result owner was produced before abort. |
| `PREDICATE::COMPLETED_PRODUCER_RESULTS_BEFORE_ABORT` | `completed_successful_producer_result_count_before_abort` | At least one producer result was returned before the later producer abort. |
| `PREDICATE::DISTINCT_SWAP_PLACES` | `left_place_id`, `right_place_id` | The two exact logical place identities differ. Address inequality is not authority. |
| `PREDICATE::GROWTH_REQUIRED_BEFORE_BEHAVIOR_PREFIX` | `replacement_carrier_acquired_before_behavior_abort` | A replacement carrier was acquired before the behavior-abort edge. |
| `PREDICATE::INPUT_LIVE_LENGTH_AT_LEAST_TWO` | `input_live_length` | The input live length is at least two. |
| `PREDICATE::NEW_LENGTH_GREATER_THAN_OLD_LENGTH` | `new_length`, `old_length` | `new_length` is greater than `old_length`. |
| `PREDICATE::OUTPUT_PAYLOAD_NONEMPTY` | `output_payload_length` | The output payload length is greater than zero. |
| `PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED` | `prestate_live_owner_roles_order_and_slot_liveness`, `selected_prefix_or_result_live_owner_roles_order_and_slot_liveness` | The selected prefix or result changes live owner roles, order, slot liveness, length, or root-bearing structural state. Value mutation through an already-valid element loan alone is not a structural change. |
| `PREDICATE::PREEXISTING_OWNER_LOGICAL_PLACE_CHANGED` | `prestate_owner_logical_place_map`, `selected_prefix_or_result_owner_logical_place_map` | At least one retained or returned pre-existing owner has a different logical destination place. |
| `PREDICATE::PREFIX_EXCHANGE_COMPLETED_BEFORE_BEHAVIOR_ABORT` | `completed_exchange_count_before_behavior_abort` | The count is greater than zero. |
| `PREDICATE::ROTATION_IS_NONIDENTITY` | `view_length`, `rotation_mid` | `view_length > 1` and `0 < rotation_mid < view_length`. |
| `PREDICATE::SORT_PERFORMS_NONTRIVIAL_PERMUTATION` | `selected_unstable_sort_exchange_count` | The selected unstable-sort trace performs at least one exchange. |
| `PREDICATE::SOURCE_DESTINATION_LIVE_PREFIX_OVERLAP_NONEMPTY` | `shared_live_prefix_length` | The shared live prefix length is greater than zero. |
| `PREDICATE::SOURCE_PAYLOAD_NONEMPTY` | `source_payload_length` | The source payload length is greater than zero. |
| `PREDICATE::SPLIT_SUFFIX_NONEMPTY` | `split_suffix_length` | The selected suffix length is greater than zero. |
| `PREDICATE::STABLE_SORT_PREFIX_RELOCATED_BEFORE_ABORT` | `stable_sort_relocation_count_before_behavior_abort` | The selected stable-sort trace completed at least one relocation before abort. |
| `PREDICATE::STABLE_SORT_RELOCATES_PAYLOAD` | `selected_stable_sort_relocation_count` | The selected stable-sort trace performs at least one relocation. |
| `PREDICATE::STABLE_SORT_REQUIRES_SCRATCH_CARRIER` | `stable_sort_scratch_carrier_acquired` | The selected stable-sort trace acquires its one scratch carrier. |
| `PREDICATE::TARGET_LIVE_RANGE_NONEMPTY` | `target_live_range_length` | The target live range length is greater than zero. |
| `PREDICATE::TOTAL_RELOCATION_PAYLOAD_NONEMPTY` | `destination_live_length`, `source_payload_length` | The sum of destination and source live payload lengths is greater than zero. |

## Trace-classifier boundary

The following guards classify an already selected algorithm or control
schedule rather than independently proving that its action is necessary:

- `PREDICATE::CHECK_REACHED_BEFORE_BEHAVIOR_ABORT`;
- `PREDICATE::GROWTH_REQUIRED_BEFORE_BEHAVIOR_PREFIX`;
- `PREDICATE::PREFIX_EXCHANGE_COMPLETED_BEFORE_BEHAVIOR_ABORT`;
- `PREDICATE::SORT_PERFORMS_NONTRIVIAL_PERMUTATION`;
- `PREDICATE::STABLE_SORT_PREFIX_RELOCATED_BEFORE_ABORT`;
- `PREDICATE::STABLE_SORT_RELOCATES_PAYLOAD`; and
- `PREDICATE::STABLE_SORT_REQUIRES_SCRATCH_CARRIER`.

Their authority kind is `TRACE_CLASSIFIER_ONLY`. They receive no positive D-2
invocation credit, no P-1 structural-parity credit, and no cost credit. A
future promotion requires a separately frozen semantic input/output guard and
algorithm schedule.

## Stable-sort trace boundary

The selected structural witness is a stable merge-family route with at most
one contiguous scratch carrier, linear scratch space, `O(n log n)` comparisons,
and linear relocation per merge level. Small or already ordered inputs may
avoid scratch or relocation. The predicates above expose those semantic
branches without installing a universal allocation, swap, focus, or payload
traffic tax.

## Deliberate exclusions

This vocabulary cannot decide the coarse `DENSE-CONVERT` direction. That
member combines allocating, borrowing, moving, and representation-reuse
contracts in one exact outcome. Its affected route cells remain explicitly
blocked by `COARSE-CONVERT-DIRECTION` until direction-specific subcontracts are
frozen.
