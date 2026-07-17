#!/usr/bin/env python3
"""Machine-closed soundness contracts for the dense Family Lock A draft.

This module refines the coarse outcome unions in the draft ledger.  It is a
research and protocol artifact only: it describes obligations for hypothetical
candidates and never constructs, selects, scores, or executes one.
"""

from __future__ import annotations

import csv
import hashlib
import importlib.util
import io
import itertools
import json
import argparse
from collections import defaultdict
from pathlib import Path
from typing import Iterable

from dense_meta5 import META5_ROWS, cumulative_meta5_ids, validate_meta5_rows
from dense_owner_decisions import OWNER_DECISIONS


HERE = Path(__file__).resolve().parent
CLOSED_COVERAGE_REGISTRY_PATH = HERE / "dense_coverage_closed_registry.py"
LITERAL_REGISTRY_LOADER_PATH = HERE / "dense_literal_registry.py"
LITERAL_REGISTRY_LOADER_SHA256 = "a8eb255184ebf560f2fcd5eab659405b08185431a224cee69bfca9e32233cdc2"
# Updated only with an independently reviewed closed-coverage registry change.
CLOSED_COVERAGE_REGISTRY_SHA256 = "84bc687641746607ba3798b8cf419f427ef4a4fe7b3a402e377287804f1024a3"
COVERAGE_OUTPUT_SHA256 = {
    "DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv": "34b1e14f611b7eee1f069bf9dbf31b0beab706fcb1dfa70b807e692b9dd53e2d",
    "DENSE-OVERLAY-BRANCH-AUTHORITY.tsv": "2d1d737501c75903fe564e46af334700703a61a00c1e7f1fac8f0058c4638738",
    "DENSE-EVIDENCE-TARGET-AUTHORITY.tsv": "687aecfaa3ef122ee1a9270a97a4ab614e4693d62a32eb69e93a9a33ad56866e",
    "DENSE-CAPABILITY-UNIT-AUTHORITY.tsv": "e58a922be6b9c917c69f813b29362d0cfed8e94b9a4709ae33a7d3ecaa774dd0",
}
CANDIDATE_IDS = (
    "C-ATOMIC-TRANSITIONS",
    "C-DERIVED-REPAIR",
    "C-LINEAR-REBUILD",
    "C-PROOF-CARRYING-STATE",
    "C-RUNTIME-TOPOLOGY",
)


def _load_shared_literal_loader():
    data = LITERAL_REGISTRY_LOADER_PATH.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != LITERAL_REGISTRY_LOADER_SHA256:
        raise ValueError(
            "shared literal-registry loader SHA-256 mismatch: "
            f"expected {LITERAL_REGISTRY_LOADER_SHA256}, got {digest}"
        )
    spec = importlib.util.spec_from_file_location(
        "dense_literal_registry_for_soundness", LITERAL_REGISTRY_LOADER_PATH
    )
    if spec is None or spec.loader is None:
        raise ValueError("cannot load SHA-locked literal-registry loader")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.load_literal_assignments


load_literal_assignments = _load_shared_literal_loader()


def _load_closed_coverage_registry(
    path: Path = CLOSED_COVERAGE_REGISTRY_PATH,
    expected_sha256: str = CLOSED_COVERAGE_REGISTRY_SHA256,
) -> dict[str, object]:
    required = {
        "SCHEMA_VERSION",
        "CLUSTER_MEMBERS",
        "EXCLUDED_MEMBERS",
        "PROTOCOL_SYNTHETIC_MEMBERS",
        "DIRECT_ROUTE_CLASSES",
        "DIRECT_EVIDENCE_ASSIGNMENTS",
        "SELECTOR_CHILD_ASSIGNMENTS",
    }
    return load_literal_assignments(path, expected_sha256, required)


_CLOSED_COVERAGE = _load_closed_coverage_registry()
CLUSTER_MEMBERS: dict[str, tuple[str, ...]] = _CLOSED_COVERAGE["CLUSTER_MEMBERS"]  # type: ignore[assignment]
EXCLUDED_MEMBERS: dict[str, str] = _CLOSED_COVERAGE["EXCLUDED_MEMBERS"]  # type: ignore[assignment]
CLOSED_PROTOCOL_SYNTHETIC_MEMBERS = frozenset(
    _CLOSED_COVERAGE["PROTOCOL_SYNTHETIC_MEMBERS"]  # type: ignore[arg-type]
)
SCHEMA_VERSION = "dense-contract-registry-v1"
COMMON_POLICY = "COMMON-NON-OD1"
OD1_RESERVE_FIRST = "OD-1-RESERVE-FIRST"
OD1_RECOVERABLE = "OD-1-RECOVERABLE-MUTATORS"
OD3_INCLUDE_ZST = "OD-3-INCLUDE-ZST"
OD3_DEFER_ZST = "OD-3-DEFER-ZST"
OD0_COMMON_SUBSTRATE = "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE"
OD0_SEPARATE_LOCKS = "OD-0-SEPARATE-PREREQUISITE-LOCKS"
OD4_EAGER_SCOPED = "OD-4-EAGER-AND-SCOPED-CONSUME"
OD4_EAGER_ONLY = "OD-4-EAGER-ONLY"
OD4_PROMOTE_LAZY = "OD-4-PROMOTE-LAZY"

CONTRACT_OUTPUT = HERE / "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv"
OWNER_ROLE_OUTPUT = HERE / "DENSE-OWNER-ROLE-REGISTRY.tsv"
COMMON_SUBSTRATE_OUTPUT = HERE / "DENSE-COMMON-SUBSTRATE-REGISTRY.tsv"
STORED_BORROW_OUTPUT = HERE / "DENSE-STORED-BORROW-ROUTE-REGISTRY.tsv"
OD4_OUTPUT = HERE / "DENSE-OD4-POLICY-REGISTRY.tsv"
OD1_OUTPUT = HERE / "DENSE-OD1-POLICY-REGISTRY.tsv"
LIFECYCLE_OUTPUT = HERE / "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv"
OPERATIONS_OUTPUT = HERE / "DENSE-CANDIDATE-OPERATION-REGISTRY.tsv"
BINDINGS_OUTPUT = HERE / "DENSE-CANDIDATE-CONTRACT-BINDINGS.tsv"
DISTINCTIONS_OUTPUT = HERE / "DENSE-CANDIDATE-DISTINCTION-REGISTRY.tsv"
ZST_OUTPUT = HERE / "DENSE-ZST-POLICY-REGISTRY.tsv"
FACT_OUTPUT = HERE / "DENSE-FACT-CHANNEL-REGISTRY.tsv"
SYNTHETIC_OUTPUT = HERE / "DENSE-SYNTHETIC-UNIT-REGISTRY.tsv"


CONTRACT_FIELDS = (
    "schema_version",
    "contract_id",
    "member_contract_id",
    "outcome_id",
    "cluster_id",
    "policy_variant_id",
    "profile_id",
    "status",
    "evidence_identity_ids",
    "trigger",
    "pre_state",
    "offered_owners",
    "behavior_call_count_order_effects",
    "commitment_point",
    "post_state",
    "result_owners",
    "returned_owners",
    "retained_owners",
    "destroyed_owners",
    "allocation_disposition",
    "borrow_invalidation",
    "fact_invalidation",
    "normal_exit_cleanup",
    "pre_abort_invariant",
    "resource_ceiling",
    "capability_ids",
    "payload_branch_ids",
    "scenario_ids",
    "commit_phase",
    "owners_before",
    "owners_after",
    "allocation_before",
    "allocation_after",
    "borrows_before",
    "borrows_after",
    "facts_before",
    "facts_after",
    "behavior_calls",
    "result_schema",
    "state_equation",
    "member_declaration_sha256",
    "owner_role_foreign_key",
    "stored_borrow_route_ids",
    "od4_policy_options",
    "zst_policy_foreign_key",
    "candidate_execution_authorized",
)

OWNER_ROLE_FIELDS = (
    "schema_version",
    "owner_role_id",
    "contract_id",
    "member_contract_id",
    "outcome_id",
    "transition_semantics_id",
    "before_owner_roles",
    "after_owner_roles",
    "owner_universe_equation",
    "normal_result_owner_equation",
    "candidate_execution_authorized",
)

COMMON_SUBSTRATE_FIELDS = (
    "schema_version",
    "policy_variant_id",
    "status",
    "sealing_contract",
    "selected_existing_contracts",
    "stateful_behavior_contract",
    "checked_allocation_contract",
    "forbidden_authority",
    "candidate_binding_rule",
    "no_tax_gate",
    "ordinary_library_closure",
    "owner_decision_status",
)

STORED_BORROW_FIELDS = (
    "schema_version",
    "route_id",
    "cluster_id",
    "member_contract_id",
    "stored_state_owner",
    "root_leaf_schema",
    "construction_transition",
    "move_transition",
    "call_transition",
    "normal_result_transition",
    "destruction_transition",
    "failure_transition",
    "region_free_zero_tax",
    "negative_trace_ids",
    "authorization_status",
)

OD4_FIELDS = (
    "schema_version",
    "policy_variant_id",
    "status",
    "mandatory_operations",
    "space_contract",
    "call_order_contract",
    "normal_exit_contract",
    "escape_contract",
    "allocation_contract",
    "claim_boundary",
    "reopening_rule",
    "owner_decision_status",
)

OD1_FIELDS = (
    "schema_version",
    "policy_variant_id",
    "member_contract_id",
    "member_class",
    "arithmetic_failure",
    "allocation_failure",
    "base_owner_result",
    "offered_owner_result",
    "last_recoverable_point",
    "first_destructive_commit",
    "default_path_result_branch",
    "unknown_length_rule",
    "status",
    "owner_decision_status",
)

LIFECYCLE_FIELDS = (
    "schema_version",
    "candidate_id",
    "meta5_delta_id",
    "cumulative_meta5_delta_ids",
    "lifecycle_class",
    "partial_state_schema",
    "master_allocation_authority",
    "allocation_release_authority",
    "incomplete_normal_exit",
    "automatic_normal_exit_action",
    "trap_action",
    "helper_rule",
    "callback_rule",
    "capture_rule",
    "escape_rule",
    "drop_rule",
    "maximum_live_ranges",
    "runtime_partial_state",
    "owning_cursor_shape",
    "owning_cursor_closure",
    "construction_authorized",
)

OPERATION_FIELDS = (
    "schema_version",
    "candidate_id",
    "meta5_delta_id",
    "cumulative_meta5_delta_ids",
    "operation_id",
    "signature",
    "precondition",
    "owner_result",
    "allocation_result",
    "borrow_result",
    "fact_result",
    "normal_exit_effect",
    "authorization_status",
)

BINDING_FIELDS = (
    "schema_version",
    "candidate_id",
    "policy_variant_id",
    "contract_id",
    "binding_kind",
    "lifecycle_class",
    "operation_id",
    "owner_result_source",
    "common_substrate_policy_options",
    "candidate_specific_semantics_allowed",
    "zst_policy_options",
    "construction_authorized",
)

DISTINCTION_FIELDS = (
    "schema_version",
    "left_candidate_id",
    "right_candidate_id",
    "distinguishing_axis",
    "left_required_property",
    "right_required_property",
    "collapse_rule",
    "construction_authorized",
)

ZST_FIELDS = (
    "schema_version",
    "policy_variant_id",
    "status",
    "payload_size",
    "payload_alignment",
    "logical_capacity",
    "payload_allocation",
    "growth_allocation",
    "length_overflow",
    "owner_identity",
    "borrow_footprint",
    "disjointness_rule",
    "pointer_rule",
    "move_rule",
    "drop_rule",
    "allocator_failure_applicability",
    "claim_boundary",
    "owner_decision_status",
)

FACT_FIELDS = (
    "schema_version",
    "fact_id",
    "fact_schema_version",
    "exact_proposition",
    "owning_root",
    "producer",
    "preconditions",
    "scope_and_version",
    "consumers",
    "invalidators",
    "move_transfer",
    "borrow_transfer",
    "call_transfer",
    "branch_join",
    "speculation_rule",
    "facts_off_semantics",
    "artifact_evidence",
    "negative_trace_ids",
    "candidate_realization_rule",
    "authorization_status",
)

SYNTHETIC_FIELDS = (
    "schema_version",
    "synthetic_identity",
    "cluster_id",
    "member_contract_id",
    "synthetic_class",
    "rationale",
    "source_authority",
    "permitted_contract_status",
    "candidate_execution_authorized",
)


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _ordered_digest(values: Iterable[str]) -> str:
    return _sha256("\n".join(values).encode("utf-8"))


