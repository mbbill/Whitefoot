#!/usr/bin/env python3
"""Verify the exact scoped-versus-full stored-borrow contract overlay."""

from __future__ import annotations

import argparse
import csv
from collections import Counter
from dataclasses import dataclass
from pathlib import Path


CLASSIFICATION_HEADER = [
    "contract_id",
    "stored_borrow_scope",
    "scope_owner_contract_ids",
    "rationale",
]
OVERLAY_HEADER = [
    "contract_id",
    "branch_id",
    "route_scope",
    "role",
    "returns_borrow_bearing_owner",
    "requires_result_provenance",
    "condition",
    "conditional_capability_ids",
    "disposition",
    "reopening_trigger",
]
VALID_CLASSIFICATIONS = {
    "ACTIVE_BR_STORED",
    "BOUNDARY_EVIDENCE_ONLY",
    "DELEGATED_TO_FAMILY_BRANCHES",
    "DEFERRED_BRANCHES",
    "FRAME_SCOPE_DEFERRED",
    "NO_STORED_BORROW_COMPLEMENT",
}
VALID_ROLES = {
    "STORED_TRANSITION",
    "BORROW_BEARING_RESULT",
    "RETAINED_STATE",
}
ROUTE_SCOPE = "REGION_FREE_BORROW_FREE"
DISPOSITION = "DEFERRED:stored-borrow-family"
REOPENING_TRIGGER = (
    "Stored-borrow Family Lock proves this exact conditional branch and reopens every "
    "shared representation, fact path, or generated-code dependency it changes."
)
BASE_SCOPE_SENTENCE = (
    "The base matrix route applies only when every retained generic value in this "
    "branch is region-free and borrow-free."
)
ACTIVE_RESULT_EXCEPTION: set[str] = set()
NORMAL_CLASSIFICATIONS = {
    "ACTIVE_BR_STORED",
    "DEFERRED_BRANCHES",
    "NO_STORED_BORROW_COMPLEMENT",
}
BOUNDARY_IDS = {
    "RAW-SAFE-LEAK-01",
    "RAW-SAFE-SPARE-01",
    "RAW-UNSAFE-ACCESS-01",
    "RAW-UNSAFE-ALIGN-01",
    "RAW-UNSAFE-INIT-01",
    "RAW-UNSAFE-LEN-01",
    "RAW-UNSAFE-TEXT-01",
    "RAW-UNSAFE-RC-01",
    "RAW-UNSAFE-BORROW-01",
}
FRAME_SCOPE_IDS = {
    "RAW-SAFE-OWNERSHIP-01",
    "RAW-UNSAFE-RECONSTRUCT-01",
}
DELEGATED_CONTRACT_ID = "ALLOC-ERROR-01"
DELEGATED_OWNER_IDS = (
    "SEQ-TRY-RESERVE-01",
    "DEQUE-RESERVE-01",
    "HEAP-RESERVE-01",
    "HMAP-RESERVE-01",
    "HSET-RESERVE-01",
    "STRING-RESERVE-01",
)
DELEGATED_OWNER_STATES = {
    "SEQ-TRY-RESERVE-01": "DEFERRED_BRANCHES",
    "DEQUE-RESERVE-01": "DEFERRED_BRANCHES",
    "HEAP-RESERVE-01": "DEFERRED_BRANCHES",
    "HMAP-RESERVE-01": "DEFERRED_BRANCHES",
    "HSET-RESERVE-01": "DEFERRED_BRANCHES",
    "STRING-RESERVE-01": "NO_STORED_BORROW_COMPLEMENT",
}
PINNED_BULK_STORED = {
    ("SEQ-APPEND-01", "BORROW_BEARING_STORED_TRANSITION"),
    ("SEQ-EXTEND-COPY-01", "BORROW_BEARING_STORED_TRANSITION"),
    ("HEAP-APPEND-01", "BORROW_BEARING_STORED_TRANSITION"),
    ("DEQUE-BULK-01", "APPEND_BORROW_BEARING_PAYLOAD"),
    ("LIST-BULK-01", "APPEND_BORROW_BEARING_PAYLOAD"),
    ("OMAP-BULK-01", "APPEND_BORROW_BEARING_ENTRIES"),
    ("OSET-BULK-01", "APPEND_CLEAR_BORROW_BEARING_PAYLOAD"),
}
PINNED_BULK_RESULT = {
    ("VIEW-CONCAT-01", "BORROW_BEARING_OWNED_RESULT"),
    ("SEQ-SPLIT-01", "BORROW_BEARING_OWNED_RESULT"),
    ("DEQUE-BULK-01", "SPLIT_OFF_BORROW_BEARING_RESULT"),
    ("LIST-BULK-01", "SPLIT_OFF_BORROW_BEARING_RESULT"),
    ("OMAP-BULK-01", "SPLIT_OFF_BORROW_BEARING_RESULT"),
    ("OSET-BULK-01", "SPLIT_OFF_BORROW_BEARING_RESULT"),
}
EXPECTED_CLASSIFICATION_COUNTS = Counter(
    {
        "ACTIVE_BR_STORED": 26,
        "BOUNDARY_EVIDENCE_ONLY": 9,
        "DEFERRED_BRANCHES": 138,
        "FRAME_SCOPE_DEFERRED": 2,
        "NO_STORED_BORROW_COMPLEMENT": 100,
        "DELEGATED_TO_FAMILY_BRANCHES": 1,
    }
)
EXPECTED_ROLE_COUNTS = Counter(
    {
        "BORROW_BEARING_RESULT": 86,
        "RETAINED_STATE": 36,
        "STORED_TRANSITION": 172,
    }
)
EXPECTED_RESULT_PROVENANCE_COUNTS = Counter({"no": 155, "yes": 139})
EXPECTED_RETURN_COUNTS = Counter({"no": 194, "yes": 100})
EXPECTED_OVERLAY_ROWS = 294
PINNED_NO_COMPLEMENT = {
    "RC-INIT-01",
    "RANGE-VALUE-FULL-01",
    "RANGE-BOUND-BORROW-01",
    "RANGE-LEGACY-INCLUSIVE-ACCESS-01",
    "RANGE-BOUNDS-PROTOCOL-01",
    "RANGE-BOUNDS-CONTAINS-01",
    "RANGE-ITER-HALFOPEN-01",
    "RANGE-ITER-FROM-01",
    "RANGE-ITER-INCLUSIVE-01",
    "ITER-ADAPT-DUPLICATE-01",
    "ITER-ADAPT-CYCLE-01",
    "TRAIT-ITER-01",
    "TRAIT-DOUBLE-01",
    "TRAIT-DEREF-01",
    "TRAIT-BORROW-01",
}

RANGE_BOUNDS_OVERLAY_CONTRACTS = {
    "VIEW-COPY-01",
    "SEQ-EXTEND-COPY-01",
    "SEQ-DRAIN-01",
    "DEQUE-RANGE-01",
    "DEQUE-DRAIN-01",
    "OMAP-RANGE-01",
    "OSET-RANGE-01",
    "STRING-PUSH-01",
    "STRING-DRAIN-01",
    "STRING-REPLACE-01",
}
ACTIVE_RANGE_BOUNDS_STATE_CONTRACTS = {
    "SEQ-EXTRACT-01",
    "SEQ-SPLICE-01",
    "OMAP-FILTER-01",
    "OSET-FILTER-01",
}
BASE_RANGE_BOUNDS_ONLY_CONTRACTS = {
    "RANGE-BOUNDS-PROTOCOL-01",
    "RANGE-BOUNDS-CONTAINS-01",
}
CLONE_EFFECT_OVERLAY_KEYS = {
    ("VIEW-CLONE-01", "CLONE_FROM_BORROW_EFFECT"),
    ("VIEW-FILL-01", "CLONE_FROM_BORROW_EFFECT"),
    ("VIEW-ALLOC-01", "BORROW_BEARING_OWNED_RESULT"),
    ("VIEW-CONCAT-01", "BORROW_BEARING_OWNED_RESULT"),
    ("INIT-WRITE-01", "CLONE_RESULT_BORROW_STATE"),
    ("SEQ-EXTEND-COPY-01", "CLONE_RESULT_BORROW_STATE"),
    ("SEQ-RESIZE-01", "CLONE_RESULT_BORROW_STATE"),
    ("DEQUE-RESIZE-01", "CLONE_RESULT_BORROW_STATE"),
    ("RC-UNIQUE-01", "MAKE_MUT_CLONE_RESULT_BORROW_STATE"),
    ("RC-UNWRAP-01", "BORROW_BEARING_OWNED_RESULT"),
    ("RANGE-BOUND-CLONE-01", "CLONED_OUTPUT_PAYLOAD"),
    ("TRAIT-CONVERT-01", "FROM_CLONED_BORROW_PAYLOAD"),
    ("TRAIT-CLONE-01", "CLONE_FRESH_OWNED_PAYLOAD"),
    ("TRAIT-CLONE-01", "CLONE_FROM_OWNED_PAYLOAD"),
    ("TRAIT-CLONE-01", "CLONE_CACHED_BORROW_BEARING_STATE"),
}


@dataclass(frozen=True)
class RowShape:
    role: str
    returned: str
    result_provenance: str


PINNED_EXACT_BRANCHES: dict[str, set[str]] = {}
PINNED_ROW_SHAPES: dict[tuple[str, str], RowShape] = {}


