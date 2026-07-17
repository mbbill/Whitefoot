#!/usr/bin/env python3
"""Build an independently classified dense outcome-route authority matrix.

This research authority builder does not import the derivation generator or its
member classifier. It reads only the frozen exact outcome registry and applies
the explicit operation/outcome rules below. Its output is reviewed before the
TSV is added to the repository.
"""

from __future__ import annotations

import csv
import hashlib
import sys
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[4]
SOURCE = ROOT / "optimizer-language-research/implementation/minimal-systems-capability/dense-family-lock-a/DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv"
TARGET = ROOT / "optimizer-language-research/implementation/minimal-systems-capability/privileged-basis/DENSE-OUTCOME-ROUTE-AUTHORITY.tsv"
PREDICATE_TARGET = ROOT / "optimizer-language-research/implementation/minimal-systems-capability/privileged-basis/DENSE-OUTCOME-ROUTE-PREDICATE-AUTHORITY.tsv"
EVIDENCE_TARGET = ROOT / "optimizer-language-research/implementation/minimal-systems-capability/privileged-basis/DENSE-EXACT-ROUTE-EVIDENCE-AUTHORITY.tsv"
CHOICE_TARGET = ROOT / "optimizer-language-research/implementation/minimal-systems-capability/privileged-basis/DENSE-CHOICE-RESOLUTION-AUTHORITY.tsv"

SCHEMA = "dense-outcome-route-authority-v1"
STATUS = "FROZEN_RESEARCH_AUTHORITY_PENDING_OWNER_REVIEW"
ROUTES = (
    "CORE-EMPTY-CARRIER",
    "CORE-TYPED-PLACE",
    "CORE-COPY",
    "CORE-PARTITION-BORROW",
    "CORE-TAKE-PUT",
    "CORE-REPLACE",
    "CORE-SWAP",
    "CORE-RESHAPE-PARTITION",
    "CORE-REPACKAGE-FULL-STORAGE",
    "PROTOCOL-CURSOR-SHARED",
    "PROTOCOL-CURSOR-UNIQ",
    "PROTOCOL-CURSOR-OWN",
    "FAIL-CHECKED-ARITH",
    "PROTOCOL-EXACT-FOCUS",
)
FIELDS = (
    "schema_version",
    "authority_entry_id",
    "contract_id",
    "member_contract_id",
    "outcome_id",
    "route_id",
    "disposition",
    "choice_set_id",
    "choice_alternative_id",
    "predicate_id",
    "blocker_ids",
    "authority_basis",
    "authority_status",
    "authority_entry_sha256",
)
PREDICATE_SCHEMA = "dense-outcome-route-predicate-authority-v1"
PREDICATE_FIELDS = (
    "schema_version",
    "predicate_id",
    "guard_kind",
    "guard_source_field_ids",
    "guard_expression",
    "true_route_disposition",
    "false_route_disposition",
    "source_anchor",
    "authority_status",
    "predicate_sha256",
)
EVIDENCE_SCHEMA = "dense-exact-route-evidence-authority-v1"
EVIDENCE_FIELDS = (
    "schema_version",
    "route_evidence_id",
    "contract_id",
    "member_contract_id",
    "outcome_id",
    "route_id",
    "evidence_kind",
    "exact_obligation_ids",
    "activation_rule",
    "source_anchor",
    "authority_status",
    "route_evidence_sha256",
)
CHOICE_SCHEMA = "dense-choice-resolution-authority-v1"
CHOICE_FIELDS = (
    "schema_version",
    "choice_resolution_id",
    "contract_id",
    "member_contract_id",
    "outcome_id",
    "choice_set_id",
    "payload_branch_id",
    "payload_size_class",
    "stored_borrow_route_id",
    "od3_policy_variant_id",
    "od4_policy_option",
    "selected_alternative_id",
    "selected_route_ids",
    "selection_basis",
    "authority_status",
    "choice_resolution_sha256",
)


def required(basis: str):
    return ("REQUIRED", "NONE", "NONE", "NONE", "NONE", basis)


def forbidden(basis: str = "FULL_COMPLEMENT_HOSTILE_INJECTION_GUARD"):
    return ("FORBIDDEN", "NONE", "NONE", "NONE", "NONE", basis)


def conditional(predicate: str, basis: str):
    return (
        "CONDITIONAL_WITH_FROZEN_PREDICATE",
        "NONE",
        "NONE",
        predicate,
        "NONE",
        basis,
    )


def choice(choice_set: str, alternative: str, basis: str):
    return (
        "REQUIRE_ONE_OF",
        choice_set,
        alternative,
        "NONE",
        "NONE",
        basis,
    )


def unresolved(blocker: str, basis: str):
    return (
        "UNRESOLVED_WITH_BLOCKER",
        "NONE",
        "NONE",
        "NONE",
        blocker,
        basis,
    )


CHECK_FAILURES = {
    "CAPACITY_OVERFLOW_TRAP",
    "CAPACITY_ERROR_RETURN",
    "BOUNDS_TRAP",
    "PRECONDITION_TRAP",
    "CHECKED_ERROR",
    "UNDERFILL_CLOSE_REJECTED",
    "OVERFILL_REJECTED",
}
ALLOC_FAILURES = {"OOM_ABORT", "ALLOCATION_ERROR_RETURN"}

COPY_MEMBERS = {"DENSE-COPY-FROM", "DENSE-COPY-WITHIN", "DENSE-INIT-COPY"}
BORROW_SUCCESS_MEMBERS = {
    "DENSE-FIXED-EACH",
    "DENSE-FIXED-VIEW",
    "DENSE-INDEX-SHARED",
    "DENSE-INDEX-UNIQ",
    "DENSE-OWNER-VIEW",
    "DENSE-VIEW-ARRAY-CHUNKS",
    "DENSE-VIEW-CONSUME-SPLIT",
    "DENSE-VIEW-DISJOINT-UNIQ",
    "DENSE-VIEW-SPLIT-CHECKED",
    "DENSE-VIEW-SPLIT-TRAP",
    "DENSE-PUSH-UNIQ",
    "DENSE-INSERT-UNIQ",
    "DENSE-COPY-FROM",
    "DENSE-COPY-WITHIN",
    "DENSE-REVERSE",
    "DENSE-ROTATE",
    "DENSE-SWAP",
    "DENSE-SWAP-WITH-VIEW",
}
BORROW_PRESENT_MEMBERS = {
    "DENSE-VIEW-AS-FIXED",
    "DENSE-VIEW-END",
    "DENSE-VIEW-END-CHUNK",
    "DENSE-VIEW-END-SPLIT",
    "DENSE-VIEW-GET-SHARED",
    "DENSE-VIEW-GET-UNIQ",
}
BORROW_CALLABLE_MEMBERS = {
    "DENSE-COMPARE",
    "DENSE-HASH-TRAVERSAL",
    "DENSE-CLONE-FROM",
    "DENSE-CONCAT",
    "DENSE-EXTEND-CLONE",
    "DENSE-EXTEND-WITHIN",
    "DENSE-FILL-CLONE",
    "DENSE-FILL-WITH",
    "DENSE-FRESH-CLONE",
    "DENSE-JOIN",
    "DENSE-REPEAT",
    "DENSE-DEDUP",
    "DENSE-DEDUP-BY",
    "DENSE-DEDUP-BY-KEY",
    "DENSE-EAGER-EXTRACT",
    "DENSE-RETAIN",
    "DENSE-RETAIN-MUT",
    "DENSE-SELECT-UNSTABLE",
    "DENSE-SORT-STABLE",
    "DENSE-SORT-STABLE-CACHED-KEY",
    "DENSE-SORT-UNSTABLE",
}

NEW_BUILDERS = {
    "DENSE-COLLECT",
    "DENSE-CONCAT",
    "DENSE-FIXED-MAP",
    "DENSE-FRESH-CLONE",
    "DENSE-JOIN",
    "DENSE-REPEAT",
}
GROWING_BUILDERS = {
    "DENSE-EXTEND-CLONE",
    "DENSE-EXTEND-ITER",
    "DENSE-EXTEND-WITHIN",
    "DENSE-RESIZE-WITH",
}
CLONE_APPEND_BUILDERS = {
    "DENSE-CONCAT",
    "DENSE-EXTEND-CLONE",
    "DENSE-EXTEND-WITHIN",
    "DENSE-FRESH-CLONE",
    "DENSE-INIT-CLONE",
    "DENSE-JOIN",
    "DENSE-REPEAT",
}
PRODUCER_APPEND_BUILDERS = {
    "DENSE-COLLECT",
    "DENSE-EXTEND-ITER",
    "DENSE-FIXED-MAP",
}
FILL_REPLACE = {"DENSE-FILL-CLONE", "DENSE-FILL-WITH"}
RESIZE_MEMBERS = {"DENSE-RESIZE-CLONE", "DENSE-RESIZE-WITH"}