def _write_tsv(path: Path, fields: tuple[str, ...], rows: list[dict[str, str]]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, delimiter="\t", fieldnames=fields, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def _read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def _read_pinned_coverage_tsv(
    path: Path,
    expected_sha256: str | None = None,
) -> list[dict[str, str]]:
    """Digest-check a consumed coverage output before parsing the same bytes."""
    expected = expected_sha256 or COVERAGE_OUTPUT_SHA256.get(path.name)
    if expected is None:
        raise ValueError(f"no reviewed coverage-output SHA-256 for {path.name}")
    data = path.read_bytes()
    actual = _sha256(data)
    if actual != expected:
        raise ValueError(
            f"coverage-output SHA-256 mismatch for {path.name}: "
            f"expected {expected}, got {actual}"
        )
    return list(csv.DictReader(io.StringIO(data.decode("utf-8")), delimiter="\t"))


def _validate_pinned_coverage_dependencies() -> None:
    for name, expected_sha256 in COVERAGE_OUTPUT_SHA256.items():
        rows = _read_pinned_coverage_tsv(HERE / name, expected_sha256)
        if not rows:
            raise ValueError(f"pinned coverage dependency is empty: {name}")


def _decision_option(decision_id: str, option_id: str) -> dict[str, object]:
    for decision in OWNER_DECISIONS:
        if decision["decision_id"] != decision_id:
            continue
        for option in decision["options"]:
            if option["option_id"] == option_id:
                return option
    raise ValueError(f"missing owner-decision option: {decision_id}/{option_id}")


def _all_members() -> set[str]:
    return {member for members in CLUSTER_MEMBERS.values() for member in members}


# Profiles are assigned by explicit member identity below.  No name, cluster,
# or prose heuristic may silently change an operation's outcome partition.
PROFILE_MEMBERS: dict[str, tuple[str, ...]] = {
    "EXCLUDED_SURFACE": tuple(sorted(EXCLUDED_MEMBERS)),
    "TOTAL": (
        "DENSE-DEFAULT", "DENSE-INTO-OWNER", "DENSE-META", "DENSE-NEW",
        "DENSE-OWNER-VIEW", "DENSE-REVERSE", "DENSE-REPLACE", "DENSE-VIEW-META",
    ),
    "TOTAL_BORROW": ("DENSE-FIXED-EACH", "DENSE-FIXED-VIEW"),
    "OPTION_BORROW": (
        "DENSE-VIEW-AS-FIXED", "DENSE-VIEW-END", "DENSE-VIEW-END-CHUNK",
        "DENSE-VIEW-END-SPLIT", "DENSE-VIEW-GET-SHARED", "DENSE-VIEW-GET-UNIQ",
    ),
    "CHECKED_BORROW": (
        "DENSE-VIEW-CONSUME-SPLIT", "DENSE-VIEW-DISJOINT-UNIQ",
        "DENSE-VIEW-SPLIT-CHECKED",
    ),
    "TRAPPING_BORROW": (
        "DENSE-INDEX-SHARED", "DENSE-INDEX-UNIQ", "DENSE-VIEW-ARRAY-CHUNKS",
        "DENSE-VIEW-SPLIT-TRAP",
    ),
    "BEHAVIOR": (
        "DENSE-COMPARE", "DENSE-DEDUP", "DENSE-DEDUP-BY", "DENSE-DEDUP-BY-KEY",
        "DENSE-EAGER-EXTRACT", "DENSE-FILL-CLONE", "DENSE-FILL-WITH",
        "DENSE-FIXED-MAP", "DENSE-HASH-TRAVERSAL", "DENSE-RETAIN",
        "DENSE-RETAIN-MUT", "DENSE-TAKE-WITH-DEFAULT",
    ),
    "CHECKED_MUTATION": (
        "DENSE-COPY-FROM", "DENSE-COPY-WITHIN", "DENSE-ROTATE", "DENSE-SWAP",
        "DENSE-SWAP-WITH-VIEW", "DENSE-INTO-FLATTENED",
    ),
    "STABLE_SORT": ("DENSE-SORT-STABLE", "DENSE-SORT-STABLE-CACHED-KEY"),
    "UNSTABLE_SORT": ("DENSE-SORT-UNSTABLE",),
    "SELECT": ("DENSE-SELECT-UNSTABLE",),
    "CLONE_FROM": ("DENSE-CLONE-FROM",),
    "ALLOCATING_BEHAVIOR": (
        "DENSE-CONCAT", "DENSE-CONVERT", "DENSE-FRESH-CLONE", "DENSE-JOIN",
        "DENSE-REPEAT",
    ),
    "INIT_COPY": ("DENSE-INIT-COPY",),
    "INIT_BEHAVIOR": ("DENSE-INIT-CLONE",),
    "ALLOCATING_CONSTRUCTOR": ("DENSE-WITH-CAPACITY",),
    "RESERVE_DIVERGENT": ("DENSE-RESERVE", "DENSE-RESERVE-EXACT"),
    "RESERVE_RECOVERABLE": ("DENSE-TRY-RESERVE", "DENSE-TRY-RESERVE-EXACT"),
    "SHRINK": ("DENSE-INTO-BOXED", "DENSE-SHRINK-TO", "DENSE-SHRINK-TO-FIT"),
    "OD1_MUTATOR": (
        "DENSE-APPEND-MOVE", "DENSE-EAGER-SPLICE", "DENSE-EXTEND-CLONE",
        "DENSE-EXTEND-ITER", "DENSE-EXTEND-WITHIN", "DENSE-INSERT",
        "DENSE-INSERT-UNIQ", "DENSE-PUSH", "DENSE-PUSH-UNIQ",
        "DENSE-RESIZE-CLONE", "DENSE-RESIZE-WITH",
    ),
    "OD1_CONSTRUCTOR": ("DENSE-COLLECT",),
    "POP": ("DENSE-POP",),
    "POP_IF": ("DENSE-POP-IF",),
    "REMOVE": ("DENSE-REMOVE",),
    "SWAP_REMOVE": ("DENSE-SWAP-REMOVE",),
    "TRUNCATE": ("DENSE-TRUNCATE",),
    "CLEAR": ("DENSE-CLEAR",),
    "SPLIT_OFF": ("DENSE-SPLIT-OFF",),
    "ITER_BORROW": ("DENSE-ITER-SHARED", "DENSE-ITER-UNIQ"),
    "ITER_OWN": ("DENSE-ITER-OWN",),
    "DROP": ("DENSE-DROP",),
}


BASE_CAPS = "ST-DENSE,EX-NORMAL,EX-ABORT,AB-SEAL,AB-GENERIC"
BORROW_CAPS = BASE_CAPS + ",BR-REBORROW,BR-INVALIDATE"
UNIQ_CAPS = BORROW_CAPS + ",BR-DISJOINT"
MOVE_CAPS = BASE_CAPS + ",OW-MOVEOUT,OW-RELOCATE,OW-DROP,BR-INVALIDATE"
ALLOC_CAPS = BASE_CAPS + ",OW-INIT,OW-RELOCATE,OW-DROP,FL-ARITH,FL-ALLOC,BR-INVALIDATE"
BEHAVIOR_CAPS = MOVE_CAPS + ",AB-STATIC,FL-BEHAVIOR"


def _declaration(
    pre_state: str,
    offered_owners: str,
    post_state: str,
    behavior: str,
    resource_ceiling: str,
    capability_ids: str,
    scenario_ids: str,
) -> dict[str, str]:
    return {
        "pre_state": pre_state,
        "offered_owners": offered_owners,
        "post_state": post_state,
        "behavior": behavior,
        "resource_ceiling": resource_ceiling,
        "capability_ids": capability_ids,
        "scenario_ids": scenario_ids,
        "payload_branch_ids": "NONE_REGION_FREE_BORROW_FREE_BASE",
    }


# Each member has one authored declaration.  These rows do not copy, hash, or
# infer semantics from the rejected coarse member ledger.
MEMBER_DECLARATIONS: dict[str, dict[str, str]] = {
    "DENSE-ALIGN-EVIDENCE": _declaration("Writer requests unchecked alignment/type-punning authority.", "NONE", "STATIC_REJECTION", "ZERO", "No runtime path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-APPEND-MOVE": _declaration("DEST and SRC are valid dense owners; no incompatible borrow is live.", "DEST owner; SRC owner", "DEST contains old DEST followed by every old SRC owner; SRC is valid empty and retains its allocation unless transferred by the frozen capacity rule.", "ZERO", "O(len(SRC)) direct owner transfers; at most one growth allocation; no clone or per-item allocation.", ALLOC_CAPS, "S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-BOX-INIT-EVIDENCE": _declaration("Box-specific partial initialization evidence is requested.", "NONE", "STATIC_EXCLUSION_TO_F-RECURSIVE", "ZERO", "No dense-family runtime path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-CLEAR": _declaration("BASE is valid.", "BASE owner", "BASE is valid with len=0 and unchanged capacity; all former live owners are destroyed once.", "ZERO", "Exactly old len destructions; no allocation; no spare-capacity scan.", MOVE_CAPS, "S-TRUNCATE-DROP,S-OWNER-DROP,S-ZST-AFFINE"),
    "DENSE-CLONE-FROM": _declaration("DST and SRC have equal length; DST is uniquely borrowed and SRC is shared-borrowed.", "Live DST owners; borrowed SRC values", "Each DST slot is clone-updated in place in increasing index order; surviving destination resources and new clone results follow Clone::clone_from exactly.", "Exactly len clone_from calls in increasing index order; effects are not assumed pure.", "O(len); no mandatory allocation beyond behavior-owned allocation; no whole-value replacement requirement.", BEHAVIOR_CAPS, "S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-COLLECT": _declaration("PRODUCER is an owned finite iterator/protocol state.", "PRODUCER owner and each yielded value owner", "Returns one valid dense owner containing yielded owners in yield order; producer is destroyed once with remaining state.", "next is called in order until terminal None; no call duplication, fusion, or reordering.", "O(n) calls and moves; geometric growth; at most one live dense allocation; no per-item allocation.", ALLOC_CAPS + ",IT-OWN,AB-STATEFUL", "S-INIT-UNDERFILL,S-INIT-OVERFILL,S-ABANDON,S-GROW-FAIL,S-ZST-AFFINE"),
    "DENSE-COMPARE": _declaration("LEFT and RIGHT are valid shared dense views.", "Shared borrows of LEFT and RIGHT", "Both views and owners remain valid; returns the exact lexicographic/equality result.", "Comparator calls proceed in increasing index order and stop at the first decisive pair; effects are preserved.", "O(min(lenL,lenR)); zero allocation; monomorphized direct calls.", BORROW_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-NESTED-OWNER,S-ABANDON"),
    "DENSE-CONCAT": _declaration("Every source view is valid and payload Clone is available.", "Shared source borrows", "Returns one new valid dense owner containing cloned values in source order; sources remain valid.", "One clone per result element in source order.", "O(total elements); one destination allocation; no per-item allocation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-INIT-UNDERFILL,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-CONVERT": _declaration("The exact conversion source is valid and its direction-specific ownership premise holds.", "Direction-specific source owner or source borrow", "Returns the direction-specific dense owner/view without minting payload borrow roots.", "Only behavior named by the exact conversion implementation may run.", "At most one allocation and O(n) moves/clones; representation-reuse directions allocate zero.", ALLOC_CAPS, "S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-COPY-FROM": _declaration("DST and SRC are equally long Copy views and do not violate the member overlap rule.", "Unique DST borrow; shared SRC borrow", "DST values equal the pre-call SRC values; owners are Copy and no affine identity is duplicated.", "ZERO", "Exactly len payload copies; zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE"),
    "DENSE-COPY-WITHIN": _declaration("BASE is uniquely borrowed; source range and destination start are in bounds.", "Unique BASE borrow", "The selected Copy range is copied with memmove overlap semantics; all other values are unchanged.", "ZERO", "Exactly range length Copy traffic; zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE"),
    "DENSE-DEDUP": _declaration("BASE is valid and uniquely owned.", "BASE owner", "BASE keeps the first owner of each adjacent equal run in order; rejected owners are destroyed once.", "Equality is called once per required adjacent comparison in traversal order.", "O(n) comparisons; at most one relocation per retained owner after the first removal; zero allocation.", BEHAVIOR_CAPS, "S-EAGER-RETAIN,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-DEDUP-BY": _declaration("BASE is valid and uniquely owned.", "BASE owner; PRED environment", "BASE keeps the frozen representative selected by each adjacent predicate result; removed owners are destroyed once.", "Predicate calls are ordered, direct, effectful, and never duplicated.", "O(n) calls; O(n) relocations; zero allocation.", BEHAVIOR_CAPS, "S-EAGER-RETAIN,S-ABANDON,S-NESTED-OWNER"),
    "DENSE-DEDUP-BY-KEY": _declaration("BASE is valid and uniquely owned.", "BASE owner; KEY environment", "BASE retains one owner per adjacent equal-key run in stable order; removed owners are destroyed once.", "Key calls follow the frozen adjacent-comparison algorithm; no persistent key array is permitted.", "O(n) key/comparison calls; O(n) relocations; zero persistent scratch.", BEHAVIOR_CAPS, "S-EAGER-RETAIN,S-ABANDON,S-NESTED-OWNER"),
    "DENSE-DEFAULT": _declaration("No input owner.", "NONE", "Returns a valid empty dense owner with no payload allocation.", "ZERO", "O(1); zero allocation.", BASE_CAPS + ",OW-INIT", "S-INIT-UNDERFILL,S-ZST-AFFINE"),
    "DENSE-DROP": _declaration("BASE is the sole valid dense owner.", "BASE owner", "BASE is dead; exactly [0,len) is destroyed and its allocation is released once when present.", "Each payload destructor runs in increasing logical index order.", "O(len) destructions; one release; no spare-capacity scan.", MOVE_CAPS, "S-OWNER-DROP,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-EAGER-EXTRACT": _declaration("BASE is valid; PRED is owned; no incompatible borrow is live.", "BASE owner; PRED environment", "Returns valid retained BASE and a valid removed dense owner; both preserve their respective stable orders.", "PRED runs exactly once per original element in increasing order and may mutate the current value.", "O(n) calls and moves; at most one relocation per retained value after first hole; at most one output allocation.", BEHAVIOR_CAPS + ",FL-ALLOC", "S-EAGER-RETAIN,S-ABANDON,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-EAGER-SPLICE": _declaration("BASE is valid; RANGE is checked; PRODUCER is finite and owned.", "BASE owner; PRODUCER owner; every yielded replacement owner", "Returns valid BASE with replacements in order and a valid owner of removed values in original order.", "PRODUCER next calls are ordered; each yielded owner is consumed once; remaining producer state is destroyed once.", "O(n+r) moves; at most one BASE growth allocation and one removed-output allocation; no per-item allocation.", ALLOC_CAPS + ",AB-STATEFUL", "S-EAGER-SPLICE,S-ABANDON,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-EXTEND-CLONE": _declaration("BASE is valid; SRC is a valid shared view; Clone is available.", "BASE owner; shared SRC borrow", "BASE appends one clone of each SRC value in order.", "Exactly len(SRC) clone calls in increasing order.", "O(len SRC) clones/moves; at most one growth allocation after exact reservation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-GROW-FAIL,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-EXTEND-ITER": _declaration("BASE and PRODUCER are valid owned states.", "BASE owner; PRODUCER owner; yielded value owners", "BASE appends the yielded prefix in order; producer reaches terminal None and is destroyed once on success.", "next is called sequentially; prior calls and effects are never replayed.", "O(n) calls/moves; geometric growth; no per-item allocation.", ALLOC_CAPS + ",AB-STATEFUL,IT-OWN", "S-INIT-UNDERFILL,S-ABANDON,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-EXTEND-WITHIN": _declaration("BASE is valid; source range is in its pre-call live prefix; Clone is available.", "BASE owner", "BASE appends clones of the pre-call source range in source order.", "Exactly source-range length clone calls in order; source indices refer to the pre-call prefix.", "O(range length); at most one growth allocation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-GROW-FAIL,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-FILL-CLONE": _declaration("VIEW is uniquely borrowed and SEED is owned.", "Unique VIEW borrow; SEED owner", "Every slot is clone-updated; nonfinal slots use clone_from and the final slot receives SEED by move; empty input destroys SEED unused.", "Exactly max(len-1,0) clone_from calls in index order.", "O(len); zero allocation mandated by storage; behavior-owned allocation remains charged.", BEHAVIOR_CAPS, "S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-FILL-WITH": _declaration("VIEW is uniquely borrowed and PRODUCER environment is owned.", "Unique VIEW borrow; PRODUCER environment", "Each slot contains the corresponding produced owner; each replaced owner is destroyed after its replacement exists.", "Exactly len calls in increasing index order; producer-before-replace.", "O(len); zero collection allocation.", BEHAVIOR_CAPS, "S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-FIXED-EACH": _declaration("ARRAY is a valid fixed array and borrow mode is fixed.", "Shared or unique ARRAY borrow", "Returns N indexed borrows; unique results have pairwise-disjoint logical footprints.", "ZERO", "O(N) borrow formation; zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-ZST-AFFINE"),
    "DENSE-FIXED-MAP": _declaration("ARRAY and FN are owned; ARRAY has N live values.", "ARRAY owner; FN environment; N input owners", "Returns owned fixed array of N output owners in source order; every input owner has one callback disposition.", "Exactly N FnMut calls in increasing index order.", "O(N); zero heap allocation mandated.", BEHAVIOR_CAPS, "S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-FIXED-VIEW": _declaration("ARRAY is a valid fixed array.", "Shared or unique ARRAY borrow", "Returns a runtime-length view of every element rooted in ARRAY.", "ZERO", "O(1) view formation; zero allocation.", BORROW_CAPS, "S-BORROW-INVALIDATE,S-ZST-AFFINE"),
    "DENSE-FRESH-CLONE": _declaration("SRC is a valid shared view and Clone is available.", "Shared SRC borrow", "Returns one new valid dense owner with one clone per source value; SRC remains unchanged.", "Exactly len clone calls in increasing order.", "O(len); one allocation; no per-item allocation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-INIT-UNDERFILL,S-GROW-FAIL,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-HASH-TRAVERSAL": _declaration("BASE and caller HASHER are valid; BASE is shared-borrowed and HASHER uniquely reborrowed.", "Shared BASE borrow; unique HASHER reborrow", "BASE remains valid; HASHER post-state follows its declared effect relation.", "One hasher call per frozen element/length component in order; no call is elided or duplicated.", "O(len); zero collection allocation; monomorphized direct calls.", BORROW_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-ABANDON,S-NESTED-OWNER"),
    "DENSE-INDEX-SHARED": _declaration("VIEW is valid; INDEX/RANGE is supplied.", "Shared VIEW borrow", "In bounds returns receiver-rooted shared result; out of bounds traps before access.", "Index behavior is statically selected and direct.", "O(1) scalar/range formation; zero allocation.", BORROW_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE,S-SIMD-DEAD-LANE"),
    "DENSE-INDEX-UNIQ": _declaration("VIEW is valid and uniquely borrowed; INDEX/RANGE is supplied.", "Unique VIEW borrow", "In bounds returns receiver-rooted unique result; out of bounds traps before access.", "Index behavior is statically selected and direct.", "O(1); zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE,S-SIMD-DEAD-LANE"),
    "DENSE-INIT-AUTHORITY-EVIDENCE": _declaration("Writer requests forgeable initialization authority.", "NONE", "STATIC_REJECTION", "ZERO", "No runtime path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-INIT-CLONE": _declaration("DEST has exactly N dead slots; SRC has N valid shared values; Clone is available.", "DEST allocation owner; shared SRC borrow", "Success returns one valid dense owner with N cloned owners; partial normal exit is forbidden or structurally cleaned by the selected lifecycle.", "Exactly N clone calls in increasing order.", "O(N); one existing destination allocation; no per-item allocation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-INIT-UNDERFILL,S-INIT-OVERFILL,S-INIT-REPEAT-CLOSE,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-INIT-COPY": _declaration("DEST has exactly N dead slots; SRC has N valid Copy values.", "DEST allocation owner; shared SRC borrow", "Success returns one valid dense owner with N copied values; overfill is rejected before write.", "ZERO", "O(N) copies; one existing destination allocation.", ALLOC_CAPS, "S-INIT-UNDERFILL,S-INIT-OVERFILL,S-INIT-REPEAT-CLOSE,S-ABANDON,S-ZST-AFFINE"),
    "DENSE-INSERT": _declaration("BASE is valid; 0<=INDEX<=len; VALUE is owned; no incompatible borrow is live.", "BASE owner; VALUE owner", "Success moves VALUE to INDEX and relocates the old suffix right once in order.", "ZERO", "O(len-index) relocations plus at most one O(len) growth; no clone.", ALLOC_CAPS, "S-INSERT-SHIFT,S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-INSERT-UNIQ": _declaration("BASE is valid; 0<=INDEX<=len; VALUE is owned; no incompatible borrow is live.", "BASE owner; VALUE owner", "As DENSE-INSERT and additionally returns a receiver-rooted unique borrow of the inserted slot.", "ZERO", "Same as DENSE-INSERT; zero additional allocation.", ALLOC_CAPS + ",BR-REBORROW", "S-INSERT-SHIFT,S-BORROW-INVALIDATE,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-INTO-BOXED": _declaration("BASE is a valid dense owner.", "BASE owner", "Returns one exact-length boxed dense owner containing every former value owner in order; the former BASE binding is dead.", "ZERO", "O(n) only if shrink relocates; at most one replacement allocation; no clone.", ALLOC_CAPS, "S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-INTO-FLATTENED": _declaration("BASE owns a dense sequence of fixed arrays [T;N].", "BASE owner", "Returns one dense owner of T in row-major order without duplicating any payload owner.", "ZERO", "O(1) representation conversion when len*N fits; zero payload traffic.", MOVE_CAPS + ",FL-ARITH", "S-FACT-STALE,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-INTO-OWNER": _declaration("BOXED is a valid exact-length dense owner.", "BOXED owner", "Returns a valid growable dense owner with identical allocation root, order, and payload owners.", "ZERO", "O(1); zero allocation and zero payload traffic.", MOVE_CAPS, "S-FACT-STALE,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-ITER-OWN": _declaration("BASE is a valid sole owner.", "BASE owner", "Returns one cursor owning the master allocation and exact live interval [front,back); yields remove one endpoint before returning its owner.", "ZERO", "O(1) per yield; close/drop destroys exactly remaining interval then releases once.", MOVE_CAPS + ",IT-OWN,FT-STATE", "S-OWN-ITER,S-ABANDON,S-OWNER-DROP,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-ITER-SHARED": _declaration("BASE is valid and shared-borrowed.", "Shared BASE borrow", "Returns one cursor retaining shared source authority; each Some result is source-rooted and terminal None does not release authority.", "ZERO", "O(1) per call; zero allocation; cursor destruction releases source authority.", BORROW_CAPS + ",IT-SHARED,FT-STATE", "S-BORROW-INVALIDATE,S-ABANDON,S-ZST-AFFINE"),
    "DENSE-ITER-UNIQ": _declaration("BASE is valid and uniquely borrowed.", "Unique BASE borrow", "Returns one cursor retaining unique source authority; live yielded unique results are pairwise disjoint by progression.", "ZERO", "O(1) per call; zero allocation; cursor destruction releases remaining source authority.", UNIQ_CAPS + ",IT-UNIQ,FT-STATE", "S-BORROW-INVALIDATE,S-ABANDON,S-ZST-AFFINE"),
    "DENSE-JOIN": _declaration("Every source view is valid and SEPARATOR is a valid borrowed view; Clone is available.", "Shared source and separator borrows", "Returns one new dense owner with source groups and separators cloned in exact order.", "One clone per output element in output order.", "O(output len); one allocation; no per-item allocation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-INIT-UNDERFILL,S-GROW-FAIL,S-ABANDON,S-ZST-AFFINE"),
    "DENSE-LAZY-DRAIN-EVIDENCE": _declaration("A repair-bearing lazy drain surface is requested.", "NONE", "STATIC_EXCLUSION_OD4_EAGER_ONLY", "ZERO", "No admitted eager-family path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-LAZY-EXTRACT-EVIDENCE": _declaration("A repair-bearing lazy extract surface is requested.", "NONE", "STATIC_EXCLUSION_OD4_EAGER_ONLY", "ZERO", "No admitted eager-family path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-LAZY-SPLICE-EVIDENCE": _declaration("A repair-bearing lazy splice surface is requested.", "NONE", "STATIC_EXCLUSION_OD4_EAGER_ONLY", "ZERO", "No admitted eager-family path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-LEN-AUTHORITY-EVIDENCE": _declaration("Writer requests forgeable live-length authority.", "NONE", "STATIC_REJECTION", "ZERO", "No runtime path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-META": _declaration("BASE is valid.", "Shared BASE borrow", "Returns exact len/capacity/empty metadata and preserves BASE.", "ZERO", "O(1); zero allocation.", BASE_CAPS + ",FT-STATE", "S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-NEW": _declaration("No input owner.", "NONE", "Returns valid empty BASE with len=0, capacity=0 for positive-size T, and no allocation.", "ZERO", "O(1); zero allocation.", BASE_CAPS + ",OW-INIT", "S-INIT-UNDERFILL,S-ZST-AFFINE"),
    "DENSE-OWNER-VIEW": _declaration("BASE is valid and borrow mode is fixed.", "Shared or unique BASE borrow", "Returns a receiver-rooted contiguous view of exactly [0,len).", "ZERO", "O(1); zero allocation.", BORROW_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-POP": _declaration("BASE is valid; no incompatible borrow is live.", "BASE owner", "Empty returns None and unchanged BASE; nonempty marks len-1 dead before returning that sole owner.", "ZERO", "O(1); zero allocation; zero destruction of returned value.", MOVE_CAPS, "S-REMOVE-SHIFT,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-POP-IF": _declaration("BASE is valid; PRED is owned; no incompatible borrow is live.", "BASE owner; PRED environment", "Empty destroys PRED unused; false preserves last owner; true marks last dead before returning its owner.", "At most one predicate call on the last live element; exactly zero when empty.", "O(1); zero allocation.", BEHAVIOR_CAPS, "S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-PUSH": _declaration("BASE is valid; VALUE is owned; no incompatible borrow is live.", "BASE owner; VALUE owner", "Success moves VALUE exactly once into old len and increments len.", "ZERO", "O(1) amortized; at most one O(len) growth and one allocation.", ALLOC_CAPS, "S-PUSH-FAIL,S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-PUSH-UNIQ": _declaration("BASE is valid; VALUE is owned; no incompatible borrow is live.", "BASE owner; VALUE owner", "As DENSE-PUSH and returns a receiver-rooted unique borrow of the inserted slot.", "ZERO", "Same as DENSE-PUSH; no additional allocation.", ALLOC_CAPS + ",BR-REBORROW", "S-PUSH-FAIL,S-BORROW-INVALIDATE,S-GROW-FAIL,S-ZST-AFFINE"),
    "DENSE-RAW-SPARE-REJECTED": _declaration("Writer requests a view of spare or dead slots as T.", "NONE", "STATIC_REJECTION", "ZERO", "No runtime path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-REMOVE": _declaration("BASE is valid; 0<=INDEX<len; no incompatible borrow is live.", "BASE owner", "Returns the owner at INDEX; suffix owners relocate left once; retained order is preserved.", "ZERO", "O(len-index) relocations; zero allocation.", MOVE_CAPS, "S-REMOVE-SHIFT,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-REPEAT": _declaration("SRC is a valid shared view; COUNT is checked; Clone is available.", "Shared SRC borrow", "Returns one new owner containing COUNT ordered clone copies of SRC.", "Exactly len(SRC)*COUNT clone calls in output order.", "O(output len); one allocation; checked multiplication; no per-item allocation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-INIT-OVERFILL,S-GROW-FAIL,S-ABANDON,S-ZST-AFFINE"),
    "DENSE-REPLACE": _declaration("PLACE is live under unique authority; VALUE is owned.", "Old PLACE owner; VALUE owner", "Atomically installs VALUE and returns the old sole owner; no placeholder or double-live state is observable.", "ZERO", "O(1) plus payload move; zero allocation.", MOVE_CAPS, "S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-RESERVE": _declaration("BASE is valid; ADDITIONAL is checked; no payload borrow is live.", "BASE owner", "Returns BASE with capacity>=len+ADDITIONAL; values and order are preserved.", "ZERO", "O(len) only on growth; at most one allocation; geometric capacity policy.", ALLOC_CAPS, "S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-RESERVE-EXACT": _declaration("BASE is valid; ADDITIONAL is checked; no payload borrow is live.", "BASE owner", "Returns BASE with capacity>=len+ADDITIONAL using the exact logical request before allocator rounding.", "ZERO", "O(len) only on growth; at most one allocation.", ALLOC_CAPS, "S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-ZST-AFFINE"),
    "DENSE-RESIZE-CLONE": _declaration("BASE is valid; NEW_LEN is checked; SEED is owned when growth is requested.", "BASE owner; SEED owner", "Shrink destroys suffix owners; growth appends clone results and moves SEED exactly into the final new slot.", "Growth calls clone_from/clone exactly max(new_len-old_len-1,0) times in order.", "O(abs length delta); at most one growth allocation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-TRUNCATE-DROP,S-PUSH-FAIL,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-RESIZE-WITH": _declaration("BASE is valid; NEW_LEN is checked; PRODUCER is owned.", "BASE owner; PRODUCER environment", "Shrink destroys suffix; growth appends one produced owner per new slot in order.", "Exactly max(new_len-old_len,0) calls; zero calls on shrink/equal.", "O(abs length delta); at most one growth allocation.", ALLOC_CAPS + ",AB-STATIC,FL-BEHAVIOR", "S-TRUNCATE-DROP,S-PUSH-FAIL,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-RETAIN": _declaration("BASE is valid; PRED is owned; no incompatible borrow is live.", "BASE owner; PRED environment", "Retains selected owners in stable order; rejected owners are destroyed once; BASE is valid on success.", "Exactly one shared predicate call per original owner in increasing order.", "O(n) calls; at most one relocation per retained owner after first hole; zero allocation.", BEHAVIOR_CAPS, "S-EAGER-RETAIN,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-RETAIN-MUT": _declaration("BASE is valid; PRED is owned; no incompatible borrow is live.", "BASE owner; PRED environment", "As retain, while each call may mutate its current sole owner before deciding retention.", "Exactly one unique predicate call per original owner in increasing order.", "O(n) calls/relocations; zero allocation.", BEHAVIOR_CAPS, "S-EAGER-RETAIN,S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-REVERSE": _declaration("VIEW is valid and uniquely borrowed.", "Unique VIEW borrow", "Owners appear in exact reverse order; each owner remains unique.", "ZERO", "floor(len/2) swaps; zero allocation.", UNIQ_CAPS + ",OW-RELOCATE", "S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-ROTATE": _declaration("VIEW is uniquely borrowed; MID is supplied.", "Unique VIEW borrow", "When MID<=len, performs the exact left/right rotation permutation; otherwise traps before mutation.", "ZERO", "O(len) moves/swaps; zero allocation.", UNIQ_CAPS + ",OW-RELOCATE", "S-FACT-STALE,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-SELECT-UNSTABLE": _declaration("VIEW is uniquely borrowed; INDEX is supplied; comparator is owned/borrowed per member.", "Unique VIEW borrow; comparator environment", "In bounds partitions values around the returned INDEX owner without stability; every owner appears once.", "Direct comparator calls follow the frozen selection algorithm.", "O(n) expected/frozen algorithm bound; zero mandatory allocation.", BEHAVIOR_CAPS, "S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-SHRINK-TO": _declaration("BASE is valid; MIN_CAPACITY is supplied; no payload borrow is live.", "BASE owner", "Capacity becomes at least max(len,MIN_CAPACITY) and no greater than old capacity; values/order preserved.", "ZERO", "O(len) only if relocating; at most one replacement allocation.", ALLOC_CAPS, "S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-SHRINK-TO-FIT": _declaration("BASE is valid; no payload borrow is live.", "BASE owner", "Capacity becomes allocator-supported exact len or remains valid if already tight; values/order preserved.", "ZERO", "O(len) only if relocating; at most one replacement allocation.", ALLOC_CAPS, "S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-ZST-AFFINE"),
    "DENSE-SORT-STABLE": _declaration("VIEW is uniquely borrowed; comparator relation is available.", "Unique VIEW borrow; comparator environment", "Returns a stable sorted permutation; every owner appears once.", "Direct comparator calls in frozen stable-sort algorithm order.", "O(n log n); charged scratch allocation; no per-item allocation.", BEHAVIOR_CAPS + ",FL-ALLOC", "S-ABANDON,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-SORT-STABLE-CACHED-KEY": _declaration("VIEW is uniquely borrowed; KEY is owned.", "Unique VIEW borrow; KEY environment", "Returns stable key order; cached keys are operation-local and destroyed once.", "Exactly one key call per original element, then direct key comparisons in frozen order.", "O(n log n); one cached-key scratch array and stable-sort scratch; both charged.", BEHAVIOR_CAPS + ",FL-ALLOC", "S-ABANDON,S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-SORT-UNSTABLE": _declaration("VIEW is uniquely borrowed; comparator relation is available.", "Unique VIEW borrow; comparator environment", "Returns an unstable sorted permutation; every owner appears once.", "Direct comparator calls in frozen unstable-sort algorithm order.", "O(n log n); zero allocation in the Rust-contract control.", BEHAVIOR_CAPS, "S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-SPLIT-OFF": _declaration("BASE is valid; 0<=INDEX<=len; no incompatible borrow is live.", "BASE owner", "Returns prefix BASE and new suffix owner; each old owner appears in exactly one result in original order.", "ZERO", "O(len-index) moves; one suffix allocation except ZST; no clone.", ALLOC_CAPS, "S-GROW-FAIL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-SWAP": _declaration("VIEW is uniquely borrowed; LEFT and RIGHT are supplied.", "Unique VIEW borrow", "In bounds equal indices are a no-op; unequal indices exchange sole owners; out of bounds traps before mutation.", "ZERO", "O(1); zero allocation.", UNIQ_CAPS + ",OW-RELOCATE", "S-SWAP-EQUAL,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-SWAP-REMOVE": _declaration("BASE is valid; 0<=INDEX<len.", "BASE owner", "Returns INDEX owner; if nonlast, last owner relocates once into INDEX; len decreases once.", "ZERO", "O(1); zero allocation.", MOVE_CAPS, "S-REMOVE-SHIFT,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-SWAP-WITH-VIEW": _declaration("LEFT and RIGHT are unique equal-length nonoverlapping views.", "Unique LEFT and RIGHT borrows", "Each pair of sole owners exchanges positions; unequal length traps before mutation.", "ZERO", "O(len) swaps; zero allocation.", UNIQ_CAPS + ",OW-RELOCATE", "S-BORROW-INVALIDATE,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-TAKE-WITH-DEFAULT": _declaration("PLACE is live under unique authority and Default is available.", "Old PLACE owner", "Constructs replacement before commitment, installs it, and returns old sole owner.", "Exactly one Default call.", "O(1) collection work; behavior-owned allocation is charged.", BEHAVIOR_CAPS, "S-ABANDON,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-TRUNCATE": _declaration("BASE is valid; NEW_LEN is supplied.", "BASE owner", "If NEW_LEN<len, destroys exactly [NEW_LEN,len) and sets len; otherwise BASE is unchanged.", "Destructors run in increasing logical index order over removed range.", "O(max(old_len-new_len,0)); zero allocation; no spare scan.", MOVE_CAPS, "S-TRUNCATE-DROP,S-NESTED-OWNER,S-ZST-AFFINE"),
    "DENSE-TRY-RESERVE": _declaration("BASE is valid; ADDITIONAL is supplied; no payload borrow is live.", "BASE owner", "Success preserves order with sufficient capacity; recoverable error returns byte-identical logical BASE and original allocation.", "ZERO", "O(len) only on successful growth; at most one allocation attempt.", ALLOC_CAPS, "S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-TRY-RESERVE-EXACT": _declaration("BASE is valid; ADDITIONAL is supplied; no payload borrow is live.", "BASE owner", "As try_reserve with exact logical request before allocator rounding.", "ZERO", "O(len) only on successful growth; at most one allocation attempt.", ALLOC_CAPS, "S-GROW-SAME-ADDRESS,S-GROW-MOVED,S-GROW-FAIL,S-ZST-AFFINE"),
    "DENSE-UNCHECKED-ACCESS-EVIDENCE": _declaration("Writer requests unchecked access.", "NONE", "STATIC_REJECTION", "ZERO", "No runtime path.", "NONE", "S-RAW-SURFACE-NEGATIVE"),
    "DENSE-VIEW-ARRAY-CHUNKS": _declaration("VIEW is valid; const chunk width N is supplied.", "Shared or unique VIEW borrow", "For N>0 returns ordered exact chunks plus remainder; N=0 traps before access.", "ZERO", "O(1) view partition; zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-ZST-AFFINE"),
    "DENSE-VIEW-AS-FIXED": _declaration("VIEW is valid; const N is supplied.", "Shared or unique VIEW borrow", "Equal length returns receiver-rooted fixed-array borrow; unequal length returns None.", "ZERO", "O(1); zero allocation.", BORROW_CAPS, "S-BORROW-INVALIDATE,S-ZST-AFFINE"),
    "DENSE-VIEW-CONSUME-SPLIT": _declaration("VIEW is valid; split specification is checked.", "Borrowed VIEW authority", "Success consumes the input view authority and returns disjoint child views covering it; error preserves input authority.", "ZERO", "O(1); zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-VIEW-DISJOINT-UNIQ": _declaration("VIEW is uniquely borrowed; finite requested footprints are supplied.", "Unique VIEW borrow", "Success returns pairwise-disjoint unique borrows; duplicate/overlap/out-of-bounds returns checked error with no child.", "ZERO", "O(k log k) or frozen checked distinctness ceiling; zero allocation for frozen small arity.", UNIQ_CAPS, "S-SWAP-EQUAL,S-BORROW-INVALIDATE,S-ZST-AFFINE"),
    "DENSE-VIEW-END": _declaration("VIEW is valid.", "Shared or unique VIEW borrow", "Nonempty returns first/last receiver-rooted borrow; empty returns None.", "ZERO", "O(1); zero allocation.", BORROW_CAPS, "S-BORROW-INVALIDATE,S-ZST-AFFINE"),
    "DENSE-VIEW-END-CHUNK": _declaration("VIEW is valid; const N>0 is supplied.", "Shared or unique VIEW borrow", "Enough elements returns first/last fixed chunk borrow; short view returns None.", "ZERO", "O(1); zero allocation.", BORROW_CAPS, "S-BORROW-INVALIDATE,S-ZST-AFFINE"),
    "DENSE-VIEW-END-SPLIT": _declaration("VIEW is valid.", "Shared or unique VIEW borrow", "Nonempty returns endpoint borrow plus disjoint remainder view; empty returns None.", "ZERO", "O(1); zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-ZST-AFFINE"),
    "DENSE-VIEW-GET-SHARED": _declaration("VIEW is valid; INDEX/RANGE is supplied.", "Shared VIEW borrow", "In bounds returns Some shared receiver-rooted borrow; out of bounds returns None.", "Index behavior is statically selected and direct.", "O(1); zero allocation.", BORROW_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE,S-SIMD-DEAD-LANE"),
    "DENSE-VIEW-GET-UNIQ": _declaration("VIEW is valid and uniquely borrowed; INDEX/RANGE is supplied.", "Unique VIEW borrow", "In bounds returns Some unique receiver-rooted borrow; out of bounds returns None.", "Index behavior is statically selected and direct.", "O(1); zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE,S-SIMD-DEAD-LANE"),
    "DENSE-VIEW-META": _declaration("VIEW is valid.", "Shared VIEW borrow", "Returns exact len/empty metadata and preserves VIEW.", "ZERO", "O(1); zero allocation.", BASE_CAPS + ",FT-STATE", "S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-VIEW-SPLIT-CHECKED": _declaration("VIEW is valid; MID is supplied.", "Shared or unique VIEW borrow", "MID<=len returns disjoint children covering VIEW; otherwise returns checked error and preserves VIEW authority.", "ZERO", "O(1); zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-VIEW-SPLIT-TRAP": _declaration("VIEW is valid; MID is supplied.", "Shared or unique VIEW borrow", "MID<=len returns disjoint children covering VIEW; otherwise traps before access.", "ZERO", "O(1); zero allocation.", UNIQ_CAPS, "S-BORROW-INVALIDATE,S-FACT-STALE,S-ZST-AFFINE"),
    "DENSE-WITH-CAPACITY": _declaration("REQUEST is a logical element capacity.", "NONE", "Returns empty valid BASE with capacity>=REQUEST and one allocation when positive-size REQUEST requires it.", "ZERO", "O(1) allocator calls; checked element-to-byte arithmetic.", ALLOC_CAPS, "S-INIT-UNDERFILL,S-GROW-FAIL,S-ZST-AFFINE"),
}


# This is an explicit per-member semantic flag.  It is deliberately not
# inferred from names, profiles, capability strings, or declaration prose.
# A flagged member has a required direct behavior edge whose abort is an exact
# outcome under EFF-4.  Destructor, Clone, producer, predicate, comparator,
# hasher, Default, and other declared effectful calls are included; statically
# selected index dispatch is not a behavior environment and is excluded.
BEHAVIOR_ABORT_MEMBERS = frozenset(
    {
        "DENSE-CLONE-FROM",
        "DENSE-COLLECT",
        "DENSE-COMPARE",
        "DENSE-CONCAT",
        "DENSE-CONVERT",
        "DENSE-DEDUP",
        "DENSE-DEDUP-BY",
        "DENSE-DEDUP-BY-KEY",
        "DENSE-DROP",
        "DENSE-EAGER-EXTRACT",
        "DENSE-EAGER-SPLICE",
        "DENSE-EXTEND-CLONE",
        "DENSE-EXTEND-ITER",
        "DENSE-EXTEND-WITHIN",
        "DENSE-FILL-CLONE",
        "DENSE-FILL-WITH",
        "DENSE-FIXED-MAP",
        "DENSE-FRESH-CLONE",
        "DENSE-HASH-TRAVERSAL",
        "DENSE-INIT-CLONE",
        "DENSE-JOIN",
        "DENSE-POP-IF",
        "DENSE-REPEAT",
        "DENSE-RESIZE-CLONE",
        "DENSE-RESIZE-WITH",
        "DENSE-RETAIN",
        "DENSE-RETAIN-MUT",
        "DENSE-SELECT-UNSTABLE",
        "DENSE-SORT-STABLE",
        "DENSE-SORT-STABLE-CACHED-KEY",
        "DENSE-SORT-UNSTABLE",
        "DENSE-TAKE-WITH-DEFAULT",
        "DENSE-TRUNCATE",
    }
)

for _member_id, _declaration_row in MEMBER_DECLARATIONS.items():
    _declaration_row["behavior_abort_applicable"] = (
        "YES" if _member_id in BEHAVIOR_ABORT_MEMBERS else "NO"
    )


# Closed semantic transition classification.  This is the foreign-key domain
# consumed by the mathematical oracle; no prose or member-name heuristic may
# select a transition.
SEMANTIC_TRANSITION_MEMBERS: dict[str, tuple[str, ...]] = {
    "STATIC_REJECT": tuple(sorted(EXCLUDED_MEMBERS)),
    "PRESERVE": (
        "DENSE-COMPARE", "DENSE-FIXED-EACH", "DENSE-FIXED-VIEW", "DENSE-HASH-TRAVERSAL",
        "DENSE-INDEX-SHARED", "DENSE-INDEX-UNIQ", "DENSE-META", "DENSE-OWNER-VIEW",
        "DENSE-VIEW-ARRAY-CHUNKS", "DENSE-VIEW-AS-FIXED", "DENSE-VIEW-CONSUME-SPLIT",
        "DENSE-VIEW-DISJOINT-UNIQ", "DENSE-VIEW-END", "DENSE-VIEW-END-CHUNK",
        "DENSE-VIEW-END-SPLIT", "DENSE-VIEW-GET-SHARED", "DENSE-VIEW-GET-UNIQ",
        "DENSE-VIEW-META", "DENSE-VIEW-SPLIT-CHECKED", "DENSE-VIEW-SPLIT-TRAP",
    ),
    "CONSTRUCT_EMPTY": ("DENSE-DEFAULT", "DENSE-NEW", "DENSE-WITH-CAPACITY"),
    "CONSTRUCT_VALUES": (
        "DENSE-COLLECT", "DENSE-CONCAT", "DENSE-CONVERT", "DENSE-FIXED-MAP",
        "DENSE-FRESH-CLONE", "DENSE-JOIN", "DENSE-REPEAT",
    ),
    "TRANSFORM": (
        "DENSE-CLONE-FROM", "DENSE-COPY-FROM", "DENSE-COPY-WITHIN", "DENSE-DEDUP",
        "DENSE-DEDUP-BY", "DENSE-DEDUP-BY-KEY", "DENSE-FILL-CLONE", "DENSE-FILL-WITH",
        "DENSE-RETAIN", "DENSE-RETAIN-MUT", "DENSE-SELECT-UNSTABLE",
        "DENSE-SORT-STABLE", "DENSE-SORT-STABLE-CACHED-KEY", "DENSE-SORT-UNSTABLE",
        "DENSE-SWAP-WITH-VIEW", "DENSE-TAKE-WITH-DEFAULT",
    ),
    "GROW_PRESERVE": (
        "DENSE-INTO-BOXED", "DENSE-RESERVE", "DENSE-RESERVE-EXACT", "DENSE-SHRINK-TO",
        "DENSE-SHRINK-TO-FIT", "DENSE-TRY-RESERVE", "DENSE-TRY-RESERVE-EXACT",
    ),
    "PUSH": ("DENSE-PUSH", "DENSE-PUSH-UNIQ"),
    "INSERT": ("DENSE-INSERT", "DENSE-INSERT-UNIQ"),
    "APPEND": ("DENSE-APPEND-MOVE",),
    "EXTEND": ("DENSE-EXTEND-CLONE", "DENSE-EXTEND-ITER", "DENSE-EXTEND-WITHIN"),
    "RESIZE": ("DENSE-RESIZE-CLONE", "DENSE-RESIZE-WITH"),
    "POP": ("DENSE-POP",),
    "POP_IF": ("DENSE-POP-IF",),
    "REMOVE": ("DENSE-REMOVE",),
    "SWAP_REMOVE": ("DENSE-SWAP-REMOVE",),
    "CLEAR": ("DENSE-CLEAR",),
    "TRUNCATE": ("DENSE-TRUNCATE",),
    "REPLACE": ("DENSE-REPLACE",),
    "REVERSE": ("DENSE-REVERSE",),
    "ROTATE": ("DENSE-ROTATE",),
    "SWAP": ("DENSE-SWAP",),
    "SPLIT_OFF": ("DENSE-SPLIT-OFF",),
    "INTO_OWNER": ("DENSE-INTO-FLATTENED", "DENSE-INTO-OWNER"),
    "OWN_ITER": ("DENSE-ITER-OWN",),
    "BORROW_ITER": ("DENSE-ITER-SHARED", "DENSE-ITER-UNIQ"),
    "DROP": ("DENSE-DROP",),
    "INIT": ("DENSE-INIT-CLONE", "DENSE-INIT-COPY"),
    "EAGER_EXTRACT": ("DENSE-EAGER-EXTRACT",),
    "EAGER_SPLICE": ("DENSE-EAGER-SPLICE",),
}


def semantic_transition_by_member() -> dict[str, str]:
    result: dict[str, str] = {}
    for transition_id, members in SEMANTIC_TRANSITION_MEMBERS.items():
        for member_id in members:
            if member_id in result:
                raise ValueError(f"duplicate semantic transition member: {member_id}")
            result[member_id] = transition_id
    if set(result) != _all_members():
        raise ValueError(
            f"semantic transition universe mismatch missing={sorted(_all_members()-set(result))} "
            f"extra={sorted(set(result)-_all_members())}"
        )
    return result


def _outcome(
    code: str,
    trigger: str,
    commit_phase: str,
    owners_before: str,
    owners_after: str,
    allocation_before: str,
    allocation_after: str,
    borrows_before: str,
    borrows_after: str,
    facts_before: str,
    facts_after: str,
    behavior_calls: str,
    result_schema: str,
    state_equation: str,
) -> dict[str, str]:
    return {
        "code": code,
        "trigger": trigger,
        "commit_phase": commit_phase,
        "owners_before": owners_before,
        "owners_after": owners_after,
        "allocation_before": allocation_before,
        "allocation_after": allocation_after,
        "borrows_before": borrows_before,
        "borrows_after": borrows_after,
        "facts_before": facts_before,
        "facts_after": facts_after,
        "behavior_calls": behavior_calls,
        "result_schema": result_schema,
        "state_equation": state_equation,
    }


PRESERVE_SUCCESS = _outcome(
    "SUCCESS", "All member preconditions hold.", "NO_DESTRUCTIVE_COMMIT",
    "BASE=live;INPUTS=live", "BASE=live;INPUTS=as_declared_by_base_contract",
    "ALLOC(BASE)=A0", "ALLOC(BASE)=A0", "B0=compatible", "B1=base_contract_result",
    "F0=valid(A0,V0)", "F1=F0 plus result facts only", "BASE_CONTRACT_EXACT_COUNT",
    "BASE_CONTRACT_RESULT", "ValidDense(BASE1) and ordered values equal the member transformation of BASE0.",
)


PROFILE_OUTCOMES: dict[str, tuple[dict[str, str], ...]] = {
    "EXCLUDED_SURFACE": (
        _outcome(
            "EXCLUDED_NO_CALL", "Any attempt to spell or invoke this evidence-only surface.",
            "NO_CALL", "NO_ADMITTED_INPUT", "NO_ADMITTED_RESULT", "NONE", "NONE",
            "NONE", "NONE", "NONE", "NONE", "ZERO", "STATIC_REJECTION",
            "No accepted program reaches a payload or allocation transition.",
        ),
    ),
    "TOTAL": (PRESERVE_SUCCESS,),
    "TOTAL_BORROW": (PRESERVE_SUCCESS,),
    "OPTION_BORROW": (
        _outcome(
            "PRESENT", "Requested element, range, chunk, or endpoint exists.", "NO_DESTRUCTIVE_COMMIT",
            "BASE=live", "BASE=live;RESULT_BORROW=live", "ALLOC(BASE)=A0", "ALLOC(BASE)=A0",
            "B0=compatible", "B1=B0 plus receiver-rooted result footprint", "F0=valid(A0,V0)",
            "F1=F0", "ZERO", "Some(receiver-rooted borrow)",
            "BASE1=BASE0 and result.root=A0 and result.version=V0.",
        ),
        _outcome(
            "ABSENT", "Requested element, range, chunk, or endpoint does not exist.", "NO_DESTRUCTIVE_COMMIT",
            "BASE=live", "BASE=live", "ALLOC(BASE)=A0", "ALLOC(BASE)=A0", "B0=compatible", "B1=B0",
            "F0=valid(A0,V0)", "F1=F0", "ZERO", "None", "BASE1=BASE0; no borrow or owner is created.",
        ),
    ),
    "CHECKED_BORROW": (
        PRESERVE_SUCCESS,
        _outcome(
            "CHECKED_ERROR", "Bounds, arity, or distinctness premise is false.", "PRECOMMIT",
            "BASE=live", "BASE=live", "ALLOC(BASE)=A0", "ALLOC(BASE)=A0", "B0=compatible", "B1=B0",
            "F0=valid(A0,V0)", "F1=F0", "ZERO", "Err(exact checked reason)",
            "BASE1=BASE0 and no result borrow is created.",
        ),
    ),
    "TRAPPING_BORROW": (
        PRESERVE_SUCCESS,
        _outcome(
            "BOUNDS_TRAP", "The required bounds, arity, or chunk-size premise is false.", "PRECOMMIT_ABORT",
            "BASE=live", "NO_RECOVERABLE_POSTSTATE", "ALLOC(BASE)=A0", "NO_RECOVERABLE_POSTSTATE",
            "B0=compatible", "NO_RECOVERABLE_POSTSTATE", "F0=valid(A0,V0)", "NO_RECOVERABLE_POSTSTATE",
            "ZERO", "TRAP", "No invalid payload access occurs before abort.",
        ),
    ),
    "BEHAVIOR": (
        PRESERVE_SUCCESS,
        _outcome(
            "BEHAVIOR_ABORT", "The exact monomorphized behavior call aborts under EFF-4.", "ABORT_DURING_BEHAVIOR",
            "BASE=valid_at_call_boundary;ENV=live", "NO_RECOVERABLE_POSTSTATE", "ALLOC(BASE)=A0",
            "NO_RECOVERABLE_POSTSTATE", "B0=compatible", "NO_RECOVERABLE_POSTSTATE", "F0=valid(A0,V0)",
            "NO_RECOVERABLE_POSTSTATE", "EXACT_PREFIX_BEFORE_ABORT", "ABORT",
            "Every pre-abort read targets a live owner; no unwind or rollback is claimed.",
        ),
    ),
    "CHECKED_MUTATION": (
        PRESERVE_SUCCESS,
        _outcome(
            "PRECONDITION_TRAP", "Length, bounds, rotation, or overlap premise is false.", "PRECOMMIT_ABORT",
            "BASE=live;INPUTS=live", "NO_RECOVERABLE_POSTSTATE", "ALLOC(BASE)=A0", "NO_RECOVERABLE_POSTSTATE",
            "B0=compatible", "NO_RECOVERABLE_POSTSTATE", "F0=valid(A0,V0)", "NO_RECOVERABLE_POSTSTATE",
            "ZERO", "TRAP", "No owner moves and no invalid access occurs before abort.",
        ),
    ),
    "STABLE_SORT": (
        PRESERVE_SUCCESS,
        _outcome(
            "CAPACITY_OVERFLOW_TRAP", "Scratch-size arithmetic overflows.", "PRECOMMIT_ABORT",
            "BASE=live", "NO_RECOVERABLE_POSTSTATE", "ALLOC(BASE)=A0", "NO_RECOVERABLE_POSTSTATE",
            "B0=exclusive", "NO_RECOVERABLE_POSTSTATE", "F0=valid(A0,V0)", "NO_RECOVERABLE_POSTSTATE",
            "ZERO", "TRAP", "BASE is valid immediately before abort.",
        ),
        _outcome(
            "OOM_ABORT", "Scratch allocation fails at the current divergent OOM boundary.", "PRECOMMIT_ABORT",
            "BASE=live", "NO_RECOVERABLE_POSTSTATE", "ALLOC(BASE)=A0", "NO_RECOVERABLE_POSTSTATE",
            "B0=exclusive", "NO_RECOVERABLE_POSTSTATE", "F0=valid(A0,V0)", "NO_RECOVERABLE_POSTSTATE",
            "ZERO", "ABORT", "BASE is valid immediately before abort.",
        ),
        _outcome(
            "BEHAVIOR_ABORT", "Comparator or cached-key behavior aborts.", "ABORT_DURING_BEHAVIOR",
            "BASE=valid_permutation_state;ENV=live", "NO_RECOVERABLE_POSTSTATE", "ALLOC(BASE)=A0",
            "NO_RECOVERABLE_POSTSTATE", "B0=exclusive", "NO_RECOVERABLE_POSTSTATE", "F0=valid(A0,V0)",
            "NO_RECOVERABLE_POSTSTATE", "EXACT_CALL_PREFIX", "ABORT",
            "Every value has exactly one owner in a valid permutation state before abort.",
        ),
    ),
    "UNSTABLE_SORT": (),
    "SELECT": (),
    "CLONE_FROM": (),
    "ALLOCATING_BEHAVIOR": (),
    "INIT_COPY": (),
    "INIT_BEHAVIOR": (),
    "ALLOCATING_CONSTRUCTOR": (),
    "RESERVE_DIVERGENT": (),
    "RESERVE_RECOVERABLE": (),
    "SHRINK": (),
    "POP": (),
    "POP_IF": (),
    "REMOVE": (),
    "SWAP_REMOVE": (),
    "TRUNCATE": (),
    "CLEAR": (),
    "SPLIT_OFF": (),
    "ITER_BORROW": (),
    "ITER_OWN": (),
    "DROP": (),
}


def _abort_outcome(code: str, trigger: str, behavior_calls: str = "ZERO") -> dict[str, str]:
    return _outcome(
        code, trigger, "PRE_ABORT_NO_NORMAL_RESULT", "DECLARED_PRESTATE_OWNERS=live",
        "NO_RECOVERABLE_POSTSTATE", "ALLOCATIONS=declared_prestate", "NO_RECOVERABLE_POSTSTATE",
        "BORROWS=declared_prestate", "NO_RECOVERABLE_POSTSTATE", "FACTS=declared_prestate",
        "NO_RECOVERABLE_POSTSTATE", behavior_calls, "TRAP_OR_ABORT",
        "ValidDense and every partial-state invariant hold immediately before abort; no invalid access precedes abort.",
    )


def _error_outcome(code: str, trigger: str, result: str) -> dict[str, str]:
    return _outcome(
        code, trigger, "PRECOMMIT_RECOVERABLE", "BASE=live;OFFERED=live", "BASE=live;OFFERED=live",
        "ALLOC(BASE)=A0", "ALLOC(BASE)=A0", "BORROWS=compatible", "BORROWS=unchanged",
        "FACTS=valid(A0,V0)", "FACTS=unchanged", "ZERO", result,
        "BASE1=BASE0; OFFERED1=OFFERED0; allocation root, len, capacity, order, borrows, and facts are unchanged.",
    )


BEHAVIOR_ABORT = _abort_outcome(
    "BEHAVIOR_ABORT", "A required direct behavior call aborts under EFF-4.", "EXACT_PREFIX_BEFORE_ABORT"
)
BOUNDS_TRAP = _abort_outcome("BOUNDS_TRAP", "A checked bounds or shape premise is false.")
OVERFLOW_TRAP = _abort_outcome("CAPACITY_OVERFLOW_TRAP", "Checked element, byte, or layout arithmetic fails.")
OOM_ABORT = _abort_outcome("OOM_ABORT", "The current allocator reports OOM on a divergent allocation edge.")

PROFILE_OUTCOMES.update(
    {
        "UNSTABLE_SORT": (PRESERVE_SUCCESS, BEHAVIOR_ABORT),
        "SELECT": (PRESERVE_SUCCESS, BOUNDS_TRAP, BEHAVIOR_ABORT),
        "CLONE_FROM": (PRESERVE_SUCCESS, BOUNDS_TRAP, BEHAVIOR_ABORT),
        "ALLOCATING_BEHAVIOR": (PRESERVE_SUCCESS, OVERFLOW_TRAP, OOM_ABORT, BEHAVIOR_ABORT),
        "INIT_COPY": (
            PRESERVE_SUCCESS,
            _error_outcome("UNDERFILL_CLOSE_REJECTED", "Completion is requested before every required slot is live.", "Err(underfill; partial state follows candidate lifecycle)"),
            _error_outcome("OVERFILL_REJECTED", "A value is offered after capacity is fully live.", "Err(overfill; offered owner returned)"),
        ),
        "INIT_BEHAVIOR": (
            PRESERVE_SUCCESS,
            _error_outcome("UNDERFILL_CLOSE_REJECTED", "Completion is requested before every required slot is live.", "Err(underfill; partial state follows candidate lifecycle)"),
            _error_outcome("OVERFILL_REJECTED", "A value is offered after capacity is fully live.", "Err(overfill; offered owner returned)"),
            BEHAVIOR_ABORT,
        ),
        "ALLOCATING_CONSTRUCTOR": (PRESERVE_SUCCESS, OVERFLOW_TRAP, OOM_ABORT),
        "RESERVE_DIVERGENT": (
            _outcome("SUCCESS_NO_GROW", "len+additional<=capacity.", "NO_DESTRUCTIVE_COMMIT", "BASE=live", "BASE=live", "ALLOC(BASE)=A0", "ALLOC(BASE)=A0", "BORROWS=none", "BORROWS=none", "FACTS=valid(A0,V0)", "FACTS=valid(A0,V0)", "ZERO", "Ok(BASE)", "BASE1=BASE0."),
            _outcome("SUCCESS_GROW", "len+additional>capacity and allocation succeeds.", "COMMIT_AFTER_NEW_OWNER_AND_ALL_RELOCATIONS", "BASE=live", "BASE'=live", "ALLOC(BASE)=A0", "ALLOC(BASE')=A1;A0=released", "BORROWS=none", "BORROWS=none", "FACTS=valid(A0,V0)", "FACTS=valid(A1,V1);old facts invalid", "ZERO", "Ok(BASE')", "Values(BASE')=Values(BASE); root A1 and version V1 are fresh even when numerical address is reused."),
            OVERFLOW_TRAP,
            OOM_ABORT,
        ),
        "RESERVE_RECOVERABLE": (
            _outcome("SUCCESS_NO_GROW", "len+additional<=capacity.", "NO_DESTRUCTIVE_COMMIT", "BASE=live", "BASE=live", "ALLOC(BASE)=A0", "ALLOC(BASE)=A0", "BORROWS=none", "BORROWS=none", "FACTS=valid(A0,V0)", "FACTS=valid(A0,V0)", "ZERO", "Ok(BASE)", "BASE1=BASE0."),
            _outcome("SUCCESS_GROW", "len+additional>capacity and allocation succeeds.", "COMMIT_AFTER_NEW_OWNER_AND_ALL_RELOCATIONS", "BASE=live", "BASE'=live", "ALLOC(BASE)=A0", "ALLOC(BASE')=A1;A0=released", "BORROWS=none", "BORROWS=none", "FACTS=valid(A0,V0)", "FACTS=valid(A1,V1);old facts invalid", "ZERO", "Ok(BASE')", "Values(BASE')=Values(BASE); root/version transfer is exact."),
            _error_outcome("CAPACITY_ERROR_RETURN", "Checked capacity/layout arithmetic fails.", "Err(CapacityError, BASE)"),
            _error_outcome("ALLOCATION_ERROR_RETURN", "Allocator acquisition fails before relocation.", "Err(AllocationError, BASE)"),
        ),
        "SHRINK": (
            _outcome("SUCCESS_NO_CHANGE", "Requested shrink does not reduce allocator-backed capacity.", "NO_DESTRUCTIVE_COMMIT", "BASE=live", "BASE=live", "ALLOC(BASE)=A0", "ALLOC(BASE)=A0", "BORROWS=none", "BORROWS=none", "FACTS=valid(A0,V0)", "FACTS=valid(A0,V0)", "ZERO", "BASE", "BASE1=BASE0."),
            _outcome("SUCCESS_RELOCATE", "Shrink uses a replacement allocation or changed allocation root.", "COMMIT_AFTER_ALL_VALUES_RELOCATE", "BASE=live", "BASE'=live", "ALLOC(BASE)=A0", "ALLOC(BASE')=A1;A0=released", "BORROWS=none", "BORROWS=none", "FACTS=valid(A0,V0)", "FACTS=valid(A1,V1)", "ZERO", "BASE'", "Values and order preserved; every old-root fact invalidated."),
            OOM_ABORT,
        ),
        "POP": (
            _outcome("EMPTY", "len=0.", "NO_DESTRUCTIVE_COMMIT", "BASE=live", "BASE=live", "ALLOC(BASE)=A0_or_none", "unchanged", "BORROWS=none", "BORROWS=none", "FACTS=valid", "FACTS=unchanged", "ZERO", "(BASE,None)", "BASE1=BASE0; no owner is returned or destroyed."),
            _outcome("VALUE_RETURNED", "len>0.", "SLOT_DEAD_BEFORE_RESULT_LIVE", "BASE=live;slot[len-1]=VALUE", "BASE'=live;RESULT=VALUE", "ALLOC(BASE)=A0", "ALLOC(BASE')=A0", "BORROWS=none", "BORROWS=none", "FACTS=valid(A0,V0)", "old facts invalid;new facts valid(A0,V1)", "ZERO", "(BASE',Some(VALUE))", "len'=len-1; returned VALUE is absent from BASE' and is not destroyed."),
        ),
        "POP_IF": (
            _outcome("EMPTY", "len=0.", "NO_DESTRUCTIVE_COMMIT", "BASE=live;PRED=live", "BASE=live;PRED=destroyed", "unchanged", "unchanged", "BORROWS=none", "BORROWS=none", "FACTS=valid", "FACTS=unchanged", "ZERO", "(BASE,None)", "PRED is destroyed unused exactly once."),
            _outcome("PREDICATE_FALSE", "len>0 and the one predicate call returns false.", "NO_PAYLOAD_COMMIT", "BASE=live;PRED=live", "BASE=live;PRED=destroyed", "unchanged", "unchanged", "BORROWS=temporary last-slot unique", "temporary borrow ended", "FACTS=valid", "FACTS=valid after declared behavior effect", "EXACTLY_ONE", "(BASE,None)", "Last owner remains in BASE with behavior-authorized mutation only."),
            _outcome("PREDICATE_TRUE", "len>0 and the one predicate call returns true.", "SLOT_DEAD_BEFORE_RESULT_LIVE", "BASE=live;last=VALUE;PRED=live", "BASE'=live;RESULT=VALUE;PRED=destroyed", "unchanged", "unchanged", "temporary last-slot unique", "temporary borrow ended", "FACTS=valid", "old facts invalid;new facts valid", "EXACTLY_ONE", "(BASE',Some(VALUE))", "VALUE is returned once and not destroyed."),
            BEHAVIOR_ABORT,
        ),
        "REMOVE": (PRESERVE_SUCCESS, BOUNDS_TRAP),
        "SWAP_REMOVE": (PRESERVE_SUCCESS, BOUNDS_TRAP),
        "TRUNCATE": (
            _outcome("NO_CHANGE", "new_len>=old_len.", "NO_DESTRUCTIVE_COMMIT", "BASE=live", "BASE=live", "unchanged", "unchanged", "BORROWS=none", "BORROWS=none", "FACTS=valid", "FACTS=unchanged", "ZERO", "BASE", "BASE1=BASE0."),
            _outcome("SUFFIX_DESTROYED", "new_len<old_len.", "LEN_SHORTENED_BEFORE_EACH_DESTRUCTOR", "BASE=live", "BASE'=live;removed owners destroyed", "unchanged", "unchanged", "BORROWS=none", "BORROWS=none", "FACTS=valid", "old facts invalid;new prefix facts valid", "EXACT_DESTRUCTOR_COUNT", "BASE'", "Exactly indices [new_len,old_len) are destroyed once."),
        ),
        "CLEAR": (
            _outcome("EMPTY_NO_CHANGE", "len=0.", "NO_DESTRUCTIVE_COMMIT", "BASE=live", "BASE=live", "unchanged", "unchanged", "BORROWS=none", "BORROWS=none", "FACTS=valid", "FACTS=unchanged", "ZERO", "BASE", "BASE1=BASE0."),
            _outcome("ALL_VALUES_DESTROYED", "len>0.", "LEN_ZEROED_BEFORE_OR_IN_LOCKSTEP_WITH_DESTRUCTION", "BASE=live", "BASE'=live;all old values destroyed", "unchanged", "unchanged", "BORROWS=none", "BORROWS=none", "FACTS=valid", "old facts invalid;empty facts valid", "EXACT_DESTRUCTOR_COUNT", "BASE'", "Exactly old [0,len) is destroyed once; allocation retained."),
        ),
        "SPLIT_OFF": (PRESERVE_SUCCESS, BOUNDS_TRAP, OVERFLOW_TRAP, OOM_ABORT),
        "ITER_BORROW": (
            _outcome("CURSOR_CREATED", "Borrowed traversal construction succeeds.", "SOURCE_AUTHORITY_MOVES_TO_CURSOR", "SOURCE_BORROW=live", "CURSOR=live", "unchanged", "unchanged", "source borrow", "cursor owns source authority", "source facts valid", "facts transfer to cursor", "ZERO", "CURSOR", "Base owner is excluded until cursor destruction and incompatible children end."),
            _outcome("NEXT_SOME", "Cursor has a next logical position.", "PROGRESSION_BEFORE_RESULT", "CURSOR=live", "CURSOR'=live;RESULT_BORROW=live", "unchanged", "unchanged", "cursor authority", "cursor authority plus result footprint", "cursor facts valid", "progression facts updated", "ZERO", "Some(result borrow)", "Unique results require pairwise-disjoint logical footprints."),
            _outcome("TERMINAL_NONE", "Cursor has no next logical position.", "LOGICAL_ONLY", "CURSOR=live", "CURSOR=live", "unchanged", "unchanged", "cursor authority", "cursor authority", "facts valid", "facts valid", "ZERO", "None", "None does not release source authority."),
            _outcome("CURSOR_DESTROYED", "Cursor is destroyed or consuming-closed.", "AUTHORITY_RELEASE", "CURSOR=live", "CURSOR=dead", "unchanged", "unchanged", "cursor authority", "source authority released after children end", "cursor facts", "cursor facts deleted", "ZERO", "Unit", "No payload owner is destroyed by borrowed traversal."),
        ),
        "ITER_OWN": (
            _outcome("CURSOR_CREATED", "BASE is consumed.", "BASE_OWNER_TO_CURSOR", "BASE=live", "CURSOR owns allocation and [0,len)", "ALLOC(BASE)=A0", "ALLOC(CURSOR)=A0", "BORROWS=none", "BORROWS=none", "FACTS=valid", "facts transfer", "ZERO", "CURSOR", "BASE binding is dead; CURSOR is sole owner."),
            _outcome("YIELD_FRONT", "front<back and next is called.", "FRONT_DEAD_BEFORE_RESULT", "CURSOR owns [front,back)", "CURSOR' owns [front+1,back);RESULT owns old front", "ALLOC=A0", "ALLOC=A0", "BORROWS=none", "BORROWS=none", "FACTS=valid", "old facts invalid;interval facts updated", "ZERO", "Some(owned value)", "Returned owner is absent from cursor live interval."),
            _outcome("YIELD_BACK", "front<back and next_back is called.", "BACK_DEAD_BEFORE_RESULT", "CURSOR owns [front,back)", "CURSOR' owns [front,back-1);RESULT owns old back-1", "ALLOC=A0", "ALLOC=A0", "BORROWS=none", "BORROWS=none", "FACTS=valid", "old facts invalid;interval facts updated", "ZERO", "Some(owned value)", "Returned owner is absent from cursor live interval."),
            _outcome("TERMINAL_NONE", "front=back.", "LOGICAL_ONLY", "CURSOR owns empty interval", "CURSOR owns empty interval", "ALLOC=A0", "ALLOC=A0", "BORROWS=none", "BORROWS=none", "FACTS=valid", "FACTS=valid", "ZERO", "None", "Allocation remains cursor-owned until close/drop."),
            _outcome("CLOSE_OR_DROP", "Cursor is consumed or abandoned in any interval state.", "DESTROY_INTERVAL_THEN_RELEASE", "CURSOR owns [front,back)", "CURSOR=dead;remaining owners destroyed", "ALLOC=A0", "A0=released exactly once", "BORROWS=none", "BORROWS=none", "FACTS=valid", "all cursor facts deleted", "EXACT_DESTRUCTOR_COUNT", "Unit", "Exactly [front,back) is destroyed; dead exterior is never read or dropped."),
        ),
        "DROP": (
            _outcome("OWNER_DESTROYED", "The sole BASE owner is destroyed.", "DESTROY_LIVE_PREFIX_THEN_RELEASE", "BASE owns values [0,len)", "BASE=dead;values destroyed", "ALLOC(BASE)=A0_or_none", "A0 released once when present", "BORROWS=none", "BORROWS=none", "FACTS=valid", "all facts deleted", "EXACT_DESTRUCTOR_COUNT", "Unit", "Exactly len live owners are destroyed in declared order; spare slots are untouched."),
        ),
    }
)


OD1_OUTCOMES: dict[str, tuple[dict[str, str], ...]] = {
    OD1_RESERVE_FIRST: (
        _outcome("SUCCESS_NO_GROW", "Required capacity is already available.", "VALUE_OR_RANGE_COMMIT", "BASE=live;OFFERED=live", "BASE'=live;OFFERED consumed as declared", "ALLOC(BASE)=A0", "ALLOC(BASE')=A0", "BORROWS=none", "BORROWS=declared result only", "FACTS=valid(A0,V0)", "old facts invalid;new facts valid(A0,V1)", "DECLARED_MEMBER_COUNT", "Success(BASE', declared result)", "Every offered owner has exactly one declared destination."),
        _outcome("SUCCESS_GROW", "Growth arithmetic succeeds and divergent allocation succeeds.", "NEW_ALLOCATION_OWNED_BEFORE_RELOCATION;OFFERED_COMMIT_AFTER_CAPACITY", "BASE=live;OFFERED=live", "BASE'=live;OFFERED consumed as declared", "ALLOC(BASE)=A0", "ALLOC(BASE')=A1;A0 released once", "BORROWS=none", "BORROWS=declared result only", "FACTS=valid(A0,V0)", "old facts invalid;new facts valid(A1,V1)", "DECLARED_MEMBER_COUNT", "Success(BASE', declared result)", "All old and offered owners reach exactly one post-state role."),
        OVERFLOW_TRAP,
        OOM_ABORT,
        BEHAVIOR_ABORT,
    ),
    OD1_RECOVERABLE: (
        _outcome("SUCCESS_NO_GROW", "Required capacity is already available.", "VALUE_OR_RANGE_COMMIT", "BASE=live;OFFERED=live", "BASE'=live;OFFERED consumed as declared", "ALLOC(BASE)=A0", "ALLOC(BASE')=A0", "BORROWS=none", "BORROWS=declared result only", "FACTS=valid(A0,V0)", "old facts invalid;new facts valid(A0,V1)", "DECLARED_MEMBER_COUNT", "Ok(BASE', declared result)", "Every offered owner has exactly one declared destination."),
        _outcome("SUCCESS_GROW", "All recoverable preparation succeeds before the first destructive commit.", "LAST_RECOVERABLE_POINT_BEFORE_ANY_OWNER_MOVE", "BASE=live;OFFERED=live", "BASE'=live;OFFERED consumed as declared", "ALLOC(BASE)=A0", "ALLOC(BASE')=A1;A0 released once", "BORROWS=none", "BORROWS=declared result only", "FACTS=valid(A0,V0)", "old facts invalid;new facts valid(A1,V1)", "DECLARED_MEMBER_COUNT", "Ok(BASE', declared result)", "All old and offered owners reach exactly one post-state role."),
        _error_outcome("CAPACITY_ERROR_RETURN", "Checked preparation arithmetic fails before commitment.", "Err(CapacityError, unchanged BASE, every offered affine owner)"),
        _error_outcome("ALLOCATION_ERROR_RETURN", "Allocation fails before commitment.", "Err(AllocationError, unchanged BASE, every offered affine owner)"),
        BEHAVIOR_ABORT,
    ),
}


# Only protocol-derived eager replacements lack a direct G0/Rust evidence
# identity.  Real excluded evidence surfaces must retain their actual evidence
# IDs; they may never be filled by a synthetic placeholder.
PROTOCOL_SYNTHETIC_MEMBERS = CLOSED_PROTOCOL_SYNTHETIC_MEMBERS
STORED_BORROW_ROUTE_BY_MEMBER = {
    "DENSE-EAGER-EXTRACT": "ACTIVE_BR_STORED::SEQ-EXTRACT-01",
    "DENSE-EAGER-SPLICE": "ACTIVE_BR_STORED::SEQ-SPLICE-01",
    "DENSE-EXTEND-ITER": "ACTIVE_BR_STORED::TRAIT-EXTEND-01",
    "DENSE-COLLECT": "ACTIVE_BR_STORED::TRAIT-COLLECT-01",
}


def _profile_by_member() -> dict[str, str]:
    result: dict[str, str] = {}
    for profile_id, members in PROFILE_MEMBERS.items():
        for member_id in members:
            if member_id in result:
                raise ValueError(f"duplicate member profile: {member_id}")
            result[member_id] = profile_id
    if set(result) != _all_members():
        raise ValueError(
            f"profile universe mismatch missing={sorted(_all_members() - set(result))} "
            f"extra={sorted(set(result) - _all_members())}"
        )
    return result


def _cluster_by_member() -> dict[str, tuple[str, ...]]:
    result: dict[str, list[str]] = defaultdict(list)
    for cluster_id, members in CLUSTER_MEMBERS.items():
        for member_id in members:
            result[member_id].append(cluster_id)
    return {key: tuple(sorted(value)) for key, value in result.items()}


def _coverage_authority() -> tuple[
    dict[tuple[str, str], tuple[str, ...]],
    dict[tuple[str, str], tuple[str, ...]],
    dict[tuple[str, str], tuple[str, ...]],
]:
    """Load exact identity/overlay/capability keys, never operation semantics."""
    cluster_by_member = _cluster_by_member()
    evidence: dict[tuple[str, str], set[str]] = defaultdict(set)
    for row in _read_pinned_coverage_tsv(HERE / "DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv"):
        member_id = row["member_contract_id"]
        if member_id not in cluster_by_member:
            raise ValueError(f"unknown authority member: {member_id}")
        if row["cluster_id"] not in cluster_by_member[member_id]:
            raise ValueError(f"authority cluster mismatch: {member_id}")
        evidence[(row["cluster_id"], member_id)].add(row["subject_identity"])

    overlays: dict[tuple[str, str], set[str]] = defaultdict(set)
    capabilities: dict[tuple[str, str], set[str]] = defaultdict(set)
    for row in _read_pinned_coverage_tsv(HERE / "DENSE-OVERLAY-BRANCH-AUTHORITY.tsv"):
        member_id = row["member_contract_id"]
        if member_id == "NONE":
            continue
        if member_id not in cluster_by_member:
            raise ValueError(f"unknown overlay member: {member_id}")
        key = (row["cluster_id"], member_id)
        overlays[key].add(row["overlay_branch_id"])
        for value in row["base_capability_ids"].split(","):
            if value:
                capabilities[key].add(value)

    expected_pairs = {
        (cluster_id, member_id)
        for cluster_id, members in CLUSTER_MEMBERS.items()
        for member_id in members
    }
    missing_pairs = expected_pairs - set(evidence)
    if {member_id for _, member_id in missing_pairs} != PROTOCOL_SYNTHETIC_MEMBERS:
        raise ValueError(
            f"synthetic evidence set changed missing={sorted(missing_pairs)} "
            f"expected={sorted(PROTOCOL_SYNTHETIC_MEMBERS)}"
        )
    if any(member_id not in PROTOCOL_SYNTHETIC_MEMBERS for _, member_id in missing_pairs):
        raise ValueError(f"non-synthetic member lacks authority: {sorted(missing_pairs)}")
    for cluster_id, member_id in missing_pairs:
        evidence[(cluster_id, member_id)].add(f"SYNTHETIC:{member_id}")
    return (
        {key: tuple(sorted(value)) for key, value in evidence.items()},
        {key: tuple(sorted(value)) for key, value in overlays.items()},
        {key: tuple(sorted(value)) for key, value in capabilities.items()},
    )


def _policy_ids(profile_id: str) -> tuple[str, ...]:
    if profile_id in {"OD1_MUTATOR", "OD1_CONSTRUCTOR"}:
        return (OD1_RESERVE_FIRST, OD1_RECOVERABLE)
    return (COMMON_POLICY,)


def _outcomes_for(
    member_id: str,
    profile_id: str,
    policy_variant_id: str,
) -> tuple[dict[str, str], ...]:
    if profile_id == "OD1_MUTATOR":
        base_outcomes = OD1_OUTCOMES[policy_variant_id]
    elif profile_id == "OD1_CONSTRUCTOR":
        base_outcomes = PROFILE_OUTCOMES["ALLOCATING_CONSTRUCTOR"]
    else:
        base_outcomes = PROFILE_OUTCOMES[profile_id]
    declaration = MEMBER_DECLARATIONS[member_id]
    behavior_abort_applicable = declaration["behavior_abort_applicable"] == "YES"
    outcomes = tuple(
        dict(outcome)
        for outcome in base_outcomes
        if outcome["code"] != "BEHAVIOR_ABORT" or behavior_abort_applicable
    )
    if behavior_abort_applicable and not any(
        outcome["code"] == "BEHAVIOR_ABORT" for outcome in outcomes
    ):
        outcomes += (dict(BEHAVIOR_ABORT),)
    if declaration["behavior"] == "ZERO":
        for outcome in outcomes:
            outcome["behavior_calls"] = "ZERO"
    if not outcomes:
        raise ValueError(f"empty outcome profile: {profile_id}")
    return outcomes


def _explicit_owner_fields(
    member_id: str,
    profile_id: str,
    outcome: dict[str, str],
    declaration: dict[str, str],
) -> dict[str, str]:
    code = outcome["code"]
    abort = code.endswith("TRAP") or code.endswith("ABORT")
    recoverable = code in {
        "CHECKED_ERROR", "UNDERFILL_CLOSE_REJECTED", "OVERFILL_REJECTED",
        "CAPACITY_ERROR_RETURN", "ALLOCATION_ERROR_RETURN",
    }
    if abort:
        post_state = "NO_RECOVERABLE_POSTSTATE"
        result_owners = "NONE_NO_NORMAL_RESULT"
        returned_owners = "NONE_NO_NORMAL_RESULT"
        retained_owners = "NONE_NO_NORMAL_RESULT"
        destroyed_owners = "NONE_CREDITED_AFTER_ABORT"
        cleanup = "NONE_AFTER_ABORT"
        pre_abort = "ValidDense or the candidate lifecycle's exact partial-state invariant; every owner has one role and no dead slot is accessed before abort."
    elif recoverable:
        post_state = "The complete pre_state is unchanged; no destructive commitment occurred."
        result_owners = "The error result owns exactly the error value plus every owner named by returned_owners."
        returned_owners = "BASE plus every affine owner named in offered_owners; no clone, destruction, or transfer."
        retained_owners = "Every pre-state owner is returned in its original role."
        destroyed_owners = "NONE"
        cleanup = "NONE; precommit state is already valid."
        pre_abort = "NOT_APPLICABLE_RECOVERABLE_RESULT"
    else:
        post_state = declaration["post_state"]
        result_owners = f"Exactly the ownership-bearing results named here, and no others: {declaration['post_state']}"
        returned_owners = "Exactly the caller-owned results named by post_state; NONE when post_state names only borrows or scalars."
        retained_owners = "Pre-state owners not explicitly returned, destroyed, or consumed remain in the exact post_state role; set difference is total and duplicate-free."
        destroyed_owners = "Exactly the owners explicitly named destroyed by post_state; NONE otherwise."
        cleanup = "The normal branch reaches the declared post_state directly; any earlier normal exit follows the selected candidate lifecycle row, never writer discipline."
        pre_abort = "NOT_APPLICABLE_NORMAL_RESULT"

    if profile_id == "DROP" or code == "OWNER_DESTROYED":
        returned_owners = "NONE"
        retained_owners = "NONE"
        destroyed_owners = "Every logical owner in pre-state [0,len), exactly once."
    elif profile_id == "POP" and code == "VALUE_RETURNED":
        returned_owners = "Exactly the former owner of slot len-1."
        destroyed_owners = "NONE"
    elif profile_id in {"REMOVE", "SWAP_REMOVE"} and code == "SUCCESS":
        returned_owners = "Exactly the former owner at INDEX."
        destroyed_owners = "NONE"
    elif profile_id == "SPLIT_OFF" and code == "SUCCESS":
        returned_owners = "Exactly two dense owners: the prefix BASE and new suffix owner; their payload-owner sets partition the pre-state set."
        destroyed_owners = "NONE"
    elif profile_id == "CLEAR" and code == "ALL_VALUES_DESTROYED":
        destroyed_owners = "Exactly every former live owner at indices [0,old_len)."
    elif profile_id == "TRUNCATE" and code == "SUFFIX_DESTROYED":
        destroyed_owners = "Exactly the former live owners at indices [new_len,old_len)."
    elif profile_id == "ITER_OWN" and code in {"YIELD_FRONT", "YIELD_BACK"}:
        returned_owners = "Exactly one endpoint payload owner removed from the cursor live interval before result liveness."
        destroyed_owners = "NONE"
    elif profile_id == "ITER_OWN" and code == "CLOSE_OR_DROP":
        returned_owners = "NONE"
        retained_owners = "NONE"
        destroyed_owners = "Exactly every owner in the pre-state live interval [front,back)."
    elif member_id == "DENSE-REPLACE" and code == "SUCCESS":
        returned_owners = "Exactly the former PLACE owner; VALUE becomes the sole PLACE owner."
        destroyed_owners = "NONE"

    return {
        "post_state": post_state,
        "result_owners": result_owners,
        "returned_owners": returned_owners,
        "retained_owners": retained_owners,
        "destroyed_owners": destroyed_owners,
        "allocation_disposition": outcome["allocation_after"],
        "borrow_invalidation": outcome["borrows_after"],
        "fact_invalidation": outcome["facts_after"],
        "normal_exit_cleanup": cleanup,
        "pre_abort_invariant": pre_abort,
    }


def build_contract_rows() -> list[dict[str, str]]:
    """Build all exact member/outcome rows without writing files."""
    profiles = _profile_by_member()
    clusters = _cluster_by_member()
    evidence, overlays, authority_caps = _coverage_authority()
    if set(MEMBER_DECLARATIONS) != _all_members():
        raise ValueError("member declaration universe mismatch")
    rows: list[dict[str, str]] = []
    for member_id in sorted(_all_members()):
        profile_id = profiles[member_id]
        declaration = MEMBER_DECLARATIONS[member_id]
        declaration_hash = _sha256(json.dumps(declaration, sort_keys=True, separators=(",", ":")).encode("utf-8"))
        for cluster_id in clusters[member_id]:
            key = (cluster_id, member_id)
            authored_caps = tuple(value for value in declaration["capability_ids"].split(",") if value and value != "NONE")
            capability_ids = tuple(dict.fromkeys(authored_caps + authority_caps.get(key, ())))
            payload_branches = overlays.get(key, ()) or (declaration["payload_branch_ids"],)
            for policy_variant_id in _policy_ids(profile_id):
                for outcome in _outcomes_for(member_id, profile_id, policy_variant_id):
                    outcome_id = f"{member_id}.{cluster_id}.OUT.{outcome['code']}"
                    contract_id = f"{member_id}::{cluster_id}::{policy_variant_id}::{outcome['code']}"
                    explicit = _explicit_owner_fields(member_id, profile_id, outcome, declaration)
                    status = "EXCLUDED_BLOCKS_NAMED_CLAIM" if profile_id == "EXCLUDED_SURFACE" else "REQUIRED_IN_LOCK"
                    if member_id in {"DENSE-EAGER-EXTRACT", "DENSE-EAGER-SPLICE"}:
                        status = "CONDITIONAL_ON_UNRESOLVED_THREE_WAY_OD-4"
                    if (
                        policy_variant_id == OD1_RECOVERABLE
                        and member_id in {"DENSE-EXTEND-ITER", "DENSE-EAGER-SPLICE"}
                    ):
                        status = (
                            "BLOCKED_UNTIL_UNKNOWN_LENGTH_RECOVERY_RULE_IS_OWNER_APPROVED"
                            if member_id == "DENSE-EXTEND-ITER"
                            else "BLOCKED_UNKNOWN_LENGTH_RECOVERY_AND_CONDITIONAL_ON_UNRESOLVED_THREE_WAY_OD-4"
                        )
                    rows.append(
                        {
                        "schema_version": SCHEMA_VERSION,
                        "contract_id": contract_id,
                        "member_contract_id": member_id,
                        "outcome_id": outcome_id,
                        "cluster_id": cluster_id,
                        "policy_variant_id": policy_variant_id,
                        "profile_id": profile_id,
                        "status": status,
                        "evidence_identity_ids": ",".join(evidence[key]),
                        "trigger": outcome["trigger"],
                        "pre_state": declaration["pre_state"],
                        "offered_owners": declaration["offered_owners"],
                        "behavior_call_count_order_effects": f"{declaration['behavior']} Outcome branch calls: {outcome['behavior_calls']}.",
                        "commitment_point": outcome["commit_phase"],
                        "post_state": explicit["post_state"],
                        "result_owners": explicit["result_owners"],
                        "returned_owners": explicit["returned_owners"],
                        "retained_owners": explicit["retained_owners"],
                        "destroyed_owners": explicit["destroyed_owners"],
                        "allocation_disposition": explicit["allocation_disposition"],
                        "borrow_invalidation": explicit["borrow_invalidation"],
                        "fact_invalidation": explicit["fact_invalidation"],
                        "normal_exit_cleanup": explicit["normal_exit_cleanup"],
                        "pre_abort_invariant": explicit["pre_abort_invariant"],
                        "resource_ceiling": declaration["resource_ceiling"],
                        "capability_ids": ",".join(capability_ids) or "NONE",
                        "payload_branch_ids": ",".join(payload_branches),
                        "scenario_ids": declaration["scenario_ids"],
                        "commit_phase": outcome["commit_phase"],
                        "owners_before": outcome["owners_before"],
                        "owners_after": outcome["owners_after"],
                        "allocation_before": outcome["allocation_before"],
                        "allocation_after": outcome["allocation_after"],
                        "borrows_before": outcome["borrows_before"],
                        "borrows_after": outcome["borrows_after"],
                        "facts_before": outcome["facts_before"],
                        "facts_after": outcome["facts_after"],
                        "behavior_calls": outcome["behavior_calls"],
                        "result_schema": outcome["result_schema"],
                        "state_equation": outcome["state_equation"],
                        "member_declaration_sha256": declaration_hash,
                        "owner_role_foreign_key": f"OWNER-ROLE::{contract_id}",
                        "stored_borrow_route_ids": STORED_BORROW_ROUTE_BY_MEMBER.get(member_id, "NONE"),
                        "od4_policy_options": (
                            f"{OD4_EAGER_SCOPED},{OD4_EAGER_ONLY},{OD4_PROMOTE_LAZY}"
                            if member_id in {"DENSE-EAGER-EXTRACT", "DENSE-EAGER-SPLICE"}
                            else "NOT_APPLICABLE"
                        ),
                        "zst_policy_foreign_key": f"{OD3_INCLUDE_ZST},{OD3_DEFER_ZST}",
                        "candidate_execution_authorized": "NO",
                        }
                    )
    return rows


OWNER_ROLE_KEYS = (
    "live_slots",
    "nested_input_owners",
    "returned_owners",
    "retained_external_owners",
    "destroyed_owners",
    "protocol_owned_state",
)

OFFERED_VALUE_MEMBERS = frozenset(
    {"DENSE-INSERT", "DENSE-INSERT-UNIQ", "DENSE-PUSH", "DENSE-PUSH-UNIQ", "DENSE-REPLACE"}
)
PRODUCER_MEMBERS = frozenset(
    {"DENSE-COLLECT", "DENSE-EAGER-SPLICE", "DENSE-EXTEND-ITER", "DENSE-FILL-WITH", "DENSE-RESIZE-WITH"}
)
CLONE_RESULT_MEMBERS = frozenset(
    {
        "DENSE-CLONE-FROM", "DENSE-CONCAT", "DENSE-EXTEND-CLONE", "DENSE-EXTEND-WITHIN",
        "DENSE-FILL-CLONE", "DENSE-FRESH-CLONE", "DENSE-INIT-CLONE", "DENSE-JOIN",
        "DENSE-REPEAT", "DENSE-RESIZE-CLONE",
    }
)


def _owner_role_before(member_id: str, transition_id: str) -> dict[str, str]:
    roles = {key: "EMPTY" for key in OWNER_ROLE_KEYS}
    if transition_id == "STATIC_REJECT":
        return roles
    if transition_id in {"CONSTRUCT_EMPTY", "CONSTRUCT_VALUES", "INIT"}:
        if transition_id == "INIT":
            roles["protocol_owned_state"] = "SET(DEST.ALLOCATION)"
        elif member_id == "DENSE-COLLECT":
            roles["protocol_owned_state"] = "SET(PRODUCER)"
        elif transition_id == "CONSTRUCT_VALUES":
            roles["protocol_owned_state"] = "SET(INPUT.AUTHORITY)"
    else:
        roles["live_slots"] = "ORDERED_SET(BASE.PAYLOAD[0..L))"
        roles["protocol_owned_state"] = "SET(BASE)"

    nested: list[str] = []
    protocol: list[str] = []
    if roles["protocol_owned_state"] != "EMPTY":
        protocol.append(roles["protocol_owned_state"].removeprefix("SET(").removesuffix(")"))
    if member_id in OFFERED_VALUE_MEMBERS:
        nested.append("OFFERED.VALUE")
    if member_id == "DENSE-APPEND-MOVE":
        nested.append("SOURCE.PAYLOAD[0..S)")
        protocol.append("SOURCE")
    if member_id in PRODUCER_MEMBERS:
        nested.append("PRODUCER.RESULTS[0..N)")
        if "PRODUCER" not in protocol:
            protocol.append("PRODUCER")
    if member_id in CLONE_RESULT_MEMBERS:
        nested.append("BEHAVIOR.RESULTS[0..K)")
    if member_id == "DENSE-FIXED-MAP":
        nested.extend(("ARRAY.PAYLOAD[0..N)", "BEHAVIOR.RESULTS[0..N)"))
        protocol.append("ARRAY")
    if member_id == "DENSE-CONVERT":
        nested.append("INPUT.PAYLOAD[0..N)")
    if member_id in {"DENSE-RESIZE-CLONE", "DENSE-FILL-CLONE"}:
        nested.append("OFFERED.SEED")
    if member_id == "DENSE-TAKE-WITH-DEFAULT":
        nested.append("BEHAVIOR.DEFAULT_RESULT")
    if nested:
        roles["nested_input_owners"] = "DISJOINT_UNION(" + ",".join(dict.fromkeys(nested)) + ")"
    if protocol:
        roles["protocol_owned_state"] = "SET(" + ",".join(dict.fromkeys(protocol)) + ")"
    if member_id in BEHAVIOR_ABORT_MEMBERS:
        roles["retained_external_owners"] = "SET(BEHAVIOR.ENV)"
    return roles


def _owner_role_after(
    member_id: str,
    transition_id: str,
    outcome_code: str,
    before: dict[str, str],
) -> dict[str, str]:
    after = {key: "EMPTY" for key in OWNER_ROLE_KEYS}
    if transition_id == "STATIC_REJECT":
        return after
    if outcome_code.endswith("TRAP") or outcome_code.endswith("ABORT"):
        return dict(before)
    if outcome_code in {
        "CHECKED_ERROR", "UNDERFILL_CLOSE_REJECTED", "OVERFILL_REJECTED",
        "CAPACITY_ERROR_RETURN", "ALLOCATION_ERROR_RETURN",
    }:
        after["live_slots"] = before["live_slots"]
        after["returned_owners"] = (
            "DISJOINT_UNION(" + ",".join(
                value for value in (
                    before["protocol_owned_state"],
                    before["nested_input_owners"],
                    before["retained_external_owners"],
                ) if value != "EMPTY"
            ) + ")"
        )
        return after

    after["retained_external_owners"] = before["retained_external_owners"]
    base_result = "SET(BASE)"
    if transition_id == "CONSTRUCT_EMPTY":
        after["live_slots"] = "EMPTY"
        after["returned_owners"] = "SET(RESULT.BASE)"
    elif transition_id in {"CONSTRUCT_VALUES", "INIT"}:
        after["live_slots"] = "ORDERED_SET(RESULT.PAYLOAD[0..N))"
        after["returned_owners"] = "SET(RESULT.BASE)"
        if member_id == "DENSE-FIXED-MAP":
            after["destroyed_owners"] = "SET(ARRAY,ARRAY.PAYLOAD[0..N))"
        elif member_id in PRODUCER_MEMBERS:
            after["destroyed_owners"] = "SET(PRODUCER)"
    elif transition_id in {"PRESERVE", "GROW_PRESERVE", "REVERSE", "ROTATE", "SWAP", "INTO_OWNER"}:
        after["live_slots"] = "ORDERED_SET(TRANSITION(BEFORE.live_slots))"
        after["returned_owners"] = base_result
    elif transition_id == "TRANSFORM":
        after["live_slots"] = f"ORDERED_SET({member_id}(BEFORE.live_slots,BEFORE.nested_input_owners))"
        after["returned_owners"] = base_result
        after["destroyed_owners"] = "SET(BEFORE.live_slots MINUS AFTER.live_slots)"
    elif transition_id in {"PUSH", "INSERT"}:
        after["live_slots"] = f"ORDERED_SET({transition_id}(BEFORE.live_slots,OFFERED.VALUE))"
        after["returned_owners"] = base_result
    elif transition_id == "APPEND":
        after["live_slots"] = "ORDERED_SET(BEFORE.live_slots ++ SOURCE.PAYLOAD[0..S))"
        after["returned_owners"] = "SET(BASE,SOURCE.EMPTY)"
    elif transition_id in {"EXTEND", "RESIZE", "EAGER_SPLICE"}:
        after["live_slots"] = f"ORDERED_SET({transition_id}(BEFORE.live_slots,BEFORE.nested_input_owners))"
        after["returned_owners"] = base_result
        if member_id in PRODUCER_MEMBERS:
            after["destroyed_owners"] = "SET(PRODUCER)"
    elif transition_id in {"POP", "POP_IF", "REMOVE", "SWAP_REMOVE", "REPLACE"}:
        if outcome_code in {"EMPTY", "PREDICATE_FALSE"}:
            after["live_slots"] = before["live_slots"]
            after["returned_owners"] = base_result
        else:
            after["live_slots"] = f"ORDERED_SET({transition_id}(BEFORE.live_slots))"
            after["returned_owners"] = "SET(BASE,RESULT.VALUE)"
    elif transition_id == "CLEAR":
        after["live_slots"] = "EMPTY"
        after["returned_owners"] = base_result
        after["destroyed_owners"] = "SET(BEFORE.live_slots)"
    elif transition_id == "TRUNCATE":
        after["live_slots"] = "ORDERED_SET(BASE.PAYLOAD[0..NEW_L))"
        after["returned_owners"] = base_result
        after["destroyed_owners"] = "SET(BASE.PAYLOAD[NEW_L..L))"
    elif transition_id == "SPLIT_OFF":
        after["live_slots"] = "ORDERED_SET(PREFIX.PAYLOAD[0..I),SUFFIX.PAYLOAD[I..L))"
        after["returned_owners"] = "SET(PREFIX,SUFFIX)"
    elif transition_id == "BORROW_ITER":
        after["live_slots"] = before["live_slots"]
        after["protocol_owned_state"] = (
            "SET(BORROW_CURSOR)" if outcome_code != "CURSOR_DESTROYED" else "SET(BASE)"
        )
    elif transition_id == "OWN_ITER":
        if outcome_code == "CURSOR_CREATED":
            after["live_slots"] = before["live_slots"]
            after["returned_owners"] = "SET(OWN_CURSOR)"
        elif outcome_code in {"YIELD_FRONT", "YIELD_BACK"}:
            after["live_slots"] = "ORDERED_SET(CURSOR.REMAINING[FRONT..BACK))"
            after["returned_owners"] = "SET(OWN_CURSOR,RESULT.VALUE)"
        elif outcome_code == "TERMINAL_NONE":
            after["returned_owners"] = "SET(OWN_CURSOR)"
        else:
            after["destroyed_owners"] = "SET(OWN_CURSOR,CURSOR.REMAINING[FRONT..BACK))"
    elif transition_id == "DROP":
        after["destroyed_owners"] = "SET(BASE,BEFORE.live_slots)"
    elif transition_id == "EAGER_EXTRACT":
        after["live_slots"] = "ORDERED_SET(RETAINED.PAYLOAD)"
        after["returned_owners"] = "SET(BASE,RESULT.REMOVED)"
    else:
        raise ValueError(f"owner-role transition is not closed: {transition_id}")
    return after


def build_owner_role_rows() -> list[dict[str, str]]:
    transitions = semantic_transition_by_member()
    rows: list[dict[str, str]] = []
    for contract in build_contract_rows():
        member_id = contract["member_contract_id"]
        transition_id = transitions[member_id]
        outcome_code = contract["outcome_id"].split(".OUT.", 1)[1]
        before = _owner_role_before(member_id, transition_id)
        after = _owner_role_after(member_id, transition_id, outcome_code, before)
        owner_role_id = contract["owner_role_foreign_key"]
        rows.append(
            {
                "schema_version": SCHEMA_VERSION,
                "owner_role_id": owner_role_id,
                "contract_id": contract["contract_id"],
                "member_contract_id": member_id,
                "outcome_id": contract["outcome_id"],
                "transition_semantics_id": f"{transition_id}.{outcome_code}",
                "before_owner_roles": json.dumps(before, sort_keys=True, separators=(",", ":")),
                "after_owner_roles": json.dumps(after, sort_keys=True, separators=(",", ":")),
                "owner_universe_equation": "DISJOINT_UNION(BEFORE.live_slots,BEFORE.nested_input_owners,BEFORE.returned_owners,BEFORE.retained_external_owners,BEFORE.destroyed_owners,BEFORE.protocol_owned_state)=DISJOINT_UNION(AFTER.live_slots,AFTER.nested_input_owners,AFTER.returned_owners,AFTER.retained_external_owners,AFTER.destroyed_owners,AFTER.protocol_owned_state)",
                "normal_result_owner_equation": (
                    "NO_NORMAL_RESULT; PRE_ABORT_PARTITION_IS_EXACT"
                    if outcome_code.endswith("TRAP") or outcome_code.endswith("ABORT")
                    else "AFTER is the exact normal result partition; extra or missing affine roles are rejected"
                ),
                "candidate_execution_authorized": "NO",
            }
        )
    return rows


def contracts_by_member() -> dict[str, tuple[dict[str, str], ...]]:
    result: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in build_contract_rows():
        result[row["member_contract_id"]].append(row)
    return {key: tuple(value) for key, value in result.items()}


def resolve_member_outcomes(
    member_contract_id: str,
    policy_variant_id: str | None = None,
) -> tuple[dict[str, str], ...]:
    rows = contracts_by_member().get(member_contract_id)
    if rows is None:
        raise KeyError(member_contract_id)
    if policy_variant_id is None:
        return rows
    selected = tuple(
        row for row in rows
        if row["policy_variant_id"] in {COMMON_POLICY, policy_variant_id}
    )
    if not selected:
        raise KeyError((member_contract_id, policy_variant_id))
    return selected


OD1_FIRST_COMMIT = {
    "DENSE-PUSH": "Mark old len live with VALUE after capacity preparation.",
    "DENSE-PUSH-UNIQ": "Mark old len live with VALUE after capacity preparation.",
    "DENSE-INSERT": "Begin the first suffix relocation after capacity preparation.",
    "DENSE-INSERT-UNIQ": "Begin the first suffix relocation after capacity preparation.",
    "DENSE-APPEND-MOVE": "Move the first SRC owner into DEST after capacity preparation.",
    "DENSE-EXTEND-CLONE": "Install the first completed clone after capacity preparation.",
    "DENSE-EXTEND-WITHIN": "Install the first completed clone after capacity preparation.",
    "DENSE-EXTEND-ITER": "Move the first yielded owner into BASE after capacity preparation.",
    "DENSE-RESIZE-CLONE": "Destroy the first removed owner on shrink or install the first growth owner.",
    "DENSE-RESIZE-WITH": "Destroy the first removed owner on shrink or install the first produced growth owner.",
    "DENSE-EAGER-SPLICE": "Move the first removed or replacement owner after all recoverable preparation.",
    "DENSE-COLLECT": "Install the first yielded owner in the new allocation.",
}


def build_od1_rows() -> list[dict[str, str]]:
    _decision_option("OD-1", OD1_RESERVE_FIRST)
    _decision_option("OD-1", OD1_RECOVERABLE)
    members = tuple(sorted(PROFILE_MEMBERS["OD1_MUTATOR"] + PROFILE_MEMBERS["OD1_CONSTRUCTOR"]))
    rows: list[dict[str, str]] = []
    for policy_id, member_id in itertools.product((OD1_RESERVE_FIRST, OD1_RECOVERABLE), members):
        constructor = member_id in PROFILE_MEMBERS["OD1_CONSTRUCTOR"]
        unknown = member_id in {"DENSE-EXTEND-ITER", "DENSE-EAGER-SPLICE", "DENSE-COLLECT"}
        if constructor:
            member_class = "CONSTRUCTOR_NOT_MUTATOR_FIXED_DIVERGENT"
            arithmetic = "Checked overflow traps before the new owner exists."
            allocation = "OOM aborts; there is no pre-existing dense owner to return."
            base_result = "NOT_APPLICABLE_NO_PREEXISTING_BASE"
            offered_result = "No recoverable result; pre-abort owner accounting remains exact."
            last_recoverable = "NONE"
            result_branch = "NO_RECOVERABLE_RESULT_BRANCH"
            status = "EXPLICITLY_OUTSIDE_RECOVERABLE_MUTATOR_SCOPE"
        elif policy_id == OD1_RESERVE_FIRST:
            member_class = "GROWTH_CAPABLE_MUTATOR"
            arithmetic = "Checked overflow traps before any owner move, destruction, callback, or allocation commitment."
            allocation = "OOM aborts; ValidDense and exact offered-owner accounting hold immediately before abort."
            base_result = "No normal failure result; a caller requiring recovery must first use DENSE-TRY-RESERVE or DENSE-TRY-RESERVE-EXACT."
            offered_result = "No normal failure result; no offered owner is read, moved, or destroyed before the pre-abort invariant."
            last_recoverable = "The separate try-reserve call returns before this mutator begins."
            result_branch = "NO_RECOVERABLE_RESULT_BRANCH"
            status = "COMPLETE_POLICY_VARIANT_PENDING_OWNER_SELECTION"
        else:
            member_class = "GROWTH_CAPABLE_MUTATOR"
            arithmetic = "Returns CapacityError before any owner move, destruction, callback, or allocation commitment."
            allocation = "Returns AllocationError before any owner move, destruction, callback, or old-allocation release."
            base_result = "Returns the byte-identical logical BASE with the same root, version, len, capacity, order, borrows, and facts."
            offered_result = "Returns every offered affine owner in its original role; no clone, destruction, or partial callback effect."
            last_recoverable = "Immediately after successful capacity acquisition and before the first destructive commit."
            result_branch = "MANDATORY_CAPACITY_OR_ALLOCATION_ERROR_BRANCH"
            status = (
                "BLOCKED_UNKNOWN_LENGTH_PRECAPACITY_RULE"
                if unknown
                else "COMPLETE_POLICY_VARIANT_PENDING_OWNER_SELECTION"
            )
        rows.append(
            {
                "schema_version": SCHEMA_VERSION,
                "policy_variant_id": policy_id,
                "member_contract_id": member_id,
                "member_class": member_class,
                "arithmetic_failure": arithmetic,
                "allocation_failure": allocation,
                "base_owner_result": base_result,
                "offered_owner_result": offered_result,
                "last_recoverable_point": last_recoverable,
                "first_destructive_commit": OD1_FIRST_COMMIT[member_id],
                "default_path_result_branch": result_branch,
                "unknown_length_rule": (
                    "General unknown-length production cannot promise unchanged BASE after a late allocation failure. "
                    "The recoverable variant therefore requires an exact checked upper bound before the first producer call; "
                    "without one, this member remains construction-blocking."
                    if unknown and policy_id == OD1_RECOVERABLE and not constructor
                    else "NOT_APPLICABLE_OR_COMPLETE"
                ),
                "status": status,
                "owner_decision_status": "UNRESOLVED_OWNER_DECISION",
            }
        )
    return rows


def build_common_substrate_rows() -> list[dict[str, str]]:
    _decision_option("OD-0", OD0_COMMON_SUBSTRATE)
    _decision_option("OD-0", OD0_SEPARATE_LOCKS)
    return [
        {
            "schema_version": SCHEMA_VERSION,
            "policy_variant_id": OD0_COMMON_SUBSTRATE,
            "status": "UNRESOLVED_RECOMMENDED_EXPERIMENT_ONLY",
            "sealing_contract": "Candidate-neutral erasable opaque/private construction sufficient for ordinary-library invariant sealing; no standard-library-only privilege.",
            "selected_existing_contracts": "AB-GENERIC,BR-REBORROW,BR-RESULT retain their already selected semantics and are charged as unimplemented common prerequisites, not candidate deltas.",
            "stateful_behavior_contract": "Direct effectful behavior may retain only its exact declared root/leaf provenance and owned call state; no partial-state or master-allocation authority is captured.",
            "checked_allocation_contract": "One identical checked F-ALLOC facade transfers an allocation-owner token on success; arithmetic/allocation failure follows the selected exact outcome; release consumes that token once.",
            "forbidden_authority": "No raw bytes, unchecked pointer, writer-set liveness, manual deallocation, forged root/version, or spare-capacity authority.",
            "candidate_binding_rule": "Every one of the five candidates binds the byte-identical observable substrate contract and adapter, including one affine owning-interval carrier with one master allocation and exactly [front,back); endpoint death precedes yield; abandonment drops the exact remainder and releases once. It has no hole, mutation, repair-to-Dense, second range, or arbitrary-liveness authority. Substrate cost is common and never attributed to one arm.",
            "no_tax_gate": "Protected programs that do not use the substrate retain identical source verdict, layout, IR, facts, calls, branches, fields, traps, and final bytes.",
            "ordinary_library_closure": "H-FLATSET/family closure remains blocked until this option is owner-selected and its experiment-only contract/no-tax artifacts pass; no production syntax is selected here.",
            "owner_decision_status": "UNRESOLVED_OWNER_DECISION",
        },
        {
            "schema_version": SCHEMA_VERSION,
            "policy_variant_id": OD0_SEPARATE_LOCKS,
            "status": "UNRESOLVED_ALTERNATE_BLOCKS_FAMILY_CLOSURE",
            "sealing_contract": "AB-SEAL is deferred to a separately approved prerequisite lock.",
            "selected_existing_contracts": "AB-GENERIC,BR-REBORROW,BR-RESULT remain selected but receive no dense-lock implementation authority.",
            "stateful_behavior_contract": "Any missing retained-state behavior support is deferred to a separate prerequisite lock.",
            "checked_allocation_contract": "The checked F-ALLOC facade is deferred to a separate prerequisite lock with identical owner/failure requirements.",
            "forbidden_authority": "The same raw/manual authority remains forbidden while prerequisites are absent.",
            "candidate_binding_rule": "Dense candidate descriptions remain conditional and cannot claim an executable ordinary-library witness; C-ATOMIC, C-LINEAR, and C-DERIVED cannot bind mandatory owning traversal without the separately locked affine interval carrier.",
            "no_tax_gate": "Each separate prerequisite lock must later prove its own protected-baseline no-tax result.",
            "ordinary_library_closure": "H-FLATSET and family closure are explicitly blocked until every prerequisite lock closes and the dense lock is rerun against their frozen interfaces.",
            "owner_decision_status": "UNRESOLVED_OWNER_DECISION",
        },
    ]


def build_stored_borrow_rows() -> list[dict[str, str]]:
    common = {
        "schema_version": SCHEMA_VERSION,
        "root_leaf_schema": "STATE has one affine outer owner and an ordered leaf ledger; each leaf is (leaf_id,mode,external_source_owner,external_root,region,version). No leaf may root in STATE storage, a receiver reborrow, or the call frame.",
        "move_transition": "Move ends the source STATE/leaf ledger before the destination STATE with byte-identical leaf IDs, external roots, regions, and versions becomes live; unique leaves are never simultaneously live.",
        "region_free_zero_tax": "When the exact monomorphized state and item types contain no borrow leaves, the leaf ledger erases to zero fields, bytes, loads, stores, branches, calls, provenance metadata, and code-size delta; owner-state operations remain the same direct operations.",
        "authorization_status": "PROTOCOL_ONLY_NO_CANDIDATE_CONSTRUCTION",
    }
    return [
        {
            **common,
            "route_id": "ACTIVE_BR_STORED::SEQ-EXTRACT-01",
            "cluster_id": "SEQ-EXTRACT-01",
            "member_contract_id": "DENSE-EAGER-EXTRACT",
            "stored_state_owner": "Owned RangeBounds R during checked range evaluation, followed by owned predicate/control state P for the complete eager traversal; no public lazy cursor is created under OD-4-EAGER-ONLY.",
            "construction_transition": "Construct R with its exact external-root leaves; each RangeBounds call consumes the prior R state and returns its exact successor; destroy R once before the first payload transition. Move P with its exact leaves into the candidate partial state.",
            "call_transition": "For each original index in increasing order, call P exactly once; surviving P leaves retain root/region/version, ended leaves end once, and relation-authorized replacement leaves name exact external roots. P never receives master allocation authority.",
            "normal_result_transition": "After the final predicate call, destroy P once with its remaining leaves; return retained BASE and RESULT.REMOVED. No R or P leaf appears in either payload result unless the declared predicate result relation explicitly produced that payload owner.",
            "destruction_transition": "Every open candidate normal exit applies its candidate lifecycle to payload ranges and destroys the currently owned P state once; R is either still in pre-payload evaluation and destroyed once, or already dead, never both.",
            "failure_transition": "Range failure returns before payload transition after exact R destruction. Predicate abort preserves the pre-abort owner/leaf partition and performs no unwind cleanup.",
            "negative_trace_ids": "ATTACK-STORED-RANGE-CALL-FRAME-LEAF,ATTACK-STORED-PREDICATE-STALE-ROOT,ATTACK-STORED-RANGE-DOUBLE-DROP,ATTACK-STORED-PREDICATE-ESCAPE",
        },
        {
            **common,
            "route_id": "ACTIVE_BR_STORED::SEQ-SPLICE-01",
            "cluster_id": "SEQ-SPLICE-01",
            "member_contract_id": "DENSE-EAGER-SPLICE",
            "stored_state_owner": "Owned RangeBounds R during checked range evaluation and owned finite replacement producer I through eager removal, replacement production, tail relocation, and completion.",
            "construction_transition": "Evaluate R through exact successor states and destroy it once before removal commitment. Move I and its exact leaves into the candidate partial state before the first producer call.",
            "call_transition": "Each next call consumes I_n and returns exactly (I_n+1,Some ITEM_n) or (I_terminal,None); ITEM leaves retain their external roots when moved into BASE; I leaves follow only the declared behavior relation.",
            "normal_result_transition": "At terminal None, destroy I_terminal once, complete tail relocation, and return BASE plus RESULT.REMOVED; every accepted ITEM owner and borrow leaf has one exact destination.",
            "destruction_transition": "An open normal exit follows the candidate lifecycle over payload ranges and destroys the exact current I state once; R is already dead after checked construction. No terminal call repeats production or I destruction.",
            "failure_transition": "Invalid range precedes payload commitment. Under recoverable OD-1, capacity/allocation failure returns unchanged BASE and exact I/offered owners before the first I call; later behavior abort has no normal result or unwind cleanup.",
            "negative_trace_ids": "ATTACK-STORED-RANGE-CALL-FRAME-LEAF,ATTACK-STORED-PRODUCER-STALE-ROOT,ATTACK-STORED-ITEM-LEAF-REBASE,ATTACK-STORED-PRODUCER-DOUBLE-DROP",
        },
        {
            **common,
            "route_id": "ACTIVE_BR_STORED::TRAIT-EXTEND-01",
            "cluster_id": "TRAIT-EXTEND-01",
            "member_contract_id": "DENSE-EXTEND-ITER",
            "stored_state_owner": "Owned finite producer I retained across every next call and capacity/payload transition; yielded ITEM owners may contain external-root borrow leaves.",
            "construction_transition": "Move I with its complete root/leaf ledger beside BASE before the first next call; under recoverable OD-1 an exact checked upper bound and capacity acquisition must complete first.",
            "call_transition": "Each next consumes I_n and yields one exact successor. Some ITEM_n moves once into BASE and retains its independent external roots; None performs zero payload move and identifies the terminal I state.",
            "normal_result_transition": "Destroy terminal I once and return BASE with yielded items in order. No producer/call-frame leaf becomes a BASE fact, allocation owner, liveness proof, or check-elision authority.",
            "destruction_transition": "Each candidate lifecycle owns and destroys the exact current I state on a permitted open exit, or rejects that exit according to its row; no writer convention supplies cleanup.",
            "failure_transition": "Recoverable preparation returns unchanged BASE and exact I before any call. Behavior abort preserves the exact current I/base partition immediately before abort and performs no unwind cleanup.",
            "negative_trace_ids": "ATTACK-STORED-PRODUCER-STALE-ROOT,ATTACK-STORED-ITEM-LEAF-REBASE,ATTACK-STORED-PRODUCER-LOSS,ATTACK-STORED-PRODUCER-EXTRA-CALL",
        },
        {
            **common,
            "route_id": "ACTIVE_BR_STORED::TRAIT-COLLECT-01",
            "cluster_id": "TRAIT-COLLECT-01",
            "member_contract_id": "DENSE-COLLECT",
            "stored_state_owner": "Owned finite producer I retained across allocation, each next call, and partial destination initialization; yielded ITEM owners may contain external-root borrow leaves.",
            "construction_transition": "Move I with its exact leaf ledger into construction authority. Acquire the checked F-ALLOC owner before writing each prepared capacity region; no raw or writer-set liveness authority is exposed.",
            "call_transition": "Each next consumes I_n and returns I_n+1 plus Some ITEM_n or terminal None; every ITEM moves once into the next live destination slot with unchanged external borrow roots.",
            "normal_result_transition": "On terminal None, destroy I once, commit the exact live prefix to RESULT.BASE, and transfer the sole allocation-owner token; no producer leaf is silently retained.",
            "destruction_transition": "A candidate open exit owns both the exact live prefix and current I state; its lifecycle rejects, exact-uses, repairs, or structurally destroys them exactly as registered.",
            "failure_transition": "Arithmetic/OOM abort preserves the pre-abort I and partial-state partition. OD-1 recoverable mutator semantics do not apply because collect has no pre-existing BASE to return.",
            "negative_trace_ids": "ATTACK-STORED-PRODUCER-STALE-ROOT,ATTACK-STORED-ITEM-LEAF-REBASE,ATTACK-STORED-PRODUCER-LOSS,ATTACK-STORED-COLLECT-PARTIAL-DROP",
        },
    ]


def build_od4_rows() -> list[dict[str, str]]:
    for option_id in (OD4_EAGER_SCOPED, OD4_EAGER_ONLY, OD4_PROMOTE_LAZY):
        _decision_option("OD-4", option_id)
    return [
        {
            "schema_version": SCHEMA_VERSION,
            "policy_variant_id": OD4_EAGER_SCOPED,
            "status": "UNRESOLVED_RECOMMENDED_OPTION",
            "mandatory_operations": "Eager owning-result extract/splice plus nonescaping direct monomorphized scoped consume/fold for extract/removal and replacement where applicable.",
            "space_contract": "Scoped noncollecting consumption uses O(1) auxiliary container state and no persistent repair cursor; eager owning-result operations retain their exact output owner.",
            "call_order_contract": "Scoped consumer is called exactly once per visited selected owner in increasing source order; early normal stop performs zero later calls; state transition order is exact and behavior effects are not replayed.",
            "normal_exit_contract": "Every normal continue, early stop, error, or lexical exit completes exact per-item disposition and restores one valid dense owner before returning.",
            "escape_contract": "Scoped receiver/range/control/consumer authority cannot be returned, stored, captured, reentered, or converted to a first-class cursor.",
            "allocation_contract": "No removed-result allocation occurs for a noncollecting scoped consumer; allocation is permitted only when the caller explicitly selects a collecting consumer. Eager owning-result operations retain their separately charged output allocation.",
            "claim_boundary": "Preserves eager owner-return and O(1)-space streaming/discard capability without claiming a persistent lazy cursor.",
            "reopening_rule": "Selecting this option requires exact scoped META-5, call-order, repair, nonescape, allocation, and hostile-oracle gates in all five arms.",
            "owner_decision_status": "UNRESOLVED_OWNER_DECISION",
        },
        {
            "schema_version": SCHEMA_VERSION,
            "policy_variant_id": OD4_EAGER_ONLY,
            "status": "UNRESOLVED_NARROWER_ALTERNATE",
            "mandatory_operations": "Eager owning-result extract/splice only.",
            "space_contract": "Removed owners require an owning result and may require O(k) storage/allocation even when the caller would discard or consume them online.",
            "call_order_contract": "Eager predicate/producer calls retain their exact registered order; no scoped streaming consumer contract is provided.",
            "normal_exit_contract": "Eager operations return repaired BASE plus owning removed result; no persistent cursor exists.",
            "escape_contract": "No scoped authority or persistent cursor is admitted.",
            "allocation_contract": "Owning removed-result allocation is part of the claim and may be mandatory.",
            "claim_boundary": "Explicitly loses streaming/discard/removal-consumer parity when eager materialization is materially slower; it cannot claim full drain/extract/splice capability.",
            "reopening_rule": "Any later streaming/discard claim reopens operation, allocation, META-5, performance, and soundness ledgers.",
            "owner_decision_status": "UNRESOLVED_OWNER_DECISION",
        },
        {
            "schema_version": SCHEMA_VERSION,
            "policy_variant_id": OD4_PROMOTE_LAZY,
            "status": "UNRESOLVED_PERSISTENT_ALTERNATE",
            "mandatory_operations": "Promote exact persistent lazy drain/extract/splice cursor contracts.",
            "space_contract": "Persistent cursor state and close/abandonment paths are primary measured obligations; no eager result allocation is mandatory for streaming use.",
            "call_order_contract": "Construction, next/next_back, repeated terminal, close, drop, and retained callable/range state receive exact outcome partitions.",
            "normal_exit_contract": "Every cursor state is safely closable/abandonable with exact tail repair, owner disposition, and allocation release.",
            "escape_contract": "Only the exact sealed cursor may escape; its complete root/version/range/callable state transfers together.",
            "allocation_contract": "Cursor and result allocation behavior is frozen per exact lazy member/outcome, including zero-allocation streaming paths.",
            "claim_boundary": "Claims full persistent lazy capability only after new exact stored-state and abandonment evidence closes.",
            "reopening_rule": "Requires new member/outcome, stored-borrow, payload-scope, lifecycle, performance, fact, META-5, and hostile-oracle registries before Lock approval.",
            "owner_decision_status": "UNRESOLVED_OWNER_DECISION",
        },
    ]


LIFECYCLE_SPECS: dict[str, dict[str, str]] = {
    "C-ATOMIC-TRANSITIONS": {
        "meta5_delta_id": "META-C-ATOMIC",
        "lifecycle_class": "LEXICAL_OPEN_EXITS_REJECTED",
        "partial_state_schema": "Sealed lexical Transition<T>{master_root,version,capacity,ordered live ranges,ordered dead ranges}; no partial-state value exists outside the scope.",
        "master_allocation_authority": "The lexical Transition<T> binder has the sole master allocation authority.",
        "allocation_release_authority": "Only transition_release may consume a payload-empty Transition<T>; transition_commit transfers authority to Dense<T>.",
        "incomplete_normal_exit": "REJECT every fallthrough, return, break, give, try, callback stop/error, or helper return that reaches the edge before exactly one transition_commit.",
        "automatic_normal_exit_action": "NONE",
        "trap_action": "Abort without commit or cleanup after preserving the exact pre-abort transition invariant.",
        "helper_rule": "The binder cannot be passed to user helpers; only closed table operations may consume/return its abstract state internally.",
        "callback_rule": "Effectful callbacks may run only without access to, capture of, or escape of the transition authority.",
        "capture_rule": "PROHIBITED",
        "escape_rule": "No return, store, field, closure environment, result, or borrow may carry Transition<T>.",
        "drop_rule": "Transition<T> has no implicit drop or repair; open normal exits are rejected.",
        "maximum_live_ranges": "Finite ranges proved by lexical transition map; no runtime per-slot state.",
        "runtime_partial_state": "NONE",
        "owning_cursor_shape": "DENSE-ITER-OWN uses only the OD-0 candidate-neutral affine single-live-interval carrier; lexical Transition<T> never escapes into it and supplies no cursor-private mechanism.",
        "owning_cursor_closure": "CONDITIONAL_ON_OD-0-COMMON-EXPERIMENTAL-SUBSTRATE; the separate-lock option blocks this arm. The common carrier owns one master allocation and exactly [front,back), drops exactly the remainder, and releases once.",
    },
    "C-LINEAR-REBUILD": {
        "meta5_delta_id": "META-C-LINEAR",
        "lifecycle_class": "TRANSITIVE_EXACT_USE",
        "partial_state_schema": "exact rebuild<T>{master_root,version,capacity,source ranges,destination ranges,offered owners,state proposition}.",
        "master_allocation_authority": "Exactly one exact rebuild<T> owns the master allocation and all partial owners.",
        "allocation_release_authority": "Only rebuild_release on a proved empty state or rebuild_commit may consume master authority.",
        "incomplete_normal_exit": "REJECT unless the exact rebuild value is consumed exactly once into commit or an exact frozen failure result on every normal path.",
        "automatic_normal_exit_action": "NONE",
        "trap_action": "Abort without exact-value consumption after preserving the pre-abort invariant.",
        "helper_rule": "Allowed only through exact parameter/result summaries; wrapping in own/affine aggregates cannot weaken exactness.",
        "callback_rule": "Callbacks cannot own or capture rebuild<T>; state may cross only helpers with exact signatures.",
        "capture_rule": "Only an exact closure/environment field with transitive exact-use checking; no ordinary capture.",
        "escape_rule": "Allowed only as an exact function result whose caller inherits the complete state proposition and exact-use obligation.",
        "drop_rule": "No implicit drop exists for exact rebuild<T>.",
        "maximum_live_ranges": "Finite proof-state ranges; exact-use checker must account all at each control-flow edge.",
        "runtime_partial_state": "NONE beyond indices/ranges already required by the algorithm.",
        "owning_cursor_shape": "DENSE-ITER-OWN uses only the OD-0 candidate-neutral affine single-live-interval carrier; exact rebuild<T> never becomes the cursor and exact-use does not cross its public calls.",
        "owning_cursor_closure": "CONDITIONAL_ON_OD-0-COMMON-EXPERIMENTAL-SUBSTRATE; the separate-lock option blocks this arm. No exact-use, repair, hole, or second range is imported into the carrier.",
    },
    "C-DERIVED-REPAIR": {
        "meta5_delta_id": "META-C-REPAIR",
        "lifecycle_class": "COMPILER_DERIVED_TOTAL_REPAIR",
        "partial_state_schema": "Sealed lexical RepairState<T>{master_root,version,capacity,exact live ranges,registered_repair_id}.",
        "master_allocation_authority": "RepairState<T> has sole master allocation authority.",
        "allocation_release_authority": "The explicit release operation or the one registered total repair may release exactly once.",
        "incomplete_normal_exit": "ACCEPT only when one registered repair exists for the exact state; emit it on every open normal exit.",
        "automatic_normal_exit_action": "Exactly one total, nonallocating, nontrapping, behavior-free repair/destruction action, surfaced in the artifact.",
        "trap_action": "Abort with no repair or cleanup after preserving the pre-abort invariant.",
        "helper_rule": "RepairState<T> cannot pass to user helpers or escape its lexical scope.",
        "callback_rule": "Callbacks cannot observe/capture state; repair itself invokes no behavior.",
        "capture_rule": "PROHIBITED",
        "escape_rule": "PROHIBITED",
        "drop_rule": "Implicit normal-exit action is exactly the registered repair; no second action or hidden flag is legal.",
        "maximum_live_ranges": "Exact finite ranges described by the registered repair proof.",
        "runtime_partial_state": "No persistent tag; cold cleanup CFG is explicit and charged.",
        "owning_cursor_shape": "DENSE-ITER-OWN uses only the OD-0 candidate-neutral affine single-live-interval carrier; RepairState<T> remains lexical and never escapes into the cursor.",
        "owning_cursor_closure": "CONDITIONAL_ON_OD-0-COMMON-EXPERIMENTAL-SUBSTRATE; the separate-lock option blocks this arm. The carrier performs structural remainder drop, never compiler-derived repair.",
    },
    "C-PROOF-CARRYING-STATE": {
        "meta5_delta_id": "META-C-PROOF",
        "lifecycle_class": "AFFINE_STRUCTURALLY_DROPPABLE_PARTITION",
        "partial_state_schema": "partition<T>{noncopyable master_root,version,capacity,zero_to_two_statically_proved_disjoint_live_ranges}.",
        "master_allocation_authority": "One affine partition<T> owns the noncopyable master allocation; no base or range owner escapes separately.",
        "allocation_release_authority": "partition_release requires zero live ranges; structural drop destroys proved ranges then releases once; partition_commit transfers master authority.",
        "incomplete_normal_exit": "Ordinary affine abandonment is legal because the partition is structurally droppable in its exact current range proposition.",
        "automatic_normal_exit_action": "Built-in structural drop of exactly zero, one, or two proved ranges, then one release; it does not repair to Dense.",
        "trap_action": "Abort without structural drop.",
        "helper_rule": "Allowed by ordinary affine move only with exact partition proposition in parameter/result summaries.",
        "callback_rule": "Callbacks cannot capture the partition or master authority; payload behavior receives only its declared value/borrow.",
        "capture_rule": "Affine capture is allowed only when the complete partition value moves and remains subject to structural drop.",
        "escape_rule": "Allowed only as partition<T> with unchanged master root/version and exact range proposition; base/ranges never escape independently.",
        "drop_rule": "Destroy exactly the statically proved ranges; never reconstruct Dense; release master once.",
        "maximum_live_ranges": "2",
        "runtime_partial_state": "No runtime tag; only algorithm indices/ranges retained for destruction may remain.",
        "owning_cursor_shape": "DENSE-ITER-OWN uses only the OD-0 candidate-neutral affine single-live-interval carrier; partition<T> is forbidden as a candidate-private substitute in measured arms.",
        "owning_cursor_closure": "CONDITIONAL_ON_OD-0-COMMON-EXPERIMENTAL-SUBSTRATE with byte-identical common adapter/artifacts/costs; the carrier has no proof-candidate hole, second range, or mutation API.",
    },
    "C-RUNTIME-TOPOLOGY": {
        "meta5_delta_id": "META-C-RUNTIME",
        "lifecycle_class": "AFFINE_STRUCTURALLY_DROPPABLE_TOPOLOGY",
        "partial_state_schema": "topology<T>{master_root,version,capacity,sealed state Dense[0,len) or Hole[0,hole_start)+[hole_end,len)}.",
        "master_allocation_authority": "One affine topology<T> owns the master allocation and sealed descriptor.",
        "allocation_release_authority": "topology_release requires empty validated ranges; structural drop selects validated ranges and releases once; topology_commit transfers authority.",
        "incomplete_normal_exit": "Ordinary affine abandonment is legal because every validated Dense/Hole state is structurally droppable.",
        "automatic_normal_exit_action": "Validate sealed metadata, destroy its one or two live ranges, then release once; never repair to Dense.",
        "trap_action": "Abort without structural drop.",
        "helper_rule": "Allowed by affine move of the complete sealed topology<T>; no field authority crosses the call.",
        "callback_rule": "Callbacks cannot capture topology authority and cannot read/write descriptor fields.",
        "capture_rule": "Affine capture only of the complete sealed topology<T> value.",
        "escape_rule": "Allowed only as topology<T>; descriptor fields and master authority cannot escape separately.",
        "drop_rule": "Validate and destroy exactly Dense or the two Hole ranges; never reconstruct Dense; release once.",
        "maximum_live_ranges": "2",
        "runtime_partial_state": "One transient Dense/Hole tag plus len,hole_start,hole_end; no bitmap, no per-slot state, and no persistent field after commit.",
        "owning_cursor_shape": "DENSE-ITER-OWN uses only the OD-0 candidate-neutral affine single-live-interval carrier; topology<T> and its Dense/Hole/Interval tags are forbidden as candidate-private substitutes in measured arms.",
        "owning_cursor_closure": "CONDITIONAL_ON_OD-0-COMMON-EXPERIMENTAL-SUBSTRATE with byte-identical common adapter/artifacts/costs; the carrier exposes only [front,back), no topology mutation or second range.",
    },
}


def build_lifecycle_rows() -> list[dict[str, str]]:
    return [
        {
            "schema_version": SCHEMA_VERSION,
            "candidate_id": candidate_id,
            "cumulative_meta5_delta_ids": ",".join(cumulative_meta5_ids(spec["meta5_delta_id"])),
            **spec,
            "construction_authorized": "NO",
        }
        for candidate_id, spec in sorted(LIFECYCLE_SPECS.items())
    ]


STATE_TYPES = {
    "C-ATOMIC-TRANSITIONS": "Transition<T>",
    "C-LINEAR-REBUILD": "exact rebuild<T>",
    "C-DERIVED-REPAIR": "RepairState<T>",
    "C-PROOF-CARRYING-STATE": "partition<T>",
    "C-RUNTIME-TOPOLOGY": "topology<T>",
}

PRIMITIVE_OPERATIONS = (
    ("BEGIN", "begin<T>(move Dense<T>) -> STATE", "Input is one ValidDense owner with no incompatible borrow.", "STATE becomes sole owner of the master allocation and all payload owners.", "Master allocation authority transfers unchanged.", "Every old payload borrow is absent.", "Facts transfer only with the same root/version and STATE owner.", "Move into partial-state authority."),
    ("INIT", "init<T>(move STATE,index:usize,move value:T) -> STATE", "index is dead, in capacity, and uniquely authorized.", "value becomes the sole live owner at index; caller value binding is dead.", "Allocation unchanged.", "No incompatible borrow may exist.", "Affected live-range facts invalidate before initialization; successor facts follow after completion.", "Initialize one dead slot."),
    ("MOVE_OUT", "move_out<T>(move STATE,index:usize) -> (STATE,T)", "index is live and uniquely authorized.", "Slot becomes dead before returned T becomes live.", "Allocation unchanged.", "Every overlapping borrow is absent.", "Affected live/range facts invalidate before the move.", "Return exactly one payload owner."),
    ("RELOCATE", "relocate<T>(move STATE,source:[usize,usize),destination:usize) -> STATE", "Source range is live; destination range is dead or exact overlap-safe successor; permutation is proved.", "Each source owner has exactly one destination and source is dead before destination liveness.", "Allocation unchanged unless invoked as the committed transfer phase after ALLOCATE.", "Every overlapping borrow is absent.", "Source/destination facts invalidate before the first payload operation; successor facts follow after the full transition.", "Direct overlap-safe relocation; no clone or source drop."),
    ("DESTROY", "destroy<T>(move STATE,range:[usize,usize)) -> STATE", "Range is exactly live and uniquely authorized.", "Every owner in range is destroyed once and range becomes dead.", "Allocation unchanged.", "Every overlapping borrow is absent.", "Range facts invalidate before the first destructor.", "Destroy exact live range in increasing logical index order."),
    ("REPLACE", "replace<T>(move STATE,index:usize,move value:T) -> (STATE,T)", "index is live and uniquely authorized; value is owned.", "value becomes sole slot owner and former owner is returned atomically.", "Allocation unchanged.", "Every overlapping borrow is absent.", "Slot facts invalidate before replacement and reestablish after.", "No placeholder, clone, double-live state, or implicit destruction."),
    ("SWAP", "swap<T>(move STATE,left:usize,right:usize) -> STATE", "Both positions are live and checked; equal is legal no-op.", "Unequal positions exchange sole owners; equal preserves one owner.", "Allocation unchanged.", "Unique authority covers the union footprint.", "Affected position facts invalidate then reestablish; equal preserves facts.", "Checked dynamic swap."),
    ("ALLOCATE_DIVERGENT", "allocate_divergent<T>(move STATE,capacity:usize) -> STATE", "Checked arithmetic succeeds and no payload owner has moved.", "STATE retains every payload owner until relocation begins.", "Success acquires one new block owner; failure aborts with pre-abort invariant and no normal result.", "No payload borrow is live.", "Old-root facts remain only until committed transfer; new-root facts require ownership.", "OD-1 reserve-first allocation edge."),
    ("ALLOCATE_RECOVERABLE", "allocate_recoverable<T>(move STATE,capacity:usize) -> Result<STATE,(STATE,AllocationError)>", "No payload owner has moved, been destroyed, or passed to behavior.", "Success returns prepared STATE; failure returns unchanged STATE and all offered owners remain external.", "Failure retains old allocation and releases any failed provisional acquisition internally.", "No payload borrow is live.", "Failure preserves prior facts exactly; success creates no payload-access fact until commit.", "OD-1 recoverable precommit edge."),
    ("RELEASE", "release<T>(move STATE) -> Unit", "STATE owns zero live payload ranges and sole master allocation authority.", "STATE becomes dead.", "Release allocation exactly once; ZST virtual root performs no allocator call.", "No borrow is live.", "Delete every STATE/root fact.", "Final allocation release."),
    ("COMMIT", "commit<T>(move STATE) -> Dense<T>", "STATE proves exactly one live prefix [0,len), one dead suffix, and ValidDense.", "Dense<T> becomes sole owner of master allocation and payload owners.", "Allocation authority transfers without acquisition or release.", "Only declared result borrows may be created after commit.", "Open-state facts end; exact Dense facts are produced for the same root and successor version.", "Sole conversion to ordinary dense owner."),
    ("RETURN_FAILURE", "return_failure<T,E>(move STATE,move offered...,error:E) -> ExactFailure<T,E>", "The exact contract is at its last recoverable point before destructive commitment.", "Returns unchanged base owner and every offered affine owner named by the exact contract.", "Old allocation/root/version are unchanged.", "Borrow ledger is unchanged and compatible.", "Fact ledger is byte-identical to the pre-call logical state.", "Only legal for a recoverable exact outcome."),
    ("EXIT_POLICY", "compiler_exit_policy<T>(move-or-open STATE) -> declared normal result", "A normal exit reaches a candidate-specific partial state.", "Owner result is exactly the lifecycle row; no writer convention fills gaps.", "Release authority follows the lifecycle row.", "No escaped incompatible borrow.", "Every open-state fact ends or transfers exactly as the lifecycle row states.", "Candidate-specific rejection, exact use, repair, or structural drop."),
)


def _candidate_slug(candidate_id: str) -> str:
    return candidate_id.removeprefix("C-").replace("-", "_")


def _primitive_operation_id(candidate_id: str, semantic_id: str) -> str:
    return f"OP-{_candidate_slug(candidate_id)}-{semantic_id}"


def _adapter_operation_id(candidate_id: str, cluster_id: str, member_id: str) -> str:
    return f"OP-{_candidate_slug(candidate_id)}-ADAPTER-{cluster_id}-{member_id}"


def _primitive_dependencies(declaration: dict[str, str], policy_variant_id: str) -> tuple[str, ...]:
    capabilities = set(declaration["capability_ids"].split(","))
    operations = ["BEGIN"]
    if "OW-INIT" in capabilities:
        operations.append("INIT")
    if "OW-MOVEOUT" in capabilities:
        operations.append("MOVE_OUT")
    if "OW-RELOCATE" in capabilities:
        operations.append("RELOCATE")
    if "OW-DROP" in capabilities:
        operations.append("DESTROY")
    if "FL-ALLOC" in capabilities:
        operations.append("ALLOCATE_RECOVERABLE" if policy_variant_id == OD1_RECOVERABLE else "ALLOCATE_DIVERGENT")
    operations.extend(("COMMIT", "EXIT_POLICY"))
    if policy_variant_id == OD1_RECOVERABLE:
        operations.append("RETURN_FAILURE")
    return tuple(dict.fromkeys(operations))


def build_operation_rows() -> list[dict[str, str]]:
    contracts = build_contract_rows()
    rows: list[dict[str, str]] = []
    for candidate_id, lifecycle in sorted(LIFECYCLE_SPECS.items()):
        state_type = STATE_TYPES[candidate_id]
        for semantic_id, signature, precondition, owner_result, allocation_result, borrow_result, fact_result, exit_effect in PRIMITIVE_OPERATIONS:
            if semantic_id == "EXIT_POLICY":
                exit_effect = lifecycle["incomplete_normal_exit"] + " Automatic action: " + lifecycle["automatic_normal_exit_action"]
            rows.append(
                {
                    "schema_version": SCHEMA_VERSION,
                    "candidate_id": candidate_id,
                    "meta5_delta_id": lifecycle["meta5_delta_id"],
                    "cumulative_meta5_delta_ids": ",".join(cumulative_meta5_ids(lifecycle["meta5_delta_id"])),
                    "operation_id": _primitive_operation_id(candidate_id, semantic_id),
                    "signature": signature.replace("STATE", state_type),
                    "precondition": precondition,
                    "owner_result": owner_result,
                    "allocation_result": allocation_result,
                    "borrow_result": borrow_result,
                    "fact_result": fact_result,
                    "normal_exit_effect": exit_effect,
                    "authorization_status": "DESCRIPTION_ONLY_CONSTRUCTION_NOT_AUTHORIZED",
                }
            )
        grouped: dict[tuple[str, str], list[dict[str, str]]] = defaultdict(list)
        for contract in contracts:
            grouped[(contract["cluster_id"], contract["member_contract_id"])].append(contract)
        for (cluster_id, member_id), member_contracts in sorted(grouped.items()):
            declaration = MEMBER_DECLARATIONS[member_id]
            policy_dependencies = {
                policy: _primitive_dependencies(declaration, policy)
                for policy in sorted({row["policy_variant_id"] for row in member_contracts})
            }
            dependency_text = ";".join(
                f"{policy}=" + ",".join(_primitive_operation_id(candidate_id, op) for op in operations)
                for policy, operations in policy_dependencies.items()
            )
            rows.append(
                {
                    "schema_version": SCHEMA_VERSION,
                    "candidate_id": candidate_id,
                    "meta5_delta_id": lifecycle["meta5_delta_id"],
                    "cumulative_meta5_delta_ids": ",".join(cumulative_meta5_ids(lifecycle["meta5_delta_id"])),
                    "operation_id": _adapter_operation_id(candidate_id, cluster_id, member_id),
                    "signature": f"adapter<{member_id}>(move-or-borrow {state_type}; {declaration['offered_owners']}) -> one exact registered outcome",
                    "precondition": declaration["pre_state"] + " Closed primitive dependencies: " + dependency_text,
                    "owner_result": "Exactly one contract_id from: " + ",".join(row["contract_id"] for row in member_contracts),
                    "allocation_result": "Exactly allocation_disposition of the selected contract_id.",
                    "borrow_result": "Exactly borrow_invalidation of the selected contract_id.",
                    "fact_result": "Exactly fact_invalidation of the selected contract_id.",
                    "normal_exit_effect": "Exactly normal_exit_cleanup of the selected contract plus the candidate lifecycle row.",
                    "authorization_status": "REFERENCE_ADAPTER_ONLY_CONSTRUCTION_NOT_AUTHORIZED",
                }
            )
    return rows


def build_binding_rows() -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for candidate_id, contract in itertools.product(sorted(LIFECYCLE_SPECS), build_contract_rows()):
        if contract["member_contract_id"] == "DENSE-ITER-OWN":
            binding_kind = "CONDITIONAL_OD0_IDENTICAL_COMMON_INTERVAL_CARRIER"
        elif contract["status"] == "EXCLUDED_BLOCKS_NAMED_CLAIM":
            binding_kind = "STATIC_REJECTION_REQUIRED"
        elif contract["status"].startswith("BLOCKED_"):
            binding_kind = "BLOCKED_OWNER_POLICY_NO_CONSTRUCTION"
        else:
            binding_kind = "PROVE_EXACT_OUTCOME"
        rows.append(
            {
                "schema_version": SCHEMA_VERSION,
                "candidate_id": candidate_id,
                "policy_variant_id": contract["policy_variant_id"],
                "contract_id": contract["contract_id"],
                "binding_kind": binding_kind,
                "lifecycle_class": LIFECYCLE_SPECS[candidate_id]["lifecycle_class"],
                "operation_id": _adapter_operation_id(candidate_id, contract["cluster_id"], contract["member_contract_id"]),
                "owner_result_source": f"{CONTRACT_OUTPUT.name}::{contract['contract_id']}",
                "common_substrate_policy_options": f"{OD0_COMMON_SUBSTRATE},{OD0_SEPARATE_LOCKS}",
                "candidate_specific_semantics_allowed": "NO",
                "zst_policy_options": f"{OD3_INCLUDE_ZST},{OD3_DEFER_ZST}",
                "construction_authorized": "NO",
            }
        )
    return rows


PAIRWISE_DISTINCTIONS = {
    ("C-ATOMIC-TRANSITIONS", "C-LINEAR-REBUILD"): ("open-exit enforcement", "Lexical open exits are rejected; authority never becomes first class.", "A first-class exact value may cross helpers but must be consumed exactly once.", "An exact token confined lexically collapses to atomic; a lexical authority passed as exact collapses to linear."),
    ("C-ATOMIC-TRANSITIONS", "C-DERIVED-REPAIR"): ("normal abandonment", "Open normal exits are rejected and no automatic action exists.", "Open normal exits run one compiler-derived total repair.", "Rejecting all open exits collapses repair to atomic; adding cleanup to atomic collapses it to repair."),
    ("C-ATOMIC-TRANSITIONS", "C-PROOF-CARRYING-STATE"): ("valid partial state", "Partial state is lexical and cannot survive an exit.", "Affine partition is always structurally droppable and may survive/escape as one master owner.", "A nonescaping partition with rejected abandonment collapses to atomic."),
    ("C-ATOMIC-TRANSITIONS", "C-RUNTIME-TOPOLOGY"): ("runtime topology", "No runtime partial-state tag or persistent partial owner exists.", "A sealed Dense/Hole runtime descriptor makes partial state droppable.", "Removing the runtime descriptor and rejecting open exits collapses topology to atomic."),
    ("C-LINEAR-REBUILD", "C-DERIVED-REPAIR"): ("must-use versus automatic action", "Every normal edge consumes the exact value; no implicit action.", "Affine abandonment is accepted through derived repair.", "Adding cleanup to exact-use or rejecting repair exits collapses the distinction."),
    ("C-LINEAR-REBUILD", "C-PROOF-CARRYING-STATE"): ("abandonment", "Abandonment is rejected transitively.", "Abandonment structurally drops proved ranges and releases once.", "Giving partition exact-use/no drop collapses to linear."),
    ("C-LINEAR-REBUILD", "C-RUNTIME-TOPOLOGY"): ("proof versus runtime state", "Exact flow proof carries all ownership state and emits no runtime topology.", "Runtime Dense/Hole metadata selects live ranges for structural drop.", "Making topology fully proof-erased and exact-use collapses to linear."),
    ("C-DERIVED-REPAIR", "C-PROOF-CARRYING-STATE"): ("repair versus structural drop", "Normal exit runs a compiler-derived repair/destruction action selected by partial state.", "Normal exit directly drops already-valid proved ranges without repairing Dense.", "A partition cleanup that reconstructs Dense collapses to repair."),
    ("C-DERIVED-REPAIR", "C-RUNTIME-TOPOLOGY"): ("compiler CFG versus sealed metadata", "Compiler emits an exact repair block and no runtime partial descriptor is required.", "Validated runtime metadata directly selects structural drop ranges; no repair block exists.", "A topology exit that runs hidden repair-to-Dense collapses to repair."),
    ("C-PROOF-CARRYING-STATE", "C-RUNTIME-TOPOLOGY"): ("static versus dynamic live ranges", "Zero-to-two ranges are statically proved and no tag selects them.", "Dense/Hole and range endpoints are runtime sealed metadata.", "A proof arm with runtime topology collapses to runtime; a runtime arm whose descriptor erases collapses to proof."),
}


def build_distinction_rows() -> list[dict[str, str]]:
    rows = []
    for (left, right), values in sorted(PAIRWISE_DISTINCTIONS.items()):
        rows.append(
            {
                "schema_version": SCHEMA_VERSION,
                "left_candidate_id": left,
                "right_candidate_id": right,
                "distinguishing_axis": values[0],
                "left_required_property": values[1],
                "right_required_property": values[2],
                "collapse_rule": values[3],
                "construction_authorized": "NO",
            }
        )
    return rows


def build_zst_rows() -> list[dict[str, str]]:
    _decision_option("OD-3", OD3_INCLUDE_ZST)
    _decision_option("OD-3", OD3_DEFER_ZST)
    return [
        {
            "schema_version": SCHEMA_VERSION,
            "policy_variant_id": OD3_INCLUDE_ZST,
            "status": "RECOMMENDED_EXACT_VARIANT_PENDING_OWNER_SELECTION",
            "payload_size": "0 bytes",
            "payload_alignment": "The payload type's declared nonzero alignment; every logical place satisfies it even when addresses coincide.",
            "logical_capacity": "usize::MAX on the exact target; len is a checked usize and push at len==usize::MAX follows the selected OD-1 overflow outcome.",
            "payload_allocation": "NONE; the dense owner holds a virtual logical root, not an allocator block.",
            "growth_allocation": "NONE; reserve, shrink, and growth never call the allocator for payload storage.",
            "length_overflow": "Checked before logical owner transfer or callback; no zero-byte exemption.",
            "owner_identity": "Each live logical index i in [0,len) owns a distinct affine token (owner_root,i,version), independent of address bits.",
            "borrow_footprint": "A borrow footprint is the logical index/range set under (owner_root,version); empty ranges have empty access footprints.",
            "disjointness_rule": "Only checked disjoint logical index/range sets establish unique compatibility; pointer inequality and pointer equality prove nothing.",
            "pointer_rule": "All or several logical places may expose equal numerical addresses; no fact, identity, ordering, alias, or distinctness authority follows.",
            "move_rule": "Moving a ZST owner changes exactly one logical slot/role and zero bytes; the source token ends before destination liveness.",
            "drop_rule": "Owner drop destroys exactly len logical owners in increasing logical index order; truncate/clear destroy their exact logical ranges; no allocator release occurs.",
            "allocator_failure_applicability": "INAPPLICABLE; allocator success/failure traces are omitted, not counted as zero-cost wins.",
            "claim_boundary": "Supports the unrestricted positive-or-zero-size region-free, borrow-free affine dense payload claim only if every candidate passes these rules.",
            "owner_decision_status": "UNRESOLVED_OWNER_DECISION",
        },
        {
            "schema_version": SCHEMA_VERSION,
            "policy_variant_id": OD3_DEFER_ZST,
            "status": "EXACT_EXCLUSION_VARIANT_PENDING_OWNER_SELECTION",
            "payload_size": "Exactly 0-byte payload instantiations are rejected at generic instantiation.",
            "payload_alignment": "NOT_APPLICABLE_REJECTED_INSTANTIATION",
            "logical_capacity": "NOT_APPLICABLE_REJECTED_INSTANTIATION",
            "payload_allocation": "NONE_BECAUSE_INSTANTIATION_IS_REJECTED",
            "growth_allocation": "NONE_BECAUSE_INSTANTIATION_IS_REJECTED",
            "length_overflow": "No ZST operation is admitted.",
            "owner_identity": "No ZST owner token is admitted by this family variant.",
            "borrow_footprint": "No ZST borrow is admitted by this family variant.",
            "disjointness_rule": "No ZST disjointness claim is made.",
            "pointer_rule": "No ZST pointer/address claim is made.",
            "move_rule": "No ZST move is admitted.",
            "drop_rule": "No ZST drop is admitted.",
            "allocator_failure_applicability": "NOT_APPLICABLE",
            "claim_boundary": "Blocks the zero-sized affine payload claim and requires generic libraries to reject or defer ZST instantiations.",
            "owner_decision_status": "UNRESOLVED_OWNER_DECISION",
        },
    ]


FACT_ROWS: tuple[dict[str, str], ...] = (
    {
        "fact_id": "FACT-DENSE-LIVE-PREFIX",
        "fact_schema_version": "1",
        "exact_proposition": "For owner O, root A, version V, len L, capacity C: 0<=L<=C and slot i is a valid readable/droppable T iff 0<=i<L.",
        "owning_root": "An exact Dense owner O, or a sealed partial-state owner explicitly normalized to the one live prefix [0,L), together with allocation/virtual root A and structural version V.",
        "producer": "Dense constructor/commit, or an explicit partial-state normalization proof that the complete live set is exactly one prefix [0,L). Hole and multi-range states cannot produce this fact.",
        "preconditions": "Sole state authority; exact owner-ledger equality LiveSet=[0,L); checked 0<=L<=C; no unfinished payload operation; no hole or additional live range.",
        "scope_and_version": "Only the dominated region where O,A,V,L,C are unchanged and live.",
        "consumers": "Checked payload access, exact live-prefix destruction, bounds-check proof, and only backend metadata implied by the quantified proposition.",
        "invalidators": "Every init, move-out, relocation, destroy, insert, remove, push, pop, truncate, clear, split, growth, shrink, owner move that changes O/A/V, and partial-state entry.",
        "move_transfer": "Transfers only with the complete owner and unchanged A,V,L,C; binding identity may change but owner identity in the report must be rewritten exactly.",
        "borrow_transfer": "A borrow receives only its exact subrange proposition; it never receives length/capacity mutation authority.",
        "call_transfer": "Requires an exact checked summary preserving O,A,V,L,C; otherwise invalidated at call.",
        "branch_join": "Retain only if every predecessor proves the identical proposition and owner/root/version values.",
        "speculation_rule": "Validity must dominate every scalar/vector load; masking the result never licenses a dead-lane load.",
        "facts_off_semantics": "Same accepted program, results, traps, owners, drops, and valid accesses; dynamic checks remain.",
        "artifact_evidence": "Emit producer transition, exact proposition values, dependency cone, every consumer, invalidator, and end site.",
        "negative_trace_ids": "ATTACK-STALE-LEN,ATTACK-BRANCH-JOIN,ATTACK-DEAD-SIMD-LANE,ATTACK-LIVE-PREFIX-HOLE",
        "candidate_realization_rule": "All candidates expose the same proposition; proof/runtime storage differs only as priced by the lifecycle/META-5 rows.",
        "authorization_status": "PROTOCOL_ONLY_NO_PRODUCTION_FACT_AUTHORITY",
    },
    {
        "fact_id": "FACT-DENSE-SLOT-LIVE",
        "fact_schema_version": "1",
        "exact_proposition": "Slot (O,A,V,i) contains exactly one valid T owner and i is in the current exact live set.",
        "owning_root": "O,A,V and logical slot i.",
        "producer": "Completed initialization, relocation destination, replacement, or validated partial-state transition.",
        "preconditions": "The source owner ended before destination liveness; i is in bounds and no conflicting owner/borrow exists.",
        "scope_and_version": "Until any event affecting i, O, A, or V.",
        "consumers": "One payload read/borrow/move/drop and exact slot metadata; never neighboring slots.",
        "invalidators": "Move-out, relocation source, replace start, destroy, structural version change, owner release, or conflicting unique borrow.",
        "move_transfer": "Ends at source before a new destination fact is produced.",
        "borrow_transfer": "Creates a bounded result fact for the exact footprint; does not transfer ownership.",
        "call_transfer": "Only through exact behavior/result relation for this owner token.",
        "branch_join": "Requires identical O,A,V,i and liveness on all incoming paths.",
        "speculation_rule": "The fact must dominate the payload load itself.",
        "facts_off_semantics": "Retain the slot-live check before access/drop.",
        "artifact_evidence": "Per-slot producer/consumer/invalidation dependency identity.",
        "negative_trace_ids": "ATTACK-MOVED-SLOT,ATTACK-WRONG-OWNER,ATTACK-WRONG-VERSION",
        "candidate_realization_rule": "No candidate may infer this fact from client-writable len or unchecked topology bytes.",
        "authorization_status": "PROTOCOL_ONLY_NO_PRODUCTION_FACT_AUTHORITY",
    },
    {
        "fact_id": "FACT-DENSE-RANGE-LIVE",
        "fact_schema_version": "1",
        "exact_proposition": "Every logical slot in half-open [B,E) under O,A,V is live exactly once and no slot outside the named union is implied live.",
        "owning_root": "O,A,V and exact endpoints B,E.",
        "producer": "Checked Dense prefix proof, partition proof, or validated topology descriptor.",
        "preconditions": "0<=B<=E<=capacity and exact owner ledger equality for the range.",
        "scope_and_version": "Until range endpoints, live membership, owner, root, or version changes.",
        "consumers": "Range access, traversal, relocation, and structural destruction over exactly [B,E).",
        "invalidators": "Any range-overlapping ownership event or root/version change.",
        "move_transfer": "A relocation consumes the source fact and produces a destination fact only after the full permutation completes.",
        "borrow_transfer": "May derive a subrange borrow fact; parent authority remains excluded according to borrow mode.",
        "call_transfer": "Exact range proposition must appear in the checked call summary.",
        "branch_join": "Identical ranges only; union/widening is forbidden without a checked proof.",
        "speculation_rule": "No load may cross B/E, including vector lanes later masked.",
        "facts_off_semantics": "Range checks and per-access validity remain.",
        "artifact_evidence": "Exact endpoints, owner/root/version, producers, consumers, and invalidation ordering.",
        "negative_trace_ids": "ATTACK-RANGE-WIDEN,ATTACK-THIRD-RANGE,ATTACK-DEAD-SIMD-LANE",
        "candidate_realization_rule": "Proof candidate permits at most two proved ranges; runtime candidate permits exactly Dense or two Hole ranges. The OD-0 common owning interval is a separate candidate-neutral carrier and never a topology<T> state.",
        "authorization_status": "PROTOCOL_ONLY_NO_PRODUCTION_FACT_AUTHORITY",
    },
    {
        "fact_id": "FACT-DENSE-CAPACITY",
        "fact_schema_version": "1",
        "exact_proposition": "Owner O exclusively retains root A for C logical T slots; for positive-size T acquired bytes satisfy checked layout, while ZST follows OD-3.",
        "owning_root": "O and allocation or virtual root A at version V.",
        "producer": "Successful allocation acquisition, representation reuse, or ZST virtual-root construction.",
        "preconditions": "One retained owner token; checked layout/capacity arithmetic; exact allocator result.",
        "scope_and_version": "Until release, reallocation/root transfer, owner destruction, or capacity change.",
        "consumers": "Capacity query, dead-slot bounds, no-grow proof, and checked allocation release.",
        "invalidators": "Growth/shrink root transfer, release, owner destruction, or capacity mutation.",
        "move_transfer": "Transfers only with master owner authority.",
        "borrow_transfer": "Payload borrows receive no allocation release or capacity mutation authority.",
        "call_transfer": "Requires exact owner/root/capacity summary.",
        "branch_join": "Identical O,A,V,C only.",
        "speculation_rule": "Capacity alone never proves slot liveness or payload validity.",
        "facts_off_semantics": "Capacity/layout and allocation-owner checks remain.",
        "artifact_evidence": "Acquisition/release credit, requested/acquired bytes, root identity, and consumers.",
        "negative_trace_ids": "ATTACK-CAPACITY-AS-LIVE,ATTACK-OLD-ROOT,ATTACK-DOUBLE-RELEASE",
        "candidate_realization_rule": "Every candidate reports the same abstract capacity; hidden extra allocation authority is forbidden.",
        "authorization_status": "PROTOCOL_ONLY_NO_PRODUCTION_FACT_AUTHORITY",
    },
    {
        "fact_id": "FACT-DENSE-NO-GROW",
        "fact_schema_version": "1",
        "exact_proposition": "For exact O,A,V,L,C and requested K, checked arithmetic proves K<=C-L, so the named operation performs no payload allocation or root change.",
        "owning_root": "O,A,V,L,C plus immutable request K.",
        "producer": "Checked subtraction/comparison under FACT-DENSE-LIVE-PREFIX and FACT-DENSE-CAPACITY.",
        "preconditions": "Both parent facts are live and arithmetic is nonwrapping.",
        "scope_and_version": "One operation branch until any L,C,O,A,V,K change.",
        "consumers": "Select no-grow reference algorithm and omit only the allocation edge, never bounds/liveness checks not otherwise proved.",
        "invalidators": "Any mutation/call changing inputs or branch join without identical proof.",
        "move_transfer": "Does not transfer independently; rederive after owner move.",
        "borrow_transfer": "No transfer.",
        "call_transfer": "Only exact monomorphized call with all values in summary.",
        "branch_join": "All incoming paths must prove the identical inequality.",
        "speculation_rule": "Cannot speculate payload access or commitment before branch proof.",
        "facts_off_semantics": "Runtime capacity comparison remains; result and allocation behavior are identical.",
        "artifact_evidence": "Arithmetic proof cone and selected allocation/no-allocation edge.",
        "negative_trace_ids": "ATTACK-STALE-NO-GROW,ATTACK-WRAP-NO-GROW",
        "candidate_realization_rule": "No candidate may replace the checked proposition with a writer assertion.",
        "authorization_status": "PROTOCOL_ONLY_NO_PRODUCTION_FACT_AUTHORITY",
    },
    {
        "fact_id": "FACT-DENSE-DISJOINT",
        "fact_schema_version": "1",
        "exact_proposition": "Footprint sets P and Q under the same O,A,V are disjoint as logical slot/range sets; addresses are irrelevant.",
        "owning_root": "O,A,V and the exact normalized footprints P,Q.",
        "producer": "Static distinct literals/ranges or one checked dynamic distinctness proof.",
        "preconditions": "Both footprints are in current live set and normalized without overflow.",
        "scope_and_version": "Until either footprint, owner, root, version, or live set changes.",
        "consumers": "Simultaneously live unique borrows and pairwise-independent transitions over exactly P,Q.",
        "invalidators": "Structural mutation, root/version change, range widening, or end of either authority.",
        "move_transfer": "Reprove for destination footprints after relocation.",
        "borrow_transfer": "Each child receives only its footprint; parent remains excluded until all children end.",
        "call_transfer": "Requires exact footprint summary; opaque calls invalidate.",
        "branch_join": "Identical normalized footprints and proof identity only.",
        "speculation_rule": "No alias metadata stronger than exact disjoint footprints.",
        "facts_off_semantics": "Dynamic overlap/distinctness check remains before simultaneous unique access.",
        "artifact_evidence": "Normalized footprints, proof producer, children, invalidation, and alias metadata sites.",
        "negative_trace_ids": "ATTACK-EQUAL-INDEX,ATTACK-OVERLAP,ATTACK-ZST-POINTER-INEQUALITY",
        "candidate_realization_rule": "All candidates use logical footprints; ZST address equality never changes the proposition.",
        "authorization_status": "PROTOCOL_ONLY_NO_PRODUCTION_FACT_AUTHORITY",
    },
    {
        "fact_id": "FACT-DENSE-ROOT-VERSION",
        "fact_schema_version": "1",
        "exact_proposition": "Physical payload authority belongs to exact abstract root A and structural version V owned by O; numerical address equality does not equate roots or versions.",
        "owning_root": "O,A,V.",
        "producer": "Owner construction, allocation acquisition, or committed root/version successor transition.",
        "preconditions": "Exact ownership transfer and prior-root invalidation complete.",
        "scope_and_version": "Until root transfer, structural version change, or owner death.",
        "consumers": "Every payload borrow/fact identity and backend provenance/alias metadata.",
        "invalidators": "Growth, shrink, split/root transfer, structural mutation named by the member, release, or owner destruction.",
        "move_transfer": "Owner binding moves may preserve A,V only when the state transition does not change structure; report new binding identity.",
        "borrow_transfer": "Borrow records exact A,V and cannot outlive its invalidator.",
        "call_transfer": "Requires exact checked summary; opaque behavior cannot retain internal root authority.",
        "branch_join": "Identical A,V only; same address is insufficient.",
        "speculation_rule": "No old-root access may be speculated after invalidation.",
        "facts_off_semantics": "Root/version checks and borrow invalidation remain; same valid-memory accesses only.",
        "artifact_evidence": "Root/version producer, transfer, invalidator, every dependent borrow/fact, and address token for same-address attacks.",
        "negative_trace_ids": "ATTACK-SAME-ADDRESS-GROW,ATTACK-OLD-ROOT,ATTACK-WRONG-VERSION",
        "candidate_realization_rule": "Every candidate exposes the same abstract root/version even if proof representation differs.",
        "authorization_status": "PROTOCOL_ONLY_NO_PRODUCTION_FACT_AUTHORITY",
    },
    {
        "fact_id": "FACT-DENSE-ZST-LOGICAL",
        "fact_schema_version": "1",
        "exact_proposition": "Under OD-3-INCLUDE-ZST, each i in [0,len) is a distinct logical affine owner token under O,V although payload size is zero and addresses may coincide.",
        "owning_root": "ZST virtual owner root O and version V; no allocator block.",
        "producer": "ZST constructor/commit and exact logical owner transitions.",
        "preconditions": "T size is zero; OD-3 include variant selected; len arithmetic checked.",
        "scope_and_version": "Until logical len/live-set/version/owner changes.",
        "consumers": "Logical move/drop cardinality, index identity, and logical footprint disjointness.",
        "invalidators": "Every logical owner transition, len change, version change, or owner death.",
        "move_transfer": "End source logical token before destination token becomes live; zero bytes move.",
        "borrow_transfer": "Borrow footprint is logical indices/ranges only.",
        "call_transfer": "Exact ZST owner/len/version summary required.",
        "branch_join": "Identical logical owner set and version only.",
        "speculation_rule": "No address comparison or zero-byte operation proves owner/disjointness facts.",
        "facts_off_semantics": "Logical owner, len, borrow, and drop checks remain.",
        "artifact_evidence": "Logical token births/moves/deaths, exact drop count, and zero allocator calls.",
        "negative_trace_ids": "ATTACK-ZST-POINTER-INEQUALITY,ATTACK-ZST-DROP-UNDERCOUNT,ATTACK-ZST-LEN-OVERFLOW",
        "candidate_realization_rule": "All candidates must preserve logical identities; no arm may encode identity only by address.",
        "authorization_status": "PROTOCOL_ONLY_NO_PRODUCTION_FACT_AUTHORITY",
    },
)


def build_fact_rows() -> list[dict[str, str]]:
    return [{"schema_version": SCHEMA_VERSION, **row} for row in FACT_ROWS]


def build_synthetic_rows() -> list[dict[str, str]]:
    clusters = _cluster_by_member()
    rows = []
    _decision_option("OD-4", OD4_EAGER_SCOPED)
    _decision_option("OD-4", OD4_EAGER_ONLY)
    _decision_option("OD-4", OD4_PROMOTE_LAZY)
    for member_id in sorted(PROTOCOL_SYNTHETIC_MEMBERS):
        for cluster_id in clusters[member_id]:
            rows.append(
                {
                    "schema_version": SCHEMA_VERSION,
                    "synthetic_identity": f"SYNTHETIC:{member_id}",
                    "cluster_id": cluster_id,
                    "member_contract_id": member_id,
                    "synthetic_class": "PROTOCOL_DERIVED_CONDITIONAL_EAGER_MEMBER",
                    "rationale": "This eager contract is protocol-derived and conditional on the unresolved three-way OD-4 decision; the recommended option also requires separately frozen scoped consume/fold contracts.",
                    "source_authority": "D13 Family Lock drafting authorization plus dense_owner_decisions OD-4-EAGER-AND-SCOPED-CONSUME, OD-4-EAGER-ONLY, and OD-4-PROMOTE-LAZY. No option is selected here; promoting lazy reopens and replaces the eager-only surface as specified by the owner decision.",
                    "permitted_contract_status": "CONDITIONAL_ON_UNRESOLVED_THREE_WAY_OD-4",
                    "candidate_execution_authorized": "NO",
                }
            )
    return rows


def _validate_complete(fields: tuple[str, ...], rows: list[dict[str, str]], name: str) -> None:
    if not rows:
        raise ValueError(f"empty registry: {name}")
    for ordinal, row in enumerate(rows, 1):
        if set(row) != set(fields):
            raise ValueError(f"{name} row {ordinal} fields mismatch")
        missing = [field for field in fields if not row[field]]
        if missing:
            raise ValueError(f"{name} row {ordinal} misses {missing}")
        if any("\t" in value or "\n" in value or "\r" in value for value in row.values()):
            raise ValueError(f"{name} row {ordinal} contains a control character")


def _authority_resolution_check(contracts: list[dict[str, str]]) -> None:
    authority_units = {
        (row["cluster_id"], row["member_contract_id"], row["subject_identity"])
        for row in _read_pinned_coverage_tsv(HERE / "DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv")
    }
    contract_units = {
        (row["cluster_id"], row["member_contract_id"], identity)
        for row in contracts
        for identity in row["evidence_identity_ids"].split(",")
        if not identity.startswith("SYNTHETIC:")
    }
    if authority_units != contract_units:
        raise ValueError(
            f"evidence-contract resolution mismatch missing={sorted(authority_units-contract_units)[:5]} "
            f"extra={sorted(contract_units-authority_units)[:5]}"
        )
    authority_pairs = {(cluster, member) for cluster, member, _ in authority_units}
    contract_pairs = {(row["cluster_id"], row["member_contract_id"]) for row in contracts}
    synthetic_pairs = {
        (row["cluster_id"], row["member_contract_id"])
        for row in build_synthetic_rows()
    }
    if contract_pairs != authority_pairs | synthetic_pairs:
        raise ValueError("coverage member pair does not resolve to exact contracts")


def validate_registries() -> dict[str, int]:
    _validate_pinned_coverage_dependencies()
    validate_meta5_rows()
    candidate_ids = set(CANDIDATE_IDS)
    if candidate_ids != set(LIFECYCLE_SPECS):
        raise ValueError("candidate lifecycle universe mismatch")

    contracts = build_contract_rows()
    owner_roles = build_owner_role_rows()
    common_substrate = build_common_substrate_rows()
    stored_borrows = build_stored_borrow_rows()
    od4 = build_od4_rows()
    od1 = build_od1_rows()
    lifecycle = build_lifecycle_rows()
    operations = build_operation_rows()
    bindings = build_binding_rows()
    distinctions = build_distinction_rows()
    zst = build_zst_rows()
    facts = build_fact_rows()
    synthetic = build_synthetic_rows()

    registries = (
        (CONTRACT_FIELDS, contracts, CONTRACT_OUTPUT.name),
        (OWNER_ROLE_FIELDS, owner_roles, OWNER_ROLE_OUTPUT.name),
        (COMMON_SUBSTRATE_FIELDS, common_substrate, COMMON_SUBSTRATE_OUTPUT.name),
        (STORED_BORROW_FIELDS, stored_borrows, STORED_BORROW_OUTPUT.name),
        (OD4_FIELDS, od4, OD4_OUTPUT.name),
        (OD1_FIELDS, od1, OD1_OUTPUT.name),
        (LIFECYCLE_FIELDS, lifecycle, LIFECYCLE_OUTPUT.name),
        (OPERATION_FIELDS, operations, OPERATIONS_OUTPUT.name),
        (BINDING_FIELDS, bindings, BINDINGS_OUTPUT.name),
        (DISTINCTION_FIELDS, distinctions, DISTINCTIONS_OUTPUT.name),
        (ZST_FIELDS, zst, ZST_OUTPUT.name),
        (FACT_FIELDS, facts, FACT_OUTPUT.name),
        (SYNTHETIC_FIELDS, synthetic, SYNTHETIC_OUTPUT.name),
    )
    for fields, rows, name in registries:
        _validate_complete(fields, rows, name)

    contract_ids = [row["contract_id"] for row in contracts]
    if len(contract_ids) != len(set(contract_ids)):
        raise ValueError("duplicate exact contract_id")
    owner_role_by_contract = {row["contract_id"]: row for row in owner_roles}
    if len(owner_role_by_contract) != len(owner_roles) or set(owner_role_by_contract) != set(contract_ids):
        raise ValueError("owner-role registry is not one-to-one with exact contracts")
    for contract in contracts:
        role = owner_role_by_contract[contract["contract_id"]]
        if role["owner_role_id"] != contract["owner_role_foreign_key"]:
            raise ValueError("owner-role foreign-key mismatch")
        for field in ("before_owner_roles", "after_owner_roles"):
            partition = json.loads(role[field])
            if tuple(sorted(partition)) != tuple(sorted(OWNER_ROLE_KEYS)):
                raise ValueError(f"owner-role partition schema mismatch: {contract['contract_id']}")
            if any(not isinstance(partition[key], str) or not partition[key] for key in OWNER_ROLE_KEYS):
                raise ValueError(f"owner-role partition value mismatch: {contract['contract_id']}")
        if role["candidate_execution_authorized"] != "NO":
            raise ValueError("owner-role registry exceeds authorization")
    forbidden = {"NORMAL_SUCCESS", "CHECKED_OR_RECOVERABLE_FAILURE", "DESTRUCTION_OR_CLOSE"}
    if any(any(marker in row["outcome_id"] for marker in forbidden) for row in contracts):
        raise ValueError("coarse union outcome leaked into exact registry")
    expected_pairs = {
        (cluster_id, member_id)
        for cluster_id, members in CLUSTER_MEMBERS.items()
        for member_id in members
    }
    actual_pairs = {(row["cluster_id"], row["member_contract_id"]) for row in contracts}
    if actual_pairs != expected_pairs:
        raise ValueError("exact contract cluster/member universe mismatch")
    if len(_all_members()) != 93:
        raise ValueError("dense exact member universe drift")
    if {row["synthetic_identity"] for row in synthetic} != {
        f"SYNTHETIC:{member}" for member in PROTOCOL_SYNTHETIC_MEMBERS
    }:
        raise ValueError("closed protocol-synthetic unit registry mismatch")
    if any(member in EXCLUDED_MEMBERS for member in PROTOCOL_SYNTHETIC_MEMBERS):
        raise ValueError("real excluded evidence surface uses synthetic authority")
    _authority_resolution_check(contracts)

    contracts_by_partition: dict[tuple[str, str, str], list[dict[str, str]]] = defaultdict(list)
    for contract in contracts:
        contracts_by_partition[
            (
                contract["cluster_id"],
                contract["member_contract_id"],
                contract["policy_variant_id"],
            )
        ].append(contract)
    for (_, member_id, _), partition in contracts_by_partition.items():
        declaration = MEMBER_DECLARATIONS[member_id]
        behavior_abort_count = sum(
            row["outcome_id"].endswith(".OUT.BEHAVIOR_ABORT") for row in partition
        )
        expected_behavior_abort_count = (
            1 if declaration["behavior_abort_applicable"] == "YES" else 0
        )
        if behavior_abort_count != expected_behavior_abort_count:
            raise ValueError(f"behavior-abort outcome partition mismatch: {member_id}")
        if declaration["behavior"] == "ZERO" and any(
            row["behavior_calls"] != "ZERO" for row in partition
        ):
            raise ValueError(f"ZERO-call member has a behavior-call outcome: {member_id}")
        if declaration["behavior"] == "ZERO" and expected_behavior_abort_count:
            raise ValueError(f"ZERO-call member permits behavior abort: {member_id}")
    if not BEHAVIOR_ABORT_MEMBERS <= set(MEMBER_DECLARATIONS):
        raise ValueError("behavior-abort applicability names an unknown member")

    if len(od1) != 2 * len(OD1_FIRST_COMMIT):
        raise ValueError("OD-1 policy/member cross-product mismatch")
    if {row["policy_variant_id"] for row in od1} != {OD1_RESERVE_FIRST, OD1_RECOVERABLE}:
        raise ValueError("OD-1 policy variant mismatch")
    if len(zst) != 2 or {row["policy_variant_id"] for row in zst} != {OD3_INCLUDE_ZST, OD3_DEFER_ZST}:
        raise ValueError("OD-3 exact variant mismatch")
    if {row["policy_variant_id"] for row in common_substrate} != {
        OD0_COMMON_SUBSTRATE, OD0_SEPARATE_LOCKS
    }:
        raise ValueError("OD-0 common-substrate variants mismatch")
    if {row["route_id"] for row in stored_borrows} != set(STORED_BORROW_ROUTE_BY_MEMBER.values()):
        raise ValueError("ACTIVE_BR_STORED semantic route registry mismatch")
    if {row["member_contract_id"] for row in stored_borrows} != set(STORED_BORROW_ROUTE_BY_MEMBER):
        raise ValueError("ACTIVE_BR_STORED member registry mismatch")
    if any("zero fields" not in row["region_free_zero_tax"] for row in stored_borrows):
        raise ValueError("stored-borrow region-free zero-tax rule is incomplete")
    if {row["policy_variant_id"] for row in od4} != {
        OD4_EAGER_SCOPED, OD4_EAGER_ONLY, OD4_PROMOTE_LAZY
    }:
        raise ValueError("OD-4 exact three-way policy registry mismatch")
    scoped_od4 = next(row for row in od4 if row["policy_variant_id"] == OD4_EAGER_SCOPED)
    scoped_text = " ".join(scoped_od4.values())
    for required in ("O(1)", "early normal stop", "cannot be returned", "No removed-result allocation"):
        if required not in scoped_text:
            raise ValueError(f"OD-4 scoped contract omits {required}")

    operation_ids = [row["operation_id"] for row in operations]
    if len(operation_ids) != len(set(operation_ids)):
        raise ValueError("duplicate candidate operation ID")
    adapter_ids = {
        _adapter_operation_id(candidate, cluster, member)
        for candidate in candidate_ids
        for cluster, member in expected_pairs
    }
    if not adapter_ids <= set(operation_ids):
        raise ValueError("missing candidate member adapter")
    if len(bindings) != len(candidate_ids) * len(contracts):
        raise ValueError("candidate-contract binding cross-product mismatch")
    binding_keys = {(row["candidate_id"], row["contract_id"]) for row in bindings}
    if len(binding_keys) != len(bindings):
        raise ValueError("duplicate candidate-contract binding")
    if any(row["operation_id"] not in set(operation_ids) for row in bindings):
        raise ValueError("candidate binding references unknown operation")
    owning_cursor_bindings = [
        row for row in bindings
        if "::DENSE-ITER-OWN" in row["operation_id"] or "DENSE-ITER-OWN::" in row["contract_id"]
    ]
    for row in owning_cursor_bindings:
        if row["binding_kind"] != "CONDITIONAL_OD0_IDENTICAL_COMMON_INTERVAL_CARRIER":
            raise ValueError(f"owning-cursor candidate closure mismatch: {row['candidate_id']}")
    cursor_adapter_by_contract: dict[str, set[str]] = defaultdict(set)
    for row in owning_cursor_bindings:
        cursor_adapter_by_contract[row["contract_id"]].add(
            row["operation_id"].split("-ADAPTER-", 1)[1]
        )
    if any(len(values) != 1 for values in cursor_adapter_by_contract.values()):
        raise ValueError("owning-cursor common adapter semantic identity mismatch")
    for row in lifecycle:
        cursor_text = row["owning_cursor_shape"] + " " + row["owning_cursor_closure"]
        if "OD-0" not in cursor_text or "candidate-neutral" not in cursor_text:
            raise ValueError(f"candidate-private owning cursor substitution: {row['candidate_id']}")
        if row["candidate_id"] in {"C-PROOF-CARRYING-STATE", "C-RUNTIME-TOPOLOGY"} and "forbidden" not in cursor_text:
            raise ValueError(f"proof/runtime cursor substitution is not forbidden: {row['candidate_id']}")
    if len(distinctions) != 10:
        raise ValueError("pairwise candidate distinction matrix incomplete")
    if len(facts) != 8 or len({row["fact_id"] for row in facts}) != 8:
        raise ValueError("fact-channel registry mismatch")
    live_prefix = next(row for row in facts if row["fact_id"] == "FACT-DENSE-LIVE-PREFIX")
    live_prefix_text = " ".join(
        live_prefix[field]
        for field in ("owning_root", "producer", "preconditions", "negative_trace_ids")
    )
    if "LiveSet=[0,L)" not in live_prefix_text or "Hole" not in live_prefix_text:
        raise ValueError("LIVE-PREFIX is not restricted away from Hole/range partial states")
    if "ATTACK-LIVE-PREFIX-HOLE" not in live_prefix["negative_trace_ids"]:
        raise ValueError("LIVE-PREFIX Hole attack is missing")
    if any(row["candidate_execution_authorized"] != "NO" for row in contracts):
        raise ValueError("contract registry exceeds authorization")
    if any(row["construction_authorized"] != "NO" for row in bindings + lifecycle + distinctions):
        raise ValueError("candidate registry exceeds authorization")

    return {
        "members": len(_all_members()),
        "evidence_bound_members": len(_all_members() - PROTOCOL_SYNTHETIC_MEMBERS),
        "synthetic_members": len(PROTOCOL_SYNTHETIC_MEMBERS),
        "cluster_member_units": len(expected_pairs),
        "contracts": len(contracts),
        "owner_roles": len(owner_roles),
        "candidate_bindings": len(bindings),
        "operations": len(operations),
        "facts": len(facts),
    }


OUTPUT_SPECS = (
    (CONTRACT_OUTPUT, CONTRACT_FIELDS, build_contract_rows),
    (OWNER_ROLE_OUTPUT, OWNER_ROLE_FIELDS, build_owner_role_rows),
    (COMMON_SUBSTRATE_OUTPUT, COMMON_SUBSTRATE_FIELDS, build_common_substrate_rows),
    (STORED_BORROW_OUTPUT, STORED_BORROW_FIELDS, build_stored_borrow_rows),
    (OD4_OUTPUT, OD4_FIELDS, build_od4_rows),
    (OD1_OUTPUT, OD1_FIELDS, build_od1_rows),
    (LIFECYCLE_OUTPUT, LIFECYCLE_FIELDS, build_lifecycle_rows),
    (OPERATIONS_OUTPUT, OPERATION_FIELDS, build_operation_rows),
    (BINDINGS_OUTPUT, BINDING_FIELDS, build_binding_rows),
    (DISTINCTIONS_OUTPUT, DISTINCTION_FIELDS, build_distinction_rows),
    (ZST_OUTPUT, ZST_FIELDS, build_zst_rows),
    (FACT_OUTPUT, FACT_FIELDS, build_fact_rows),
    (SYNTHETIC_OUTPUT, SYNTHETIC_FIELDS, build_synthetic_rows),
)


def write_registries() -> dict[str, int]:
    counts = validate_registries()
    for path, fields, builder in OUTPUT_SPECS:
        _write_tsv(path, fields, builder())
    return counts


def check_generated_files() -> dict[str, int]:
    counts = validate_registries()
    for path, fields, builder in OUTPUT_SPECS:
        if not path.exists():
            raise ValueError(f"missing generated registry: {path.name}")
        with path.open(encoding="utf-8", newline="") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            if tuple(reader.fieldnames or ()) != fields:
                raise ValueError(f"generated header drift: {path.name}")
            actual = list(reader)
        if actual != builder():
            raise ValueError(f"stale generated registry: {path.name}")
    return counts


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate generated bytes without writing")
    args = parser.parse_args()
    counts = check_generated_files() if args.check else write_registries()
    print("dense exact contract registry: PASS " + json.dumps(counts, sort_keys=True))


if __name__ == "__main__":
    main()