def pin_contract(
    contract_id: str,
    *,
    transitions: tuple[str, ...] = (),
    results: tuple[str, ...] = (),
    retained_results: tuple[str, ...] = (),
    retained_internal: tuple[str, ...] = (),
    internal_results: tuple[str, ...] = (),
) -> None:
    """Register an independently adjudicated exact branch and role partition."""

    branch_ids = (
        transitions
        + results
        + retained_results
        + retained_internal
        + internal_results
    )
    if len(branch_ids) != len(set(branch_ids)):
        raise ValueError(f"duplicate semantic branch pin for {contract_id}")
    if contract_id in PINNED_EXACT_BRANCHES:
        raise ValueError(f"duplicate semantic contract pin for {contract_id}")
    PINNED_EXACT_BRANCHES[contract_id] = set(branch_ids)
    for branch_id in transitions:
        PINNED_ROW_SHAPES[(contract_id, branch_id)] = RowShape(
            "STORED_TRANSITION", "no", "no"
        )
    for branch_id in results:
        PINNED_ROW_SHAPES[(contract_id, branch_id)] = RowShape(
            "BORROW_BEARING_RESULT", "yes", "yes"
        )
    for branch_id in retained_results:
        PINNED_ROW_SHAPES[(contract_id, branch_id)] = RowShape(
            "RETAINED_STATE", "yes", "yes"
        )
    for branch_id in retained_internal:
        PINNED_ROW_SHAPES[(contract_id, branch_id)] = RowShape(
            "RETAINED_STATE", "no", "no"
        )
    for branch_id in internal_results:
        PINNED_ROW_SHAPES[(contract_id, branch_id)] = RowShape(
            "STORED_TRANSITION", "no", "yes"
        )


# Arrays and every stateful callable/key carrier whose environment is not already
# covered by an active BR-STORED base row.
pin_contract(
    "ARR-MAP-01",
    transitions=("CALLABLE_ENV_BORROW_STATE", "INPUT_ARRAY_BORROW_PAYLOAD"),
    results=("OUTPUT_ARRAY_BORROW_PAYLOAD",),
)
for contract_id in ("VIEW-SEARCH-02", "VIEW-ORDER-CHECK-01", "DEQUE-SEARCH-01"):
    pin_contract(
        contract_id,
        transitions=("CALLABLE_ENV_BORROW_STATE",),
        internal_results=("KEY_RESULT_BORROW_STATE",),
    )
pin_contract(
    "VIEW-SORT-01",
    transitions=(
        "BORROW_BEARING_STORED_TRANSITION",
        "CALLABLE_ENV_BORROW_STATE",
    ),
    internal_results=("CACHED_KEY_BORROW_STATE", "KEY_RESULT_BORROW_STATE"),
)
pin_contract(
    "VIEW-SORT-02",
    transitions=(
        "BORROW_BEARING_STORED_TRANSITION",
        "CALLABLE_ENV_BORROW_STATE",
    ),
    internal_results=("KEY_RESULT_BORROW_STATE",),
)
pin_contract(
    "VIEW-SELECT-01",
    transitions=(
        "BORROW_BEARING_STORED_TRANSITION",
        "CALLABLE_ENV_BORROW_STATE",
    ),
    internal_results=("KEY_RESULT_BORROW_STATE",),
)
for contract_id in (
    "VIEW-FILL-01",
    "SEQ-RESIZE-01",
    "SEQ-RETAIN-01",
    "DEQUE-RESIZE-01",
    "DEQUE-RETAIN-01",
    "HEAP-RETAIN-01",
):
    pin_contract(
        contract_id,
        transitions=("BORROW_BEARING_STORED_TRANSITION", "CALLABLE_ENV_BORROW_STATE"),
    )
for contract_id in (
    "TEXT-SEARCH-01",
    "TEXT-TRIM-01",
    "TEXT-REPLACE-01",
    "STRING-RETAIN-01",
    "REF-GUARD-01",
):
    pin_contract(contract_id, transitions=("CALLABLE_ENV_BORROW_STATE",))
for contract_id in ("SEQ-POP-01", "DEQUE-POP-01", "RC-CYCLIC-01"):
    pin_contract(
        contract_id,
        transitions=("CALLABLE_ENV_BORROW_STATE",),
        results=("BORROW_BEARING_OWNED_RESULT",),
    )
pin_contract(
    "SEQ-DEDUP-01",
    transitions=(
        "BORROW_BEARING_STORED_TRANSITION",
        "CALLABLE_ENV_BORROW_STATE",
    ),
    internal_results=("KEY_RESULT_BORROW_STATE",),
)