MOVE_GROW_SINGLE = {"DENSE-INSERT", "DENSE-INSERT-UNIQ", "DENSE-PUSH", "DENSE-PUSH-UNIQ"}
MOVE_GROW_RANGE = {"DENSE-APPEND-MOVE", "DENSE-SPLIT-OFF"}
REALLOCATE_MEMBERS = {
    "DENSE-INTO-BOXED",
    "DENSE-RESERVE",
    "DENSE-RESERVE-EXACT",
    "DENSE-SHRINK-TO",
    "DENSE-SHRINK-TO-FIT",
    "DENSE-TRY-RESERVE",
    "DENSE-TRY-RESERVE-EXACT",
}
COMPACT_MEMBERS = {
    "DENSE-DEDUP",
    "DENSE-DEDUP-BY",
    "DENSE-DEDUP-BY-KEY",
    "DENSE-EAGER-EXTRACT",
    "DENSE-EAGER-SPLICE",
    "DENSE-RETAIN",
    "DENSE-RETAIN-MUT",
}
TAKE_PUT_COMPACT_MEMBERS = {
    "DENSE-DEDUP",
    "DENSE-DEDUP-BY",
    "DENSE-DEDUP-BY-KEY",
    "DENSE-EAGER-EXTRACT",
    "DENSE-RETAIN",
    "DENSE-RETAIN-MUT",
}
TAKE_PUT_ABORT_PREFIX_MEMBERS = {
    "DENSE-COLLECT",
    "DENSE-CONCAT",
    "DENSE-EAGER-SPLICE",
    "DENSE-EXTEND-CLONE",
    "DENSE-EXTEND-ITER",
    "DENSE-EXTEND-WITHIN",
    "DENSE-FRESH-CLONE",
    "DENSE-INIT-CLONE",
    "DENSE-JOIN",
    "DENSE-REPEAT",
    "DENSE-RESIZE-CLONE",
    "DENSE-RESIZE-WITH",
}
STABLE_SORT = {"DENSE-SORT-STABLE", "DENSE-SORT-STABLE-CACHED-KEY"}
UNSTABLE_SORT = {"DENSE-SELECT-UNSTABLE", "DENSE-SORT-UNSTABLE"}
PERMUTE_DIRECT = {"DENSE-REVERSE", "DENSE-SWAP", "DENSE-SWAP-WITH-VIEW"}
DISPOSE_MEMBERS = {"DENSE-CLEAR", "DENSE-DROP", "DENSE-TRUNCATE"}

PREDICATE_SPECS = {
    "PREDICATE::ABORTING_BEHAVIOR_USES_SOURCE_LOAN": (
        "runtime.aborting_behavior_kind",
        "runtime.aborting_behavior_kind in {CLONE,CLONE_FROM,CACHED_KEY_EXTRACTION}",
    ),
    "PREDICATE::CHECK_REACHED_BEFORE_BEHAVIOR_ABORT": (
        "runtime.completed_behavior_prefix,runtime.next_checked_arithmetic_position",
        "runtime.completed_behavior_prefix_reaches_next_checked_arithmetic_position",
    ),
    "PREDICATE::CLONE_EXPANSION_REQUIRES_SOURCE_LOAN": (
        "runtime.new_length,runtime.old_length",
        "runtime.new_length>runtime.old_length&&runtime.new_length-runtime.old_length>1",
    ),
    "PREDICATE::COMPLETED_OWNER_PRODUCING_RESULTS_BEFORE_ABORT": (
        "runtime.completed_owner_producing_behavior_result_count_before_abort",
        "runtime.completed_owner_producing_behavior_result_count_before_abort>0",
    ),
    "PREDICATE::COMPLETED_PRODUCER_RESULTS_BEFORE_ABORT": (
        "runtime.completed_successful_producer_result_count_before_abort",
        "runtime.completed_successful_producer_result_count_before_abort>0",
    ),
    "PREDICATE::DISTINCT_SWAP_PLACES": (
        "runtime.left_place_id,runtime.right_place_id",
        "runtime.left_place_id!=runtime.right_place_id",
    ),
    "PREDICATE::GROWTH_REQUIRED_BEFORE_BEHAVIOR_PREFIX": (
        "runtime.replacement_carrier_acquired_before_behavior_abort",
        "runtime.replacement_carrier_acquired_before_behavior_abort==true",
    ),
    "PREDICATE::INPUT_LIVE_LENGTH_AT_LEAST_TWO": (
        "runtime.input_live_length",
        "runtime.input_live_length>=2",
    ),
    "PREDICATE::NEW_LENGTH_GREATER_THAN_OLD_LENGTH": (
        "runtime.new_length,runtime.old_length",
        "runtime.new_length>runtime.old_length",
    ),
    "PREDICATE::OUTPUT_PAYLOAD_NONEMPTY": (
        "runtime.output_payload_length",
        "runtime.output_payload_length>0",
    ),
    "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED": (
        "runtime.prestate_live_owner_roles_order_and_slot_liveness,runtime.selected_prefix_or_result_live_owner_roles_order_and_slot_liveness",
        "runtime.prestate_live_owner_roles_order_and_slot_liveness!=runtime.selected_prefix_or_result_live_owner_roles_order_and_slot_liveness",
    ),
    "PREDICATE::PREEXISTING_OWNER_LOGICAL_PLACE_CHANGED": (
        "runtime.prestate_owner_logical_place_map,runtime.selected_prefix_or_result_owner_logical_place_map",
        "exists retained_or_returned_owner whose selected logical place differs from prestate",
    ),
    "PREDICATE::PREFIX_EXCHANGE_COMPLETED_BEFORE_BEHAVIOR_ABORT": (
        "runtime.completed_exchange_count_before_behavior_abort",
        "runtime.completed_exchange_count_before_behavior_abort>0",
    ),
    "PREDICATE::ROTATION_IS_NONIDENTITY": (
        "runtime.view_length,runtime.rotation_mid",
        "runtime.view_length>1&&runtime.rotation_mid>0&&runtime.rotation_mid<runtime.view_length",
    ),
    "PREDICATE::SORT_PERFORMS_NONTRIVIAL_PERMUTATION": (
        "runtime.selected_unstable_sort_exchange_count",
        "runtime.selected_unstable_sort_exchange_count>0",
    ),
    "PREDICATE::SOURCE_DESTINATION_LIVE_PREFIX_OVERLAP_NONEMPTY": (
        "runtime.shared_live_prefix_length",
        "runtime.shared_live_prefix_length>0",
    ),
    "PREDICATE::SOURCE_PAYLOAD_NONEMPTY": (
        "runtime.source_payload_length",
        "runtime.source_payload_length>0",
    ),
    "PREDICATE::SPLIT_SUFFIX_NONEMPTY": (
        "runtime.split_suffix_length",
        "runtime.split_suffix_length>0",
    ),
    "PREDICATE::STABLE_SORT_PREFIX_RELOCATED_BEFORE_ABORT": (
        "runtime.stable_sort_relocation_count_before_behavior_abort",
        "runtime.stable_sort_relocation_count_before_behavior_abort>0",
    ),
    "PREDICATE::STABLE_SORT_RELOCATES_PAYLOAD": (
        "runtime.selected_stable_sort_relocation_count",
        "runtime.selected_stable_sort_relocation_count>0",
    ),
    "PREDICATE::STABLE_SORT_REQUIRES_SCRATCH_CARRIER": (
        "runtime.stable_sort_scratch_carrier_acquired",
        "runtime.stable_sort_scratch_carrier_acquired==true",
    ),
    "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY": (
        "runtime.target_live_range_length",
        "runtime.target_live_range_length>0",
    ),
    "PREDICATE::TOTAL_RELOCATION_PAYLOAD_NONEMPTY": (
        "runtime.destination_live_length,runtime.source_payload_length",
        "runtime.destination_live_length+runtime.source_payload_length>0",
    ),
}

TRACE_CLASSIFIER_PREDICATES = {
    "PREDICATE::CHECK_REACHED_BEFORE_BEHAVIOR_ABORT",
    "PREDICATE::GROWTH_REQUIRED_BEFORE_BEHAVIOR_PREFIX",
    "PREDICATE::PREFIX_EXCHANGE_COMPLETED_BEFORE_BEHAVIOR_ABORT",
    "PREDICATE::SORT_PERFORMS_NONTRIVIAL_PERMUTATION",
    "PREDICATE::STABLE_SORT_PREFIX_RELOCATED_BEFORE_ABORT",
    "PREDICATE::STABLE_SORT_RELOCATES_PAYLOAD",
    "PREDICATE::STABLE_SORT_REQUIRES_SCRATCH_CARRIER",
}


def suffix(row: dict[str, str]) -> str:
    return row["outcome_id"].rsplit(".OUT.", 1)[-1]


def classify(row: dict[str, str], route: str):
    member = row["member_contract_id"]
    outcome = suffix(row)
    if row["status"] == "EXCLUDED_BLOCKS_NAMED_CLAIM":
        return forbidden("EXCLUDED_OUTCOME_HAS_NO_RUNTIME_ROUTE")

    # Typed-place formation belongs to carrier formation, not to a separate
    # dense operation invocation.  Keeping every cell negative prevents it
    # from becoming a duplicate action credit.
    if route == "CORE-TYPED-PLACE":
        return forbidden("TYPED_PLACE_IS_CARRIER_PRECONDITION_NOT_OUTCOME_ACTION")

    if route == "CORE-COPY":
        if member in COPY_MEMBERS and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "SOURCE_PRESERVING_COPY_RUNS_ONLY_FOR_A_NONEMPTY_TARGET_RANGE",
            )
        return forbidden("COPY_ONLY_ON_SOURCE_PRESERVING_SUCCESS")

    if route == "CORE-EMPTY-CARRIER":
        if member in {"DENSE-DEFAULT", "DENSE-NEW", "DENSE-WITH-CAPACITY"} and outcome == "SUCCESS":
            return required("EXACT_EMPTY_OWNER_CONSTRUCTION")
        if member in NEW_BUILDERS and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
            return required("NEW_OUTPUT_CARRIER_EXISTS_BEFORE_PAYLOAD_PRODUCTION")
        if member == "DENSE-EAGER-EXTRACT" and outcome == "SUCCESS":
            return required("REMOVED_OUTPUT_OWNER_REQUIRES_A_DISTINCT_CARRIER_EVEN_WHEN_EMPTY")
        if member == "DENSE-EAGER-SPLICE" and outcome == "SUCCESS_NO_GROW":
            return required("REMOVED_OUTPUT_OWNER_REQUIRES_A_DISTINCT_CARRIER_EVEN_WITHOUT_BASE_GROWTH")
        if member in {
            "DENSE-APPEND-MOVE",
            "DENSE-EAGER-SPLICE",
            "DENSE-EXTEND-CLONE",
            "DENSE-EXTEND-ITER",
            "DENSE-EXTEND-WITHIN",
            "DENSE-INSERT",
            "DENSE-INSERT-UNIQ",
            "DENSE-PUSH",
            "DENSE-PUSH-UNIQ",
            "DENSE-RESERVE",
            "DENSE-RESERVE-EXACT",
            "DENSE-RESIZE-CLONE",
            "DENSE-RESIZE-WITH",
            "DENSE-TRY-RESERVE",
            "DENSE-TRY-RESERVE-EXACT",
        } and outcome == "SUCCESS_GROW":
            return required("SUCCESSFUL_GROWTH_ACQUIRES_REPLACEMENT_CARRIER")
        if member in {
            "DENSE-EAGER-SPLICE",
            "DENSE-EXTEND-CLONE",
            "DENSE-EXTEND-ITER",
            "DENSE-EXTEND-WITHIN",
            "DENSE-RESIZE-CLONE",
            "DENSE-RESIZE-WITH",
        } and outcome == "BEHAVIOR_ABORT":
            return conditional(
                "PREDICATE::GROWTH_REQUIRED_BEFORE_BEHAVIOR_PREFIX",
                "BEHAVIOR_ABORT_MAY_FOLLOW_GROWTH_ACQUISITION",
            )
        if member in {"DENSE-INTO-BOXED", "DENSE-SHRINK-TO", "DENSE-SHRINK-TO-FIT"} and outcome == "SUCCESS_RELOCATE":
            return required("SUCCESSFUL_RELOCATION_ACQUIRES_REPLACEMENT_CARRIER")
        if member == "DENSE-SPLIT-OFF" and outcome == "SUCCESS":
            return required("SPLIT_RESULT_REQUIRES_AN_EXACT_NEW_CARRIER")
        if member in STABLE_SORT and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
            return conditional(
                "PREDICATE::STABLE_SORT_REQUIRES_SCRATCH_CARRIER",
                "SELECTED_STABLE_MERGE_TRACE_ACQUIRES_ONE_SCRATCH_CARRIER_ONLY_ABOVE_ITS_THRESHOLD",
            )
        if member == "DENSE-CONVERT" and outcome == "SUCCESS":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_SUCCESS_DOES_NOT_SELECT_ALLOCATING_VERSUS_REUSE_DIRECTION",
            )
        if member == "DENSE-CONVERT" and outcome == "BEHAVIOR_ABORT":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_ABORT_DOES_NOT_SELECT_REPRESENTATION_DIRECTION",
            )
        return forbidden("NO_NEW_VACANT_CARRIER_IN_EXACT_OUTCOME")

    if route == "CORE-PARTITION-BORROW":
        if member == "DENSE-INIT-CLONE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "CLONE_INITIALIZATION_LOANS_SOURCE_ELEMENTS_ONLY_FOR_A_NONEMPTY_TARGET_RANGE",
            )
        if member == "DENSE-INIT-CLONE" and outcome == "BEHAVIOR_ABORT":
            return required("CLONE_INITIALIZATION_ABORT_PROVES_A_SOURCE_ELEMENT_LOAN")
        if member == "DENSE-INIT-COPY" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "COPY_INITIALIZATION_LOANS_SOURCE_ELEMENTS_ONLY_FOR_A_NONEMPTY_TARGET_RANGE",
            )
        if member == "DENSE-RESIZE-CLONE" and outcome in {"SUCCESS_NO_GROW", "SUCCESS_GROW"}:
            return conditional(
                "PREDICATE::CLONE_EXPANSION_REQUIRES_SOURCE_LOAN",
                "RESIZE_CLONE_LOANS_THE_SEED_ONLY_WHEN_THE_FROZEN_CLONE_CALL_COUNT_IS_NONZERO",
            )
        if member == "DENSE-RESIZE-CLONE" and outcome == "BEHAVIOR_ABORT":
            return conditional(
                "PREDICATE::ABORTING_BEHAVIOR_USES_SOURCE_LOAN",
                "RESIZE_CLONE_ABORT_MAY_BE_A_SOURCE_LOANING_CLONE_OR_A_NONLOANING_DISPOSER",
            )
        if member == "DENSE-FILL-CLONE" and outcome == "BEHAVIOR_ABORT":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "FILL_CLONE_FORMS_TARGET_AND_SEED_LOANS_ONLY_FOR_A_NONEMPTY_TARGET_RANGE",
            )
        if member in {"DENSE-COPY-FROM", "DENSE-COPY-WITHIN"} and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "COPY_FORMS_ELEMENT_PLACE_LOANS_ONLY_FOR_A_NONEMPTY_TARGET_RANGE",
            )
        if member == "DENSE-FIXED-EACH" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "INDEXED_BORROWS_ARE_FORMED_ONLY_FOR_A_NONEMPTY_FIXED_RANGE",
            )
        if member == "DENSE-SWAP" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::DISTINCT_SWAP_PLACES",
                "DISJOINT_PLACE_LOANS_ARE_NEEDED_ONLY_FOR_DISTINCT_SWAP_PLACES",
            )
        if member == "DENSE-SWAP-WITH-VIEW" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "TWO_VIEW_PLACE_LOANS_ARE_NEEDED_ONLY_FOR_A_NONEMPTY_RANGE",
            )
        if member == "DENSE-REVERSE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::INPUT_LIVE_LENGTH_AT_LEAST_TWO",
                "REVERSE_FORMS_DISJOINT_PAIR_LOANS_ONLY_WHEN_AT_LEAST_ONE_EXCHANGE_EXISTS",
            )
        if member == "DENSE-ROTATE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::ROTATION_IS_NONIDENTITY",
                "SELECTED_GCD_ROTATION_FORMS_PLACE_LOANS_ONLY_FOR_A_NONIDENTITY_PERMUTATION",
            )
        if member == "DENSE-CLONE-FROM" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::SOURCE_DESTINATION_LIVE_PREFIX_OVERLAP_NONEMPTY",
                "CLONE_FROM_LOANS_AN_ELEMENT_PAIR_ONLY_FOR_A_NONEMPTY_EQUAL_LENGTH_PREFIX",
            )
        if member == "DENSE-COMPARE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::SOURCE_DESTINATION_LIVE_PREFIX_OVERLAP_NONEMPTY",
                "COMPARE_LOANS_AN_ELEMENT_PAIR_ONLY_FOR_A_NONEMPTY_SHARED_PREFIX",
            )
        if member in {"DENSE-CONCAT", "DENSE-FRESH-CLONE", "DENSE-JOIN", "DENSE-REPEAT"} and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::OUTPUT_PAYLOAD_NONEMPTY",
                "CLONE_BUILD_LOANS_AN_ELEMENT_ONLY_FOR_A_NONEMPTY_OUTPUT_PAYLOAD",
            )
        if member in {"DENSE-DEDUP", "DENSE-DEDUP-BY", "DENSE-DEDUP-BY-KEY", "DENSE-SELECT-UNSTABLE", "DENSE-SORT-STABLE", "DENSE-SORT-UNSTABLE"} and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::INPUT_LIVE_LENGTH_AT_LEAST_TWO",
                "ADJACENT_OR_COMPARATOR_LOANS_REQUIRE_AT_LEAST_TWO_LIVE_INPUTS",
            )
        if member in {"DENSE-EAGER-EXTRACT", "DENSE-HASH-TRAVERSAL", "DENSE-RETAIN", "DENSE-RETAIN-MUT", "DENSE-SORT-STABLE-CACHED-KEY"} and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::SOURCE_PAYLOAD_NONEMPTY",
                "ELEMENT_BEHAVIOR_RECEIVES_A_LOAN_ONLY_FOR_A_NONEMPTY_SOURCE_PAYLOAD",
            )
        if member == "DENSE-SORT-STABLE-CACHED-KEY" and outcome == "BEHAVIOR_ABORT":
            return conditional(
                "PREDICATE::ABORTING_BEHAVIOR_USES_SOURCE_LOAN",
                "CACHED_KEY_ABORT_MAY_OCCUR_IN_A_PAYLOAD_LOANING_KEY_CALL_OR_A_SCRATCH_KEY_COMPARISON",
            )
        if member in {"DENSE-EXTEND-CLONE", "DENSE-EXTEND-WITHIN"} and outcome == "SUCCESS_NO_GROW":
            return conditional(
                "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED",
                "IN_CAPACITY_CLONE_APPEND_LOANS_ELEMENTS_ONLY_WHEN_PAYLOAD_OR_LENGTH_CHANGES",
            )
        if member in {"DENSE-FILL-CLONE", "DENSE-FILL-WITH"} and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "FILL_FORMS_A_TARGET_PLACE_LOAN_ONLY_FOR_A_NONEMPTY_RANGE",
            )
        if member == "DENSE-FILL-WITH" and outcome == "BEHAVIOR_ABORT":
            return forbidden("FILL_WITH_PRODUCER_ABORTS_BEFORE_ANY_TARGET_PLACE_LOAN")
        if member in BORROW_SUCCESS_MEMBERS and outcome.startswith("SUCCESS"):
            return required("EXACT_BORROW_PARTITION_OR_DISJOINT_ACCESS_SUCCESS")
        if member in BORROW_PRESENT_MEMBERS and outcome == "PRESENT":
            return required("EXACT_PRESENT_RESULT_BORROW")
        if member in BORROW_CALLABLE_MEMBERS and (
            outcome.startswith("SUCCESS") or outcome == "BEHAVIOR_ABORT"
        ):
            return required("EXACT_CALLABLE_RECEIVES_LIVE_ELEMENT_LOAN")
        if member == "DENSE-POP-IF" and outcome in {
            "PREDICATE_FALSE",
            "PREDICATE_TRUE",
            "BEHAVIOR_ABORT",
        }:
            return required("EXACT_PREDICATE_RECEIVES_LAST_ELEMENT_LOAN")
        if member in {"DENSE-ITER-SHARED", "DENSE-ITER-UNIQ"} and outcome in {
            "CURSOR_CREATED",
            "NEXT_SOME",
        }:
            return required("BORROWING_CURSOR_FORMS_OR_SPLITS_EXACT_LOAN")
        if member == "DENSE-CONVERT" and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_OUTCOME_DOES_NOT_SELECT_OWNED_VERSUS_BORROWED_DIRECTION",
            )
        return forbidden("NO_PAYLOAD_PARTITION_OR_RESULT_BORROW_IN_EXACT_OUTCOME")

    if route == "PROTOCOL-CURSOR-SHARED":
        if member == "DENSE-ITER-SHARED":
            return required("SHARED_CURSOR_STATE_TRANSITION")
        return forbidden()
    if route == "PROTOCOL-CURSOR-UNIQ":
        if member == "DENSE-ITER-UNIQ":
            return required("UNIQUE_CURSOR_STATE_TRANSITION")
        return forbidden()
    if route == "PROTOCOL-CURSOR-OWN":
        if member == "DENSE-ITER-OWN":
            return required("OWNING_CURSOR_STATE_TRANSITION")
        return forbidden()

    if route == "CORE-RESHAPE-PARTITION":
        if member == "DENSE-INTO-FLATTENED" and outcome == "SUCCESS":
            return required("EXACT_ZERO_TRAFFIC_PARTITION_RESHAPE")
        if member == "DENSE-INTO-OWNER" and outcome == "SUCCESS":
            return unresolved(
                "INTO-OWNER-ZST-CAPACITY-RESHAPE-NOT-DERIVED",
                "POSITIVE_SIZE_REPACKAGE_AND_ZST_LOGICAL_CAPACITY_RESHAPE_ARE_NOT_SPLIT",
            )
        if member == "DENSE-INTO-BOXED" and outcome == "SUCCESS_NO_CHANGE":
            return unresolved(
                "INTO-BOXED-NO-CHANGE-FULLNESS-AND-ZST-SUBCONTRACTS-NOT-SPLIT",
                "FULL_POSITIVE_STORAGE_AND_ZST_LOGICAL_CAPACITY_RESHAPE_ARE_NOT_SPLIT",
            )
        if member == "DENSE-CONVERT" and outcome == "SUCCESS":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_SUCCESS_DOES_NOT_SELECT_RESHAPE_DIRECTION",
            )
        if member == "DENSE-CONVERT" and outcome == "BEHAVIOR_ABORT":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_ABORT_DOES_NOT_SELECT_REPRESENTATION_DIRECTION",
            )
        return forbidden("RESHAPE_ONLY_ON_PROVED_LAYOUT_BIJECTION")

    if route == "CORE-REPACKAGE-FULL-STORAGE":
        if member == "DENSE-INTO-OWNER" and outcome == "SUCCESS":
            return unresolved(
                "INTO-OWNER-ZST-CAPACITY-RESHAPE-NOT-DERIVED",
                "POSITIVE_SIZE_REPACKAGE_AND_ZST_LOGICAL_CAPACITY_RESHAPE_ARE_NOT_SPLIT",
            )
        if member == "DENSE-INTO-BOXED" and outcome == "SUCCESS_NO_CHANGE":
            return unresolved(
                "INTO-BOXED-NO-CHANGE-FULLNESS-AND-ZST-SUBCONTRACTS-NOT-SPLIT",
                "SOURCE_DOES_NOT_FREEZE_POSITIVE_FULLNESS_OR_ZST_LOGICAL_CAPACITY_CHANGE",
            )
        if member == "DENSE-CONVERT" and outcome == "SUCCESS":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_SUCCESS_DOES_NOT_SELECT_FULL_STORAGE_REPACKAGE_DIRECTION",
            )
        if member == "DENSE-CONVERT" and outcome == "BEHAVIOR_ABORT":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_ABORT_DOES_NOT_SELECT_REPRESENTATION_DIRECTION",
            )
        return forbidden("REPACKAGE_REQUIRES_EXACT_SAME_FULL_STORAGE_ROOT")

    if route == "CORE-REPLACE":
        if member in {"DENSE-REPLACE", "DENSE-TAKE-WITH-DEFAULT"} and outcome == "SUCCESS":
            return required("EXACT_LIVE_PLACE_REPLACEMENT")
        if member in FILL_REPLACE and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "FILL_REPLACES_EACH_EXISTING_LIVE_TARGET",
            )
        if member == "DENSE-FILL-CLONE" and outcome == "BEHAVIOR_ABORT":
            return forbidden(
                "FILL_CLONE_ABORTS_INSIDE_CLONE_FROM_BEFORE_THE_FINAL_OUTER_SEED_REPLACEMENT"
            )
        if member in FILL_REPLACE and outcome == "BEHAVIOR_ABORT":
            return conditional(
                "PREDICATE::COMPLETED_PRODUCER_RESULTS_BEFORE_ABORT",
                "FILL_ABORT_MAY_FOLLOW_A_REPLACED_PREFIX",
            )
        return forbidden("NO_EXACT_LIVE_PLACE_REPLACEMENT")

    if route == "CORE-SWAP":
        if member == "DENSE-SWAP" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::DISTINCT_SWAP_PLACES",
                "SWAP_EXCHANGES_VALUES_ONLY_WHEN_THE_TWO_EXACT_PLACES_DIFFER",
            )
        if member == "DENSE-SWAP-WITH-VIEW" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "TWO_VIEW_EXCHANGE_RUNS_ONLY_FOR_A_NONEMPTY_EQUAL_LENGTH_RANGE",
            )
        if member == "DENSE-REVERSE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::INPUT_LIVE_LENGTH_AT_LEAST_TWO",
                "REVERSE_EXCHANGES_PAIRS_ONLY_WHEN_THE_INPUT_HAS_AT_LEAST_TWO_VALUES",
            )
        if member == "DENSE-ROTATE" and outcome == "SUCCESS":
            return forbidden("SELECTED_GENERIC_ROTATION_WITNESS_USES_GCD_TAKE_PUT_NOT_SWAP")
        if member in UNSTABLE_SORT and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::SORT_PERFORMS_NONTRIVIAL_PERMUTATION",
                "UNSTABLE_SORT_USES_DISJOINT_EXCHANGES",
            )
        if member in UNSTABLE_SORT and outcome == "BEHAVIOR_ABORT":
            return conditional(
                "PREDICATE::PREFIX_EXCHANGE_COMPLETED_BEFORE_BEHAVIOR_ABORT",
                "UNSTABLE_SORT_ABORT_MAY_FOLLOW_EXCHANGES",
            )
        if member in STABLE_SORT and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
            return forbidden("SELECTED_STABLE_MERGE_TRACE_USES_RELOCATION_NOT_SWAP")
        return forbidden("NO_SELECTED_SWAP_ROUTE_IN_EXACT_OUTCOME")

    if route == "CORE-TAKE-PUT":
        # Source-preserving Copy must never be weakened into relocation.
        if member in COPY_MEMBERS:
            return forbidden("CORE_COPY_PRESERVES_SOURCE_AND_FORBIDS_RELOCATION")
        if member in MOVE_GROW_SINGLE and outcome in {"SUCCESS_NO_GROW", "SUCCESS_GROW"}:
            return required("EXACT_OFFERED_OWNER_INSERTION_AND_RELOCATION")
        if member == "DENSE-APPEND-MOVE" and outcome == "SUCCESS_NO_GROW":
            return conditional(
                "PREDICATE::SOURCE_PAYLOAD_NONEMPTY",
                "APPEND_MOVES_OWNERS_ONLY_WHEN_THE_SOURCE_PAYLOAD_IS_NONEMPTY",
            )
        if member == "DENSE-APPEND-MOVE" and outcome == "SUCCESS_GROW":
            return conditional(
                "PREDICATE::TOTAL_RELOCATION_PAYLOAD_NONEMPTY",
                "APPEND_GROWTH_MOVES_OWNERS_ONLY_WHEN_DESTINATION_OR_SOURCE_PAYLOAD_IS_NONEMPTY",
            )
        if member in {
            "DENSE-EAGER-SPLICE",
            "DENSE-EXTEND-CLONE",
            "DENSE-EXTEND-ITER",
            "DENSE-EXTEND-WITHIN",
        } and outcome == "SUCCESS_NO_GROW":
            return conditional(
                "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED",
                "IN_CAPACITY_APPEND_MOVES_OWNERS_ONLY_WHEN_PAYLOAD_OR_LENGTH_CHANGES",
            )
        if member in {
            "DENSE-EAGER-SPLICE",
            "DENSE-EXTEND-CLONE",
            "DENSE-EXTEND-ITER",
            "DENSE-EXTEND-WITHIN",
        } and outcome == "SUCCESS_GROW":
            return required("EXACT_PREFIX_PRODUCTION_AND_INSERTION_ROUTE")
        if member in {
            "DENSE-COLLECT",
            "DENSE-CONCAT",
            "DENSE-FIXED-MAP",
            "DENSE-FRESH-CLONE",
            "DENSE-JOIN",
            "DENSE-REPEAT",
        } and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::OUTPUT_PAYLOAD_NONEMPTY",
                "NEW_BUILD_MOVES_OWNERS_ONLY_WHEN_THE_OUTPUT_PAYLOAD_IS_NONEMPTY",
            )
        if member == "DENSE-INIT-CLONE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "INITIALIZATION_MOVES_OWNERS_ONLY_WHEN_THE_EXACT_TARGET_RANGE_IS_NONEMPTY",
            )
        if member == "DENSE-FIXED-MAP" and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
            return required("FIXED_MAP_TAKES_INPUT_OWNER_BEFORE_OWNED_CALLBACK")
        if member in TAKE_PUT_ABORT_PREFIX_MEMBERS and outcome == "BEHAVIOR_ABORT":
            return conditional(
                "PREDICATE::COMPLETED_OWNER_PRODUCING_RESULTS_BEFORE_ABORT",
                "ABORT_MAY_FOLLOW_A_VALID_WRITTEN_OR_RELOCATED_PREFIX",
            )
        if member in RESIZE_MEMBERS and outcome == "SUCCESS_NO_GROW":
            return conditional(
                "PREDICATE::NEW_LENGTH_GREATER_THAN_OLD_LENGTH",
                "IN_CAPACITY_RESIZE_WRITES_ONLY_THE_NEW_SUFFIX",
            )
        if member in RESIZE_MEMBERS and outcome == "SUCCESS_GROW":
            return required("GROWING_RESIZE_RELOCATES_OLD_AND_WRITES_NEW_OWNERS")
        if member in TAKE_PUT_COMPACT_MEMBERS and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
            return conditional(
                "PREDICATE::PREEXISTING_OWNER_LOGICAL_PLACE_CHANGED",
                "COMPACTION_OR_EXTRACTION_MOVES_ONLY_WHEN_A_LIVE_HOLE_IS_REPAIRED",
            )
        if member == "DENSE-POP" and outcome == "VALUE_RETURNED":
            return required("POP_RETURNS_THE_SOLE_LAST_OWNER")
        if member == "DENSE-POP-IF" and outcome == "PREDICATE_TRUE":
            return required("POP_IF_TRUE_RETURNS_THE_SOLE_LAST_OWNER")
        if member in {"DENSE-REMOVE", "DENSE-SWAP-REMOVE"} and outcome == "SUCCESS":
            return required("REMOVE_RETURNS_ONE_OWNER_AND_REPAIRS_DENSE_PREFIX")
        if member in {
            "DENSE-RESERVE",
            "DENSE-RESERVE-EXACT",
            "DENSE-TRY-RESERVE",
            "DENSE-TRY-RESERVE-EXACT",
        } and outcome == "SUCCESS_GROW":
            return conditional(
                "PREDICATE::SOURCE_PAYLOAD_NONEMPTY",
                "GROWTH_TRANSFERS_THE_LIVE_PREFIX_ONLY_WHEN_IT_IS_NONEMPTY",
            )
        if member in {"DENSE-SHRINK-TO", "DENSE-SHRINK-TO-FIT"} and outcome == "SUCCESS_RELOCATE":
            return conditional(
                "PREDICATE::SOURCE_PAYLOAD_NONEMPTY",
                "SHRINK_TRANSFERS_THE_LIVE_PREFIX_ONLY_WHEN_IT_IS_NONEMPTY",
            )
        if member == "DENSE-INTO-BOXED" and outcome == "SUCCESS_RELOCATE":
            return conditional(
                "PREDICATE::SOURCE_PAYLOAD_NONEMPTY",
                "BOX_RELOCATION_TRANSFERS_THE_LIVE_PREFIX_ONLY_WHEN_IT_IS_NONEMPTY",
            )
        if member == "DENSE-SPLIT-OFF" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::SPLIT_SUFFIX_NONEMPTY",
                "SPLIT_OFF_TRANSFERS_OWNERS_ONLY_WHEN_THE_SELECTED_SUFFIX_IS_NONEMPTY",
            )
        if member == "DENSE-ITER-OWN" and outcome in {"YIELD_FRONT", "YIELD_BACK"}:
            return required("OWNING_CURSOR_YIELDS_ONE_SOLE_OWNER")
        if member in STABLE_SORT and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
            predicate = (
                "PREDICATE::STABLE_SORT_RELOCATES_PAYLOAD"
                if outcome == "SUCCESS"
                else "PREDICATE::STABLE_SORT_PREFIX_RELOCATED_BEFORE_ABORT"
            )
            return conditional(
                predicate,
                "SELECTED_STABLE_MERGE_TRACE_USES_ONE_SCRATCH_CARRIER_AND_LINEAR_RELOCATION",
            )
        if member == "DENSE-ROTATE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::ROTATION_IS_NONIDENTITY",
                "SELECTED_GENERIC_GCD_ROTATION_RELOCATES_OWNERS_ONLY_FOR_A_NONIDENTITY_PERMUTATION",
            )
        if member == "DENSE-CONVERT" and outcome == "SUCCESS":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_SUCCESS_DOES_NOT_SELECT_MOVE_BUILD_DIRECTION",
            )
        if member == "DENSE-CONVERT" and outcome == "BEHAVIOR_ABORT":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_ABORT_DOES_NOT_SELECT_REPRESENTATION_DIRECTION",
            )
        return forbidden("NO_OWNER_TRANSFER_IN_EXACT_OUTCOME")

    if route == "FAIL-CHECKED-ARITH":
        # The frozen crosswalk maps only FL-ARITH (the alias of FL-CAPACITY)
        # to this route. Bounds and disjointness are place/partition facts and
        # must not become duplicate checked-arithmetic action credit.
        if member in STABLE_SORT:
            if outcome == "BEHAVIOR_ABORT":
                return conditional(
                    "PREDICATE::CHECK_REACHED_BEFORE_BEHAVIOR_ABORT",
                    "SELECTED_STABLE_MERGE_TRACE_MAY_REACH_SCRATCH_SIZE_ARITHMETIC_BEFORE_CALLBACK_ABORT",
                )
            return required("ROUTE_EVIDENCE_PLACEHOLDER")
        if "FL-ARITH" in row["capability_ids"].split(","):
            if member == "DENSE-CONVERT" and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
                return unresolved(
                    "COARSE-CONVERT-DIRECTION",
                    "CONVERSION_OUTCOME_DOES_NOT_SELECT_AN_ARITHMETIC_APPLICABILITY_DIRECTION",
                )
            if outcome == "BEHAVIOR_ABORT":
                return conditional(
                    "PREDICATE::CHECK_REACHED_BEFORE_BEHAVIOR_ABORT",
                    "THE_EXACT_BEHAVIOR_PREFIX_MAY_PRECEDE_OR_FOLLOW_ITS_NEXT_SIZE_LAYOUT_OR_CAPACITY_CHECK",
                )
            return required("ROUTE_EVIDENCE_PLACEHOLDER")
        return forbidden("NO_FL_ARITH_CAPABILITY_IN_EXACT_SOURCE_CONTRACT")

    if route == "PROTOCOL-EXACT-FOCUS":
        if member == "DENSE-CONVERT" and outcome == "SUCCESS":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_SUCCESS_DOES_NOT_SELECT_AN_OPEN_FOCUS_DIRECTION",
            )
        if member == "DENSE-CONVERT" and outcome == "BEHAVIOR_ABORT":
            return unresolved(
                "COARSE-CONVERT-DIRECTION",
                "CONVERSION_ABORT_DOES_NOT_SELECT_REPRESENTATION_DIRECTION",
            )
        if member == "DENSE-CLONE-FROM" and outcome == "SUCCESS":
            return forbidden("LOCKED_EQUAL_LENGTH_CLONE_FROM_USES_ATOMIC_IN_PLACE_CLONE_UPDATE")
        if member == "DENSE-CLONE-FROM" and outcome == "BEHAVIOR_ABORT":
            return forbidden("LOCKED_EQUAL_LENGTH_CLONE_FROM_REMAINS_SEALED_ACROSS_EACH_DIRECT_CALL")
        if outcome in CHECK_FAILURES | ALLOC_FAILURES | {
            "SUCCESS_NO_CHANGE",
            "SUCCESS_NO_GROW",
            "EMPTY",
            "PREDICATE_FALSE",
            "ABSENT",
            "NO_CHANGE",
            "EMPTY_NO_CHANGE",
            "TERMINAL_NONE",
            "CURSOR_DESTROYED",
        }:
            # Some SUCCESS_NO_GROW outcomes below perform definite insertion;
            # they are handled before this generic no-change rule.
            if member not in MOVE_GROW_SINGLE and not (
                member in {"DENSE-APPEND-MOVE", "DENSE-EAGER-SPLICE", "DENSE-EXTEND-CLONE", "DENSE-EXTEND-ITER", "DENSE-EXTEND-WITHIN", "DENSE-RESIZE-CLONE", "DENSE-RESIZE-WITH"}
                and outcome == "SUCCESS_NO_GROW"
            ):
                return forbidden("EXACT_PRECOMMIT_OR_NO_CHANGE_OUTCOME_NEEDS_NO_OPEN_FOCUS")
        if member in MOVE_GROW_SINGLE and outcome in {"SUCCESS_NO_GROW", "SUCCESS_GROW"}:
            return required("EXACT_INSERTION_OR_RELOCATION_FOCUS")
        if member == "DENSE-APPEND-MOVE" and outcome == "SUCCESS_GROW":
            return required("APPEND_GROWTH_IMPLIES_A_NONEMPTY_SOURCE_OWNER_SEQUENCE")
        if member == "DENSE-APPEND-MOVE" and outcome == "SUCCESS_NO_GROW":
            return conditional(
                "PREDICATE::SOURCE_PAYLOAD_NONEMPTY",
                "APPEND_OPENS_FOCUS_ONLY_FOR_OWNER_TRANSFER",
            )
        if member == "DENSE-SPLIT-OFF" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::SPLIT_SUFFIX_NONEMPTY",
                "SPLIT_OPENS_FOCUS_ONLY_FOR_OWNER_TRANSFER",
            )
        if member == "DENSE-FIXED-MAP" and outcome == "BEHAVIOR_ABORT":
            return required("ROUTE_EVIDENCE_PLACEHOLDER")
        if member in NEW_BUILDERS and outcome in {"SUCCESS", "BEHAVIOR_ABORT"}:
            predicate = (
                "PREDICATE::OUTPUT_PAYLOAD_NONEMPTY"
                if outcome == "SUCCESS"
                else "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED"
            )
            return conditional(
                predicate,
                "PARTIAL_NEW_OWNER_USES_FOCUS_ONLY_WHEN_A_LIVE_PAYLOAD_PREFIX_EXISTS",
            )
        if member == "DENSE-INIT-CLONE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "CLONE_INITIALIZATION_USES_FOCUS_ONLY_FOR_A_NONEMPTY_EXACT_TARGET_RANGE",
            )
        if member in GROWING_BUILDERS | CLONE_APPEND_BUILDERS | PRODUCER_APPEND_BUILDERS | RESIZE_MEMBERS:
            if outcome in {"SUCCESS_GROW"}:
                return required("ROUTE_EVIDENCE_PLACEHOLDER")
            if outcome in {"SUCCESS", "SUCCESS_NO_GROW"}:
                return conditional(
                    "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED",
                    "FOCUS_IS_NEEDED_ONLY_WHEN_THE_OWNER_STATE_CHANGES",
                )
            if outcome == "BEHAVIOR_ABORT":
                return conditional(
                    "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED",
                    "BEHAVIOR_ABORT_USES_FOCUS_ONLY_FOR_AN_ALREADY_COMMITTED_PREFIX",
                )
        if member in FILL_REPLACE:
            return forbidden("EACH_ATOMIC_REPLACE_PRESERVES_THE_SEALED_DENSE_INVARIANT")
        if member in COMPACT_MEMBERS:
            if outcome in {"SUCCESS", "SUCCESS_NO_GROW"}:
                return conditional(
                    "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED",
                    "FOCUS_IS_NEEDED_ONLY_WHEN_THE_OWNER_STATE_CHANGES",
                )
            if outcome == "SUCCESS_GROW":
                return required("ROUTE_EVIDENCE_PLACEHOLDER")
            if outcome == "BEHAVIOR_ABORT":
                return conditional(
                    "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED",
                    "COMPACTION_ABORT_USES_FOCUS_ONLY_AFTER_A_LIVE_HOLE_REPAIR",
                )
        if member == "DENSE-POP" and outcome == "VALUE_RETURNED":
            return required("POP_OWNER_TRANSFER_IS_ACCOUNTED_INSIDE_EXACT_FOCUS")
        if member == "DENSE-POP-IF" and outcome == "PREDICATE_TRUE":
            return required("POP_IF_OWNER_TRANSFER_IS_ACCOUNTED_INSIDE_EXACT_FOCUS")
        if member == "DENSE-POP-IF" and outcome == "BEHAVIOR_ABORT":
            return forbidden("PREDICATE_ABORTS_BEFORE_POP_MUTATION")
        if member in {"DENSE-REMOVE", "DENSE-SWAP-REMOVE"} and outcome == "SUCCESS":
            return required("ROUTE_EVIDENCE_PLACEHOLDER")
        if member in {"DENSE-REPLACE", "DENSE-TAKE-WITH-DEFAULT"} and outcome == "SUCCESS":
            return forbidden("ATOMIC_REPLACE_PRESERVES_THE_SEALED_DENSE_INVARIANT")
        if member == "DENSE-TAKE-WITH-DEFAULT" and outcome == "BEHAVIOR_ABORT":
            return forbidden("DEFAULT_ABORTS_BEFORE_REPLACEMENT")
        if member in PERMUTE_DIRECT and outcome == "SUCCESS":
            return forbidden("ATOMIC_SWAP_PRESERVES_THE_SEALED_DENSE_INVARIANT")
        if member == "DENSE-ROTATE" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::ROTATION_IS_NONIDENTITY",
                "SELECTED_GENERIC_GCD_ROTATION_OPENS_FOCUS_ONLY_FOR_A_NONIDENTITY_PERMUTATION",
            )
        if member in STABLE_SORT:
            if outcome == "SUCCESS":
                return conditional(
                    "PREDICATE::STABLE_SORT_RELOCATES_PAYLOAD",
                    "STABLE_SORT_USES_FOCUS_ONLY_WHEN_THE_SELECTED_MERGE_TRACE_RELOCATES_PAYLOAD",
                )
            if outcome == "BEHAVIOR_ABORT":
                return conditional(
                    "PREDICATE::STABLE_SORT_PREFIX_RELOCATED_BEFORE_ABORT",
                    "STABLE_SORT_RESEALS_BEFORE_EACH_CALLBACK_AND_CREDITS_ONLY_A_COMPLETED_RELOCATION_PREFIX",
                )
        if member in UNSTABLE_SORT:
            return forbidden("SELECTED_UNSTABLE_TRACE_USES_ATOMIC_SWAP_WITHOUT_OPEN_FOCUS")
        if member in REALLOCATE_MEMBERS and outcome in {"SUCCESS_GROW", "SUCCESS_RELOCATE"}:
            return required("ROOT_REPLACEMENT_ALWAYS_OPENS_FOCUS_TO_TRANSFER_AND_RESEAL_THE_OPAQUE_OWNER")
        if member == "DENSE-INIT-COPY" and outcome == "SUCCESS":
            return conditional(
                "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                "COPY_INITIALIZATION_USES_FOCUS_ONLY_FOR_A_NONEMPTY_LIVE_PREFIX",
            )
        if member == "DENSE-INIT-CLONE":
            if outcome == "SUCCESS":
                return conditional(
                    "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY",
                    "CLONE_INITIALIZATION_USES_FOCUS_ONLY_FOR_A_NONEMPTY_LIVE_PREFIX",
                )
            if outcome == "BEHAVIOR_ABORT":
                return conditional(
                    "PREDICATE::OWNER_PROTOCOL_STRUCTURAL_STATE_CHANGED",
                    "INITIALIZATION_ABORT_USES_FOCUS_ONLY_FOR_A_COMPLETED_LIVE_PREFIX",
                )
        if member in DISPOSE_MEMBERS:
            if outcome in {"ALL_VALUES_DESTROYED", "OWNER_DESTROYED", "SUFFIX_DESTROYED", "BEHAVIOR_ABORT"}:
                return required("ROUTE_EVIDENCE_PLACEHOLDER")
        return forbidden("NO_TEMPORARILY_OPEN_AGGREGATE_STATE_IN_EXACT_OUTCOME")

    raise AssertionError(route)