# Map, set, entry, and owning-iteration paths. These partitions intentionally
# distinguish storage, destruction, returned ownership, and retained hasher state.
MAP_INSERT_TRANSITIONS = (
    "ABSENT_STORE_KEY_VALUE",
    "OCCUPIED_DROP_OFFERED_KEY",
    "OCCUPIED_STORE_OFFERED_VALUE",
)
MAP_INSERT_RESULTS = (
    "EXIT_RECOVERABLE_FAILURE",
    "OCCUPIED_RETURN_DISPLACED_VALUE",
)
pin_contract(
    "OMAP-INSERT-01", transitions=MAP_INSERT_TRANSITIONS, results=MAP_INSERT_RESULTS
)
pin_contract(
    "HMAP-INSERT-01",
    transitions=MAP_INSERT_TRANSITIONS,
    results=MAP_INSERT_RESULTS,
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "OMAP-REMOVE-01",
    transitions=("REMOVE_DROP_KEY",),
    results=(
        "POP_ENDPOINT_PAIR_RESULT",
        "REMOVE_ENTRY_PAIR_RESULT",
        "REMOVE_VALUE_RESULT",
    ),
)
pin_contract(
    "HMAP-REMOVE-01",
    transitions=("REMOVE_DROP_KEY",),
    results=("REMOVE_ENTRY_PAIR_RESULT", "REMOVE_VALUE_RESULT"),
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
SET_INSERT_TRANSITIONS = (
    "INSERT_DUPLICATE_DROP_OFFERED",
    "INSERT_NOVEL_STORE",
    "REPLACE_ABSENT_STORE",
    "REPLACE_PRESENT_STORE_OFFERED",
)
SET_INSERT_RESULTS = (
    "EXIT_RECOVERABLE_FAILURE",
    "REPLACE_PRESENT_RETURN_DISPLACED",
)
pin_contract(
    "OSET-INSERT-01", transitions=SET_INSERT_TRANSITIONS, results=SET_INSERT_RESULTS
)
pin_contract(
    "HSET-INSERT-01",
    transitions=SET_INSERT_TRANSITIONS,
    results=SET_INSERT_RESULTS,
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "OSET-REMOVE-01",
    transitions=("REMOVE_DROP_STORED",),
    results=("POP_ENDPOINT_RESULT", "TAKE_RESULT"),
)
pin_contract(
    "HSET-REMOVE-01",
    transitions=("REMOVE_DROP_STORED",),
    results=("TAKE_RESULT",),
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
MAP_ITER_TRANSITIONS = (
    "OWNING_INTO_KEYS_OMITTED_VALUE_DROP",
    "OWNING_INTO_VALUES_OMITTED_KEY_DROP",
)
MAP_ITER_RESULTS = (
    "OWNING_INTOITER_PAIR_RESULT",
    "OWNING_INTO_KEYS_RESULT",
    "OWNING_INTO_VALUES_RESULT",
)
pin_contract(
    "OMAP-ITER-01", transitions=MAP_ITER_TRANSITIONS, results=MAP_ITER_RESULTS
)
pin_contract(
    "HMAP-ITER-01",
    transitions=MAP_ITER_TRANSITIONS + ("OWNING_HASH_BUILDER_DROP",),
    results=MAP_ITER_RESULTS,
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "OSET-RANGE-01",
    transitions=("BORROW_BEARING_RANGE_DESCRIPTOR",),
    results=("OWNING_INTOITER_BORROW_BEARING_RESULT",),
)
pin_contract(
    "HSET-ITER-01",
    transitions=("OWNING_HASH_BUILDER_DROP",),
    results=("OWNING_INTOITER_BORROW_BEARING_RESULT",),
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "MAP-ENTRY-01",
    transitions=(
        "AND_MODIFY_CALLABLE_ENV",
        "AND_MODIFY_STORED_VALUE",
        "INSERT_ENTRY_STORE_OR_REPLACE",
        "OCCUPIED_ENTRY_CANDIDATE_DROP",
        "OR_DEFAULT_VACANT_STORE_KEY_VALUE",
        "OR_INSERT_OCCUPIED_DROP_OFFERED_VALUE",
        "OR_INSERT_VACANT_STORE_KEY_VALUE",
        "OR_INSERT_WITH_CALLABLE_ENV",
        "OR_INSERT_WITH_KEY_CALLABLE_ENV",
        "OR_INSERT_WITH_KEY_VACANT_STORE_KEY_VALUE",
        "OR_INSERT_WITH_VACANT_STORE_KEY_VALUE",
    ),
    results=("EXIT_RECOVERABLE_FAILURE",),
    retained_results=("VACANT_GUARD_CANDIDATE_KEY",),
    retained_internal=("HASH_ENTRY_STORED_HASHER_STATE",),
)
pin_contract(
    "MAP-OCCUPIED-01",
    transitions=("INSERT_STORE_OFFERED_VALUE", "REMOVE_DROP_KEY"),
    results=(
        "INSERT_RETURN_DISPLACED_VALUE",
        "REMOVE_ENTRY_PAIR_RESULT",
        "REMOVE_VALUE_RESULT",
    ),
    retained_internal=("HASH_GUARD_MAP_HASHER_STATE",),
)
pin_contract(
    "MAP-VACANT-01",
    transitions=("EXIT_PAYLOAD_DROP", "MEMBER_INSERT", "MEMBER_INSERT_ENTRY"),
    results=("EXIT_RECOVERABLE_FAILURE", "MEMBER_INTO_KEY"),
    retained_internal=("HASH_GUARD_MAP_HASHER_STATE",),
)

# Hash-builder state is a retained fact channel even when an operation neither
# hashes nor returns the builder itself.
pin_contract(
    "HMAP-META-01",
    transitions=("EVENTUAL_BORROW_BEARING_PAYLOAD_DROP",),
    retained_results=("HASH_BUILDER_BORROW_BEARING_RESULT",),
    retained_internal=("EXISTING_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "HSET-META-01",
    transitions=("EVENTUAL_BORROW_BEARING_PAYLOAD_DROP",),
    retained_results=("HASH_BUILDER_BORROW_BEARING_RESULT",),
    retained_internal=("EXISTING_HASH_BUILDER_BORROW_STATE",),
)
for contract_id in ("HMAP-RESERVE-01", "HSET-RESERVE-01"):
    pin_contract(
        contract_id,
        transitions=("BORROW_BEARING_STORED_TRANSITION",),
        retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
    )
for contract_id in (
    "HMAP-LOOKUP-01",
    "HMAP-DISJOINT-01",
    "HSET-LOOKUP-01",
):
    pin_contract(
        contract_id, retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",)
    )
pin_contract(
    "HMAP-DRAIN-01",
    transitions=("CLEAR_BORROW_BEARING_ENTRIES",),
    results=("DRAIN_BORROW_BEARING_RESULT",),
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "HSET-DRAIN-01",
    transitions=("CLEAR_BORROW_BEARING_PAYLOAD",),
    results=("DRAIN_BORROW_BEARING_RESULT",),
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "SET-REL-01",
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "SET-ALG-02",
    retained_results=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "TRAIT-INDEX-01",
    retained_internal=("STORED_HASH_BUILDER_BORROW_STATE",),
)
pin_contract(
    "TRAIT-INTOITER-01",
    transitions=("OWNING_HASH_BUILDER_DROP",),
    results=("OWNING_ENTRANCE_BORROW_BEARING_RESULT",),
)
pin_contract(
    "TRAIT-CMP-01",
    retained_internal=(
        "CALLER_HASHER_BORROW_STATE",
        "STORED_HASH_BUILDER_BORROW_STATE",
    ),
)

# Erased owners, weak handles, parsing, RefCell state, and clone helper state.
for contract_id in ("BOX-DOWNCAST-01", "RC-DOWNCAST-01"):
    pin_contract(
        contract_id,
        results=("FAILURE_ERASED_BORROW_PAYLOAD", "SUCCESS_CONCRETE_BORROW_PAYLOAD"),
    )
pin_contract(
    "RC-WEAK-01",
    results=("DOWNGRADE_BORROW_PAYLOAD_HANDLE", "UPGRADE_BORROW_PAYLOAD_HANDLE"),
)
pin_contract(
    "TEXT-PARSE-01",
    results=("ERROR_OWNED_BORROW_PAYLOAD", "SELF_OWNED_BORROW_PAYLOAD"),
)
pin_contract(
    "REFCELL-OWNER-01", results=("NEW_INTO_INNER_BORROW_BEARING_RESULT",)
)
pin_contract(
    "REFCELL-REPLACE-01",
    transitions=("CALLABLE_ENV_BORROW_STATE", "SWAP_BORROW_PAYLOAD_STATE"),
    results=("RETURNED_BORROW_PAYLOAD",),
)
pin_contract(
    "TRAIT-CLONE-01",
    transitions=("CLONE_FROM_OWNED_PAYLOAD",),
    results=("CLONE_FRESH_OWNED_PAYLOAD", "CLONE_SHARED_HANDLE_PAYLOAD"),
    retained_results=("CLONE_CACHED_BORROW_BEARING_STATE",),
)
pin_contract(
    "TRAIT-CONVERT-01",
    results=(
        "FROM_CLONED_BORROW_PAYLOAD",
        "FROM_OWNED_BORROW_PAYLOAD",
        "TRY_FROM_OWNED_BORROW_PAYLOAD_OK",
        "TRY_FROM_BORROW_PAYLOAD_ERROR",
    ),
)
pin_contract("TRAIT-DEFAULT-01", results=("DEFAULT_LIVE_BORROW_PAYLOAD",))

# Generic range values and every stable-safe RangeBounds consumer. Step-constrained
# iterator contracts remain explicitly outside this complement.
for contract_id, branch_id in (
    ("RANGE-VALUE-HALFOPEN-01", "RANGE_ENDPOINTS"),
    ("RANGE-VALUE-FROM-01", "RANGE_FROM_START"),
    ("RANGE-VALUE-INCLUSIVE-01", "RANGE_INCLUSIVE_ENDPOINTS"),
    ("RANGE-VALUE-TO-INCLUSIVE-01", "RANGE_TO_INCLUSIVE_END"),
    ("RANGE-VALUE-TO-EXCLUSIVE-01", "RANGE_TO_END"),
    ("RANGE-BOUND-VALUE-01", "INCLUDED_EXCLUDED_PAYLOAD"),
    ("RANGE-LEGACY-HALFOPEN-STATE-01", "RANGE_ENDPOINTS"),
    ("RANGE-LEGACY-FROM-STATE-01", "RANGE_FROM_START"),
    ("RANGE-LEGACY-INCLUSIVE-STATE-01", "RANGE_INCLUSIVE_ENDPOINTS"),
):
    pin_contract(contract_id, retained_results=(branch_id,))
pin_contract(
    "RANGE-BOUND-CLONE-01",
    transitions=("CLONED_INPUT_BOUND_REF",),
    results=("CLONED_OUTPUT_PAYLOAD",),
)
pin_contract(
    "RANGE-BOUND-MAP-01",
    transitions=("CALLABLE_ENV_BORROW_STATE", "MAP_INPUT_PAYLOAD"),
    results=("MAP_OUTPUT_PAYLOAD",),
)
pin_contract(
    "RANGE-LEGACY-INCLUSIVE-INTO-01", results=("INTO_INNER_ENDPOINTS",)
)
for contract_id in ("VIEW-COPY-01", "SEQ-EXTEND-COPY-01"):
    pin_contract(
        contract_id,
        transitions=(
            "BORROW_BEARING_RANGE_DESCRIPTOR",
            "BORROW_BEARING_STORED_TRANSITION",
        ),
    )
for contract_id in ("SEQ-DRAIN-01", "DEQUE-DRAIN-01"):
    pin_contract(
        contract_id,
        transitions=("BORROW_BEARING_RANGE_DESCRIPTOR",),
        results=("BORROW_BEARING_OWNED_RESULT",),
    )
for contract_id in (
    "DEQUE-RANGE-01",
    "OMAP-RANGE-01",
    "STRING-PUSH-01",
    "STRING-DRAIN-01",
    "STRING-REPLACE-01",
):
    pin_contract(contract_id, transitions=("BORROW_BEARING_RANGE_DESCRIPTOR",))
pin_contract(
    "INIT-WRITE-01", transitions=("BORROW_BEARING_STORED_TRANSITION",)
)
pin_contract(
    "VIEW-CLONE-01", transitions=("BORROW_BEARING_STORED_TRANSITION",)
)
pin_contract(
    "RC-UNIQUE-01", transitions=("BORROW_BEARING_STORED_TRANSITION",)
)
pin_contract(
    "MEM-TAKE-01", results=("BORROW_BEARING_OWNED_RESULT",)
)

CALLABLE_STATE_CONTRACTS = {
    contract_id
    for contract_id, branches in PINNED_EXACT_BRANCHES.items()
    if "CALLABLE_ENV_BORROW_STATE" in branches
}
KEY_STATE_CONTRACTS = {
    contract_id
    for contract_id, branches in PINNED_EXACT_BRANCHES.items()
    if "KEY_RESULT_BORROW_STATE" in branches
}
STORED_HASH_STATE_CONTRACTS = {
    "HMAP-RESERVE-01",
    "HMAP-LOOKUP-01",
    "HMAP-DISJOINT-01",
    "HMAP-INSERT-01",
    "HMAP-REMOVE-01",
    "HMAP-ITER-01",
    "HMAP-DRAIN-01",
    "HSET-RESERVE-01",
    "HSET-LOOKUP-01",
    "HSET-INSERT-01",
    "HSET-REMOVE-01",
    "HSET-ITER-01",
    "HSET-DRAIN-01",
    "SET-REL-01",
    "SET-ALG-02",
    "TRAIT-INDEX-01",
    "TRAIT-CMP-01",
}
HASH_BEHAVIOR_STATE_CONTRACTS = {
    "HMAP-RESERVE-01",
    "HMAP-LOOKUP-01",
    "HMAP-DISJOINT-01",
    "HMAP-INSERT-01",
    "HMAP-REMOVE-01",
    "HSET-RESERVE-01",
    "HSET-LOOKUP-01",
    "HSET-INSERT-01",
    "HSET-REMOVE-01",
    "SET-REL-01",
    "SET-ALG-02",
    "TRAIT-INDEX-01",
    "TRAIT-CMP-01",
}
CALLER_HASHER_STATE_CONTRACTS = {"TRAIT-CMP-01"}
EXISTING_HASH_STATE_CONTRACTS = {"HMAP-META-01", "HSET-META-01"}
HASH_GUARD_STATE_CONTRACTS = {"MAP-OCCUPIED-01", "MAP-VACANT-01"}
GENERATED_HASHER_RESULT_CONTRACTS = {
    "HMAP-RESERVE-01",
    "HMAP-LOOKUP-01",
    "HMAP-DISJOINT-01",
    "HMAP-INSERT-01",
    "HMAP-REMOVE-01",
    "HSET-RESERVE-01",
    "HSET-LOOKUP-01",
    "HSET-INSERT-01",
    "HSET-REMOVE-01",
    "MAP-ENTRY-01",
    "SET-REL-01",
    "SET-ALG-02",
    "TRAIT-INDEX-01",
    "TRAIT-CMP-01",
}

for contract_id in GENERATED_HASHER_RESULT_CONTRACTS:
    branch_id = "GENERATED_HASHER_BORROW_STATE"
    branches = PINNED_EXACT_BRANCHES.get(contract_id)
    if branches is None:
        raise ValueError(f"generated-hasher contract lacks an exact partition: {contract_id}")
    if branch_id in branches:
        raise ValueError(f"duplicate generated-hasher branch pin: {contract_id}")
    branches.add(branch_id)
    PINNED_ROW_SHAPES[(contract_id, branch_id)] = RowShape(
        "STORED_TRANSITION", "no", "yes"
    )

ADDED_TRANSITION_BRANCHES = {
    "VIEW-CLONE-01": ("CLONE_FROM_BORROW_EFFECT",),
    "VIEW-FILL-01": ("CLONE_FROM_BORROW_EFFECT",),
}
INTERNAL_RESULT_BRANCHES = {
    "VIEW-FILL-01": ("PRODUCER_RESULT_BORROW_STATE",),
    "INIT-WRITE-01": ("CLONE_RESULT_BORROW_STATE",),
    "SEQ-EXTEND-COPY-01": ("CLONE_RESULT_BORROW_STATE",),
    "SEQ-RESIZE-01": (
        "CLONE_RESULT_BORROW_STATE",
        "PRODUCER_RESULT_BORROW_STATE",
    ),
    "DEQUE-RESIZE-01": (
        "CLONE_RESULT_BORROW_STATE",
        "PRODUCER_RESULT_BORROW_STATE",
    ),
    "RC-UNIQUE-01": ("MAKE_MUT_CLONE_RESULT_BORROW_STATE",),
    "REFCELL-REPLACE-01": (
        "INSTALLED_CALLBACK_RESULT_BORROW_STATE",
        "INSTALLED_DEFAULT_RESULT_BORROW_STATE",
    ),
    "MEM-TAKE-01": ("INSTALLED_DEFAULT_RESULT_BORROW_STATE",),
    "TEXT-SEARCH-01": ("PATTERN_SEARCHER_BORROW_STATE",),
    "TEXT-TRIM-01": ("PATTERN_SEARCHER_BORROW_STATE",),
    "TEXT-REPLACE-01": ("PATTERN_SEARCHER_BORROW_STATE",),
}

for contract_id, branch_ids in ADDED_TRANSITION_BRANCHES.items():
    branches = PINNED_EXACT_BRANCHES.get(contract_id)
    if branches is None:
        raise ValueError(f"added transition lacks exact partition: {contract_id}")
    for branch_id in branch_ids:
        if branch_id in branches:
            raise ValueError(f"duplicate added transition pin: {contract_id}/{branch_id}")
        branches.add(branch_id)
        PINNED_ROW_SHAPES[(contract_id, branch_id)] = RowShape(
            "STORED_TRANSITION", "no", "no"
        )

for contract_id, branch_ids in INTERNAL_RESULT_BRANCHES.items():
    branches = PINNED_EXACT_BRANCHES.get(contract_id)
    if branches is None:
        raise ValueError(f"internal result lacks exact partition: {contract_id}")
    for branch_id in branch_ids:
        if branch_id in branches:
            raise ValueError(f"duplicate internal-result pin: {contract_id}/{branch_id}")
        branches.add(branch_id)
        PINNED_ROW_SHAPES[(contract_id, branch_id)] = RowShape(
            "STORED_TRANSITION", "no", "yes"
        )

for branch_id in (
    "OR_INSERT_WITH_VACANT_STORE_KEY_VALUE",
    "OR_INSERT_WITH_KEY_VACANT_STORE_KEY_VALUE",
    "OR_DEFAULT_VACANT_STORE_KEY_VALUE",
):
    PINNED_ROW_SHAPES[("MAP-ENTRY-01", branch_id)] = RowShape(
        "STORED_TRANSITION", "no", "yes"
    )

# Required fragments are semantic pins, not a byte-for-byte copy of the builder.
# They preserve the provenance and destruction distinctions that caused the audit.
CONDITION_PINS: dict[tuple[str, str], tuple[str, ...]] = {
    ("ARR-MAP-01", "INPUT_ARRAY_BORROW_PAYLOAD"): (
        "consumed [t; n]",
        "destroyed exactly once",
    ),
    ("ARR-MAP-01", "OUTPUT_ARRAY_BORROW_PAYLOAD"): (
        "returned [u; n]",
        "provenance is authorized by the selected callable",
    ),
    ("OMAP-INSERT-01", "OCCUPIED_DROP_OFFERED_KEY"): (
        "canonical stored k survives",
        "offered k",
        "destroyed",
    ),
    ("HMAP-INSERT-01", "OCCUPIED_DROP_OFFERED_KEY"): (
        "canonical stored k survives",
        "offered k",
        "destroyed",
    ),
    ("OSET-INSERT-01", "INSERT_DUPLICATE_DROP_OFFERED"): (
        "canonical stored representative",
        "destroys offered t",
    ),
    ("HSET-INSERT-01", "INSERT_DUPLICATE_DROP_OFFERED"): (
        "canonical stored representative",
        "destroys offered t",
    ),
    ("TEXT-PARSE-01", "SELF_OWNED_BORROW_PAYLOAD"): (
        "independently valid static borrow leaves",
        "promoted zero-sized empty-reference root",
        "no result leaf can derive from the call-scoped input text",
        "empty footprint grants no storage or disjointness authority",
    ),
    ("TEXT-PARSE-01", "ERROR_OWNED_BORROW_PAYLOAD"): (
        "independently valid static borrow leaves",
        "promoted zero-sized empty-reference root",
        "no result leaf can derive from the call-scoped input text",
        "empty footprint grants no storage or disjointness authority",
    ),
    ("TRAIT-CONVERT-01", "FROM_OWNED_BORROW_PAYLOAD"): (
        "moves or representation-reuses",
        "pre-existing payload borrow leaf preserves its exact external or promoted-empty root",
        "no fresh borrow derives from consumed container or allocation storage",
    ),
    ("TRAIT-CONVERT-01", "FROM_CLONED_BORROW_PAYLOAD"): (
        "explicitly clones payload from a borrowed source",
        "separately frozen clone result-provenance relation",
        "may select, swap, or coalesce independently valid roots",
        "no result leaf derives from temporary source-view, receiver, container, or call-frame storage",
    ),
    ("TRAIT-CONVERT-01", "TRY_FROM_OWNED_BORROW_PAYLOAD_OK"): (
        "successful owned tryfrom conversion",
        "pre-existing payload borrow leaf preserves its exact external or promoted-empty root",
        "no fresh borrow derives from consumed container or allocation storage",
    ),
    ("TRAIT-CONVERT-01", "TRY_FROM_BORROW_PAYLOAD_ERROR"): (
        "recoverable failure returns the original owner or another error owner",
        "pre-existing payload borrow leaf preserves its exact external or promoted-empty root",
        "no fresh borrow derives from transient conversion state",
    ),
    ("RC-WEAK-01", "DOWNGRADE_BORROW_PAYLOAD_HANDLE"): (
        "preserving t's live borrow relations",
        "without keeping the payload alive",
        "weak::new is excluded",
    ),
    ("RC-WEAK-01", "UPGRADE_BORROW_PAYLOAD_HANDLE"): (
        "successful weak::upgrade",
        "failed upgrade and weak::new are excluded",
    ),
    ("REFCELL-REPLACE-01", "RETURNED_BORROW_PAYLOAD"): (
        "replace, replace_with, or take",
        "swap is excluded",
    ),
    ("REFCELL-REPLACE-01", "SWAP_BORROW_PAYLOAD_STATE"): (
        "exchanges two live t payloads",
        "returns no t owner",
    ),
    ("TRAIT-CLONE-01", "CLONE_CACHED_BORROW_BEARING_STATE"): (
        "non-cursor state",
        "declared clone result-provenance relation",
        "source-map-only cursor clone remains covered by br-cursor and is excluded",
    ),
    ("TRAIT-CLONE-01", "CLONE_FRESH_OWNED_PAYLOAD"): (
        "declared clone result-provenance relation",
        "may select, swap, or coalesce independently valid roots",
        "no leaf derives from temporary receiver or container storage",
    ),
    ("TRAIT-CLONE-01", "CLONE_FROM_OWNED_PAYLOAD"): (
        "same clone source and destination owners remain valid",
        "surviving leaves keep their roots",
        "every overwritten or otherwise ended leaf ends once",
        "every new leaf is authorized",
        "reused destination allocation or storage grants no provenance",
    ),
    ("RANGE-BOUND-CLONE-01", "CLONED_OUTPUT_PAYLOAD"): (
        "declared clone result-provenance relation",
        "may select, swap, or coalesce independently valid roots",
        "no leaf derives from the temporary bound receiver, borrowed referent view, or call frame",
    ),
    ("TRAIT-DEFAULT-01", "DEFAULT_LIVE_BORROW_PAYLOAD"): (
        "associated default result-provenance relation",
        "independently valid static, global, promoted, or otherwise declared root",
        "no leaf is fabricated from the new owner storage, temporary receiver, or call frame",
    ),
    ("VIEW-ALLOC-01", "BORROW_BEARING_OWNED_RESULT"): (
        "slice::into_vec transfers existing payload owners",
        "slice::to_vec obtains every produced t through the declared clone result-provenance relation",
        "may select, swap, or coalesce independent roots",
        "may not derive from temporary source-view, receiver, allocation, or call-frame storage",
    ),
    ("VIEW-CONCAT-01", "BORROW_BEARING_OWNED_RESULT"): (
        "selected concat, join, copy, or clone result-provenance relation",
        "no produced leaf derives from temporary source-view, receiver, output-allocation, or call-frame storage",
    ),
    ("RC-UNWRAP-01", "BORROW_BEARING_OWNED_RESULT"): (
        "unique try_unwrap, into_inner, or unwrap_or_clone branch moves the existing t",
        "unwrap_or_clone's shared fallback instead obtains t through the declared clone result-provenance relation",
        "may not derive a leaf from temporary receiver, rc allocation, or call-frame storage",
    ),
    ("RC-CYCLIC-01", "BORROW_BEARING_OWNED_RESULT"): (
        "stores the t returned by its fnonce producer",
        "producer result-provenance relation",
        "declared captures or independent static, global, or promoted roots",
        "never from the temporary provisional &weak<t> argument, weak identity, callable storage, or call frame",
    ),
    ("MEM-TAKE-01", "BORROW_BEARING_OWNED_RESULT"): (
        "returns the pre-existing old t owner",
        "preserves every old borrow-leaf root exactly",
        "separately recorded default-result branch",
    ),
}
for contract_id in ("BOX-DOWNCAST-01", "RC-DOWNCAST-01"):
    CONDITION_PINS[(contract_id, "SUCCESS_CONCRETE_BORROW_PAYLOAD")] = (
        "successful downcast returns the same",
        "static lifetime",
        "live static borrow leaves",
    )
    CONDITION_PINS[(contract_id, "FAILURE_ERASED_BORROW_PAYLOAD")] = (
        "failed downcast returns the exact",
        "static lifetime",
        "live static borrow leaves",
    )
for contract_id in CALLABLE_STATE_CONTRACTS:
    CONDITION_PINS.setdefault(
        (contract_id, "CALLABLE_ENV_BORROW_STATE"),
        ("environment contains live borrow leaves", "destroyed exactly once"),
    )
for contract_id in KEY_STATE_CONTRACTS:
    CONDITION_PINS[(contract_id, "KEY_RESULT_BORROW_STATE")] = (
        "contains live borrow leaves",
        "through every comparison",
        "destroyed exactly once",
    )
for contract_id in GENERATED_HASHER_RESULT_CONTRACTS:
    fragments = (
        "each buildhasher::build_hasher call returns exactly one call-local h owner",
        "post-call leaves retained by the same valid s owner jointly follow the declared buildhasher result and behavior-effect relations",
        "every surviving s leaf keeps its exact root",
        "each newly installed s leaf and each initial h leaf follows those relations",
        "a unique leaf moved from s into h ends in s before becoming live in h",
        "never simultaneously live in both",
        "no h leaf derives from the call-scoped &s receiver",
        "address or storage of an s field, or the call frame",
        "an existing leaf value stored in s may transfer with its independent external root",
        "same h owner remains valid",
        "declared hasher behavior-effect relation",
        "required hash::hash and hasher calls",
        "call-scoped reborrows",
        "destroyed exactly once before the operation or lazy cursor step completes",
        "h or s provenance never becomes payload or public result-borrow provenance",
        "hash output may influence the logical probe, destination, or boolean result",
        "alone mints no occupancy, liveness, uniqueness, or check-elision fact",
        "occupancy metadata gates payload access",
        "eq gates equivalence",
    )
    if contract_id == "TRAIT-CMP-01":
        fragments = (
            "only the hashmap and hashset equality branches iterate the left operand",
            "right-hand stored s through get or contains",
            "left-hand s remains retained and unreborrowed",
            "length-mismatch and empty-equal paths create zero h owners",
            "each performed right-hand probe creates exactly one h",
            "hash implementation branch instead uses caller-owned h without buildhasher",
            "all other comparison branches use neither",
        ) + fragments
    CONDITION_PINS[(contract_id, "GENERATED_HASHER_BORROW_STATE")] = fragments
for contract_id, branch_ids in INTERNAL_RESULT_BRANCHES.items():
    for branch_id in branch_ids:
        if branch_id == "CLONE_RESULT_BORROW_STATE":
            fragments = (
                "declared clone result-provenance relation",
                "never derives from",
                "call frame",
            )
        elif branch_id == "PRODUCER_RESULT_BORROW_STATE":
            fragments = (
                "callable result-provenance relation",
                "never derives from",
                "call frame",
            )
        elif branch_id == "MAKE_MUT_CLONE_RESULT_BORROW_STATE":
            fragments = (
                "multiple-strong branch",
                "declared clone-result relation",
                "weak-only relocation branch performs no clone and is excluded",
            )
        elif branch_id == "INSTALLED_CALLBACK_RESULT_BORROW_STATE":
            fragments = (
                "replace_with installs the t returned by fnonce",
                "callable result-provenance relation",
                "temporary &mut t argument",
            )
        elif branch_id == "INSTALLED_DEFAULT_RESULT_BORROW_STATE":
            fragments = (
                "installs the t returned by default",
                "default result-provenance relation",
                "never derives from the displaced old t",
            )
        elif branch_id == "PATTERN_SEARCHER_BORROW_STATE":
            fragments = (
                "searcher state containing live borrow leaves",
                "pattern result-provenance relation",
                "never derives from the temporary pattern receiver or call frame",
            )
        else:
            raise ValueError(f"unclassified internal-result pin: {contract_id}/{branch_id}")
        CONDITION_PINS[(contract_id, branch_id)] = fragments
for contract_id in ("VIEW-CLONE-01", "VIEW-FILL-01"):
    CONDITION_PINS[(contract_id, "CLONE_FROM_BORROW_EFFECT")] = (
        "clone_from returns no t",
        "same clone source and destination owners remain valid",
        "surviving leaves keep their roots",
        "every overwritten or otherwise ended leaf ends once",
        "every new leaf is authorized",
        "reused destination allocation or storage grants no provenance",
    )
for branch_id, relation in {
    "OR_INSERT_WITH_VACANT_STORE_KEY_VALUE": "callable result-provenance relation",
    "OR_INSERT_WITH_KEY_VACANT_STORE_KEY_VALUE": "callable result-provenance relation",
    "OR_DEFAULT_VACANT_STORE_KEY_VALUE": "default result-provenance relation",
}.items():
    CONDITION_PINS[("MAP-ENTRY-01", branch_id)] = (
        relation,
        "never derives from",
        "call frame",
    )
for contract_id in STORED_HASH_STATE_CONTRACTS:
    if contract_id == "SET-REL-01":
        fragments = (
            "one or both sets' stored buildhasher states",
            "neither owner or its evolving leaf state grants payload or boolean-result authority",
        )
    elif contract_id == "SET-ALG-02":
        fragments = (
            "lazy hashset algebra cursor",
            "s remains source-owned",
            "grants no yielded-payload provenance",
        )
    elif contract_id == "TRAIT-INDEX-01":
        fragments = (
            "only the hashmap index branch",
            "returned value borrow derives from map storage, never from s",
        )
    elif contract_id == "TRAIT-CMP-01":
        fragments = (
            "only hashmap and hashset equality branches iterate the left operand",
            "right operand's stored buildhasher s through get or contains probes",
            "left s remains retained and unreborrowed",
            "length-mismatch and empty-equal paths perform zero build_hasher calls",
            "each performed right-hand probe creates exactly one generated h",
            "other comparison and caller-hasher hash branches are excluded",
            "neither s nor its evolving leaves grant relation-result or payload authority",
        )
    elif contract_id in {"HMAP-ITER-01", "HSET-ITER-01"}:
        fragments = (
            "only borrowed hash iterators",
            "s is not invoked, moved, or destroyed",
            "never stored in the returned cursor",
            "owning-iterator branch destroys s before returning its cursor state",
        )
    elif contract_id in {"HMAP-DRAIN-01", "HSET-DRAIN-01"}:
        fragments = (
            "drain and clear preserve source-owned buildhasher s",
            "without invoking, moving, or destroying it",
            "no returned drain cursor retains s",
        )
    else:
        fragments = (
            "operation invokes stored buildhasher state containing live borrow leaves",
            "exact eventual destruction",
            "neither old nor new s leaf state grants payload or result provenance",
        )
    if contract_id in HASH_BEHAVIOR_STATE_CONTRACTS:
        fragments = (
            "same stored buildhasher s owner remains valid",
            "declared buildhasher behavior-effect relation",
            "every surviving s leaf keeps its exact root",
            "every newly installed s leaf follows that relation",
            "unique leaf transferred from s into generated h ends in s before it becomes live in h",
            "never simultaneously live in both owners",
        ) + fragments
    CONDITION_PINS[(contract_id, "STORED_HASH_BUILDER_BORROW_STATE")] = fragments
for contract_id in EXISTING_HASH_STATE_CONTRACTS:
    CONDITION_PINS[(contract_id, "EXISTING_HASH_BUILDER_BORROW_STATE")] = (
        "existing",
        "stored buildhasher state contains live borrow leaves",
        "remains valid across the call",
    )
for contract_id in HASH_GUARD_STATE_CONTRACTS:
    CONDITION_PINS[(contract_id, "HASH_GUARD_MAP_HASHER_STATE")] = (
        "only the hashmap",
        "stored buildhasher state containing live borrow leaves",
        "across",
    )
CONDITION_PINS[("MAP-ENTRY-01", "HASH_ENTRY_STORED_HASHER_STATE")] = (
    "only the hashmap entry path",
    "same s owner remains map-owned and valid",
    "declared buildhasher behavior-effect relation",
    "every surviving s leaf keeps its exact root",
    "every newly installed s leaf follows that relation",
    "unique leaf transferred from s into generated h ends in s before it becomes live in h",
    "never simultaneously live in both owners",
    "neither old nor new s leaf state grants payload or result provenance",
)
CONDITION_PINS[("TRAIT-INTOITER-01", "OWNING_HASH_BUILDER_DROP")] = (
    "only owning hashmap and hashset",
    "destroy stored buildhasher state",
    "before returning the payload cursor",
    "no owning cursor retains s",
    "borrowed entrances preserve",
)
for contract_id in ("HMAP-ITER-01", "HSET-ITER-01"):
    CONDITION_PINS[(contract_id, "OWNING_HASH_BUILDER_DROP")] = (
        "destroys stored buildhasher state containing live borrow leaves exactly once",
        "before returning the iterator cursor",
        "no owning cursor retains s",
    )
CONDITION_PINS[("TRAIT-CMP-01", "CALLER_HASHER_BORROW_STATE")] = (
    "only the hash implementation branch",
    "caller-owned mutable hasher h",
    "same h owner remains valid after every normal hasher call",
    "declared hasher behavior-effect relation",
    "every surviving leaf keeps its exact root",
    "every newly installed leaf follows that relation",
    "ended unique leaf is never simultaneously live with its replacement",
    "grants no payload, relation-result, occupancy, uniqueness, or check-elision authority",
)
for contract_id, fragments in {
    "RANGE-BOUND-MAP-01": (
        "included and excluded invoke it exactly once",
        "unbounded invokes it zero times",
        "every normal route destroys the environment exactly once",
    ),
    "RC-CYCLIC-01": (
        "invokes it exactly once on every normal route",
        "destroys it exactly once",
    ),
    "REF-GUARD-01": (
        "invokes it exactly once on every normal route",
        "ref::clone has no environment",
    ),
    "REFCELL-REPLACE-01": (
        "invokes it exactly once on every normal route",
        "replace, take, and swap have no environment",
    ),
}.items():
    CONDITION_PINS[(contract_id, "CALLABLE_ENV_BORROW_STATE")] = fragments
for contract_id, branches in PINNED_EXACT_BRANCHES.items():
    if "BORROW_BEARING_RANGE_DESCRIPTOR" in branches:
        CONDITION_PINS[(contract_id, "BORROW_BEARING_RANGE_DESCRIPTOR")] = (
            "user-defined rangebounds descriptor r containing live borrow leaves",
            "calls rangebounds only while r is live",
            "r is destroyed exactly once",
            "no returned cursor or payload provenance derives from r",
        )


def read_tsv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if reader.fieldnames is None:
            raise ValueError(f"{path}: missing header")
        return reader.fieldnames, list(reader)


def verify(root: Path) -> tuple[list[str], Counter[str], Counter[str]]:
    errors: list[str] = []
    census_header, census_rows = read_tsv(root / "RUST-DATA-CONTRACT-CENSUS.tsv")
    matrix_header, matrix_rows = read_tsv(root / "DERIVATION-MATRIX.tsv")
    registry_header, registry_rows = read_tsv(
        root / "CAPABILITY-OBLIGATION-REGISTRY.tsv"
    )
    classification_header, classification_rows = read_tsv(
        root / "PAYLOAD-SCOPE-CLASSIFICATION.tsv"
    )
    overlay_header, overlay_rows = read_tsv(root / "PAYLOAD-SCOPE-OVERLAY.tsv")

    if not census_header or census_header[0] != "contract_id":
        errors.append("census contract_id key is missing")
    if not matrix_header or matrix_header[0] != "contract_id":
        errors.append("matrix contract_id key is missing")
    if not registry_header or registry_header[0] != "capability_id":
        errors.append("capability registry key is missing")
    if classification_header != CLASSIFICATION_HEADER:
        errors.append(f"classification header differs: {classification_header}")
    if overlay_header != OVERLAY_HEADER:
        errors.append(f"overlay header differs: {overlay_header}")

    census_ids = [row.get("contract_id", "") for row in census_rows]
    matrix_ids = [row.get("contract_id", "") for row in matrix_rows]
    classification_ids = [row.get("contract_id", "") for row in classification_rows]
    registry_ids = [row.get("capability_id", "") for row in registry_rows]
    for name, values in (
        ("census", census_ids),
        ("matrix", matrix_ids),
        ("payload-scope classification", classification_ids),
        ("capability registry", registry_ids),
    ):
        if len(values) != len(set(values)):
            errors.append(f"{name} contains duplicate IDs")
        if "" in values:
            errors.append(f"{name} contains an empty ID")
    if census_ids != matrix_ids:
        errors.append("census and matrix contract order differs")
    if classification_ids != census_ids:
        errors.append("payload-scope classification is not an exact census-order partition")

    census_set = set(census_ids)
    matrix = {row.get("contract_id", ""): row for row in matrix_rows}
    registry_set = set(registry_ids)
    registry_rank = {value: index for index, value in enumerate(registry_ids)}
    classification_rows_by_id = {
        row.get("contract_id", ""): row for row in classification_rows
    }
    classification = {
        contract_id: row.get("stored_borrow_scope", "")
        for contract_id, row in classification_rows_by_id.items()
    }
    classification_counts = Counter(classification.values())
    unknown_classifications = sorted(set(classification_counts) - VALID_CLASSIFICATIONS)
    if unknown_classifications:
        errors.append(f"unknown stored-borrow classifications: {unknown_classifications}")

    overlay_keys: list[tuple[str, str]] = []
    overlay_by_key: dict[tuple[str, str], dict[str, str]] = {}
    actual_by_contract: dict[str, set[str]] = {}
    contracts_by_branch: dict[str, set[str]] = {}
    role_counts: Counter[str] = Counter()
    result_provenance_counts: Counter[str] = Counter()
    return_counts: Counter[str] = Counter()
    overlay_contracts: set[str] = set()
    for row in overlay_rows:
        contract_id = row.get("contract_id", "")
        branch_id = row.get("branch_id", "")
        key = (contract_id, branch_id)
        overlay_keys.append(key)
        if key not in overlay_by_key:
            overlay_by_key[key] = row
        overlay_contracts.add(contract_id)
        actual_by_contract.setdefault(contract_id, set()).add(branch_id)
        contracts_by_branch.setdefault(branch_id, set()).add(contract_id)
        if contract_id not in matrix:
            errors.append(f"{contract_id}/{branch_id}: unknown contract")
            continue
        if not branch_id or branch_id.upper() != branch_id:
            errors.append(f"{contract_id}/{branch_id}: branch_id is not canonical upper case")

        role = row.get("role", "")
        role_counts[role] += 1
        if role not in VALID_ROLES:
            errors.append(f"{contract_id}/{branch_id}: unknown role {role!r}")
        returned = row.get("returns_borrow_bearing_owner", "")
        return_counts[returned] += 1
        if returned not in {"yes", "no"}:
            errors.append(f"{contract_id}/{branch_id}: invalid result marker {returned!r}")
        result_provenance = row.get("requires_result_provenance", "")
        result_provenance_counts[result_provenance] += 1
        if result_provenance not in {"yes", "no"}:
            errors.append(
                f"{contract_id}/{branch_id}: invalid result-provenance marker "
                f"{result_provenance!r}"
            )
        if role == "BORROW_BEARING_RESULT" and returned != "yes":
            errors.append(f"{contract_id}/{branch_id}: result role is not marked returned")
        if returned == "yes" and result_provenance != "yes":
            errors.append(
                f"{contract_id}/{branch_id}: returned borrow-bearing owner lacks "
                "result provenance"
            )
        if role == "BORROW_BEARING_RESULT" and result_provenance != "yes":
            errors.append(
                f"{contract_id}/{branch_id}: result role lacks result provenance"
            )
        if role == "STORED_TRANSITION" and returned != "no":
            errors.append(f"{contract_id}/{branch_id}: stored transition invents a result")
        if row.get("route_scope") != ROUTE_SCOPE:
            errors.append(f"{contract_id}/{branch_id}: route scope differs")
        if row.get("disposition") != DISPOSITION:
            errors.append(f"{contract_id}/{branch_id}: disposition differs")
        if row.get("reopening_trigger") != REOPENING_TRIGGER:
            errors.append(f"{contract_id}/{branch_id}: reopening trigger differs")

        condition = row.get("condition", "")
        lowered = condition.lower()
        if BASE_SCOPE_SENTENCE.lower() not in lowered:
            errors.append(f"{contract_id}/{branch_id}: base-scope condition is incomplete")
        if "borrow" not in lowered:
            errors.append(f"{contract_id}/{branch_id}: condition omits the borrow predicate")
        for forbidden in (
            "copy means borrow-free",
            "'static means borrow-free",
            "cheaply relocatable",
        ):
            if forbidden in lowered:
                errors.append(
                    f"{contract_id}/{branch_id}: forbidden scope synonym {forbidden!r}"
                )

        base = {
            value
            for value in matrix[contract_id].get("capability_ids", "").split(",")
            if value
        }
        desired = {"BR-PROV", "BR-STORED"}
        if result_provenance == "yes":
            desired.add("BR-RESULT")
        missing_registry = sorted(desired - registry_set)
        if missing_registry:
            errors.append(
                f"{contract_id}/{branch_id}: required capabilities absent from registry "
                f"{missing_registry}"
            )
        expected = sorted(
            desired - base,
            key=lambda capability: registry_rank.get(capability, len(registry_rank)),
        )
        raw_actual = row.get("conditional_capability_ids", "")
        actual = raw_actual.split(",") if raw_actual else []
        unknown_conditional = sorted(set(actual) - registry_set)
        if unknown_conditional:
            errors.append(
                f"{contract_id}/{branch_id}: unknown conditional capabilities "
                f"{unknown_conditional}"
            )
        if len(actual) != len(set(actual)):
            errors.append(f"{contract_id}/{branch_id}: duplicate conditional capabilities")
        if actual != expected:
            errors.append(
                f"{contract_id}/{branch_id}: conditional capabilities {actual} differ "
                f"from {expected}"
            )
        if not expected:
            errors.append(
                f"{contract_id}/{branch_id}: conditional capability difference is empty"
            )
        if "BR-STORED" in base:
            errors.append(f"{contract_id}/{branch_id}: active BR-STORED row duplicated in overlay")
        if classification.get(contract_id) != "DEFERRED_BRANCHES":
            errors.append(f"{contract_id}/{branch_id}: classification is not DEFERRED_BRANCHES")

    if len(overlay_keys) != len(set(overlay_keys)):
        errors.append("payload-scope overlay contains duplicate branch keys")

    # Scope-owner policy is checked independently of aggregate state counts. Normal
    # rows have no indirection; evidence and frame rows are self-owned; the sole
    # delegated row has a closed, ordered, unique set of ordinary-family owners.
    owner_claims: dict[str, str] = {}
    for contract_id, row in classification_rows_by_id.items():
        scope = row.get("stored_borrow_scope", "")
        raw_owners = row.get("scope_owner_contract_ids", "")
        rationale = row.get("rationale", "").lower()
        if scope in NORMAL_CLASSIFICATIONS:
            if raw_owners != "NONE":
                errors.append(f"{contract_id}: normal classification owner is not NONE")
            owners: tuple[str, ...] = ()
        elif raw_owners in {"", "NONE"}:
            errors.append(f"{contract_id}: non-normal classification lacks scope owner")
            owners = ()
        else:
            owners = tuple(raw_owners.split(";"))
            if any(not owner for owner in owners):
                errors.append(f"{contract_id}: scope owner list contains an empty ID")
            if len(owners) != len(set(owners)):
                errors.append(f"{contract_id}: scope owner list contains duplicate IDs")
            unknown_owners = sorted(set(owners) - census_set)
            if unknown_owners:
                errors.append(f"{contract_id}: unknown scope owner IDs {unknown_owners}")
            for owner in owners:
                prior = owner_claims.setdefault(owner, contract_id)
                if prior != contract_id:
                    errors.append(
                        f"{contract_id}: scope owner {owner} is already claimed by {prior}"
                    )

        capabilities = {
            value
            for value in matrix.get(contract_id, {}).get("capability_ids", "").split(",")
            if value
        }
        status = matrix.get(contract_id, {}).get("status_code", "")
        if scope == "ACTIVE_BR_STORED":
            if not {"BR-PROV", "BR-STORED"} <= capabilities:
                errors.append(f"{contract_id}: ACTIVE_BR_STORED lacks BR-PROV/BR-STORED")
            if contract_id not in ACTIVE_RESULT_EXCEPTION and "BR-RESULT" not in capabilities:
                errors.append(f"{contract_id}: active borrow-bearing result lacks BR-RESULT")
            if contract_id in overlay_contracts:
                errors.append(f"{contract_id}: active row also has deferred overlay branches")
        elif scope == "DEFERRED_BRANCHES":
            if contract_id not in overlay_contracts:
                errors.append(f"{contract_id}: deferred classification has no overlay branch")
            if status in {"E", "P"}:
                errors.append(
                    f"{contract_id}: unrestricted E/P claim is forbidden while a "
                    "stored-borrow branch is deferred"
                )
        elif scope == "NO_STORED_BORROW_COMPLEMENT":
            if contract_id in overlay_contracts:
                errors.append(f"{contract_id}: no-complement row has an overlay branch")
            if "BR-STORED" in capabilities:
                errors.append(f"{contract_id}: no-complement row carries active BR-STORED")
            if contract_id in {
                "RANGE-ITER-HALFOPEN-01",
                "RANGE-ITER-FROM-01",
                "RANGE-ITER-INCLUSIVE-01",
            }:
                for fragment in (
                    "exact 21-type standard step set",
                    "step and trustedstep are unstable",
                    "every listed type is borrow-free copy",
                    "not from shared-receiver purity",
                    "ae3f9307f4b4972f418561ae2a0311586eb3dde782359b8aaef3244915256464",
                ):
                    if fragment not in rationale:
                        errors.append(
                            f"{contract_id}: sealed Step no-complement rationale lost "
                            f"{fragment!r}"
                        )
        elif scope == "BOUNDARY_EVIDENCE_ONLY":
            if owners != (contract_id,):
                errors.append(f"{contract_id}: boundary evidence is not self-owned")
            if contract_id in overlay_contracts:
                errors.append(f"{contract_id}: boundary evidence gained a derivation branch")
            if status != "BOUNDARY":
                errors.append(f"{contract_id}: boundary classification has status {status}")
            for fragment in ("boundary evidence", "no derivation complement"):
                if fragment not in rationale:
                    errors.append(
                        f"{contract_id}: boundary rationale omits policy fragment {fragment!r}"
                    )
        elif scope == "FRAME_SCOPE_DEFERRED":
            if owners != (contract_id,):
                errors.append(f"{contract_id}: frame scope is not self-owned")
            if contract_id in overlay_contracts:
                errors.append(f"{contract_id}: frame scope gained an ordinary overlay branch")
            if status != "FRAME":
                errors.append(f"{contract_id}: frame classification has status {status}")
            for fragment in (
                "frame/abi authority",
                "no ordinary derivation complement",
                "cannot close a safe-library capability route",
            ):
                if fragment not in rationale:
                    errors.append(
                        f"{contract_id}: frame rationale omits policy fragment {fragment!r}"
                    )
        elif scope == "DELEGATED_TO_FAMILY_BRANCHES":
            if contract_id != DELEGATED_CONTRACT_ID:
                errors.append(f"{contract_id}: unexpected delegated classification")
            if owners != DELEGATED_OWNER_IDS:
                errors.append(
                    f"{contract_id}: delegated owners {owners} differ from "
                    f"{DELEGATED_OWNER_IDS}"
                )
            if contract_id in overlay_contracts:
                errors.append(f"{contract_id}: delegated row gained an independent branch")
            if "BR-STORED" in capabilities:
                errors.append(f"{contract_id}: delegated row gained active BR-STORED")
            if status in {"E", "P"}:
                errors.append(
                    f"{contract_id}: delegated scope cannot carry unrestricted {status}"
                )
            if "no independent payload operation" not in rationale:
                errors.append(f"{contract_id}: delegated rationale invents an independent route")

    boundary_actual = {
        contract_id
        for contract_id, scope in classification.items()
        if scope == "BOUNDARY_EVIDENCE_ONLY"
    }
    if boundary_actual != BOUNDARY_IDS:
        errors.append(
            f"boundary evidence IDs {sorted(boundary_actual)} differ from "
            f"{sorted(BOUNDARY_IDS)}"
        )
    matrix_boundary_ids = {
        contract_id
        for contract_id, row in matrix.items()
        if row.get("status_code") == "BOUNDARY"
    }
    if matrix_boundary_ids != BOUNDARY_IDS:
        errors.append(
            f"matrix boundary IDs {sorted(matrix_boundary_ids)} differ from "
            f"{sorted(BOUNDARY_IDS)}"
        )
    frame_actual = {
        contract_id
        for contract_id, scope in classification.items()
        if scope == "FRAME_SCOPE_DEFERRED"
    }
    if frame_actual != FRAME_SCOPE_IDS:
        errors.append(
            f"frame-scope IDs {sorted(frame_actual)} differ from {sorted(FRAME_SCOPE_IDS)}"
        )
    delegated_actual = {
        contract_id
        for contract_id, scope in classification.items()
        if scope == "DELEGATED_TO_FAMILY_BRANCHES"
    }
    if delegated_actual != {DELEGATED_CONTRACT_ID}:
        errors.append(
            f"delegated IDs {sorted(delegated_actual)} differ from "
            f"{[DELEGATED_CONTRACT_ID]}"
        )
    for owner, expected_state in DELEGATED_OWNER_STATES.items():
        actual_state = classification.get(owner)
        if actual_state != expected_state:
            errors.append(
                f"{DELEGATED_CONTRACT_ID}: owner {owner} state {actual_state!r} differs "
                f"from {expected_state!r}"
            )

    active_from_matrix = {
        contract_id
        for contract_id, row in matrix.items()
        if "BR-STORED" in row.get("capability_ids", "").split(",")
    }
    active_from_classification = {
        contract_id
        for contract_id, scope in classification.items()
        if scope == "ACTIVE_BR_STORED"
    }
    if active_from_classification != active_from_matrix:
        errors.append(
            "ACTIVE_BR_STORED is not the exact matrix BR-STORED set: "
            f"missing={sorted(active_from_matrix - active_from_classification)}, "
            f"extra={sorted(active_from_classification - active_from_matrix)}"
        )

    for contract_id in PINNED_NO_COMPLEMENT:
        if classification.get(contract_id) != "NO_STORED_BORROW_COMPLEMENT":
            errors.append(f"{contract_id}: pinned no-complement classification differs")
    for contract_id in ACTIVE_RANGE_BOUNDS_STATE_CONTRACTS:
        if classification.get(contract_id) != "ACTIVE_BR_STORED":
            errors.append(f"{contract_id}: retained RangeBounds state is not active BR-STORED")
    for contract_id in BASE_RANGE_BOUNDS_ONLY_CONTRACTS:
        if classification.get(contract_id) != "NO_STORED_BORROW_COMPLEMENT":
            errors.append(f"{contract_id}: base-only RangeBounds row gained an overlay")
        if contract_id in actual_by_contract:
            errors.append(f"{contract_id}: pinned no-complement contract gained a branch")

    # Exact semantic partitions are independent of generation counts: a missing
    # branch cannot be hidden by adding an unrelated row elsewhere.
    for contract_id, expected_branches in PINNED_EXACT_BRANCHES.items():
        actual_branches = actual_by_contract.get(contract_id, set())
        if actual_branches != expected_branches:
            errors.append(
                f"{contract_id}: exact branches {sorted(actual_branches)} differ from "
                f"{sorted(expected_branches)}"
            )
    for key, shape in PINNED_ROW_SHAPES.items():
        row = overlay_by_key.get(key)
        if row is None:
            continue
        if (
            row.get("role") != shape.role
            or row.get("returns_borrow_bearing_owner") != shape.returned
            or row.get("requires_result_provenance") != shape.result_provenance
        ):
            errors.append(
                f"{key[0]}/{key[1]}: role/public-result/provenance shape "
                f"{(row.get('role'), row.get('returns_borrow_bearing_owner'), row.get('requires_result_provenance'))} differs "
                f"from {(shape.role, shape.returned, shape.result_provenance)}"
            )

    exact_carriers = {
        "CALLABLE_ENV_BORROW_STATE": CALLABLE_STATE_CONTRACTS,
        "KEY_RESULT_BORROW_STATE": KEY_STATE_CONTRACTS,
        "CACHED_KEY_BORROW_STATE": {"VIEW-SORT-01"},
        "STORED_HASH_BUILDER_BORROW_STATE": STORED_HASH_STATE_CONTRACTS,
        "EXISTING_HASH_BUILDER_BORROW_STATE": EXISTING_HASH_STATE_CONTRACTS,
        "HASH_GUARD_MAP_HASHER_STATE": HASH_GUARD_STATE_CONTRACTS,
        "HASH_ENTRY_STORED_HASHER_STATE": {"MAP-ENTRY-01"},
        "CALLER_HASHER_BORROW_STATE": CALLER_HASHER_STATE_CONTRACTS,
        "GENERATED_HASHER_BORROW_STATE": GENERATED_HASHER_RESULT_CONTRACTS,
        "CLONE_RESULT_BORROW_STATE": {
            "INIT-WRITE-01",
            "SEQ-EXTEND-COPY-01",
            "SEQ-RESIZE-01",
            "DEQUE-RESIZE-01",
        },
        "PRODUCER_RESULT_BORROW_STATE": {
            "VIEW-FILL-01",
            "SEQ-RESIZE-01",
            "DEQUE-RESIZE-01",
        },
        "PATTERN_SEARCHER_BORROW_STATE": {
            "TEXT-SEARCH-01",
            "TEXT-TRIM-01",
            "TEXT-REPLACE-01",
        },
        "MAKE_MUT_CLONE_RESULT_BORROW_STATE": {"RC-UNIQUE-01"},
        "INSTALLED_CALLBACK_RESULT_BORROW_STATE": {"REFCELL-REPLACE-01"},
        "INSTALLED_DEFAULT_RESULT_BORROW_STATE": {
            "REFCELL-REPLACE-01",
            "MEM-TAKE-01",
        },
        "BORROW_BEARING_RANGE_DESCRIPTOR": {
            *RANGE_BOUNDS_OVERLAY_CONTRACTS,
        },
    }
    for branch_id, expected_contracts in exact_carriers.items():
        actual_contracts = contracts_by_branch.get(branch_id, set())
        if actual_contracts != expected_contracts:
            errors.append(
                f"{branch_id}: carrier set {sorted(actual_contracts)} differs from "
                f"{sorted(expected_contracts)}"
            )

    for key, fragments in CONDITION_PINS.items():
        row = overlay_by_key.get(key)
        if row is None:
            continue
        condition = row.get("condition", "").lower()
        for fragment in fragments:
            if fragment.lower() not in condition:
                errors.append(
                    f"{key[0]}/{key[1]}: condition omits semantic pin {fragment!r}"
                )
    missing_clone_effect_keys = CLONE_EFFECT_OVERLAY_KEYS - set(overlay_by_key)
    if missing_clone_effect_keys:
        errors.append(
            "Clone source-effect overlay keys are missing: "
            f"{sorted(missing_clone_effect_keys)}"
        )
    clone_from_keys = {
        ("VIEW-CLONE-01", "CLONE_FROM_BORROW_EFFECT"),
        ("VIEW-FILL-01", "CLONE_FROM_BORROW_EFFECT"),
        ("TRAIT-CLONE-01", "CLONE_FROM_OWNED_PAYLOAD"),
    }
    for key in sorted(CLONE_EFFECT_OVERLAY_KEYS & set(overlay_by_key)):
        condition = overlay_by_key[key].get("condition", "").lower()
        fragments = (
            (
                "same clone source and destination owners remain valid",
                "post-call source and destination leaves jointly follow that relation",
                "surviving leaves keep their roots",
                "every overwritten or otherwise ended leaf ends once",
                "reused destination allocation or storage grants no provenance",
            )
            if key in clone_from_keys
            else (
                "same clone source owner remains valid on normal return",
                "post-call source leaves and result leaves jointly follow",
                "surviving source leaves keep their roots",
                "every ended leaf ends once",
                "unique leaf transferred to the result ends in the source before result liveness",
            )
        )
        for fragment in fragments:
            if fragment not in condition:
                errors.append(
                    f"{key[0]}/{key[1]}: Clone source-effect condition omits {fragment!r}"
                )
    for contract_id in {
        "RANGE-BOUND-MAP-01",
        "RC-CYCLIC-01",
        "REF-GUARD-01",
        "REFCELL-REPLACE-01",
    }:
        condition = overlay_by_key.get(
            (contract_id, "CALLABLE_ENV_BORROW_STATE"), {}
        ).get("condition", "").lower()
        for forbidden in ("zero-or-more invocation path", "including a not-invoked branch"):
            if forbidden in condition:
                errors.append(
                    f"{contract_id}/CALLABLE_ENV_BORROW_STATE: exact call partition "
                    f"contains overbroad phrase {forbidden!r}"
                )
    for key, row in overlay_by_key.items():
        if (
            key[0] == "TRAIT-CLONE-01"
            and key[1] != "CLONE_CACHED_BORROW_BEARING_STATE"
            and "source-map-only" in row.get("condition", "").lower()
        ):
            errors.append(f"{key[0]}/{key[1]}: source-map-only exclusion leaked branches")

    # AB-STATEFUL cannot float without a stored-borrow proof. It is either already
    # active in the base row or has the exact callable/hasher complement below.
    for contract_id in (
        CALLABLE_STATE_CONTRACTS
        | HASH_BEHAVIOR_STATE_CONTRACTS
        | CALLER_HASHER_STATE_CONTRACTS
        | {"TRAIT-INTOITER-01"}
    ):
        capabilities = set(matrix[contract_id].get("capability_ids", "").split(","))
        if "AB-STATEFUL" not in capabilities:
            errors.append(
                f"{contract_id}: adjudicated callable/hasher state lacks AB-STATEFUL"
            )
    ab_required: dict[str, set[str]] = {
        contract_id: {"CALLABLE_ENV_BORROW_STATE"}
        for contract_id in CALLABLE_STATE_CONTRACTS
    }
    for contract_id in STORED_HASH_STATE_CONTRACTS:
        if "AB-STATEFUL" in matrix[contract_id].get("capability_ids", "").split(","):
            ab_required.setdefault(contract_id, set()).add(
                "STORED_HASH_BUILDER_BORROW_STATE"
            )
    ab_required["MAP-ENTRY-01"] = {
        "AND_MODIFY_CALLABLE_ENV",
        "OR_INSERT_WITH_CALLABLE_ENV",
        "OR_INSERT_WITH_KEY_CALLABLE_ENV",
        "HASH_ENTRY_STORED_HASHER_STATE",
    }
    ab_required["TRAIT-CMP-01"] = {
        "CALLER_HASHER_BORROW_STATE",
        "STORED_HASH_BUILDER_BORROW_STATE",
    }
    ab_required["TRAIT-INTOITER-01"] = {"OWNING_HASH_BUILDER_DROP"}
    for contract_id, row in matrix.items():
        capabilities = set(row.get("capability_ids", "").split(","))
        if "AB-STATEFUL" not in capabilities or "BR-STORED" in capabilities:
            continue
        required = ab_required.get(contract_id)
        if required is None:
            errors.append(
                f"{contract_id}: AB-STATEFUL lacks active BR-STORED and an adjudicated "
                "callable/hasher overlay"
            )
            continue
        missing = required - actual_by_contract.get(contract_id, set())
        if missing:
            errors.append(
                f"{contract_id}: AB-STATEFUL overlay lacks {sorted(missing)}"
            )

    overlay_key_set = set(overlay_keys)
    for key in PINNED_BULK_STORED:
        if key not in overlay_key_set:
            errors.append(f"missing pinned bulk stored transition {key}")
        elif overlay_by_key[key].get("role") != "STORED_TRANSITION":
            errors.append(f"{key}: pinned bulk stored role differs")
    for key in PINNED_BULK_RESULT:
        if key not in overlay_key_set:
            errors.append(f"missing pinned bulk result transition {key}")
        elif overlay_by_key[key].get("role") != "BORROW_BEARING_RESULT":
            errors.append(f"{key}: pinned bulk result role differs")

    # Aggregate pins detect accidental generator drift, but all high-risk semantics
    # above are checked by identities, exact branch partitions, and condition facts.
    if classification_counts != EXPECTED_CLASSIFICATION_COUNTS:
        errors.append(
            "stored-borrow classification counts differ: "
            f"{dict(classification_counts)} != {dict(EXPECTED_CLASSIFICATION_COUNTS)}"
        )
    if len(overlay_rows) != EXPECTED_OVERLAY_ROWS:
        errors.append(
            f"payload-scope overlay has {len(overlay_rows)} rows, "
            f"expected {EXPECTED_OVERLAY_ROWS}"
        )
    if role_counts != EXPECTED_ROLE_COUNTS:
        errors.append(
            f"payload-scope role counts differ: {dict(role_counts)} != "
            f"{dict(EXPECTED_ROLE_COUNTS)}"
        )
    if result_provenance_counts != EXPECTED_RESULT_PROVENANCE_COUNTS:
        errors.append(
            "payload-scope result-provenance marker counts differ: "
            f"{dict(result_provenance_counts)} != "
            f"{dict(EXPECTED_RESULT_PROVENANCE_COUNTS)}"
        )
    if return_counts != EXPECTED_RETURN_COUNTS:
        errors.append(
            "payload-scope public-return marker counts differ: "
            f"{dict(return_counts)} != {dict(EXPECTED_RETURN_COUNTS)}"
        )

    return errors, classification_counts, role_counts


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="minimal-systems-capability artifact directory",
    )
    args = parser.parse_args()
    errors, classification_counts, role_counts = verify(args.root)
    if errors:
        for error in errors:
            print(f"payload-scope overlay: FAIL: {error}")
        return 1
    print(
        "payload-scope overlay: PASS — "
        + ", ".join(f"{key}={classification_counts[key]}" for key in sorted(classification_counts))
        + "; branches "
        + ", ".join(f"{key}={role_counts[key]}" for key in sorted(role_counts))
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