def row_hash(row: dict[str, str]) -> str:
    material = "\t".join(row[field] for field in FIELDS[:-1])
    return hashlib.sha256(material.encode()).hexdigest()


def build_rows():
    with SOURCE.open(newline="") as handle:
        source_rows = list(csv.DictReader(handle, delimiter="\t"))
    assert len(source_rows) == 303
    rows = []
    for source in sorted(source_rows, key=lambda item: item["contract_id"]):
        for route in ROUTES:
            disposition, choice_set, alternative, predicate, blockers, basis = classify(source, route)
            if route in {"FAIL-CHECKED-ARITH", "PROTOCOL-EXACT-FOCUS"} and disposition in {
                "REQUIRED",
                "REQUIRE_ONE_OF",
            }:
                basis = f"ROUTE-EVIDENCE::{source['contract_id']}::{route}"
            authority_id = f"OUTCOME-ROUTE::{source['contract_id']}::{route}"
            if disposition == "REQUIRE_ONE_OF":
                authority_id += f"::{choice_set}::{alternative}"
            row = {
                "schema_version": SCHEMA,
                "authority_entry_id": authority_id,
                "contract_id": source["contract_id"],
                "member_contract_id": source["member_contract_id"],
                "outcome_id": source["outcome_id"],
                "route_id": route,
                "disposition": disposition,
                "choice_set_id": choice_set,
                "choice_alternative_id": alternative,
                "predicate_id": predicate,
                "blocker_ids": blockers,
                "authority_basis": basis,
                "authority_status": STATUS,
                "authority_entry_sha256": "",
            }
            row["authority_entry_sha256"] = row_hash(row)
            rows.append(row)
    assert len(rows) == 4242
    rows.sort(key=lambda row: row["authority_entry_id"])
    return source_rows, rows


def hash_fields(row: dict[str, str], fields: tuple[str, ...], hash_field: str) -> str:
    return hashlib.sha256(
        "\t".join(row[field] for field in fields if field != hash_field).encode()
    ).hexdigest()


def build_predicate_rows(authority_rows):
    referenced = {
        row["predicate_id"]
        for row in authority_rows
        if row["disposition"] == "CONDITIONAL_WITH_FROZEN_PREDICATE"
    }
    assert referenced == set(PREDICATE_SPECS), (referenced - set(PREDICATE_SPECS), set(PREDICATE_SPECS) - referenced)
    rows = []
    for predicate_id in sorted(PREDICATE_SPECS):
        source_fields, expression = PREDICATE_SPECS[predicate_id]
        row = {
            "schema_version": PREDICATE_SCHEMA,
            "predicate_id": predicate_id,
            "guard_kind": (
                "TRACE_CLASSIFIER_ONLY"
                if predicate_id in TRACE_CLASSIFIER_PREDICATES
                else "RUNTIME_BRANCH_GUARD"
            ),
            "guard_source_field_ids": source_fields,
            "guard_expression": expression,
            "true_route_disposition": "REQUIRED",
            "false_route_disposition": "FORBIDDEN",
            "source_anchor": f"DENSE-OUTCOME-RUNTIME-BRANCH-VOCABULARY.md::{predicate_id}",
            "authority_status": STATUS,
            "predicate_sha256": "",
        }
        row["predicate_sha256"] = hash_fields(row, PREDICATE_FIELDS, "predicate_sha256")
        rows.append(row)
    return rows


def checked_arithmetic_obligations(member: str) -> tuple[str, ...]:
    if member == "DENSE-APPEND-MOVE":
        return ("CHECK-LIVE-LEN-PLUS-SOURCE-LEN", "CHECK-GROWTH-CAPACITY", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member in {"DENSE-COLLECT", "DENSE-EXTEND-ITER"}:
        return ("CHECK-NEXT-LIVE-LEN", "CHECK-GROWTH-CAPACITY", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member in {"DENSE-CONCAT", "DENSE-JOIN"}:
        return ("CHECK-OUTPUT-LENGTH-SUM", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member == "DENSE-CONVERT":
        return ("CHECK-DIRECTION-OUTPUT-LENGTH", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member == "DENSE-EAGER-SPLICE":
        return ("CHECK-LIVE-LEN-MINUS-REMOVED", "CHECK-LIVE-LEN-PLUS-INSERTED", "CHECK-GROWTH-CAPACITY", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member in {"DENSE-EXTEND-CLONE", "DENSE-EXTEND-WITHIN"}:
        return ("CHECK-LIVE-LEN-PLUS-SOURCE-LEN", "CHECK-GROWTH-CAPACITY", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member == "DENSE-FRESH-CLONE":
        return ("CHECK-SOURCE-LENGTH", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member in {"DENSE-INIT-CLONE", "DENSE-INIT-COPY"}:
        return ("CHECK-INITIALIZED-LIVE-COUNT", "CHECK-INITIALIZED-LIVE-COUNT-EQUALS-CAPACITY")
    if member in {"DENSE-INSERT", "DENSE-INSERT-UNIQ", "DENSE-PUSH", "DENSE-PUSH-UNIQ"}:
        return ("CHECK-LIVE-LEN-PLUS-ONE", "CHECK-GROWTH-CAPACITY", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member == "DENSE-INTO-BOXED":
        return ("CHECK-BOXED-LIVE-EXTENT", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member == "DENSE-INTO-FLATTENED":
        return ("CHECK-FLATTENED-ELEMENT-PRODUCT", "CHECK-FLATTENED-LAYOUT-BIJECTION")
    if member == "DENSE-REPEAT":
        return ("CHECK-SOURCE-LEN-TIMES-REPEAT-COUNT", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member in {"DENSE-RESERVE", "DENSE-RESERVE-EXACT", "DENSE-TRY-RESERVE", "DENSE-TRY-RESERVE-EXACT"}:
        return ("CHECK-LIVE-LEN-PLUS-ADDITIONAL", "CHECK-REQUESTED-CAPACITY", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member in {"DENSE-RESIZE-CLONE", "DENSE-RESIZE-WITH"}:
        return ("CHECK-REQUESTED-NEW-LENGTH", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member in {"DENSE-SHRINK-TO", "DENSE-SHRINK-TO-FIT"}:
        return ("CHECK-SHRINK-TARGET-AT-LEAST-LIVE-LEN", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member == "DENSE-SPLIT-OFF":
        return ("CHECK-SPLIT-INDEX", "CHECK-SPLIT-SUFFIX-LENGTH", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member in STABLE_SORT:
        return ("CHECK-STABLE-SORT-SCRATCH-LENGTH", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    if member == "DENSE-WITH-CAPACITY":
        return ("CHECK-REQUESTED-CAPACITY", "CHECK-CARRIER-BYTE-EXTENT", "CHECK-CARRIER-LAYOUT")
    raise AssertionError(f"missing exact checked-arithmetic obligations for {member}")


def exact_focus_obligations(member: str, outcome: str) -> tuple[str, ...]:
    if member in {"DENSE-CLEAR", "DENSE-DROP", "DENSE-TRUNCATE"}:
        tail = "FOCUS-PRE-ABORT-SAFE" if outcome == "BEHAVIOR_ABORT" else "FOCUS-CONSUME-OR-RESEAL"
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-DISPOSE-EXACT-LIVE-SET", tail)
    if member == "DENSE-FIXED-MAP":
        return ("FOCUS-OPEN-INPUT-OWNER", "FOCUS-TAKE-NEXT-INPUT", "FOCUS-PRE-ABORT-SAFE")
    if member == "DENSE-APPEND-MOVE":
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-RELOCATE-LIVE-PREFIX", "FOCUS-APPEND-SOURCE-OWNERS", "FOCUS-RESEAL-NEW-CARRIER", "FOCUS-RELEASE-OLD-CARRIER")
    if member in {"DENSE-INSERT", "DENSE-INSERT-UNIQ"}:
        if outcome == "SUCCESS_GROW":
            return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-RELOCATE-LIVE-PREFIX", "FOCUS-INSERT-OFFERED-OWNER", "FOCUS-RESEAL-NEW-CARRIER", "FOCUS-RELEASE-OLD-CARRIER")
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-SHIFT-SUFFIX", "FOCUS-INSERT-OFFERED-OWNER", "FOCUS-RESEAL-DENSE-PREFIX")
    if member in {"DENSE-PUSH", "DENSE-PUSH-UNIQ"}:
        if outcome == "SUCCESS_GROW":
            return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-RELOCATE-LIVE-PREFIX", "FOCUS-PUT-OFFERED-OWNER", "FOCUS-RESEAL-NEW-CARRIER", "FOCUS-RELEASE-OLD-CARRIER")
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-PUT-OFFERED-OWNER", "FOCUS-COMMIT-LENGTH", "FOCUS-RESEAL-DENSE-PREFIX")
    if member in {"DENSE-EAGER-SPLICE", "DENSE-EXTEND-CLONE", "DENSE-EXTEND-ITER", "DENSE-EXTEND-WITHIN", "DENSE-RESIZE-CLONE", "DENSE-RESIZE-WITH"}:
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-RELOCATE-LIVE-PREFIX", "FOCUS-WRITE-SELECTED-PAYLOAD", "FOCUS-RESEAL-NEW-CARRIER", "FOCUS-RELEASE-OLD-CARRIER")
    if member in {"DENSE-INTO-BOXED", "DENSE-SHRINK-TO", "DENSE-SHRINK-TO-FIT", "DENSE-RESERVE", "DENSE-RESERVE-EXACT", "DENSE-TRY-RESERVE", "DENSE-TRY-RESERVE-EXACT"}:
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-RELOCATE-LIVE-PREFIX", "FOCUS-RESEAL-NEW-CARRIER", "FOCUS-RELEASE-OLD-CARRIER")
    if member in {"DENSE-POP", "DENSE-POP-IF"}:
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-TAKE-LAST-OWNER", "FOCUS-COMMIT-LENGTH", "FOCUS-RESEAL-AND-RETURN-OWNER")
    if member == "DENSE-REMOVE":
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-TAKE-SELECTED-OWNER", "FOCUS-SHIFT-SUFFIX", "FOCUS-RESEAL-AND-RETURN-OWNER")
    if member == "DENSE-SWAP-REMOVE":
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-SWAP-SELECTED-WITH-LAST", "FOCUS-TAKE-LAST-OWNER", "FOCUS-RESEAL-AND-RETURN-OWNER")
    if member == "DENSE-ROTATE":
        return ("FOCUS-OPEN-SEALED-OWNER", "FOCUS-TAKE-PUT-ROTATION-CYCLE", "FOCUS-RESEAL-DENSE-PREFIX")
    raise AssertionError(f"missing exact focus obligations for {member}/{outcome}")


def build_evidence_rows(source_rows, authority_rows):
    source_by_contract = {row["contract_id"]: row for row in source_rows}
    rows = []
    for authority in authority_rows:
        route = authority["route_id"]
        if route not in {"FAIL-CHECKED-ARITH", "PROTOCOL-EXACT-FOCUS"}:
            continue
        if authority["disposition"] not in {"REQUIRED", "REQUIRE_ONE_OF"}:
            continue
        source = source_by_contract[authority["contract_id"]]
        member = source["member_contract_id"]
        outcome = suffix(source)
        if route == "FAIL-CHECKED-ARITH":
            obligations = checked_arithmetic_obligations(member)
            kind = "EXACT_CHECKED_ARITH_OBLIGATION"
            activation = (
                "EXACT_OUTCOME_SELECTS_FIRST_FAILING_NAMED_OBLIGATION"
                if outcome in {"CAPACITY_OVERFLOW_TRAP", "CAPACITY_ERROR_RETURN", "UNDERFILL_CLOSE_REJECTED", "OVERFILL_REJECTED"}
                else "EXACT_OUTCOME_REQUIRES_ALL_NAMED_OBLIGATIONS_IN_DECLARED_ORDER"
            )
            anchor_fields = "trigger+capability_ids+commitment_point"
        else:
            obligations = exact_focus_obligations(member, outcome)
            kind = "EXACT_FOCUS_TRACE_SELECTION"
            activation = "EXACT_OUTCOME_REQUIRES_ALL_NAMED_OBLIGATIONS_IN_DECLARED_ORDER"
            anchor_fields = "pre_state+commitment_point+post_state+pre_abort_invariant"
        evidence_id = f"ROUTE-EVIDENCE::{source['contract_id']}::{route}"
        assert authority["authority_basis"] == evidence_id
        row = {
            "schema_version": EVIDENCE_SCHEMA,
            "route_evidence_id": evidence_id,
            "contract_id": source["contract_id"],
            "member_contract_id": member,
            "outcome_id": source["outcome_id"],
            "route_id": route,
            "evidence_kind": kind,
            "exact_obligation_ids": ",".join(obligations),
            "activation_rule": activation,
            "source_anchor": f"DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv::{source['contract_id']}::{anchor_fields}",
            "authority_status": STATUS,
            "route_evidence_sha256": "",
        }
        row["route_evidence_sha256"] = hash_fields(row, EVIDENCE_FIELDS, "route_evidence_sha256")
        rows.append(row)
    rows.sort(key=lambda row: row["route_evidence_id"])
    return rows


def build_choice_rows(source_rows):
    # The exact matrix freezes one generic GCD take/put witness for Rotate.
    # Faster data-dependent stack-buffer and block-swap variants remain
    # derivable from the same public basis but are deliberately not encoded as
    # an invocation choice until their selector and cost policy are measured.
    assert any(
        row["member_contract_id"] == "DENSE-ROTATE" and suffix(row) == "SUCCESS"
        for row in source_rows
    )
    return []


def validate(source_rows, rows, quiet=False):
    keys = {(row["contract_id"], row["route_id"]) for row in rows}
    expected = {(row["contract_id"], route) for row in source_rows for route in ROUTES}
    assert keys == expected and len(keys) == 4242
    by_contract = defaultdict(dict)
    for row in rows:
        by_contract[row["contract_id"]][row["route_id"]] = row
    for source in source_rows:
        matrix = by_contract[source["contract_id"]]
        member = source["member_contract_id"]
        outcome = suffix(source)
        if member in COPY_MEMBERS and outcome == "SUCCESS":
            assert matrix["CORE-COPY"]["disposition"] == "CONDITIONAL_WITH_FROZEN_PREDICATE"
            assert matrix["CORE-COPY"]["predicate_id"] == "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY"
            assert matrix["CORE-TAKE-PUT"]["disposition"] == "FORBIDDEN"
        if member in COPY_MEMBERS and outcome != "SUCCESS":
            assert matrix["CORE-COPY"]["disposition"] == "FORBIDDEN"
        if member == "DENSE-POP" and outcome == "EMPTY":
            assert matrix["CORE-TAKE-PUT"]["disposition"] == "FORBIDDEN"
        if member == "DENSE-POP" and outcome == "VALUE_RETURNED":
            assert matrix["CORE-TAKE-PUT"]["disposition"] == "REQUIRED"
        if member == "DENSE-FILL-CLONE" and outcome == "BEHAVIOR_ABORT":
            assert matrix["CORE-PARTITION-BORROW"]["disposition"] == "CONDITIONAL_WITH_FROZEN_PREDICATE"
            assert matrix["CORE-PARTITION-BORROW"]["predicate_id"] == "PREDICATE::TARGET_LIVE_RANGE_NONEMPTY"
        if member == "DENSE-EAGER-EXTRACT" and outcome == "SUCCESS":
            assert matrix["CORE-EMPTY-CARRIER"]["disposition"] == "REQUIRED"
        if member == "DENSE-EAGER-SPLICE" and outcome in {"SUCCESS_NO_GROW", "SUCCESS_GROW"}:
            assert matrix["CORE-EMPTY-CARRIER"]["disposition"] == "REQUIRED"
        if member == "DENSE-INTO-OWNER":
            assert matrix["CORE-REPACKAGE-FULL-STORAGE"]["disposition"] == "UNRESOLVED_WITH_BLOCKER"
            assert matrix["CORE-RESHAPE-PARTITION"]["disposition"] == "UNRESOLVED_WITH_BLOCKER"
            assert matrix["CORE-TAKE-PUT"]["disposition"] == "FORBIDDEN"
        if member == "DENSE-INTO-BOXED" and outcome == "SUCCESS_NO_CHANGE":
            assert matrix["CORE-REPACKAGE-FULL-STORAGE"]["disposition"] == "UNRESOLVED_WITH_BLOCKER"
            assert matrix["CORE-RESHAPE-PARTITION"]["disposition"] == "UNRESOLVED_WITH_BLOCKER"
        if member == "DENSE-INTO-BOXED" and outcome in {"SUCCESS_RELOCATE", "OOM_ABORT"}:
            assert matrix["CORE-REPACKAGE-FULL-STORAGE"]["disposition"] == "FORBIDDEN"
        if member == "DENSE-INTO-FLATTENED" and outcome == "SUCCESS":
            assert matrix["CORE-RESHAPE-PARTITION"]["disposition"] == "REQUIRED"
            assert matrix["CORE-TAKE-PUT"]["disposition"] == "FORBIDDEN"
        if member == "DENSE-INTO-FLATTENED" and outcome == "PRECONDITION_TRAP":
            assert matrix["CORE-RESHAPE-PARTITION"]["disposition"] == "FORBIDDEN"
    if not quiet:
        print("matrix_rows", len(rows), file=sys.stderr)
        for route in ROUTES:
            print(route, dict(sorted(Counter(row["disposition"] for row in rows if row["route_id"] == route).items())), file=sys.stderr)
        print("predicates", dict(sorted(Counter(row["predicate_id"] for row in rows if row["predicate_id"] != "NONE").items())), file=sys.stderr)
        print("blockers", dict(sorted(Counter(row["blocker_ids"] for row in rows if row["blocker_ids"] != "NONE").items())), file=sys.stderr)


def render(rows):
    lines = ["\t".join(FIELDS)]
    lines.extend("\t".join(row[field] for field in FIELDS) for row in rows)
    return "\n".join(lines) + "\n"


def render_table(rows, fields):
    lines = ["\t".join(fields)]
    lines.extend("\t".join(row[field] for field in fields) for row in rows)
    return lines


def verify_published_tables(authority, predicates, evidence, choices):
    tables = (
        (TARGET, authority, FIELDS),
        (PREDICATE_TARGET, predicates, PREDICATE_FIELDS),
        (EVIDENCE_TARGET, evidence, EVIDENCE_FIELDS),
        (CHOICE_TARGET, choices, CHOICE_FIELDS),
    )
    for path, rows, fields in tables:
        expected = "\n".join(render_table(rows, fields)) + "\n"
        if path.read_text() != expected:
            raise SystemExit(f"published authority differs from independent builder: {path}")
        print(f"verified {path.name}: {len(rows)} rows")


def render_table_patch(target, rows, fields, start, end):
    lines = render_table(rows, fields)
    print("*** Begin Patch")
    if start == 0:
        print(f"*** Add File: {target}")
        for line in lines[: end + 1]:
            print("+" + line)
    else:
        print(f"*** Update File: {target}")
        print("@@")
        print(" " + lines[start])
        for line in lines[start + 1 : end + 1]:
            print("+" + line)
    print("*** End Patch")


def render_patch(content: str):
    print("*** Begin Patch")
    print(f"*** Add File: {TARGET}")
    for line in content.splitlines():
        print("+" + line)
    print("*** End Patch")


if __name__ == "__main__":
    source, authority = build_rows()
    validate(
        source,
        authority,
        quiet=any(
            argument.startswith("patch") or argument == "verify"
            for argument in sys.argv[1:]
        ),
    )
    predicates = build_predicate_rows(authority)
    evidence = build_evidence_rows(source, authority)
    choices = build_choice_rows(source)
    if len(sys.argv) == 2 and sys.argv[1] == "patch":
        render_patch(render(authority))
    elif len(sys.argv) == 4 and sys.argv[1] == "patch-authority-segment":
        render_table_patch(TARGET, authority, FIELDS, int(sys.argv[2]), int(sys.argv[3]))
    elif len(sys.argv) == 2 and sys.argv[1] == "patch-predicates":
        render_table_patch(PREDICATE_TARGET, predicates, PREDICATE_FIELDS, 0, len(predicates))
    elif len(sys.argv) == 2 and sys.argv[1] == "patch-evidence":
        render_table_patch(EVIDENCE_TARGET, evidence, EVIDENCE_FIELDS, 0, len(evidence))
    elif len(sys.argv) == 2 and sys.argv[1] == "patch-choices":
        render_table_patch(CHOICE_TARGET, choices, CHOICE_FIELDS, 0, len(choices))
    elif len(sys.argv) == 2 and sys.argv[1] == "verify":
        verify_published_tables(authority, predicates, evidence, choices)
    elif len(sys.argv) == 2 and sys.argv[1] == "summary":
        print("predicate_rows", len(predicates), file=sys.stderr)
        print("evidence_rows", len(evidence), file=sys.stderr)
        print("choice_rows", len(choices), file=sys.stderr)
    else:
        sys.stdout.write(render(authority))
