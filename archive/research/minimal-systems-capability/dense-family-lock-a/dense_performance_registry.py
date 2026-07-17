#!/usr/bin/env python3
"""Generate the frozen, research-only Dense Family Lock A performance protocol.

This generator consumes one exact frozen contract registry.  It writes protocol
data and immutable input plans only.  It never constructs a candidate, builds a
candidate compiler, runs a pilot, measures a candidate, scores a result, or
authorizes any later stage.
"""

from __future__ import annotations

import csv
import hashlib
import itertools
import json
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any, Iterable


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
EXACT_CONTRACT_REGISTRY = HERE / "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv"
CONTRACT_SOURCE = HERE / "dense_contract_registry.py"

FROZEN_EXACT_CONTRACT_SHA256 = (
    "3016206708a63b858b655e81de0e5c08e21055b0c4aa4c1ce37c2561c73e3418"
)
FROZEN_CONTRACT_SOURCE_SHA256 = (
    "31dda5ccfd33202860022946fdf456404b104a14118ffcd286406595e2da2d06"
)
RUST_VERSION = "1.97.0"
RUST_COMMIT = "2d8144b7880597b6e6d3dfd63a9a9efae3f533d3"
RUSTC_SHA256 = "54c9e14255b2861a31bece6b5de29ef3fa715d0e87e27c440d05ed258bb24bf4"
RAWVEC_SHA256 = "ed847fcc6d237a1ddf95dc54bc8a3823ae89bea125f3e45cb8bb54ae0000fff0"
RUST_API_INVENTORY_SHA256 = (
    "804cc68c716af043a06fc314bcb132c8f5cce4181bb6ad4214823b5225068fb6"
)
FROZEN_CANDIDATE_OPERATIONS_SHA256 = (
    "e6309a06014fbc573974786e91597344d29c665b463bd4589553d1e8a55812bd"
)
FROZEN_CANDIDATE_BINDINGS_SHA256 = (
    "efb9336256340fd4b49177ba738fa3de1099f3e9f8157e77b6232979afee01d3"
)
FROZEN_CANDIDATE_LIFECYCLE_SHA256 = (
    "63f6c1e0cad521ea718b40b9570857e2401a2e83e4d36d1d58785b7276a041df"
)
FROZEN_COMMON_SUBSTRATE_SHA256 = (
    "99fa5fc0c0ad44033c360027a0b2d5caf2bdb65253013776995f21e145e28e3f"
)
FROZEN_OD4_SHA256 = (
    "073c206be16fb9d85cfd7d90bd3743c21633b30ef1a93fd21828d3a1a5938bdd"
)
BASELINE_COMMIT = "58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1"
PENDING = "PENDING_EXTERNAL"
NO_CROSSOVER = "OD-5-NO-CROSSOVER"
MASK64 = (1 << 64) - 1

CANDIDATES = (
    "C-ATOMIC-TRANSITIONS",
    "C-LINEAR-REBUILD",
    "C-DERIVED-REPAIR",
    "C-PROOF-CARRYING-STATE",
    "C-RUNTIME-TOPOLOGY",
)
TREATMENTS = CANDIDATES + ("RUST-1.97",)
NATIVE_TARGETS = ("TARGET-AARCH64-DARWIN", "TARGET-X86_64-LINUX")
ALL_TARGETS = NATIVE_TARGETS + ("TARGET-I686-STRUCTURAL",)
ALL_PAYLOAD_IDS = (
    "P-U8",
    "P-U64",
    "P-ROW24",
    "P-ROW56",
    "P-AFFINE24",
    "P-AFFINE64",
    "P-AFFINE256",
    "P-ZST-AFFINE",
    "P-BEHAVIOR",
)
PAYLOAD_CODES = (
    "U8",
    "U64",
    "ROW24",
    "ROW56",
    "AFFINE24",
    "AFFINE64",
    "AFFINE256",
    "ZST",
    "BEHAVIOR",
)

OUTPUTS = {
    "dispositions": "DENSE-PERFORMANCE-DISPOSITIONS.tsv",
    "operation_gates": "DENSE-PERFORMANCE-OPERATION-GATES.tsv",
    "owner_branches": "DENSE-PERFORMANCE-OWNER-BRANCHES.tsv",
    "common_substrate": "DENSE-PERFORMANCE-COMMON-SUBSTRATE.tsv",
    "algorithms": "DENSE-PERFORMANCE-ALGORITHMS.tsv",
    "references": "DENSE-PERFORMANCE-REFERENCE-ROUTES.tsv",
    "payloads": "DENSE-PERFORMANCE-PAYLOADS.tsv",
    "targets": "DENSE-PERFORMANCE-TARGETS.tsv",
    "layouts": "DENSE-PERFORMANCE-LAYOUTS.tsv",
    "allocators": "DENSE-PERFORMANCE-ALLOCATORS.tsv",
    "growth": "DENSE-PERFORMANCE-GROWTH-POLICIES.tsv",
    "endpoints": "DENSE-PERFORMANCE-ENDPOINTS.tsv",
    "counter_policies": "DENSE-PERFORMANCE-COUNTER-POLICIES.tsv",
    "structural": "DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv",
    "controls": "DENSE-PERFORMANCE-CONTROLS.tsv",
    "failures": "DENSE-PERFORMANCE-FAILURE-POLICIES.tsv",
    "warmups": "DENSE-PERFORMANCE-WARMUPS.tsv",
    "repetitions": "DENSE-PERFORMANCE-REPETITIONS.tsv",
    "facts": "DENSE-PERFORMANCE-FACTS-POLICIES.tsv",
    "generators": "DENSE-PERFORMANCE-GENERATORS.tsv",
    "schedules": "DENSE-PERFORMANCE-SCHEDULES.tsv",
    "distributions": "DENSE-PERFORMANCE-DISTRIBUTIONS.tsv",
    "blockers": "DENSE-PERFORMANCE-BLOCKERS.tsv",
    "matrix": "DENSE-PERFORMANCE-MATRIX.tsv",
    "descriptors": "DENSE-PERFORMANCE-INPUT-DESCRIPTORS.jsonl",
    "inputs": "DENSE-PERFORMANCE-INPUTS.jsonl",
    "statistics": "DENSE-PERFORMANCE-STATISTICS.json",
    "protocol": "DENSE-PERFORMANCE-PROTOCOL.md",
    "summary": "DENSE-PERFORMANCE-REGISTRY-SUMMARY.json",
}

SCHEMAS = {
    "dispositions": [
        "performance_unit_id", "contract_id", "member_contract_id",
        "outcome_id", "cluster_id", "policy_variant_id", "profile_id",
        "source_contract_status", "source_contract_sha256",
        "derivation_kind", "disposition", "representative_operation_gate_id",
        "structural_gate_ids", "functional_oracle_id", "exact_reason",
        "status",
    ],
    "operation_gates": [
        "operation_gate_id", "member_contract_id", "policy_variant_id",
        "profile_id", "representative_contract_ids",
        "representative_contract_set_sha256", "rust_reference_route_id",
        "rust_floor_upper_ratio", "required_shape_ids",
        "required_native_target_ids", "primary_cell_ids",
        "primary_cell_set_sha256", "derivation_reason", "status",
    ],
    "owner_branches": [
        "branch_id", "branch_class", "od0_option_id", "od1_option_id",
        "od2_option_id", "od3_option_id", "od4_option_id",
        "od5_option_id", "required_target_ids", "required_payload_ids",
        "primary_cell_count", "primary_cell_ids_sha256", "power_plan_id",
        "selection_scope", "blocked_or_reopening_reason", "status",
    ],
    "common_substrate": [
        "candidate_id", "od0_option_id", "sealing_contract",
        "generic_contract", "reborrow_contract", "result_provenance_contract",
        "allocator_contract", "affine_interval_contract",
        "owning_cursor_contract", "cost_accounting_contract",
        "private_substitution_rule", "substrate_contract_sha256",
        "cost_model_sha256", "status", "blocking_fact",
    ],
    "algorithms": [
        "algorithm_id", "operation_gate_id", "member_contract_id",
        "contract_ids", "exact_trigger_and_prestate",
        "exact_commit_and_poststate", "behavior_schedule",
        "allocation_and_capacity_rule", "resource_ceiling",
        "rust_reference_route_id", "member_declaration_sha256", "status",
    ],
    "references": [
        "reference_route_id", "reference_kind", "contract_route",
        "rust_version", "rust_commit", "source_paths", "source_sha256s",
        "rawvec_sha256", "rustc_sha256", "baseline_commit",
        "capacity_and_allocator_rule", "adapter_rule", "status",
        "blocking_fact",
    ],
    "payloads": [
        "payload_id", "payload_code", "semantic_class", "declared_bytes",
        "declared_alignment", "behavior_contract", "logical_byte_rule",
        "capacity_rule", "allocator_byte_rule", "destruction_rule",
        "latency_rule", "status",
    ],
    "targets": [
        "target_id", "triple", "measurement_scope", "machine_identity",
        "os_kernel_libc", "cpu_features", "power_affinity_thermal_noise",
        "layout_id", "toolchain_identity", "required_for_selection",
        "status", "blocking_fact",
    ],
    "layouts": [
        "layout_id", "triple", "rust_1_97_datalayout",
        "candidate_module_equality_rule", "pointer_bits", "endianness",
        "payload_layout_rule", "protected_control_rule", "status",
        "blocking_fact",
    ],
    "allocators": [
        "allocator_id", "applies_to", "implementation_identity",
        "adapter_sha256", "common_across_treatments_rule",
        "requested_byte_rule", "usable_byte_rule", "failure_rule",
        "zst_rule", "status", "blocking_fact",
    ],
    "growth": [
        "growth_policy_id", "applies_to", "capacity_formula",
        "allocation_formula", "overflow_rule", "zst_rule", "rust_source_pin",
        "status",
    ],
    "endpoints": [
        "endpoint_id", "unit", "direction", "observation",
        "estimator", "selection_role", "pooling_rule",
        "missing_data_rule", "status",
    ],
    "counter_policies": [
        "counter_policy_id", "target_id", "counter_names",
        "collection_command_template", "availability_probe",
        "repetition_and_group_rule", "raw_schema_rule",
        "unavailable_counter_rule", "claim_rule", "status", "blocking_fact",
    ],
    "structural": [
        "structural_gate_id", "applies_to", "collection_method",
        "pass_rule", "failure_consequence", "status", "blocking_fact",
    ],
    "controls": [
        "control_id", "role", "source_authority", "source_sha256",
        "artifact_authorities", "layout_oracle", "code_shape_oracle",
        "structural_oracle", "equality_rule", "required_target_ids",
        "status", "blocking_fact",
    ],
    "failures": [
        "failure_policy_id", "commit_point", "arithmetic_failure",
        "allocation_failure", "callback_failure", "owner_disposition",
        "timing_rule", "status",
    ],
    "warmups": [
        "warmup_id", "applies_to", "untimed_trace_count",
        "cache_state_rule", "candidate_symmetry", "status",
    ],
    "repetitions": [
        "repetition_id", "applies_to", "block_count_rule",
        "observations_per_block", "fresh_process_rule", "stopping_rule",
        "raw_retention_rule", "status", "blocking_fact",
    ],
    "facts": [
        "facts_policy_id", "primary_treatments", "facts_off_role",
        "facts_off_pairing", "required_reports", "selection_use",
        "mismatch_rule", "status", "blocking_fact",
    ],
    "generators": [
        "generator_id", "schema", "seed_rule", "operation_encoding",
        "trace_hash_rule", "oracle_hash_rule", "authorization", "status",
    ],
    "schedules": [
        "schedule_id", "shape_id", "operation_order",
        "operation_count_rule", "reset_rule", "status",
    ],
    "distributions": [
        "shape_id", "workload_family", "exact_definition",
        "seed_use", "independence_rule", "post_result_changes", "status",
    ],
    "blockers": [
        "blocker_id", "kind", "missing_fact", "required_resolution",
        "earliest_blocked_stage", "applicable_owner_branch_ids",
        "mechanism_failure_rule", "status",
    ],
    "matrix": [
        "cell_id", "cell_role", "operation_gate_id", "performance_unit_id",
        "contract_id", "member_contract_id", "outcome_id",
        "policy_variant_id", "derivation_reason_sha256", "shape_id",
        "payload_id", "payload_code", "target_id", "layout_id",
        "owner_branch_ids", "algorithm_id", "rust_reference_route_id",
        "rust_floor_upper_ratio", "initial_len", "initial_capacity",
        "request_count", "operation_count", "logical_bytes",
        "allocator_id", "growth_policy_id", "failure_policy_id",
        "primary_endpoint_id", "descriptive_endpoint_ids",
        "structural_gate_ids", "counter_policy_id", "warmup_id",
        "repetition_id", "facts_policy_id", "generator_id",
        "trace_sha256", "oracle_sha256", "status", "blocker_ids",
        "candidate_execution_authorized",
    ],
}


class ProtocolError(RuntimeError):
    """Raised when an authority is absent, mutable, incomplete, or inconsistent."""


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("ascii")


def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def digest_value(value: Any) -> str:
    return digest_bytes(canonical_bytes(value))


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if not rows or any(None in row for row in rows):
        raise ProtocolError(f"invalid or empty TSV authority: {path}")
    return rows


def write_tsv(
    path: Path, fields: list[str], rows: Iterable[dict[str, Any]]
) -> None:
    materialized = list(rows)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle, fieldnames=fields, delimiter="\t", lineterminator="\n"
        )
        writer.writeheader()
        for row in materialized:
            if set(row) != set(fields):
                raise ProtocolError(
                    f"schema mismatch for {path.name}: "
                    f"missing={sorted(set(fields)-set(row))} "
                    f"extra={sorted(set(row)-set(fields))}"
                )
            writer.writerow(row)


def write_jsonl(path: Path, rows: Iterable[dict[str, Any]]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        for row in rows:
            handle.write(canonical_bytes(row).decode("ascii") + "\n")


def source_row_digest(row: dict[str, str], fields: list[str]) -> str:
    return digest_bytes(
        "\t".join(row[field] for field in fields).encode("utf-8")
    )


def assert_frozen_contract_inputs() -> list[dict[str, str]]:
    if not EXACT_CONTRACT_REGISTRY.is_file() or not CONTRACT_SOURCE.is_file():
        raise ProtocolError("frozen exact contract inputs are absent")
    actual_registry = digest_bytes(EXACT_CONTRACT_REGISTRY.read_bytes())
    actual_source = digest_bytes(CONTRACT_SOURCE.read_bytes())
    if actual_registry != FROZEN_EXACT_CONTRACT_SHA256:
        raise ProtocolError(
            "exact contract registry drifted; performance generation is blocked: "
            f"expected={FROZEN_EXACT_CONTRACT_SHA256} actual={actual_registry}"
        )
    if actual_source != FROZEN_CONTRACT_SOURCE_SHA256:
        raise ProtocolError(
            "contract generator drifted; performance generation is blocked: "
            f"expected={FROZEN_CONTRACT_SOURCE_SHA256} actual={actual_source}"
        )
    downstream = {
        "DENSE-CANDIDATE-OPERATION-REGISTRY.tsv":
            FROZEN_CANDIDATE_OPERATIONS_SHA256,
        "DENSE-CANDIDATE-CONTRACT-BINDINGS.tsv":
            FROZEN_CANDIDATE_BINDINGS_SHA256,
        "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv":
            FROZEN_CANDIDATE_LIFECYCLE_SHA256,
        "DENSE-COMMON-SUBSTRATE-REGISTRY.tsv":
            FROZEN_COMMON_SUBSTRATE_SHA256,
        "DENSE-OD4-POLICY-REGISTRY.tsv": FROZEN_OD4_SHA256,
    }
    for spelling, expected in downstream.items():
        path = HERE / spelling
        if not path.is_file() or digest_bytes(path.read_bytes()) != expected:
            raise ProtocolError(
                f"frozen downstream authority drifted: {spelling}"
            )
    rows = read_tsv(EXACT_CONTRACT_REGISTRY)
    if len(rows) != 303:
        raise ProtocolError(f"expected 303 frozen contracts, found {len(rows)}")
    ids = [row["contract_id"] for row in rows]
    if len(ids) != len(set(ids)):
        raise ProtocolError("duplicate exact contract ID")
    if any(row["candidate_execution_authorized"] != "NO" for row in rows):
        raise ProtocolError("exact contract registry authorizes candidate execution")
    return rows


PRIMARY_OUTCOMES = frozenset({
    "ABSENT",
    "ALL_VALUES_DESTROYED",
    "NEXT_SOME",
    "OWNER_DESTROYED",
    "PREDICATE_FALSE",
    "PREDICATE_TRUE",
    "PRESENT",
    "SUCCESS",
    "SUCCESS_GROW",
    "SUCCESS_NO_CHANGE",
    "SUCCESS_NO_GROW",
    "SUCCESS_RELOCATE",
    "SUFFIX_DESTROYED",
    "VALUE_RETURNED",
    "YIELD_BACK",
    "YIELD_FRONT",
})
STRUCTURAL_OUTCOMES = frozenset({
    "CLOSE_OR_DROP",
    "CURSOR_CREATED",
    "CURSOR_DESTROYED",
    "EMPTY",
    "EMPTY_NO_CHANGE",
    "NO_CHANGE",
    "TERMINAL_NONE",
})
FUNCTIONAL_OUTCOMES = frozenset({
    "ALLOCATION_ERROR_RETURN",
    "BEHAVIOR_ABORT",
    "BOUNDS_TRAP",
    "CAPACITY_ERROR_RETURN",
    "CAPACITY_OVERFLOW_TRAP",
    "CHECKED_ERROR",
    "OOM_ABORT",
    "OVERFILL_REJECTED",
    "PRECONDITION_TRAP",
    "UNDERFILL_CLOSE_REJECTED",
})


def outcome_label(row: dict[str, str]) -> str:
    return row["outcome_id"].rsplit(".", 1)[-1]


def operation_gate_id(member: str, policy: str) -> str:
    return "OPG-" + digest_value([member, policy])[:20].upper()


def classify_contract(row: dict[str, str]) -> tuple[str, str, str]:
    label = outcome_label(row)
    if label == "EXCLUDED_NO_CALL":
        return (
            "EXCLUDED",
            "EXCLUDED",
            "Exact excluded surface has no executable route and blocks any "
            "performance claim that would require it.",
        )
    if label in PRIMARY_OUTCOMES:
        return (
            "REPRESENTATIVE",
            "TIMED_PRIMARY",
            "Exact normal outcome is an independent same-shape Rust-floor "
            "representative; every target and workload shape keeps its own "
            "upper-ratio gate.",
        )
    if label in STRUCTURAL_OUTCOMES:
        return (
            "STRUCTURAL",
            "STRUCTURAL_ONLY",
            "Exact state-only, empty, cursor-lifecycle, or repeated-terminal "
            "outcome is admitted only by its own deterministic structural "
            "oracle and cannot supply a timing benefit.",
        )
    if label in FUNCTIONAL_OUTCOMES:
        return (
            "FUNCTIONAL",
            "FUNCTIONAL_ONLY",
            "Exact failure, rejection, trap, or abort outcome is covered by "
            "the exact soundness oracle and failure-injection gate; it is "
            "never pooled into successful timing.",
        )
    raise ProtocolError(
        f"contract has no explicit performance derivation: {row['contract_id']}"
    )


def derive_dispositions(
    contract_rows: list[dict[str, str]],
) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    fields = list(contract_rows[0])
    authority: list[dict[str, str]] = []
    joined: list[dict[str, str]] = []
    for source in contract_rows:
        kind, disposition, reason = classify_contract(source)
        source_sha = source_row_digest(source, fields)
        gate_id = (
            operation_gate_id(
                source["member_contract_id"], source["policy_variant_id"]
            )
            if disposition == "TIMED_PRIMARY"
            else "NONE"
        )
        structural_ids = (
            "SG-EXACT-ORACLE,SG-OWNER-EVENTS,SG-ALLOCATION"
            if kind == "REPRESENTATIVE"
            else "SG-EXACT-ORACLE,SG-OWNER-EVENTS"
            if kind == "STRUCTURAL"
            else "NONE"
        )
        functional_id = (
            "DENSE-SOUNDNESS-EXACT-CONTRACT" if kind == "FUNCTIONAL" else "NONE"
        )
        exact_reason = (
            f"{reason} Contract {source['contract_id']} has frozen outcome "
            f"{outcome_label(source)}, profile {source['profile_id']}, policy "
            f"{source['policy_variant_id']}, and source row {source_sha}."
        )
        auth = {
            "contract_id": source["contract_id"],
            "derivation_kind": kind,
            "disposition": disposition,
            "representative_operation_gate_id": gate_id,
            "structural_gate_ids": structural_ids,
            "functional_oracle_id": functional_id,
            "source_contract_sha256": source_sha,
            "exact_reason": exact_reason,
            "status": "FROZEN_PROTOCOL",
        }
        authority.append(auth)
        joined.append({
            "performance_unit_id": "PERFUNIT-" + source_sha[:20].upper(),
            "contract_id": source["contract_id"],
            "member_contract_id": source["member_contract_id"],
            "outcome_id": source["outcome_id"],
            "cluster_id": source["cluster_id"],
            "policy_variant_id": source["policy_variant_id"],
            "profile_id": source["profile_id"],
            "source_contract_status": source["status"],
            "source_contract_sha256": source_sha,
            "derivation_kind": kind,
            "disposition": disposition,
            "representative_operation_gate_id": gate_id,
            "structural_gate_ids": structural_ids,
            "functional_oracle_id": functional_id,
            "exact_reason": exact_reason,
            "status": "FROZEN_PROTOCOL",
        })
    if len(authority) != len(contract_rows):
        raise ProtocolError("contract derivation is not total")
    return authority, joined


def owner_branch_base_rows() -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for od1, od2, od3 in itertools.product(
        ("OD-1-RESERVE-FIRST", "OD-1-RECOVERABLE-MUTATORS"),
        ("OD-2-DUAL-NATIVE", "OD-2-MAC-TARGET-LOCAL"),
        ("OD-3-INCLUDE-ZST", "OD-3-DEFER-ZST"),
    ):
        branch_id = "BR-" + digest_value([od1, od2, od3])[:16].upper()
        targets = (
            ",".join(NATIVE_TARGETS)
            if od2 == "OD-2-DUAL-NATIVE"
            else "TARGET-AARCH64-DARWIN"
        )
        payloads = (
            ",".join(ALL_PAYLOAD_IDS)
            if od3 == "OD-3-INCLUDE-ZST"
            else ",".join(
                payload for payload in ALL_PAYLOAD_IDS
                if payload != "P-ZST-AFFINE"
            )
        )
        rows.append({
            "branch_id": branch_id,
            "branch_class": "ACTIVE_POWER_BRANCH",
            "od0_option_id": "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE",
            "od1_option_id": od1,
            "od2_option_id": od2,
            "od3_option_id": od3,
            "od4_option_id": "OD-4-EAGER-AND-SCOPED-CONSUME",
            "od5_option_id": NO_CROSSOVER,
            "required_target_ids": targets,
            "required_payload_ids": payloads,
            "primary_cell_count": "PENDING_MATRIX",
            "primary_cell_ids_sha256": "PENDING_MATRIX",
            "power_plan_id": "POWER-" + branch_id,
            "selection_scope": (
                "DUAL_NATIVE" if od2 == "OD-2-DUAL-NATIVE"
                else "AARCH64_TARGET_LOCAL_ONLY"
            ),
            "blocked_or_reopening_reason": "NONE",
            "status": "UNRESOLVED_OWNER_DECISION",
        })
    blocked = (
        (
            "BLOCKED-OD0-SEPARATE",
            "OD-0-SEPARATE-PREREQUISITE-LOCKS",
            "OD-4-EAGER-AND-SCOPED-CONSUME",
            NO_CROSSOVER,
            "Separate prerequisite locks block candidate construction and "
            "require a regenerated comparison after their exact artifacts close.",
        ),
        (
            "REOPEN-OD4-EAGER-ONLY",
            "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE",
            "OD-4-EAGER-ONLY",
            NO_CROSSOVER,
            "Eager-only changes the required capability and removes scoped "
            "consume cells; it requires a distinct contract and power registry.",
        ),
        (
            "REOPEN-OD4-PROMOTE-LAZY",
            "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE",
            "OD-4-PROMOTE-LAZY",
            NO_CROSSOVER,
            "Persistent lazy cursors add lifecycle and abandonment operations; "
            "new exact contracts, cells, multiplicity, and power are required.",
        ),
        (
            "REOPEN-OD5-CROSSOVER",
            "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE",
            "OD-4-EAGER-AND-SCOPED-CONSUME",
            "OD-5-ENUMERATED-CROSSOVER",
            "A crossover is a sixth candidate. It reopens treatment count, "
            "Williams rows, power, global multiplicity, and every cell assignment.",
        ),
    )
    for branch_id, od0, od4, od5, reason in blocked:
        rows.append({
            "branch_id": branch_id,
            "branch_class": "BLOCKED_OR_REOPEN_REQUIRED",
            "od0_option_id": od0,
            "od1_option_id": "NOT_APPLICABLE",
            "od2_option_id": "NOT_APPLICABLE",
            "od3_option_id": "NOT_APPLICABLE",
            "od4_option_id": od4,
            "od5_option_id": od5,
            "required_target_ids": "NONE",
            "required_payload_ids": "NONE",
            "primary_cell_count": "0",
            "primary_cell_ids_sha256": digest_value([]),
            "power_plan_id": "NONE",
            "selection_scope": "NO_SELECTION_PROTOCOL",
            "blocked_or_reopening_reason": reason,
            "status": "BLOCKED_REQUIRES_NEW_LOCK",
        })
    return rows


def active_branch_ids(
    branches: list[dict[str, str]],
    policy_variant_id: str,
    payload_id: str,
    target_id: str,
) -> list[str]:
    result = []
    for branch in branches:
        if branch["branch_class"] != "ACTIVE_POWER_BRANCH":
            continue
        if (
            policy_variant_id.startswith("OD-1-")
            and policy_variant_id != branch["od1_option_id"]
        ):
            continue
        if target_id not in branch["required_target_ids"].split(","):
            continue
        if payload_id not in branch["required_payload_ids"].split(","):
            continue
        result.append(branch["branch_id"])
    return sorted(result)


SUBSTRATE_CONTRACT = {
    "od0_option_id": "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE",
    "sealing_contract": (
        "One erasable user-defined opaque representation seal with no "
        "standard-library-only privilege."
    ),
    "generic_contract": (
        "One byte-identical direct monomorphized generic and retained-state "
        "behavior-call contract."
    ),
    "reborrow_contract": (
        "One exact call-scoped reborrow relation with no root forgery."
    ),
    "result_provenance_contract": (
        "One exact declared result-provenance relation; call frames and "
        "container addresses mint no payload root."
    ),
    "allocator_contract": (
        "One checked allocation-owner facade, identical adapter, capacity "
        "policy, failure schedule, and accounting in all five arms and Rust."
    ),
    "affine_interval_contract": (
        "One affine single-live-interval owner over [front,back) and one master "
        "allocation; endpoint death precedes yield and abandonment drops the "
        "exact remainder then releases once."
    ),
    "owning_cursor_contract": (
        "Owning traversal uses the common affine interval carrier in every arm; "
        "no candidate-private cursor, allocator, repair state, or liveness map."
    ),
    "cost_accounting_contract": (
        "Layout fields, compiler work, calls, branches, checks, code bytes, "
        "allocator traffic, cursor operations, and destruction are charged "
        "identically to each arm; common cost is never subtracted."
    ),
    "private_substitution_rule": (
        "Any private allocator, private owning cursor, privileged seal, "
        "different generic path, or uncharged common helper rejects the arm."
    ),
}


def common_substrate_rows() -> list[dict[str, str]]:
    contract_sha = digest_value(SUBSTRATE_CONTRACT)
    cost_sha = digest_value({
        "cost": SUBSTRATE_CONTRACT["cost_accounting_contract"],
        "allocator": SUBSTRATE_CONTRACT["allocator_contract"],
        "cursor": SUBSTRATE_CONTRACT["owning_cursor_contract"],
    })
    return [
        {
            "candidate_id": candidate,
            **SUBSTRATE_CONTRACT,
            "substrate_contract_sha256": contract_sha,
            "cost_model_sha256": cost_sha,
            "status": "FROZEN_RESEARCH_PROTOCOL",
            "blocking_fact": "PENDING_EXTERNAL_COMMON_SUBSTRATE_ARTIFACTS",
        }
        for candidate in CANDIDATES
    ]


REFERENCE_ROUTES = [
    {
        "reference_route_id": "REF-RUST-API-INVENTORY",
        "reference_kind": "PINNED_RUST_DECLARATION_AND_SOURCE_ROUTE",
        "contract_route": (
            "Each operation gate binds its exact member_declaration_sha256 to "
            "the canonical stable Rust 1.97 inventory and exact contract row."
        ),
        "rust_version": RUST_VERSION,
        "rust_commit": RUST_COMMIT,
        "source_paths": (
            "optimizer-language-research/implementation/minimal-systems-"
            "capability/RUST-1.97.0-API-INVENTORY.tsv"
        ),
        "source_sha256s": RUST_API_INVENTORY_SHA256,
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": (
            "Every allocating route also binds REF-RUST-RAWVEC and the one "
            "common counted system allocator adapter."
        ),
        "adapter_rule": (
            "One direct safe Rust call with the exact caller-visible contract; "
            "no algorithm rewrite, private helper substitution, LTO, PGO, "
            "native tuning, or unchecked harness path."
        ),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "reference_route_id": "REF-RUST-VEC",
        "reference_kind": "PINNED_RUST_SOURCE",
        "contract_route": (
            "Rust Vec public operations and initialized-prefix implementation."
        ),
        "rust_version": RUST_VERSION,
        "rust_commit": RUST_COMMIT,
        "source_paths": "library/alloc/src/vec/mod.rs",
        "source_sha256s": (
            "93e3f8bad8de514ccd23c84d9c036405f32cf0b24c40e11f69f012b5264ad08e"
        ),
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": (
            "GROW-RUST-1.97 or GROW-RUST-EXACT-1.97 with the common allocator; "
            "capacity and usable bytes are retained as raw observations."
        ),
        "adapter_rule": "One direct safe public Rust operation per cell.",
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "reference_route_id": "REF-RUST-RAWVEC",
        "reference_kind": "PINNED_RUST_SOURCE",
        "contract_route": "Exact Rust 1.97 RawVec capacity and allocation policy.",
        "rust_version": RUST_VERSION,
        "rust_commit": RUST_COMMIT,
        "source_paths": "library/alloc/src/raw_vec/mod.rs",
        "source_sha256s": RAWVEC_SHA256,
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": (
            "Minimum nonzero capacity is 8 for one-byte elements, 4 for "
            "elements through 1024 bytes, and 1 otherwise; amortized growth "
            "uses max(required,2*old,minimum); exact reserve requests required."
        ),
        "adapter_rule": "Used only through the pinned safe Rust route.",
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "reference_route_id": "REF-RUST-EXTRACT",
        "reference_kind": "PINNED_RUST_SOURCE",
        "contract_route": (
            "Consume Rust extract_if to completion into the eager result owner."
        ),
        "rust_version": RUST_VERSION,
        "rust_commit": RUST_COMMIT,
        "source_paths": "library/alloc/src/vec/extract_if.rs",
        "source_sha256s": (
            "d12c2600203936735b761d04eeb7cc520e8aeb91f699e9f1174f42d3af19ea43"
        ),
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": (
            "Exact eager result allocation is charged; scoped-consume cells "
            "use a separately frozen no-result-allocation symmetric adapter."
        ),
        "adapter_rule": (
            "The adapter only exhausts the cursor and folds removed owners; "
            "predicate order and repair work are unchanged."
        ),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "reference_route_id": "REF-RUST-SPLICE",
        "reference_kind": "PINNED_RUST_SOURCE",
        "contract_route": (
            "Consume Rust Vec::splice to completion into the eager result owner."
        ),
        "rust_version": RUST_VERSION,
        "rust_commit": RUST_COMMIT,
        "source_paths": (
            "library/alloc/src/vec/splice.rs;library/alloc/src/vec/mod.rs"
        ),
        "source_sha256s": (
            "1d2fe169c4c4ed18c8407378052e66bae92d55d769ebbe509d37e80dfc35e9e4;"
            "93e3f8bad8de514ccd23c84d9c036405f32cf0b24c40e11f69f012b5264ad08e"
        ),
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": (
            "Base and removed-result allocation calls, requested layouts, "
            "transient old/new bytes, and final capacities are all charged."
        ),
        "adapter_rule": (
            "The adapter only exhausts the cursor and folds removed owners."
        ),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "reference_route_id": "REF-RUST-INTOITER",
        "reference_kind": "PINNED_RUST_SOURCE",
        "contract_route": "Rust shared, unique, and owning traversal routes.",
        "rust_version": RUST_VERSION,
        "rust_commit": RUST_COMMIT,
        "source_paths": (
            "library/alloc/src/vec/into_iter.rs;library/core/src/slice/iter.rs"
        ),
        "source_sha256s": (
            "87132ec3ed996091ac45b47649a288e812d40345076f9ff7f0771fb7ca51e1df;"
            "83205f82241154964b1aecef87913ec92ce8c0b99285337c04e37840594f9251"
        ),
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": (
            "Shared and unique traversal allocate zero; owning traversal "
            "releases the original allocation once after endpoint exhaustion."
        ),
        "adapter_rule": (
            "One direct fold with the same payload role and no collection."
        ),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "reference_route_id": "REF-RUST-SORT-STABLE",
        "reference_kind": "PINNED_RUST_SOURCE_CLOSURE",
        "contract_route": (
            "Rust 1.97 stable sort plus all reachable stable-sort helpers."
        ),
        "rust_version": RUST_VERSION,
        "rust_commit": RUST_COMMIT,
        "source_paths": (
            "library/core/src/slice/sort/stable/mod.rs;"
            "library/core/src/slice/sort/stable/drift.rs;"
            "library/core/src/slice/sort/stable/merge.rs;"
            "library/core/src/slice/sort/stable/quicksort.rs;"
            "library/core/src/slice/sort/stable/tiny.rs"
        ),
        "source_sha256s": (
            "3f2b2d8543a1cb91c9352e041bed49949a31cbd4b1030ec1a82cca1304cb22d3;"
            "5992b53abed257aa1c2dc2bb811d6a525294471e7a98d434f161d376100bf9c9;"
            "d98c59ba9932ff64ce5dca62bf28433ddfa9b604c7b3e4c226f29b673f8b00c0;"
            "4a41df6bc7c4e0a6fcb655fde9bf73ed62f3a675ca805f2f766919ce249d8f4c;"
            "d4ebcbff3d23b09feedeb5e4670f2a40c1e9ad0618937ea41391bdd5e2645da2"
        ),
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": (
            "Every scratch allocation and byte is charged with the common adapter."
        ),
        "adapter_rule": (
            "Comparator or key calls, input order, scratch, and result digest "
            "are exact for each independent sort shape."
        ),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "reference_route_id": "REF-RUST-SORT-UNSTABLE",
        "reference_kind": "PINNED_RUST_SOURCE_CLOSURE",
        "contract_route": (
            "Rust 1.97 unstable sort plus all reachable unstable-sort helpers."
        ),
        "rust_version": RUST_VERSION,
        "rust_commit": RUST_COMMIT,
        "source_paths": (
            "library/core/src/slice/sort/unstable/mod.rs;"
            "library/core/src/slice/sort/unstable/ipnsort.rs;"
            "library/core/src/slice/sort/unstable/quicksort.rs"
        ),
        "source_sha256s": (
            "23fa044fde8d3542f5f298654923ed2426930373259e6f71670c75b80501cf9a;"
            "8fc55e114a3715dafb6b6d336b9450534ce7d45314d71faa9bd79a0c7d3e8a13;"
            "6e126fcfaaee8e0b7f70a9d94dc8a512a2ba28c93811db621ec2784a62501ccb"
        ),
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": "No scratch allocation is permitted.",
        "adapter_rule": (
            "Comparator calls, input order, and result multiset are exact for "
            "each independent sort shape."
        ),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "reference_route_id": "REF-PROTECTED-BASELINE",
        "reference_kind": "PINNED_XLANG_BASELINE",
        "contract_route": "B-FIX and B-P2 exact no-tax source and artifact gates.",
        "rust_version": "NOT_APPLICABLE",
        "rust_commit": "NOT_APPLICABLE",
        "source_paths": (
            "prototype/democ/examples/soa_kernel.xl;compiler/sources.txt;"
            "experiments/data-layout-owning-sequence/BASELINE.md"
        ),
        "source_sha256s": (
            "3770cfb059bb723a1c906309af600a9ae8ce63f1fc115ff955b27a284f57b97b;"
            "17c28914ec3cd109f0411cc8a83423623c1541be239e753e91144a66bea93f65;"
            "f32a35e05e519ee863c0769f5a50845b6fc74811bb2abfb8da8df8837ef637bb"
        ),
        "rawvec_sha256": RAWVEC_SHA256,
        "rustc_sha256": RUSTC_SHA256,
        "baseline_commit": BASELINE_COMMIT,
        "capacity_and_allocator_rule": (
            "Historical allocation and layout are exact; no new adapter exists."
        ),
        "adapter_rule": (
            "Two clean builds, exact artifacts, empty normalization allowlist."
        ),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    },
]


PAYLOADS = [
    (
        "P-U8", "U8", "COPY_SCALAR", "1", "1", "No behavior",
        "logical_bytes=len", "Normal RawVec capacity", "Count requested and usable bytes",
        "No drop behavior", "Timed separating witness required",
    ),
    (
        "P-U64", "U64", "COPY_SCALAR", "8", "8", "No behavior",
        "logical_bytes=8*len", "Normal RawVec capacity", "Count requested and usable bytes",
        "No drop behavior", "Timed separating witness required",
    ),
    (
        "P-ROW24", "ROW24", "COPY_RECORD", "24", "8", "No behavior",
        "logical_bytes=24*len", "Normal RawVec capacity", "Count requested and usable bytes",
        "No drop behavior", "Timed separating witness required",
    ),
    (
        "P-ROW56", "ROW56", "COPY_RECORD", "56", "8", "No behavior",
        "logical_bytes=56*len", "Normal RawVec capacity", "Count requested and usable bytes",
        "No drop behavior", "Timed separating witness required",
    ),
    (
        "P-AFFINE24", "AFFINE24", "AFFINE_DROP", "24", "8",
        "Unique logical identity and counted destruction",
        "logical_bytes=24*len", "Normal RawVec capacity", "Count requested and usable bytes",
        "Every owner destroyed exactly once", "Timed separating witness required",
    ),
    (
        "P-AFFINE64", "AFFINE64", "NESTED_RESOURCE", "64", "8",
        "Nested allocation owner and counted destruction",
        "logical_bytes=64*len", "Normal RawVec capacity", "Count outer and nested bytes",
        "Every outer and nested owner destroyed exactly once",
        "Timed separating witness required",
    ),
    (
        "P-AFFINE256", "AFFINE256", "AFFINE_LARGE", "256", "8",
        "Unique logical identity and counted destruction",
        "logical_bytes=256*len", "Normal RawVec capacity", "Count requested and usable bytes",
        "Every owner destroyed exactly once", "Timed separating witness required",
    ),
    (
        "P-ZST-AFFINE", "ZST", "ZERO_SIZED_AFFINE", "0", "1",
        "Unique index identity and counted destruction with zero payload bytes",
        "logical_bytes=0", "capacity=usize::MAX for the target pointer width",
        "Exactly zero allocate, grow, shrink, free, requested, usable, live, and peak bytes",
        "Every logical owner destroyed exactly once by index",
        "Operation latency may be timed; allocator bytes are structural equality only",
    ),
    (
        "P-BEHAVIOR", "BEHAVIOR", "COUNTED_BEHAVIOR", "24", "8",
        "Counted Clone, clone_from, comparison, key, mutation, and destruction",
        "logical_bytes=24*len", "Normal RawVec capacity", "Count behavior-owned bytes separately",
        "Every owner destroyed exactly once", "Timed separating witness required",
    ),
]


def payload_rows() -> list[dict[str, str]]:
    fields = [
        "payload_id", "payload_code", "semantic_class", "declared_bytes",
        "declared_alignment", "behavior_contract", "logical_byte_rule",
        "capacity_rule", "allocator_byte_rule", "destruction_rule",
        "latency_rule",
    ]
    return [
        dict(zip(fields, values), status="FROZEN_PROTOCOL")
        for values in PAYLOADS
    ]


LAYOUTS = [
    {
        "layout_id": "LAYOUT-AARCH64-DARWIN",
        "triple": "aarch64-apple-darwin",
        "rust_1_97_datalayout": (
            "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-"
            "n32:64-S128-Fn32"
        ),
        "candidate_module_equality_rule": (
            "Every candidate, facts mode, Rust control, and harness module "
            "emits this exact DataLayout."
        ),
        "pointer_bits": "64",
        "endianness": "little",
        "payload_layout_rule": (
            "Size, alignment, offsets, and stride match field by field."
        ),
        "protected_control_rule": (
            "B-FIX, B-P2, H-FLATSET, W-SMALL, and W-GAP all have an exact "
            "structural cell on this layout."
        ),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_AARCH64_CANDIDATE_MODULES",
    },
    {
        "layout_id": "LAYOUT-X86_64-LINUX",
        "triple": "x86_64-unknown-linux-gnu",
        "rust_1_97_datalayout": (
            "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-"
            "f80:128-n8:16:32:64-S128"
        ),
        "candidate_module_equality_rule": (
            "Every candidate, facts mode, Rust control, and harness module "
            "emits this exact DataLayout."
        ),
        "pointer_bits": "64",
        "endianness": "little",
        "payload_layout_rule": (
            "Size, alignment, offsets, and stride match field by field."
        ),
        "protected_control_rule": (
            "B-FIX, B-P2, H-FLATSET, W-SMALL, and W-GAP all have an exact "
            "structural cell on this layout."
        ),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_X86_RUNNER",
    },
    {
        "layout_id": "LAYOUT-I686-LINUX",
        "triple": "i686-unknown-linux-gnu",
        "rust_1_97_datalayout": (
            "e-m:e-p:32:32-p270:32:32-p271:32:32-p272:64:64-i128:128-"
            "f64:32:64-f80:32-n8:16:32-S128"
        ),
        "candidate_module_equality_rule": (
            "Every structural candidate and Rust module emits this exact DataLayout."
        ),
        "pointer_bits": "32",
        "endianness": "little",
        "payload_layout_rule": (
            "Size, alignment, offsets, stride, isize ceiling, and usize "
            "arithmetic match exactly."
        ),
        "protected_control_rule": (
            "B-FIX, B-P2, H-FLATSET, W-SMALL, and W-GAP all have an exact "
            "structural cell on this layout."
        ),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_I686_TOOLCHAIN",
    },
]

TARGETS = [
    {
        "target_id": "TARGET-AARCH64-DARWIN",
        "triple": "aarch64-apple-darwin",
        "measurement_scope": "TIMED_AND_STRUCTURAL",
        "machine_identity": "FVV39V0J1W; arm64 T8132; macOS 26.5.1 build 25F80",
        "os_kernel_libc": (
            "Darwin 25.5.0; exact dyld, libc, and allocator image hashes pending"
        ),
        "cpu_features": "generic/default; exact emitted feature string pending",
        "power_affinity_thermal_noise": (
            "Exact power, affinity, thermal, frequency, and noise probes pending"
        ),
        "layout_id": "LAYOUT-AARCH64-DARWIN",
        "toolchain_identity": (
            f"rustc {RUST_VERSION} sha256 {RUSTC_SHA256}; exact xlang, linker, "
            "runtime, counter tool, and configuration hashes pending"
        ),
        "required_for_selection": "YES_NATIVE_IUT",
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_AARCH64_ENVIRONMENT",
    },
    {
        "target_id": "TARGET-X86_64-LINUX",
        "triple": "x86_64-unknown-linux-gnu",
        "measurement_scope": "TIMED_AND_STRUCTURAL",
        "machine_identity": PENDING,
        "os_kernel_libc": PENDING,
        "cpu_features": "generic/default; exact emitted feature string pending",
        "power_affinity_thermal_noise": PENDING,
        "layout_id": "LAYOUT-X86_64-LINUX",
        "toolchain_identity": (
            f"Rust source {RUST_COMMIT}; exact rustc, xlang, linker, libc, "
            "runtime, perf, and configuration hashes pending"
        ),
        "required_for_selection": "YES_NATIVE_IUT_FOR_DUAL_BRANCH",
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_X86_RUNNER",
    },
    {
        "target_id": "TARGET-I686-STRUCTURAL",
        "triple": "i686-unknown-linux-gnu",
        "measurement_scope": "STRUCTURAL_AND_ARITHMETIC_ONLY",
        "machine_identity": "No timing runner required",
        "os_kernel_libc": "No execution environment required",
        "cpu_features": "generic/default",
        "power_affinity_thermal_noise": "NOT_APPLICABLE",
        "layout_id": "LAYOUT-I686-LINUX",
        "toolchain_identity": (
            f"Rust source {RUST_COMMIT}; exact cross-tool and xlang hashes pending"
        ),
        "required_for_selection": "YES_STRUCTURAL",
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_I686_TOOLCHAIN",
    },
]

TARGET_LAYOUT = {row["target_id"]: row["layout_id"] for row in TARGETS}
TARGET_BITS = {
    row["target_id"]: int(next(
        layout["pointer_bits"] for layout in LAYOUTS
        if layout["layout_id"] == row["layout_id"]
    ))
    for row in TARGETS
}

ALLOCATORS = [
    {
        "allocator_id": "ALLOC-COMMON-COUNTED-SYSTEM-V1",
        "applies_to": "Every positive-size treatment and Rust reference",
        "implementation_identity": (
            "One target-local system allocator behind one byte-identical "
            "counted adapter shared by all five arms and Rust"
        ),
        "adapter_sha256": PENDING,
        "common_across_treatments_rule": (
            "Source, binary, owner tag, failure ordinal, usable-byte query, and "
            "call route are byte-identical; candidate-private substitution rejects."
        ),
        "requested_byte_rule": "Record size and alignment before every call.",
        "usable_byte_rule": (
            "Record target allocator usable bytes from the one frozen primary "
            "method; retain raw request and response without normalization."
        ),
        "failure_rule": (
            "Injected failure occurs at the exact call ordinal before commit; "
            "real OOM follows the exact member contract."
        ),
        "zst_rule": "Not applicable to ZST.",
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_ALLOCATOR_ADAPTER",
    },
    {
        "allocator_id": "ALLOC-NONE-ZST",
        "applies_to": "P-ZST-AFFINE",
        "implementation_identity": "No allocator route",
        "adapter_sha256": "NOT_APPLICABLE",
        "common_across_treatments_rule": (
            "Every treatment has exactly zero allocator calls and bytes."
        ),
        "requested_byte_rule": "Exactly zero",
        "usable_byte_rule": "Exactly zero",
        "failure_rule": "Only checked usize length overflow applies.",
        "zst_rule": (
            "capacity=usize::MAX; zero allocate, grow, shrink, free, requested, "
            "usable, live, peak, and transient allocator bytes."
        ),
        "status": "FROZEN_PROTOCOL",
        "blocking_fact": "NONE",
    },
    {
        "allocator_id": "ALLOC-PROTECTED-HISTORICAL",
        "applies_to": "B-FIX and B-P2",
        "implementation_identity": (
            f"Exact historical artifacts at {BASELINE_COMMIT}"
        ),
        "adapter_sha256": "NOT_APPLICABLE",
        "common_across_treatments_rule": (
            "No new adapter, allocation route, or accounting subtraction."
        ),
        "requested_byte_rule": "Exact historical requested bytes",
        "usable_byte_rule": "Not used to relax artifact equality",
        "failure_rule": "Exact historical behavior",
        "zst_rule": "NOT_APPLICABLE",
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    },
]

GROWTH = [
    {
        "growth_policy_id": "GROW-RUST-1.97",
        "applies_to": "Positive-size amortized growth",
        "capacity_formula": (
            "required=checked(len+request); minimum=8 if size==1 else "
            "4 if size<=1024 else 1; new=max(required,checked(2*old),minimum)"
        ),
        "allocation_formula": (
            "requested_bytes=checked(new_capacity*size); alignment=align(T); "
            "allocator usable bytes remain separate"
        ),
        "overflow_rule": (
            "The first failed add, double, multiply, layout, isize, or usize "
            "premise exits before allocation or payload access."
        ),
        "zst_rule": "Use GROW-ZST-USIZE-MAX.",
        "rust_source_pin": (
            f"{RUST_COMMIT}:library/alloc/src/raw_vec/mod.rs:{RAWVEC_SHA256}"
        ),
        "status": "FROZEN_PROTOCOL",
    },
    {
        "growth_policy_id": "GROW-RUST-EXACT-1.97",
        "applies_to": "Positive-size reserve_exact and try_reserve_exact",
        "capacity_formula": "required=checked(len+request); new=required",
        "allocation_formula": (
            "requested_bytes=checked(required*size); alignment=align(T); "
            "allocator usable bytes remain separate"
        ),
        "overflow_rule": (
            "The first failed add, multiply, layout, isize, or usize premise "
            "exits before allocation or payload access."
        ),
        "zst_rule": "Use GROW-ZST-USIZE-MAX.",
        "rust_source_pin": (
            f"{RUST_COMMIT}:library/alloc/src/raw_vec/mod.rs:{RAWVEC_SHA256}"
        ),
        "status": "FROZEN_PROTOCOL",
    },
    {
        "growth_policy_id": "GROW-NONE",
        "applies_to": "No-grow, read, traversal, edit, and structural cells",
        "capacity_formula": "Initial capacity remains exact.",
        "allocation_formula": "No growth request.",
        "overflow_rule": "Operation-specific checked indices remain active.",
        "zst_rule": "Use GROW-ZST-USIZE-MAX for ZST.",
        "rust_source_pin": "Operation member declaration plus exact contract row",
        "status": "FROZEN_PROTOCOL",
    },
    {
        "growth_policy_id": "GROW-ZST-USIZE-MAX",
        "applies_to": "P-ZST-AFFINE",
        "capacity_formula": "capacity=usize::MAX for target pointer width",
        "allocation_formula": "No allocation and zero bytes",
        "overflow_rule": "len+request remains checked against usize::MAX",
        "zst_rule": (
            "Logical owner count changes by index; address is never identity."
        ),
        "rust_source_pin": (
            f"{RUST_COMMIT}:library/alloc/src/raw_vec/mod.rs:{RAWVEC_SHA256}"
        ),
        "status": "FROZEN_PROTOCOL",
    },
]


ENDPOINTS = [
    {
        "endpoint_id": "END-RAW-TRACE-NS",
        "unit": "nanoseconds per complete trace",
        "direction": "LOWER_IS_BETTER",
        "observation": (
            "One raw positive monotonic interval around one exact trace in one "
            "fresh child; no empty-harness subtraction."
        ),
        "estimator": (
            "Exact scheduled-mixture count of strict raw-integer successes "
            "under the registered timing cross-product"
        ),
        "selection_role": "PRIMARY_NONINFERIORITY_AND_REGISTERED_BENEFIT",
        "pooling_rule": (
            "No workload, shape, payload, target, size, or operation pooling."
        ),
        "missing_data_rule": (
            "Missing, duplicate, nonpositive, malformed, or incomplete rows "
            "invalidate the complete six-treatment block."
        ),
        "status": "FROZEN_PROTOCOL",
    },
    {
        "endpoint_id": "END-PEAK-ACQUIRED-BYTES",
        "unit": "allocator-reported subject-owned usable bytes",
        "direction": "LOWER_IS_BETTER",
        "observation": (
            "Peak live usable bytes including old/new transient growth."
        ),
        "estimator": (
            "Exact scheduled-mixture count of strict raw-integer successes "
            "under the registered memory cross-product and zero-byte rule"
        ),
        "selection_role": "REGISTERED_GLOBAL_BENEFIT",
        "pooling_rule": "No cross-cell or cross-target pooling.",
        "missing_data_rule": "Missing adapter data invalidates the block.",
        "status": PENDING,
    },
]

for scope in ("TRACE", "OP"):
    for quantile in ("P50", "P95", "P99"):
        ENDPOINTS.append({
            "endpoint_id": f"END-{scope}-LATENCY-{quantile}",
            "unit": (
                "nanoseconds per complete trace"
                if scope == "TRACE" else "nanoseconds per logical operation"
            ),
            "direction": "LOWER_IS_BETTER",
            "observation": (
                f"Nearest-rank {quantile.lower()} from retained raw valid "
                f"{scope.lower()} observations inside the exact cell."
            ),
            "estimator": (
                "nearest-rank order statistic ceil(q*n), clamped to [1,n]"
            ),
            "selection_role": "DESCRIPTIVE_ONLY",
            "pooling_rule": (
                "Reported separately for every operation, shape, payload, "
                "size, target, and treatment; never enters selection."
            ),
            "missing_data_rule": (
                "Report NOT_AVAILABLE with exact cause; never substitute zero, "
                "impute, or borrow another cell."
            ),
            "status": "FROZEN_PROTOCOL",
        })

ENDPOINTS.extend([
    {
        "endpoint_id": "END-HARDWARE-COUNTERS",
        "unit": "raw target counter events",
        "direction": "DESCRIPTIVE",
        "observation": (
            "Cycles, instructions, branches, branch misses, cache events, and "
            "TLB events collected under the exact target counter policy."
        ),
        "estimator": (
            "Raw count and count per logical operation; p50/p95/p99 descriptive"
        ),
        "selection_role": "DESCRIPTIVE_ONLY",
        "pooling_rule": "No event, group, target, or workload pooling.",
        "missing_data_rule": (
            "Unavailable or multiplexed-beyond-threshold events are explicit "
            "NOT_AVAILABLE rows with probe evidence; never zero."
        ),
        "status": PENDING,
    },
    {
        "endpoint_id": "END-CODE-UNION-BYTES",
        "unit": "unique reachable linked text bytes",
        "direction": "LOWER_IS_BETTER",
        "observation": (
            "Set union of linked text intervals reachable from mandatory roots."
        ),
        "estimator": "One deterministic value per target and treatment",
        "selection_role": "STRUCTURAL_RUST_FLOOR",
        "pooling_rule": (
            "Shared interval counted once; unresolved indirect reachability "
            "fails closed."
        ),
        "missing_data_rule": "Missing reachability or relocation data fails.",
        "status": PENDING,
    },
    {
        "endpoint_id": "END-STRUCTURAL-COUNTERS",
        "unit": "exact counts",
        "direction": "EXACT_CEILINGS",
        "observation": (
            "Allocations, requested/usable/live/peak bytes, initialized, "
            "moved, cloned, dropped values, checks, branches, calls, facts, "
            "behavior calls, metadata, and source passes."
        ),
        "estimator": "Exact deterministic equality or registered ceiling",
        "selection_role": "STRUCTURAL_ADMISSION",
        "pooling_rule": "Every counter remains cell-local.",
        "missing_data_rule": "Any required missing counter fails admission.",
        "status": PENDING,
    },
])


COUNTER_POLICIES = [
    {
        "counter_policy_id": "COUNTERS-AARCH64-DARWIN",
        "target_id": "TARGET-AARCH64-DARWIN",
        "counter_names": (
            "cycles,instructions,branches,branch_misses,l1d_loads,"
            "l1d_load_misses,l2_loads,l2_load_misses,llc_loads,"
            "llc_load_misses,dtlb_loads,dtlb_load_misses,itlb_loads,"
            "itlb_load_misses"
        ),
        "collection_command_template": (
            "PENDING exact signed counter-tool binary and argument vector; "
            "one child PID, one event group, no shell expansion"
        ),
        "availability_probe": (
            "Freeze counter-tool version/hash, event listing, permission state, "
            "PMU identity, group schedulability, time_enabled, and time_running."
        ),
        "repetition_and_group_rule": (
            "One identically ordered event group per timed child; reject a row "
            "when time_running/time_enabled<0.99 or a required event is scaled."
        ),
        "raw_schema_rule": (
            "Retain event name, raw value, unit, enabled_ns, running_ns, PID, "
            "command hash, tool hash, and availability status."
        ),
        "unavailable_counter_rule": (
            "Write NOT_AVAILABLE plus probe digest and reason; never write zero, "
            "impute, rename an event, or substitute a proxy."
        ),
        "claim_rule": (
            "Counters are descriptive. Missing counters remove that descriptive "
            "claim but never relax timing or structural admission."
        ),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_AARCH64_COUNTER_PROTOCOL",
    },
    {
        "counter_policy_id": "COUNTERS-X86_64-LINUX",
        "target_id": "TARGET-X86_64-LINUX",
        "counter_names": (
            "cycles,instructions,branches,branch_misses,cache_references,"
            "cache_misses,L1-dcache-loads,L1-dcache-load-misses,"
            "LLC-loads,LLC-load-misses,dTLB-loads,dTLB-load-misses,"
            "iTLB-loads,iTLB-load-misses"
        ),
        "collection_command_template": (
            "PENDING exact perf binary/hash and argv; perf stat --no-big-num "
            "--field-separator tab --pid CHILD_PID with one frozen event group"
        ),
        "availability_probe": (
            "Freeze perf version/hash, perf list output, perf_event_paranoid, "
            "PMU identity, kernel, group schedulability, enabled, and running."
        ),
        "repetition_and_group_rule": (
            "One identically ordered group per child; reject a row when "
            "time_running/time_enabled<0.99 or a required event is scaled."
        ),
        "raw_schema_rule": (
            "Retain raw perf fields, event, value, unit, enabled_ns, running_ns, "
            "PID, argv hash, perf hash, and availability status."
        ),
        "unavailable_counter_rule": (
            "Write NOT_AVAILABLE plus probe digest and reason; never write zero, "
            "impute, rename an event, or substitute a proxy."
        ),
        "claim_rule": (
            "Counters are descriptive. Missing counters remove that descriptive "
            "claim but never relax timing or structural admission."
        ),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_X86_COUNTER_PROTOCOL",
    },
    {
        "counter_policy_id": "COUNTERS-NOT-APPLICABLE-I686",
        "target_id": "TARGET-I686-STRUCTURAL",
        "counter_names": "NONE",
        "collection_command_template": "NOT_APPLICABLE",
        "availability_probe": "NOT_APPLICABLE",
        "repetition_and_group_rule": "No execution or timing.",
        "raw_schema_rule": "One structural NOT_APPLICABLE record.",
        "unavailable_counter_rule": (
            "Counters are structurally outside this target; no zero is emitted."
        ),
        "claim_rule": "No hardware-counter claim.",
        "status": "FROZEN_PROTOCOL",
        "blocking_fact": "NONE",
    },
]


STRUCTURAL_GATES = [
    (
        "SG-EXACT-ORACLE",
        "Every cell",
        "Compare result, exact contract state equation, owner roles, allocation "
        "disposition, behavior-call sequence, and failure result.",
        "Exact equality; no invalid access before a trap or abort.",
        "Reject the cell and candidate before interpreting timing.",
        PENDING,
        "PENDING_EXTERNAL_HARNESS",
    ),
    (
        "SG-OWNER-EVENTS",
        "Every ownership-bearing cell",
        "Record initialize, move-out, relocate, replace, clone, clone_from, "
        "behavior, and destruction by logical owner ID.",
        "Every owner has exactly one destination or destruction; no hidden "
        "clone, duplicate liveness, loss, or post-death access.",
        "Reject the candidate.",
        PENDING,
        "PENDING_EXTERNAL_HARNESS",
    ),
    (
        "SG-ALLOCATION",
        "Every positive-size cell",
        "Record common-adapter calls, layouts, requested/usable/live/peak/"
        "transient bytes, failures, releases, and owner tags.",
        "Exact algorithm ceiling, same-shape call equivalence, and no "
        "candidate-private allocator or uncharged allocation.",
        "Reject the candidate.",
        PENDING,
        "PENDING_EXTERNAL_ALLOCATOR_ADAPTER",
    ),
    (
        "SG-ZST",
        "Every P-ZST-AFFINE cell",
        "Count logical owners and every allocator event and byte field.",
        "capacity=usize::MAX; all allocator calls and byte fields are exactly "
        "zero; move/drop counts remain exact by logical index.",
        "Reject the candidate; zero bytes never supply a memory benefit.",
        "FROZEN_PROTOCOL",
        "NONE",
    ),
    (
        "SG-OPERATION-RUST-FLOOR",
        "Every standalone canonical primary operation cell",
        "Exact scheduled-mixture sign decision on raw within-block ratios for "
        "the full Williams-by-five-salt cycle of the exact target, payload, "
        "size, and workload shape.",
        "The directed candidate/Rust claim passes at 1/5000 in every cell; "
        "50*candidate_elapsed_ns < 51*rust_elapsed_ns is the strict success.",
        "Reject the operation gate and candidate.",
        PENDING,
        "PENDING_EXTERNAL_REFERENCE_PILOT",
    ),
    (
        "SG-CODE-UNION",
        "Every selected target",
        "Build relocation-normalized reachable-text graph and interval union.",
        "No unresolved indirect edge; mandatory union <=1.10 times Rust.",
        "Reject the Rust-floor claim.",
        PENDING,
        "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    ),
    (
        "SG-COMMON-SUBSTRATE",
        "All five candidates",
        "Compare substrate interface, source/binary hashes, layouts, costs, "
        "allocator route, generic calls, provenance, and owning cursor.",
        "All five substrate and cost hashes are identical; no private "
        "allocator, cursor, seal, generic path, or result-provenance substitute.",
        "Reject the arm before construction or measurement.",
        PENDING,
        "PENDING_EXTERNAL_COMMON_SUBSTRATE_ARTIFACTS",
    ),
    (
        "SG-FACTS",
        "Every candidate cell",
        "Compare facts-on/off source, accepted semantics, producers, consumers, "
        "invalidators, retained checks, and optimized bodies.",
        "Exact source and semantics; every fact path accounted; indeterminate "
        "fails closed.",
        "Reject the candidate.",
        PENDING,
        "PENDING_EXTERNAL_FACT_REPORTS",
    ),
    (
        "SG-B-FIX",
        "B-FIX on all three target layouts",
        "Compare source, diagnostics, DataLayout, raw IR, normalized hot "
        "instructions, calls, traps, alias metadata, vectorization, and counters.",
        "Exact protected historical identity; normalization allowlist is empty.",
        "Reject the candidate before any score.",
        PENDING,
        "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    ),
    (
        "SG-B-P2",
        "B-P2 on all three target layouts",
        "Compare canonical source, output, DataLayout, raw IR, optimized access "
        "bodies, bounds, aliases, vectorization, calls, and counters.",
        "Exact historical identity; no generation, retirement, reuse, metadata "
        "load, branch, indirection, call, or new fact.",
        "Reject the candidate before any score.",
        PENDING,
        "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    ),
    (
        "SG-H-FLATSET",
        "H-FLATSET on all three target layouts",
        "Validate sorted-set oracle, comparison counts, moves, allocations, "
        "metadata, public dependency budget, and code.",
        "Exact hidden oracle; no FAM-DENSE import, candidate recognition, "
        "per-slot occupancy, dummy payload, or per-item allocation.",
        "Block dense closure.",
        PENDING,
        "PENDING_EXTERNAL_H_FLATSET_CUSTODY",
    ),
    (
        "SG-W-SMALL",
        "W-SMALL on all three target layouts",
        "Validate lengths 0..32, inline/spill ownership, allocation, first-spill "
        "moves, failure, removal, and destruction.",
        "Exact visible witness ceilings and no candidate recognition.",
        "Block W-SMALL.",
        PENDING,
        "PENDING_EXTERNAL_W_SMALL_FIXTURE",
    ),
    (
        "SG-W-GAP",
        "W-GAP on all three target layouts",
        "Validate prefix/suffix/gap oracle, exact move distance, growth, "
        "materialization, allocation, and destruction.",
        "Front, middle, and back routes meet exact ceilings with no per-slot tag.",
        "Block W-GAP.",
        PENDING,
        "PENDING_EXTERNAL_W_GAP_FIXTURE",
    ),
    (
        "SG-ARITHMETIC-32-64",
        "32-bit and 64-bit boundary cells",
        "Evaluate checked add, double, multiply, layout, isize, and usize "
        "premises without allocation or payload access.",
        "Last-valid succeeds and first-invalid exits at the exact failed premise.",
        "Reject the candidate.",
        "FROZEN_PROTOCOL",
        "NONE",
    ),
]


def structural_rows() -> list[dict[str, str]]:
    fields = [
        "structural_gate_id", "applies_to", "collection_method", "pass_rule",
        "failure_consequence", "status", "blocking_fact",
    ]
    return [dict(zip(fields, row)) for row in STRUCTURAL_GATES]


CONTROLS = [
    {
        "control_id": "CTRL-OPERATION-RUST-FLOOR",
        "role": "PRIMARY_SAME_SHAPE_OPERATION",
        "source_authority": (
            f"Rust {RUST_VERSION} at {RUST_COMMIT}; exact operation gate and "
            "member declaration"
        ),
        "source_sha256": RUST_API_INVENTORY_SHA256,
        "artifact_authorities": (
            "REF-RUST-API-INVENTORY;REF-RUST-VEC;REF-RUST-RAWVEC"
        ),
        "layout_oracle": (
            "Payload ABI, initial state, capacity, event stream, and result "
            "layout match across all six facts-on treatments."
        ),
        "code_shape_oracle": (
            "Every adapter, allocation, helper, check, branch, call, and "
            "reachable byte is counted."
        ),
        "structural_oracle": (
            "SG-EXACT-ORACLE, SG-OWNER-EVENTS, SG-ALLOCATION or SG-ZST, "
            "SG-FACTS, SG-CODE-UNION, and SG-OPERATION-RUST-FLOOR."
        ),
        "equality_rule": (
            "Same operation, exact outcome, shape, target, payload, size, "
            "capacity, allocator, growth, endpoint, and common substrate."
        ),
        "required_target_ids": ",".join(NATIVE_TARGETS),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "control_id": "CTRL-PAYLOAD-SEPARATOR",
        "role": "PRIMARY_TIMED_PAYLOAD_SEPARATOR",
        "source_authority": (
            "Nine frozen payload declarations and exact same-shape Rust routes"
        ),
        "source_sha256": digest_value(PAYLOADS),
        "artifact_authorities": "DENSE-PERFORMANCE-PAYLOADS.tsv",
        "layout_oracle": (
            "Declared size, alignment, offsets, behavior, identity, and drop "
            "semantics match on each target."
        ),
        "code_shape_oracle": (
            "Each payload has an independent native-target latency cell; "
            "P-ZST-AFFINE retains zero allocator calls and bytes."
        ),
        "structural_oracle": (
            "Exact payload result, owner/drop events, allocation bytes, and "
            "operation Rust floor."
        ),
        "equality_rule": (
            "No payload result is inferred from another size or semantic class."
        ),
        "required_target_ids": ",".join(NATIVE_TARGETS),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_HARNESS",
    },
    {
        "control_id": "B-FIX",
        "role": "PROTECTED_NO_TAX",
        "source_authority": (
            f"{BASELINE_COMMIT}:prototype/democ/examples/soa_kernel.xl"
        ),
        "source_sha256": (
            "3770cfb059bb723a1c906309af600a9ae8ce63f1fc115ff955b27a284f57b97b"
        ),
        "artifact_authorities": (
            "democ.py=211f7caee393ac5822df71cdd0777a2c9b77dc2fafd47d4f3fd4d0aeed9c5336;"
            "facts-off-IR=3069:dfe27e6ac18799b2ac5e4d6f382fea3f979be04bd269ef64fe224c3c73d42d7c;"
            "facts-on-IR=3779:fa80c462223036a8a2b67d0aabee4e232d326ee6c3327a8a8d6118cdbec20f5d;"
            "assembly=c9e73da35ad2581548688f6ffea411a9189e7a2397edd204ae0a3f8263ca4ffb"
        ),
        "layout_oracle": (
            "Target-derived equivalent of the exact two-word owner and one "
            "allocation; no capacity, occupancy, generation, or policy field."
        ),
        "code_shape_oracle": (
            "Exact per-target raw IR and normalized assembly, bounds/traps, "
            "aliases, vectorization, calls, and instructions."
        ),
        "structural_oracle": (
            "Exact identity on AArch64, x86-64, and i686 structural layouts."
        ),
        "equality_rule": (
            "Source, verdict, diagnostics, layout, per-mode IR, code, and "
            "counters are exact; normalization allowlist empty."
        ),
        "required_target_ids": ",".join(ALL_TARGETS),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    },
    {
        "control_id": "B-P2",
        "role": "PROTECTED_NO_TAX",
        "source_authority": (
            f"{BASELINE_COMMIT}:compiler/sources.txt canonical concatenation"
        ),
        "source_sha256": (
            "17c28914ec3cd109f0411cc8a83423623c1541be239e753e91144a66bea93f65"
        ),
        "artifact_authorities": (
            "source=1029044 bytes;"
            "facts-off-IR=1860733:23baa6cce795a7c8c21b66af2c2c01dbbeade8e40b5fe7dda64966db9f8e615a;"
            "facts-on-IR=2229127:0cde7c30e63ea4e60277ed76fb50940012b9b900d04abfc4385ceb4816e95001;"
            "output=211374 tokens:105550 AST"
        ),
        "layout_oracle": (
            "Target-derived exact 30 fixed two-word columns and non-reused "
            "index width; no capacity, generation, recycling, or policy field."
        ),
        "code_shape_oracle": (
            "Exact per-target access bodies, bounds, aliases, vectorization, "
            "calls, IR, and counters."
        ),
        "structural_oracle": (
            "Exact identity on AArch64, x86-64, and i686 structural layouts."
        ),
        "equality_rule": (
            "No generation, retirement, reuse, metadata load, branch, "
            "indirection, call, fact, or output change."
        ),
        "required_target_ids": ",".join(ALL_TARGETS),
        "status": "FROZEN_SOURCE_PIN",
        "blocking_fact": "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    },
    {
        "control_id": "H-FLATSET",
        "role": "HELD_OUT_DENSE_CLOSURE",
        "source_authority": "WITNESS-REGISTRY.md visible contract",
        "source_sha256": "PENDING_EXTERNAL_CUSTODY_HASH",
        "artifact_authorities": (
            "Hidden source, test, oracle, generator, and access-log hashes pending"
        ),
        "layout_oracle": (
            "One contiguous dense payload owner plus O(1) metadata; no per-slot tag."
        ),
        "code_shape_oracle": (
            "Binary search and direct shifts; all comparisons, moves, "
            "allocations, checks, and bytes counted."
        ),
        "structural_oracle": "SG-H-FLATSET exact hidden oracle",
        "equality_rule": (
            "Visible contract and dependency budget fixed before custody."
        ),
        "required_target_ids": ",".join(ALL_TARGETS),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_H_FLATSET_CUSTODY",
    },
    {
        "control_id": "W-SMALL",
        "role": "VISIBLE_TOPOLOGY_WITNESS",
        "source_authority": "WITNESS-REGISTRY.md visible contract",
        "source_sha256": "PENDING_EXTERNAL_FIXTURE_HASH",
        "artifact_authorities": "Visible source, tests, generator, and hashes pending",
        "layout_oracle": (
            "Frozen inline N; zero heap through N and one dense owner after spill."
        ),
        "code_shape_oracle": (
            "Every length 0..32, first spill, failure, pop/remove, and drop."
        ),
        "structural_oracle": "SG-W-SMALL exact visible oracle",
        "equality_rule": (
            "Identical ordinary-library source and public capability surface."
        ),
        "required_target_ids": ",".join(ALL_TARGETS),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_W_SMALL_FIXTURE",
    },
    {
        "control_id": "W-GAP",
        "role": "VISIBLE_TOPOLOGY_WITNESS",
        "source_authority": "WITNESS-REGISTRY.md visible contract",
        "source_sha256": "PENDING_EXTERNAL_FIXTURE_HASH",
        "artifact_authorities": "Visible source, tests, generator, and hashes pending",
        "layout_oracle": (
            "One owner with live prefix and suffix around one gap; no per-slot tag."
        ),
        "code_shape_oracle": (
            "Front, middle, back edit/move routes and final materialization."
        ),
        "structural_oracle": "SG-W-GAP exact visible oracle",
        "equality_rule": (
            "Identical ordinary-library source and public capability surface."
        ),
        "required_target_ids": ",".join(ALL_TARGETS),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_W_GAP_FIXTURE",
    },
]


FAILURES = [
    {
        "failure_policy_id": "FP-NORMAL",
        "commit_point": "Exact contract commitment_point",
        "arithmetic_failure": "Excluded from the successful timed row",
        "allocation_failure": "Excluded from the successful timed row",
        "callback_failure": "Excluded from the successful timed row",
        "owner_disposition": "Exact normal post-state and owner roles",
        "timing_rule": "Only the named normal outcome enters this cell.",
        "status": "FROZEN_PROTOCOL",
    },
    {
        "failure_policy_id": "FP-FUNCTIONAL-ONLY",
        "commit_point": "Exact contract commitment_point",
        "arithmetic_failure": "Exact checked error or trap outcome",
        "allocation_failure": "Exact allocation error or OOM outcome",
        "callback_failure": "Exact behavior abort outcome",
        "owner_disposition": (
            "Exact returned owners or pre-abort invariant from the contract row"
        ),
        "timing_rule": (
            "Functional and failure-injection oracle only; never pooled into "
            "successful timing."
        ),
        "status": "FROZEN_PROTOCOL",
    },
    {
        "failure_policy_id": "FP-PROTECTED-IDENTITY",
        "commit_point": "Historical",
        "arithmetic_failure": "Historical",
        "allocation_failure": "Historical",
        "callback_failure": "Historical",
        "owner_disposition": "Historical",
        "timing_rule": "Structural only",
        "status": "FROZEN_PROTOCOL",
    },
]

WARMUPS = [
    {
        "warmup_id": "WARMUP-EXACT-3",
        "applies_to": "No-grow reads, borrows, edits, and repeated traversal",
        "untimed_trace_count": "3",
        "cache_state_rule": (
            "Run the identical trace exactly three times, then reconstruct the "
            "subject; no adaptive warmup."
        ),
        "candidate_symmetry": "Same descriptor and order for all six treatments.",
        "status": "FROZEN_PROTOCOL",
    },
    {
        "warmup_id": "WARMUP-NONE-FRESH",
        "applies_to": "Allocation, growth, clone, sort, drop, and witness cells",
        "untimed_trace_count": "0",
        "cache_state_rule": "Fresh child and subject for every observation.",
        "candidate_symmetry": "Same rule for all six treatments.",
        "status": "FROZEN_PROTOCOL",
    },
    {
        "warmup_id": "WARMUP-NONE-STRUCTURAL",
        "applies_to": "Protected and structural cells",
        "untimed_trace_count": "0",
        "cache_state_rule": "Two clean deterministic output directories.",
        "candidate_symmetry": "Same build graph for all candidates.",
        "status": "FROZEN_PROTOCOL",
    },
]

REPETITIONS = [
    {
        "repetition_id": "REP-WILLIAMS-POWER",
        "applies_to": "Every native-target primary cell",
        "block_count_rule": (
            "Choose 60, 90, or 120 complete blocks only from the frozen "
            "reference-only exact empirical power calculation; if none reaches the "
            "power floor, return PROTOCOL_INFEASIBLE and reopen before construction."
        ),
        "observations_per_block": (
            "6 fresh children, one per facts-on candidate or Rust treatment"
        ),
        "fresh_process_rule": "Every observation is one fresh child.",
        "stopping_rule": (
            "Fixed selected count; no extension, early stop, timeout selection, "
            "best-of-N, or candidate-result power update."
        ),
        "raw_retention_rule": (
            "Retain every complete and incomplete row. A candidate scheduled "
            "unusable slot is a sign failure and cannot reduce fixed n; a "
            "reference-pilot unusable row makes the branch PROTOCOL_INFEASIBLE."
        ),
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_REFERENCE_PILOT",
    },
    {
        "repetition_id": "REP-STRUCTURAL-2",
        "applies_to": "Every structural and protected control",
        "block_count_rule": "Exactly two clean deterministic builds",
        "observations_per_block": "1 artifact graph",
        "fresh_process_rule": "Fresh output directory",
        "stopping_rule": "Exactly two; disagreement fails.",
        "raw_retention_rule": "Retain both graphs and reports.",
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    },
]

FACTS = [
    {
        "facts_policy_id": "FACTS-PRIMARY-6-DIAGNOSTIC-OFF",
        "primary_treatments": ";".join(
            f"{candidate}:facts-on" for candidate in CANDIDATES
        ) + ";RUST-1.97",
        "facts_off_role": (
            "Separate post-primary diagnostic campaign; not a seventh treatment, "
            "never interleaved with a Williams block, and never enters selection."
        ),
        "facts_off_pairing": (
            "After every primary row and machine-state artifact is frozen, run a "
            "separate randomized facts-on/off campaign with its own commitment, "
            "assignment manifest, global order, and raw-row schema. It cannot "
            "change, replace, or trigger any primary observation."
        ),
        "required_reports": (
            "Source, accepted semantics, producers, dependencies, consumers, "
            "invalidators, checks, optimized-body digest, and counters."
        ),
        "selection_use": "Facts-on only; facts-off cannot rescue a failure.",
        "mismatch_rule": "Any mismatch or indeterminate accounting rejects.",
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_FACT_REPORTS",
    },
    {
        "facts_policy_id": "FACTS-STRUCTURAL",
        "primary_treatments": "Every candidate configuration required by the gate",
        "facts_off_role": "Exact protected authority when applicable",
        "facts_off_pairing": "NOT_APPLICABLE",
        "required_reports": "Exact raw and normalized artifact identities",
        "selection_use": "Structural admission only",
        "mismatch_rule": "Reject",
        "status": PENDING,
        "blocking_fact": "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    },
]

GENERATORS = [
    {
        "generator_id": "GEN-DENSE-PLAN-V3",
        "schema": "xlang-dense-performance-input-v3",
        "seed_rule": (
            "seed64=first eight bytes of SHA256(cell_id || domain), big-endian; "
            "zero maps to 0x9e3779b97f4a7c15"
        ),
        "operation_encoding": (
            "Canonical JSON exact contract, event stream, state, capacity, "
            "shape, payload, size, target, branch set, and operation count."
        ),
        "trace_hash_rule": "SHA256 of canonical ASCII trace_plan",
        "oracle_hash_rule": "SHA256 of canonical ASCII oracle_plan",
        "authorization": "candidate_execution_authorized=false",
        "status": "FROZEN_PROTOCOL",
    },
    {
        "generator_id": "GEN-PROTECTED-V3",
        "schema": "xlang-dense-protected-input-v3",
        "seed_rule": "No random seed",
        "operation_encoding": (
            "Exact source, target layout, artifact authorities, and comparison graph"
        ),
        "trace_hash_rule": "SHA256 of canonical ASCII trace_plan",
        "oracle_hash_rule": "SHA256 of canonical ASCII oracle_plan",
        "authorization": "candidate_execution_authorized=false",
        "status": "FROZEN_PROTOCOL",
    },
]


SHAPE_DEFINITIONS: dict[str, tuple[str, str, str]] = {
    "BASE": (
        "CANONICAL_OPERATION",
        "One exact canonical operation outcome at 4096 logical payload bytes.",
        "No aggregation with any other shape.",
    ),
    "EDIT-FRONT": (
        "EDIT_INDEX",
        "Index 0 on a nonempty valid owner.",
        "Independent primary cell.",
    ),
    "EDIT-MIDDLE": (
        "EDIT_INDEX",
        "Index floor(len/2) on a nonempty valid owner.",
        "Independent primary cell.",
    ),
    "EDIT-BACK": (
        "EDIT_INDEX",
        "Last valid index, or insertion index len.",
        "Independent primary cell.",
    ),
    "SWAP-EQUAL": (
        "SWAP_INDEX",
        "left=right=floor(len/2); exact no-op owner identity.",
        "Independent primary cell.",
    ),
    "SWAP-FRONT-BACK": (
        "SWAP_INDEX",
        "left=0 and right=len-1.",
        "Independent primary cell.",
    ),
    "SWAP-ADJACENT-MIDDLE": (
        "SWAP_INDEX",
        "The two valid adjacent middle indices.",
        "Independent primary cell.",
    ),
    "CLONE-FROM-DST-SHORTER": (
        "CLONE_FROM_LENGTH",
        "destination length=floor(source length/2).",
        "Independent primary cell.",
    ),
    "CLONE-FROM-EQUAL": (
        "CLONE_FROM_LENGTH",
        "destination length=source length.",
        "Independent primary cell.",
    ),
    "CLONE-FROM-DST-LONGER": (
        "CLONE_FROM_LENGTH",
        "destination length=source length+floor(source length/2).",
        "Independent primary cell.",
    ),
    "RETAIN-10": (
        "SURVIVOR_SELECTIVITY",
        "Exactly 10 percent survivors with stable seeded positions.",
        "Independent primary cell; never pooled with 50 or 90 percent.",
    ),
    "RETAIN-50": (
        "SURVIVOR_SELECTIVITY",
        "Exactly 50 percent survivors with stable seeded positions.",
        "Independent primary cell; never pooled with 10 or 90 percent.",
    ),
    "RETAIN-90": (
        "SURVIVOR_SELECTIVITY",
        "Exactly 90 percent survivors with stable seeded positions.",
        "Independent primary cell; never pooled with 10 or 50 percent.",
    ),
    "SORT-RANDOM": (
        "SORT_INPUT",
        "Seeded random permutation of unique keys.",
        "Independent primary cell.",
    ),
    "SORT-SORTED": (
        "SORT_INPUT",
        "Strictly increasing keys.",
        "Independent primary cell.",
    ),
    "SORT-REVERSE": (
        "SORT_INPUT",
        "Strictly decreasing keys.",
        "Independent primary cell.",
    ),
    "SORT-ORGAN-PIPE": (
        "SORT_INPUT",
        "Increasing to the midpoint then decreasing.",
        "Independent primary cell.",
    ),
    "SORT-DUPLICATE-90": (
        "SORT_INPUT",
        "Exactly 90 percent duplicate keys at seeded positions.",
        "Independent primary cell.",
    ),
    "SCOPED-CONTINUE": (
        "OD4_SCOPED_CONSUME",
        "Consumer continues through every selected owner in source order.",
        "Independent primary cell with zero removed-result allocation.",
    ),
    "SCOPED-EARLY-STOP": (
        "OD4_SCOPED_CONSUME",
        "Consumer stops after exactly 50 percent of selected owners; repair "
        "completes before return.",
        "Independent primary cell with zero removed-result allocation.",
    ),
}

for position, replacement in itertools.product(
    ("FRONT", "MIDDLE", "BACK"), ("SHORTER", "EQUAL", "LONGER")
):
    SHAPE_DEFINITIONS[f"SPLICE-{position}-REPL-{replacement}"] = (
        "SPLICE_RANGE_REPLACEMENT",
        f"{position.lower()} removal range with {replacement.lower()} replacement.",
        "Independent primary cell; position and replacement length never pool.",
    )

for boundary in (64, 4096, 262144, 4194304, 67108864):
    SHAPE_DEFINITIONS[f"BYTE-{boundary}"] = (
        "BYTE_BOUNDARY",
        f"Exactly {boundary} logical payload bytes.",
        "Independent primary boundary cell.",
    )

for payload_code in PAYLOAD_CODES:
    SHAPE_DEFINITIONS[f"PAYLOAD-{payload_code}"] = (
        "PAYLOAD_SEPARATOR",
        f"Timed separating witness for {payload_code}.",
        "Independent native-target primary cell.",
    )

for control in ("B-FIX", "B-P2", "H-FLATSET", "W-SMALL", "W-GAP"):
    SHAPE_DEFINITIONS[f"PROTECTED-{control}"] = (
        "PROTECTED_CONTROL",
        f"Exact structural {control} control on the target layout.",
        "Independent structural layout cell.",
    )

SHAPE_DEFINITIONS["ARITHMETIC-32-LAST-FIRST"] = (
    "ARITHMETIC_BOUNDARY",
    "i686 last-valid and first-invalid add, multiply, layout, isize, and usize tuples.",
    "Independent structural cell.",
)
SHAPE_DEFINITIONS["ARITHMETIC-64-LAST-FIRST"] = (
    "ARITHMETIC_BOUNDARY",
    "64-bit last-valid and first-invalid add, multiply, layout, isize, and usize tuples.",
    "Independent structural cell on each native layout.",
)


def distribution_rows() -> list[dict[str, str]]:
    return [
        {
            "shape_id": shape,
            "workload_family": values[0],
            "exact_definition": values[1],
            "seed_use": (
                "Seed fixes values and positions without changing the shape."
                if "seed" in values[1].lower() or values[0] in {
                    "CANONICAL_OPERATION", "EDIT_INDEX", "SORT_INPUT",
                    "SURVIVOR_SELECTIVITY", "PAYLOAD_SEPARATOR",
                }
                else "No randomness"
            ),
            "independence_rule": values[2],
            "post_result_changes": "FORBIDDEN",
            "status": "FROZEN_PROTOCOL",
        }
        for shape, values in sorted(SHAPE_DEFINITIONS.items())
    ]


def schedule_rows() -> list[dict[str, str]]:
    return [
        {
            "schedule_id": "SCHED-" + shape,
            "shape_id": shape,
            "operation_order": definition[1],
            "operation_count_rule": (
                "Exact operation_count from the immutable descriptor"
            ),
            "reset_rule": (
                "Fresh subject per observation; no hidden rebuild inside timer."
            ),
            "status": "FROZEN_PROTOCOL",
        }
        for shape, definition in sorted(SHAPE_DEFINITIONS.items())
    ]


PIPELINE_STAGE_ORDER = (
    "REFERENCE_PILOT",
    "CANDIDATE_CONSTRUCTION",
    "CANDIDATE_FREEZE_B",
)
SIDE_STAGES = ("DESCRIPTIVE_COUNTER_REPORT",)


def active_owner_branch_rows() -> list[dict[str, str]]:
    return [
        row for row in owner_branch_base_rows()
        if row["branch_class"] == "ACTIVE_POWER_BRANCH"
    ]


ALL_ACTIVE_BRANCH_IDS = tuple(sorted(
    row["branch_id"] for row in active_owner_branch_rows()
))
DUAL_NATIVE_BRANCH_IDS = tuple(sorted(
    row["branch_id"] for row in active_owner_branch_rows()
    if row["od2_option_id"] == "OD-2-DUAL-NATIVE"
))


BLOCKERS = [
    (
        "PENDING_EXTERNAL_OWNER_AUTHORIZATION",
        "AUTHORIZATION",
        "D13 does not authorize candidate construction, Freeze B, pilot, or score.",
        "A separate explicit owner authorization after exact-hash hostile review.",
        "REFERENCE_PILOT",
        "Absence ends the workflow; it is not permission to infer authority.",
    ),
    (
        "PENDING_EXTERNAL_OWNER_BRANCH_SELECTION",
        "OWNER_DECISION",
        "OD-0 through OD-5 remain unresolved. The eight active power branches "
        "exist only under OD-0 common substrate, OD-4 scoped consume, and OD-5 "
        "no-crossover.",
        "The owner either accepts those three protocol conditions and selects "
        "exactly one OD-1/OD-2/OD-3 branch, or the lock reopens before any pilot.",
        "REFERENCE_PILOT",
        "No branch may borrow data or power from another branch; a rejected "
        "protocol condition requires a regenerated lock.",
    ),
    (
        "PENDING_EXTERNAL_REPOSITORY_BASELINE",
        "REPOSITORY_BASELINE",
        "No clean common candidate starting commit, permitted generated state, "
        "worktree-status digest, or equality rule is frozen.",
        "Freeze one exact repository commit and allowed-status manifest for all "
        "five isolated authors before the first candidate prompt.",
        "CANDIDATE_CONSTRUCTION",
        "Any unequal or contaminated starting tree invalidates construction; no "
        "arm may receive a repaired or later baseline.",
    ),
    (
        "PENDING_EXTERNAL_CANDIDATE_AUTHOR_IDENTITIES",
        "AUTHORSHIP",
        "The five isolated candidate authors and conflict/access attestations "
        "are not named.",
        "Freeze one distinct author identity per arm and exact access boundaries.",
        "CANDIDATE_CONSTRUCTION",
        "An unstaffed arm is mechanism failure; another author may not repair it.",
    ),
    (
        "PENDING_EXTERNAL_SERVICE_SNAPSHOTS",
        "SERVICE",
        "Exact model/service/provider snapshots, system prompts, tools, and "
        "retention terms are not frozen.",
        "Freeze exact service identities and immutable request/response logs.",
        "CANDIDATE_CONSTRUCTION",
        "No adjacent service or later model may substitute after a failure.",
    ),
    (
        "PENDING_EXTERNAL_DISCLOSURE_AUTHORITY",
        "DISCLOSURE",
        "No authority exists to disclose repository bytes to an external service.",
        "Record exact disclosure scope, recipient, retention, and owner approval.",
        "CANDIDATE_CONSTRUCTION",
        "No prompt may be sent without explicit authority.",
    ),
    (
        "PENDING_EXTERNAL_CONSTRUCTION_BUDGET",
        "RESOURCE_PROTOCOL",
        "No defensible equal interaction, compute, wall-clock, repair, or tuning "
        "budget is owner-frozen; inventing an arbitrary budget is forbidden.",
        "Owner freezes equal service-native budgets and termination rules before "
        "the first prompt, or declares construction infeasible.",
        "CANDIDATE_CONSTRUCTION",
        "Exhaustion or inability to implement is the arm result; no extra budget.",
    ),
    (
        "PENDING_EXTERNAL_FEEDBACK_PROTOCOL",
        "REPAIR_PROTOCOL",
        "Exact candidate-visible diagnostics, repair rounds, tuning access, and "
        "allowed defect corrections are not frozen.",
        "Freeze identical materials and a finite service-native protocol before "
        "construction; mechanism changes remain forbidden.",
        "CANDIDATE_CONSTRUCTION",
        "A mechanism that cannot pass within the frozen protocol fails.",
    ),
    (
        "PENDING_EXTERNAL_COMMON_SUBSTRATE_ARTIFACTS",
        "COMMON_SUBSTRATE",
        "The byte-identical OD-0 substrate implementation and cost artifacts "
        "for all five arms do not exist.",
        "Freeze one source/binary/interface/cost hash set and prove equality.",
        "CANDIDATE_CONSTRUCTION",
        "No candidate-private allocator, seal, generic, provenance, or cursor.",
    ),
    (
        "PENDING_EXTERNAL_OD4_SCOPED_CONTRACT",
        "OD4_REFERENCE_ADAPTER",
        "The scoped-consume policy hash is frozen, but its executable symmetric "
        "Rust reference adapter does not exist.",
        "Freeze exact Rust source, binary, interface, behavior-oracle, and cost "
        "hashes without changing the OD-4 policy bytes.",
        "REFERENCE_PILOT",
        "A missing or asymmetric scoped Rust adapter makes the reference pilot "
        "protocol infeasible.",
    ),
    (
        "PENDING_EXTERNAL_OD4_CANDIDATE_ARTIFACTS",
        "OD4_CANDIDATE_CONTRACT",
        "The exact scoped-consume META-5 and compiler artifacts for candidate "
        "construction do not exist.",
        "Freeze the candidate-visible contract, compiler surface, diagnostics, "
        "and lowering inputs without changing the OD-4 policy bytes.",
        "CANDIDATE_CONSTRUCTION",
        "If the exact scoped contract cannot be implemented, the branch fails.",
    ),
    (
        "PENDING_EXTERNAL_H_FLATSET_CUSTODY",
        "HELD_OUT_CUSTODY",
        "Independent custodian, hidden source/test/oracle/generator hashes, "
        "storage, access log, and disclosure time are absent.",
        "Freeze custodian identity and hashes outside candidate-visible paths.",
        "CANDIDATE_CONSTRUCTION",
        "Any leak rotates hidden bytes under the unchanged visible contract.",
    ),
    (
        "PENDING_EXTERNAL_AARCH64_ENVIRONMENT",
        "TARGET",
        "Exact power, affinity, thermal, noise, dyld, libc, allocator image, "
        "CPU features, Rust compiler, linker, runtime, target tools, flags, and "
        "executable identities are absent.",
        "Freeze environment probes, toolchain binaries and configurations, build "
        "commands, executable hashes, and invalidation rules.",
        "REFERENCE_PILOT",
        "Unavailable environment blocks the relevant branch.",
    ),
    (
        "PENDING_EXTERNAL_X86_RUNNER",
        "TARGET",
        "No exact x86-64 machine, kernel, libc, allocator, isolation, counter "
        "tool, or toolchain identity is frozen.",
        "Provision and hash the native runner.",
        "REFERENCE_PILOT",
        "Dual-native branches fail closed; Mac-local branches remain target-local.",
    ),
    (
        "PENDING_EXTERNAL_I686_TOOLCHAIN",
        "TARGET",
        "No exact i686 structural compiler and layout artifact set is frozen.",
        "Freeze tools, flags, modules, and structural reports.",
        "CANDIDATE_CONSTRUCTION",
        "Every branch still requires i686 structural admission.",
    ),
    (
        "PENDING_EXTERNAL_ALLOCATOR_ADAPTER",
        "ALLOCATOR",
        "Common counted adapter source, binary, usable-byte method, and failure "
        "injector hashes do not exist.",
        "Construct and independently review one symmetric adapter.",
        "REFERENCE_PILOT",
        "No private or approximate byte accounting may substitute.",
    ),
    (
        "PENDING_EXTERNAL_HARNESS",
        "FIXTURE",
        "Executable harness, Rust adapters, payload declarations, and exact "
        "oracles do not exist.",
        "Freeze source, binaries, inputs, commands, and oracle hashes.",
        "REFERENCE_PILOT",
        "A missing exact adapter blocks its operation; no aggregate proxy.",
    ),
    (
        "PENDING_EXTERNAL_REFERENCE_PILOT",
        "STATISTICS",
        "The fixed four-cycle six-pseudo-Rust raw campaign, whole-cycle supports, "
        "memory eligibility ledger, and selected branch block count are absent.",
        "Run only the preregistered reference pilot and exact empirical DP after "
        "every applicable per-branch REFERENCE_PILOT prerequisite in the frozen "
        "stage protocol is resolved.",
        "CANDIDATE_CONSTRUCTION",
        "If 60, 90, and 120 blocks all miss worst-case power, return "
        "PROTOCOL_INFEASIBLE and reopen before any candidate observation.",
    ),
    (
        "PENDING_EXTERNAL_RANDOMIZATION_CUSTODY",
        "RANDOMIZATION_CUSTODY",
        "No independent custodian, committed 32-byte seed, byte-grammar test, "
        "assignment manifests, custody log, release record, or runner receipt exists.",
        "Freeze the exact custody identities, commitment, seed generation, manifest "
        "joins, and release procedure before the reference pilot.",
        "REFERENCE_PILOT",
        "Any custody gap, early disclosure, manifest mismatch, or rerandomization "
        "makes the branch PROTOCOL_INFEASIBLE; no replacement seed or run.",
    ),
    (
        "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "POWER_ENGINE",
        "The exact empirical-DP engine source, binary, compiler, flags, task "
        "manifest, arithmetic tests, and fixed resource evidence are absent.",
        "Freeze and independently verify the engine and every task, operation, "
        "memory, and monotonic wall-time ceiling before the reference pilot.",
        "REFERENCE_PILOT",
        "A missing artifact, exceeded ceiling, signal, nonzero exit, or incomplete "
        "manifest returns PROTOCOL_INFEASIBLE; no adjacent engine or partial result.",
    ),
    (
        "PENDING_EXTERNAL_FACT_REPORTS",
        "FACT_CHANNEL",
        "Candidate fact producers, consumers, invalidators, and reports do not exist.",
        "During authorized construction, produce the exact report schema, "
        "producer/consumer/invalidation records, and facts-on/off artifacts; "
        "Candidate Freeze B then pins them.",
        "CANDIDATE_FREEZE_B",
        "Indeterminate fact accounting rejects the candidate.",
    ),
    (
        "PENDING_EXTERNAL_CANDIDATE_BUILDS",
        "ARTIFACT",
        "No candidate source, compiler, command, or binary is authorized or built.",
        "After construction authority, produce exact candidate source, compiler, "
        "commands, binaries, and construction reports; Candidate Freeze B then "
        "pins those completed artifacts.",
        "CANDIDATE_FREEZE_B",
        "A nonbuilding arm is a mechanism result, not permission to change it.",
    ),
    (
        "PENDING_EXTERNAL_AARCH64_COUNTER_PROTOCOL",
        "COUNTERS",
        "Exact AArch64 event availability and counter command are absent.",
        "Freeze the target counter policy evidence.",
        "DESCRIPTIVE_COUNTER_REPORT",
        "Unavailable counters are explicit missing data, never zero.",
    ),
    (
        "PENDING_EXTERNAL_X86_COUNTER_PROTOCOL",
        "COUNTERS",
        "Exact x86 event availability and perf command are absent.",
        "Freeze the target counter policy evidence.",
        "DESCRIPTIVE_COUNTER_REPORT",
        "Unavailable counters are explicit missing data, never zero.",
    ),
    (
        "PENDING_EXTERNAL_AARCH64_CANDIDATE_MODULES",
        "LAYOUT",
        "Candidate and harness AArch64 modules do not exist.",
        "Freeze exact modules and field-by-field layout reports.",
        "CANDIDATE_FREEZE_B",
        "Layout uncertainty rejects the candidate.",
    ),
    (
        "PENDING_EXTERNAL_W_SMALL_FIXTURE",
        "VISIBLE_WITNESS",
        "W-SMALL source, N, tests, and generator hashes are absent.",
        "Freeze the ordinary-library fixture.",
        "CANDIDATE_CONSTRUCTION",
        "Missing witness blocks its obligation.",
    ),
    (
        "PENDING_EXTERNAL_W_GAP_FIXTURE",
        "VISIBLE_WITNESS",
        "W-GAP source, tests, and generator hashes are absent.",
        "Freeze the ordinary-library fixture.",
        "CANDIDATE_CONSTRUCTION",
        "Missing witness blocks its obligation.",
    ),
]


def blocker_rows() -> list[dict[str, str]]:
    fields = [
        "blocker_id", "kind", "missing_fact", "required_resolution",
        "earliest_blocked_stage", "mechanism_failure_rule",
    ]
    rows = []
    for values in BLOCKERS:
        item = dict(zip(fields, values), status="BLOCKING")
        branch_ids = (
            DUAL_NATIVE_BRANCH_IDS
            if item["blocker_id"] in {
                "PENDING_EXTERNAL_X86_RUNNER",
                "PENDING_EXTERNAL_X86_COUNTER_PROTOCOL",
            }
            else ALL_ACTIVE_BRANCH_IDS
        )
        item["applicable_owner_branch_ids"] = ",".join(branch_ids)
        rows.append(item)
    return rows


def stage_prerequisite_protocol() -> dict[str, Any]:
    rows = blocker_rows()
    stage_index = {
        stage: index for index, stage in enumerate(PIPELINE_STAGE_ORDER)
    }
    known_stages = set(PIPELINE_STAGE_ORDER) | set(SIDE_STAGES)
    if any(row["earliest_blocked_stage"] not in known_stages for row in rows):
        raise ProtocolError("blocker uses an unknown stage")
    if len({row["blocker_id"] for row in rows}) != len(rows):
        raise ProtocolError("duplicate operational blocker")

    per_branch: dict[str, dict[str, Any]] = {}
    for branch_id in ALL_ACTIVE_BRANCH_IDS:
        applicable = [
            row for row in rows
            if branch_id in row["applicable_owner_branch_ids"].split(",")
        ]
        pipeline: dict[str, Any] = {}
        for stage in PIPELINE_STAGE_ORDER:
            direct = sorted(
                row["blocker_id"] for row in applicable
                if row["earliest_blocked_stage"] == stage
            )
            cumulative = sorted(
                row["blocker_id"] for row in applicable
                if (
                    row["earliest_blocked_stage"] in stage_index
                    and stage_index[row["earliest_blocked_stage"]]
                    <= stage_index[stage]
                )
            )
            pipeline[stage] = {
                "direct_blocker_ids": direct,
                "cumulative_blocker_ids": cumulative,
            }
        side = {
            stage: sorted(
                row["blocker_id"] for row in applicable
                if row["earliest_blocked_stage"] == stage
            )
            for stage in SIDE_STAGES
        }
        per_branch[branch_id] = {
            "pipeline": pipeline,
            "side_stages": side,
        }
    return {
        "schema": "xlang-dense-stage-prerequisites-v1",
        "pipeline_stage_order": list(PIPELINE_STAGE_ORDER),
        "side_stages": list(SIDE_STAGES),
        "earliest_stage_rule": (
            "earliest_blocked_stage is the first pipeline gate that an unresolved "
            "applicable row blocks; it transitively blocks every later pipeline "
            "stage. Side stages never gate candidate selection."
        ),
        "branch_applicability_rule": (
            "A blocker applies only to the exact owner_branch_ids recorded in its "
            "row. Every branch must resolve its complete cumulative set before "
            "entering a pipeline stage."
        ),
        "per_owner_branch": per_branch,
    }


SORT_MEMBERS = {
    "DENSE-SORT-STABLE",
    "DENSE-SORT-STABLE-CACHED-KEY",
    "DENSE-SORT-UNSTABLE",
}
RETAIN_MEMBERS = {
    "DENSE-RETAIN",
    "DENSE-RETAIN-MUT",
    "DENSE-EAGER-EXTRACT",
    "DENSE-DEDUP",
    "DENSE-DEDUP-BY",
    "DENSE-DEDUP-BY-KEY",
}
EDIT_MEMBERS = {
    "DENSE-INSERT",
    "DENSE-INSERT-UNIQ",
    "DENSE-REMOVE",
    "DENSE-SWAP-REMOVE",
    "DENSE-COPY-WITHIN",
    "DENSE-ROTATE",
    "DENSE-SELECT-UNSTABLE",
    "DENSE-VIEW-SPLIT-CHECKED",
    "DENSE-VIEW-SPLIT-TRAP",
    "DENSE-VIEW-CONSUME-SPLIT",
    "DENSE-VIEW-DISJOINT-UNIQ",
    "DENSE-VIEW-ARRAY-CHUNKS",
}


def shapes_for_member(member: str) -> list[str]:
    if member in SORT_MEMBERS:
        return [
            "SORT-RANDOM", "SORT-SORTED", "SORT-REVERSE",
            "SORT-ORGAN-PIPE", "SORT-DUPLICATE-90",
        ]
    if member in RETAIN_MEMBERS:
        return ["RETAIN-10", "RETAIN-50", "RETAIN-90"]
    if member in EDIT_MEMBERS:
        return ["EDIT-FRONT", "EDIT-MIDDLE", "EDIT-BACK"]
    if member == "DENSE-SWAP":
        return ["SWAP-EQUAL", "SWAP-FRONT-BACK", "SWAP-ADJACENT-MIDDLE"]
    if member == "DENSE-CLONE-FROM":
        return [
            "CLONE-FROM-DST-SHORTER",
            "CLONE-FROM-EQUAL",
            "CLONE-FROM-DST-LONGER",
        ]
    if member == "DENSE-EAGER-SPLICE":
        return sorted(
            shape for shape in SHAPE_DEFINITIONS
            if shape.startswith("SPLICE-")
        )
    return ["BASE"]


def rust_route_for_member(member: str) -> str:
    if member == "DENSE-EAGER-EXTRACT":
        return "REF-RUST-EXTRACT"
    if member == "DENSE-EAGER-SPLICE":
        return "REF-RUST-SPLICE"
    if member in {"DENSE-ITER-SHARED", "DENSE-ITER-UNIQ", "DENSE-ITER-OWN"}:
        return "REF-RUST-INTOITER"
    if member in {"DENSE-SORT-STABLE", "DENSE-SORT-STABLE-CACHED-KEY"}:
        return "REF-RUST-SORT-STABLE"
    if member == "DENSE-SORT-UNSTABLE":
        return "REF-RUST-SORT-UNSTABLE"
    if member.startswith("DENSE-"):
        return "REF-RUST-VEC"
    return "REF-RUST-API-INVENTORY"


def default_payload_for(row: dict[str, str]) -> str:
    profile = row["profile_id"]
    member = row["member_contract_id"]
    if profile in {
        "BEHAVIOR", "ALLOCATING_BEHAVIOR", "CLONE_FROM",
        "INIT_BEHAVIOR", "POP_IF", "STABLE_SORT", "UNSTABLE_SORT", "SELECT",
    }:
        return "P-BEHAVIOR"
    if profile in {
        "OD1_MUTATOR", "DROP", "CLEAR", "TRUNCATE", "REMOVE",
        "SWAP_REMOVE", "SHRINK", "ITER_OWN",
    } or member in {"DENSE-APPEND-MOVE", "DENSE-EAGER-SPLICE"}:
        return "P-AFFINE64"
    if profile in {
        "TRAPPING_BORROW", "OPTION_BORROW", "CHECKED_BORROW", "ITER_BORROW",
        "TOTAL_BORROW",
    }:
        return "P-ROW24"
    if profile in {"CHECKED_MUTATION", "INIT_COPY"}:
        return "P-ROW56"
    return "P-U64"


def build_operation_gates_and_algorithms(
    contract_rows: list[dict[str, str]],
    dispositions: list[dict[str, str]],
) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    by_contract = {row["contract_id"]: row for row in contract_rows}
    primary = [
        row for row in dispositions if row["disposition"] == "TIMED_PRIMARY"
    ]
    grouped: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in primary:
        grouped[row["representative_operation_gate_id"]].append(row)
    executable_members = {
        row["member_contract_id"] for row in dispositions
        if row["disposition"] != "EXCLUDED"
    }
    represented_members = {row["member_contract_id"] for row in primary}
    if executable_members != represented_members:
        raise ProtocolError(
            "standalone canonical operation has no primary Rust-floor "
            f"representative: {sorted(executable_members-represented_members)}"
        )
    gates: list[dict[str, str]] = []
    algorithms: list[dict[str, str]] = []
    for gate_id, rows in sorted(grouped.items()):
        rows.sort(key=lambda row: row["contract_id"])
        members = {row["member_contract_id"] for row in rows}
        policies = {row["policy_variant_id"] for row in rows}
        profiles = {row["profile_id"] for row in rows}
        if len(members) != 1 or len(policies) != 1 or len(profiles) != 1:
            raise ProtocolError(f"operation gate identity collision: {gate_id}")
        member = next(iter(members))
        policy = next(iter(policies))
        profile = next(iter(profiles))
        contract_ids = [row["contract_id"] for row in rows]
        route = rust_route_for_member(member)
        shape_ids = shapes_for_member(member)
        gates.append({
            "operation_gate_id": gate_id,
            "member_contract_id": member,
            "policy_variant_id": policy,
            "profile_id": profile,
            "representative_contract_ids": ",".join(contract_ids),
            "representative_contract_set_sha256": digest_value(contract_ids),
            "rust_reference_route_id": route,
            "rust_floor_upper_ratio": "1.02",
            "required_shape_ids": ",".join(shape_ids),
            "required_native_target_ids": ",".join(NATIVE_TARGETS),
            "primary_cell_ids": "PENDING_MATRIX",
            "primary_cell_set_sha256": "PENDING_MATRIX",
            "derivation_reason": (
                f"Every exact normal representative of {member} under {policy} "
                "has an independent same-shape upper-ratio gate on each "
                "selected native target. No other operation or workload can "
                "mask a failure."
            ),
            "status": "FROZEN_PROTOCOL",
        })
        exact = [by_contract[contract_id] for contract_id in contract_ids]
        algorithms.append({
            "algorithm_id": "ALG-" + gate_id[4:],
            "operation_gate_id": gate_id,
            "member_contract_id": member,
            "contract_ids": ",".join(contract_ids),
            "exact_trigger_and_prestate": " || ".join(
                f"{row['contract_id']}: trigger={row['trigger']}; "
                f"pre={row['pre_state']}" for row in exact
            ),
            "exact_commit_and_poststate": " || ".join(
                f"{row['contract_id']}: commit={row['commitment_point']}; "
                f"post={row['post_state']}; equation={row['state_equation']}"
                for row in exact
            ),
            "behavior_schedule": " || ".join(
                f"{row['contract_id']}: {row['behavior_call_count_order_effects']}"
                for row in exact
            ),
            "allocation_and_capacity_rule": " || ".join(
                f"{row['contract_id']}: {row['allocation_disposition']}"
                for row in exact
            ),
            "resource_ceiling": " || ".join(
                f"{row['contract_id']}: {row['resource_ceiling']}"
                for row in exact
            ),
            "rust_reference_route_id": route,
            "member_declaration_sha256": ",".join(sorted({
                row["member_declaration_sha256"] for row in exact
            })),
            "status": "FROZEN_PROTOCOL",
        })
    gates.append({
        "operation_gate_id": "OPG-OD4-SCOPED-CONSUME",
        "member_contract_id": "PROTOCOL-OD4-SCOPED-CONSUME",
        "policy_variant_id": "OD-4-EAGER-AND-SCOPED-CONSUME",
        "profile_id": "PROTOCOL_SYNTHETIC",
        "representative_contract_ids": "OD4-POLICY-SCOPED-CONSUME",
        "representative_contract_set_sha256": FROZEN_OD4_SHA256,
        "rust_reference_route_id": "REF-RUST-EXTRACT",
        "rust_floor_upper_ratio": "1.02",
        "required_shape_ids": "SCOPED-CONTINUE,SCOPED-EARLY-STOP",
        "required_native_target_ids": ",".join(NATIVE_TARGETS),
        "primary_cell_ids": "PENDING_MATRIX",
        "primary_cell_set_sha256": "PENDING_MATRIX",
        "derivation_reason": (
            "The frozen OD4 scoped-consume policy requires independent "
            "continue and early-stop same-shape primary gates, zero mandatory "
            "removed-result allocation, deterministic call order, and repaired "
            "Dense state before normal return."
        ),
        "status": "FROZEN_RESEARCH_PROTOCOL_BLOCKED_ON_EXECUTABLE_ARTIFACT",
    })
    algorithms.append({
        "algorithm_id": "ALG-OD4-SCOPED-CONSUME",
        "operation_gate_id": "OPG-OD4-SCOPED-CONSUME",
        "member_contract_id": "PROTOCOL-OD4-SCOPED-CONSUME",
        "contract_ids": "OD4-POLICY-SCOPED-CONSUME",
        "exact_trigger_and_prestate": (
            "Valid dense owner, exact range/predicate, retained direct consumer, "
            "and no escaping authority."
        ),
        "exact_commit_and_poststate": (
            "Move each selected owner once in source order; on continue, early "
            "stop, or normal error, repair to one valid dense owner before return."
        ),
        "behavior_schedule": (
            "Consumer called exactly once for each visited selected owner; no "
            "later call after early stop."
        ),
        "allocation_and_capacity_rule": (
            "O(1) auxiliary container state and zero removed-result allocation "
            "unless caller explicitly selects collection outside this cell."
        ),
        "resource_ceiling": (
            "One source pass; direct compaction; exact behavior calls; no "
            "persistent cursor or second repair state."
        ),
        "rust_reference_route_id": "REF-RUST-EXTRACT",
        "member_declaration_sha256": FROZEN_OD4_SHA256,
        "status": "FROZEN_RESEARCH_PROTOCOL",
    })
    return gates, algorithms


def payload_size(payload_id: str) -> int:
    return int(next(
        row[3] for row in PAYLOADS if row[0] == payload_id
    ))


def payload_code(payload_id: str) -> str:
    return next(row[1] for row in PAYLOADS if row[0] == payload_id)


def state_dimensions(
    row: dict[str, str],
    payload_id: str,
    shape_id: str,
) -> tuple[int, int, int, int, int]:
    size = payload_size(payload_id)
    if shape_id.startswith("BYTE-"):
        logical_bytes = int(shape_id.split("-", 1)[1])
        length = logical_bytes if size == 0 else max(1, logical_bytes // size)
    elif shape_id == "PROTECTED-B-FIX" or shape_id == "PROTECTED-B-P2":
        return 0, 0, 0, 0, 0
    elif shape_id.startswith("PROTECTED-"):
        length = 32
        logical_bytes = length * max(size, 1)
    else:
        length = 4096 if size == 0 else max(16, 4096 // size)
        logical_bytes = length * size
    label = outcome_label(row) if row.get("outcome_id") else "SUCCESS"
    if payload_id == "P-ZST-AFFINE":
        capacity = (1 << TARGET_BITS[row["target_id"]]) - 1
    elif label in {"SUCCESS_GROW", "SUCCESS_RELOCATE"}:
        capacity = length
    else:
        capacity = max(length, 2 * length)
    request = 1 if label in {"SUCCESS_GROW", "SUCCESS_RELOCATE"} else max(1, length // 8)
    operations = 1
    if row.get("member_contract_id") in {
        "DENSE-ITER-SHARED", "DENSE-ITER-UNIQ", "DENSE-ITER-OWN",
        "PROTOCOL-OD4-SCOPED-CONSUME",
    }:
        operations = length
    if shape_id.startswith("RETAIN-"):
        operations = length
    if shape_id.startswith("SORT-"):
        operations = 1
    return length, capacity, request, operations, logical_bytes


def growth_policy_for(
    member: str, outcome: str, payload_id: str
) -> str:
    if payload_id == "P-ZST-AFFINE":
        return "GROW-ZST-USIZE-MAX"
    if "RESERVE-EXACT" in member:
        return "GROW-RUST-EXACT-1.97"
    if outcome in {"SUCCESS_GROW", "SUCCESS_RELOCATE"}:
        return "GROW-RUST-1.97"
    return "GROW-NONE"


def common_cell_blockers(target_id: str, payload_id: str) -> list[str]:
    result = [
        "PENDING_EXTERNAL_OWNER_AUTHORIZATION",
        "PENDING_EXTERNAL_OWNER_BRANCH_SELECTION",
        "PENDING_EXTERNAL_REPOSITORY_BASELINE",
        "PENDING_EXTERNAL_CANDIDATE_AUTHOR_IDENTITIES",
        "PENDING_EXTERNAL_SERVICE_SNAPSHOTS",
        "PENDING_EXTERNAL_DISCLOSURE_AUTHORITY",
        "PENDING_EXTERNAL_CONSTRUCTION_BUDGET",
        "PENDING_EXTERNAL_FEEDBACK_PROTOCOL",
        "PENDING_EXTERNAL_COMMON_SUBSTRATE_ARTIFACTS",
        "PENDING_EXTERNAL_HARNESS",
        "PENDING_EXTERNAL_RANDOMIZATION_CUSTODY",
        "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "PENDING_EXTERNAL_REFERENCE_PILOT",
        "PENDING_EXTERNAL_FACT_REPORTS",
        "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    ]
    if payload_id != "P-ZST-AFFINE":
        result.append("PENDING_EXTERNAL_ALLOCATOR_ADAPTER")
    if target_id == "TARGET-AARCH64-DARWIN":
        result.extend([
            "PENDING_EXTERNAL_AARCH64_ENVIRONMENT",
            "PENDING_EXTERNAL_AARCH64_CANDIDATE_MODULES",
            "PENDING_EXTERNAL_AARCH64_COUNTER_PROTOCOL",
        ])
    elif target_id == "TARGET-X86_64-LINUX":
        result.extend([
            "PENDING_EXTERNAL_X86_RUNNER",
            "PENDING_EXTERNAL_X86_COUNTER_PROTOCOL",
        ])
    else:
        result.append("PENDING_EXTERNAL_I686_TOOLCHAIN")
    return sorted(set(result))


def make_descriptor_and_cell(
    role: str,
    operation_gate: dict[str, str],
    disposition: dict[str, str],
    source: dict[str, str],
    shape_id: str,
    payload_id: str,
    target_id: str,
    branches: list[dict[str, str]],
    extra_blockers: Iterable[str] = (),
) -> tuple[dict[str, str], dict[str, Any]]:
    synthetic = source.get("contract_id") == "OD4-POLICY-SCOPED-CONSUME"
    member = source["member_contract_id"]
    policy = source["policy_variant_id"]
    outcome = (
        outcome_label(source) if not synthetic else "SCOPED_NORMAL"
    )
    branch_ids = active_branch_ids(
        branches, policy, payload_id, target_id
    )
    if synthetic:
        branch_ids = sorted(
            row["branch_id"] for row in branches
            if row["branch_class"] == "ACTIVE_POWER_BRANCH"
            and target_id in row["required_target_ids"].split(",")
            and payload_id in row["required_payload_ids"].split(",")
        )
    if not branch_ids:
        raise ProtocolError(
            f"primary cell belongs to no owner branch: {member} {payload_id} {target_id}"
        )
    provisional = {
        "role": role,
        "gate": operation_gate["operation_gate_id"],
        "contract": source["contract_id"],
        "shape": shape_id,
        "payload": payload_id,
        "target": target_id,
    }
    cell_id = "DPERF-" + digest_value(provisional)[:20].upper()
    dimension_source = {
        **source,
        "target_id": target_id,
    }
    initial_len, initial_capacity, request_count, operation_count, logical_bytes = (
        state_dimensions(dimension_source, payload_id, shape_id)
    )
    route = operation_gate["rust_reference_route_id"]
    algorithm_id = (
        "ALG-OD4-SCOPED-CONSUME"
        if synthetic else "ALG-" + operation_gate["operation_gate_id"][4:]
    )
    allocator_id = (
        "ALLOC-NONE-ZST" if payload_id == "P-ZST-AFFINE"
        else "ALLOC-COMMON-COUNTED-SYSTEM-V1"
    )
    growth_id = growth_policy_for(member, outcome, payload_id)
    trace_plan = {
        "schema": "xlang-dense-performance-trace-plan-v3",
        "cell_id": cell_id,
        "cell_role": role,
        "operation_gate_id": operation_gate["operation_gate_id"],
        "contract_id": source["contract_id"],
        "source_contract_sha256": disposition["source_contract_sha256"],
        "member_contract_id": member,
        "outcome_id": source["outcome_id"],
        "policy_variant_id": policy,
        "shape_id": shape_id,
        "shape_definition": SHAPE_DEFINITIONS[shape_id][1],
        "payload_id": payload_id,
        "payload_code": payload_code(payload_id),
        "target_id": target_id,
        "layout_id": TARGET_LAYOUT[target_id],
        "owner_branch_ids": branch_ids,
        "initial_state": {
            "len": initial_len,
            "capacity": initial_capacity,
            "logical_bytes": logical_bytes,
        },
        "operation": {
            "request_count": request_count,
            "operation_count": operation_count,
            "trigger": source["trigger"],
            "commitment_point": source["commitment_point"],
        },
        "rust_reference_route_id": route,
        "allocator_id": allocator_id,
        "growth_policy_id": growth_id,
        "common_substrate_sha256": common_substrate_rows()[0][
            "substrate_contract_sha256"
        ],
        "crossover_policy": NO_CROSSOVER,
        "candidate_execution_authorized": False,
    }
    oracle_plan = {
        "schema": "xlang-dense-performance-oracle-plan-v3",
        "cell_id": cell_id,
        "contract_id": source["contract_id"],
        "post_state": source["post_state"],
        "result_owners": source["result_owners"],
        "allocation_disposition": source["allocation_disposition"],
        "behavior_calls": source["behavior_call_count_order_effects"],
        "state_equation": source["state_equation"],
        "resource_ceiling": source["resource_ceiling"],
        "rust_floor_upper_ratio": "1.02",
        "no_pooling": True,
        "zst_allocator_bytes_structural_only": payload_id == "P-ZST-AFFINE",
        "candidate_execution_authorized": False,
    }
    trace_sha = digest_value(trace_plan)
    oracle_sha = digest_value(oracle_plan)
    structural = [
        "SG-EXACT-ORACLE",
        "SG-OWNER-EVENTS",
        "SG-OPERATION-RUST-FLOOR",
        "SG-CODE-UNION",
        "SG-COMMON-SUBSTRATE",
        "SG-FACTS",
        "SG-ZST" if payload_id == "P-ZST-AFFINE" else "SG-ALLOCATION",
    ]
    blockers = sorted(set(
        common_cell_blockers(target_id, payload_id)
        + list(extra_blockers)
    ))
    cell = {
        "cell_id": cell_id,
        "cell_role": role,
        "operation_gate_id": operation_gate["operation_gate_id"],
        "performance_unit_id": disposition["performance_unit_id"],
        "contract_id": source["contract_id"],
        "member_contract_id": member,
        "outcome_id": source["outcome_id"],
        "policy_variant_id": policy,
        "derivation_reason_sha256": digest_value(disposition["exact_reason"]),
        "shape_id": shape_id,
        "payload_id": payload_id,
        "payload_code": payload_code(payload_id),
        "target_id": target_id,
        "layout_id": TARGET_LAYOUT[target_id],
        "owner_branch_ids": ",".join(branch_ids),
        "algorithm_id": algorithm_id,
        "rust_reference_route_id": route,
        "rust_floor_upper_ratio": "1.02",
        "initial_len": str(initial_len),
        "initial_capacity": str(initial_capacity),
        "request_count": str(request_count),
        "operation_count": str(operation_count),
        "logical_bytes": str(logical_bytes),
        "allocator_id": allocator_id,
        "growth_policy_id": growth_id,
        "failure_policy_id": "FP-NORMAL",
        "primary_endpoint_id": "END-RAW-TRACE-NS",
        "descriptive_endpoint_ids": (
            "END-TRACE-LATENCY-P50,END-TRACE-LATENCY-P95,"
            "END-TRACE-LATENCY-P99,END-OP-LATENCY-P50,"
            "END-OP-LATENCY-P95,END-OP-LATENCY-P99,"
            "END-HARDWARE-COUNTERS,END-PEAK-ACQUIRED-BYTES,"
            "END-CODE-UNION-BYTES,END-STRUCTURAL-COUNTERS"
        ),
        "structural_gate_ids": ",".join(structural),
        "counter_policy_id": (
            "COUNTERS-AARCH64-DARWIN"
            if target_id == "TARGET-AARCH64-DARWIN"
            else "COUNTERS-X86_64-LINUX"
        ),
        "warmup_id": (
            "WARMUP-NONE-FRESH"
            if outcome in {"SUCCESS_GROW", "SUCCESS_RELOCATE"}
            or member in SORT_MEMBERS
            else "WARMUP-EXACT-3"
        ),
        "repetition_id": "REP-WILLIAMS-POWER",
        "facts_policy_id": "FACTS-PRIMARY-6-DIAGNOSTIC-OFF",
        "generator_id": "GEN-DENSE-PLAN-V3",
        "trace_sha256": trace_sha,
        "oracle_sha256": oracle_sha,
        "status": PENDING,
        "blocker_ids": ",".join(blockers),
        "candidate_execution_authorized": "NO",
    }
    descriptor = {
        "schema": "xlang-dense-performance-input-v3",
        "cell_id": cell_id,
        "trace_plan": trace_plan,
        "trace_sha256": trace_sha,
        "oracle_plan": oracle_plan,
        "oracle_sha256": oracle_sha,
        "seeds": {
            domain: f"0x{derive_seed(cell_id, domain):016x}"
            for domain in ("trace", "payload", "order", "layout")
        },
        "candidate_execution_authorized": False,
    }
    return cell, descriptor


def derive_seed(cell_id: str, domain: str) -> int:
    raw = hashlib.sha256(
        (cell_id + "\0" + domain).encode("ascii")
    ).digest()
    value = int.from_bytes(raw[:8], "big")
    return value or 0x9E3779B97F4A7C15


PAYLOAD_WITNESS_MEMBERS = {
    "P-U8": "DENSE-SWAP",
    "P-U64": "DENSE-POP",
    "P-ROW24": "DENSE-COPY-FROM",
    "P-ROW56": "DENSE-COPY-WITHIN",
    "P-AFFINE24": "DENSE-CLEAR",
    "P-AFFINE64": "DENSE-REMOVE",
    "P-AFFINE256": "DENSE-SWAP-REMOVE",
    "P-ZST-AFFINE": "DENSE-ITER-OWN",
    "P-BEHAVIOR": "DENSE-CLONE-FROM",
}


def choose_primary_disposition(
    dispositions: list[dict[str, str]],
    member: str,
) -> dict[str, str]:
    rows = sorted(
        (
            row for row in dispositions
            if row["member_contract_id"] == member
            and row["disposition"] == "TIMED_PRIMARY"
            and row["policy_variant_id"] == "COMMON-NON-OD1"
        ),
        key=lambda row: (
            0 if outcome_label(row) == "SUCCESS" else 1,
            row["contract_id"],
        ),
    )
    if not rows:
        raise ProtocolError(f"no common primary witness contract for {member}")
    return rows[0]


def make_control_cell(
    control: dict[str, str],
    target_id: str,
    branches: list[dict[str, str]],
) -> tuple[dict[str, str], dict[str, Any]]:
    control_id = control["control_id"]
    shape_id = "PROTECTED-" + control_id
    payload_id = (
        "P-U8" if control_id in {"B-FIX", "B-P2"}
        else "P-AFFINE64"
    )
    branch_ids = sorted(
        row["branch_id"] for row in branches
        if row["branch_class"] == "ACTIVE_POWER_BRANCH"
        and (
            target_id == "TARGET-I686-STRUCTURAL"
            or target_id in row["required_target_ids"].split(",")
        )
    )
    seed = {
        "role": "PROTECTED_STRUCTURAL",
        "control": control_id,
        "target": target_id,
    }
    cell_id = "DPERF-" + digest_value(seed)[:20].upper()
    trace_plan = {
        "schema": "xlang-dense-protected-trace-plan-v3",
        "cell_id": cell_id,
        "control_id": control_id,
        "shape_id": shape_id,
        "target_id": target_id,
        "layout_id": TARGET_LAYOUT[target_id],
        "source_authority": control["source_authority"],
        "source_sha256": control["source_sha256"],
        "artifact_authorities": control["artifact_authorities"],
        "owner_branch_ids": branch_ids,
        "candidate_execution_authorized": False,
    }
    oracle_plan = {
        "schema": "xlang-dense-protected-oracle-plan-v3",
        "cell_id": cell_id,
        "control_id": control_id,
        "layout_oracle": control["layout_oracle"],
        "code_shape_oracle": control["code_shape_oracle"],
        "structural_oracle": control["structural_oracle"],
        "equality_rule": control["equality_rule"],
        "candidate_execution_authorized": False,
    }
    gate = {
        "B-FIX": "SG-B-FIX",
        "B-P2": "SG-B-P2",
        "H-FLATSET": "SG-H-FLATSET",
        "W-SMALL": "SG-W-SMALL",
        "W-GAP": "SG-W-GAP",
    }[control_id]
    blockers = [
        "PENDING_EXTERNAL_OWNER_AUTHORIZATION",
        "PENDING_EXTERNAL_REPOSITORY_BASELINE",
        "PENDING_EXTERNAL_CANDIDATE_BUILDS",
    ]
    if control_id == "H-FLATSET":
        blockers.append("PENDING_EXTERNAL_H_FLATSET_CUSTODY")
    if control_id == "W-SMALL":
        blockers.append("PENDING_EXTERNAL_W_SMALL_FIXTURE")
    if control_id == "W-GAP":
        blockers.append("PENDING_EXTERNAL_W_GAP_FIXTURE")
    if target_id == "TARGET-AARCH64-DARWIN":
        blockers.extend([
            "PENDING_EXTERNAL_AARCH64_ENVIRONMENT",
            "PENDING_EXTERNAL_AARCH64_CANDIDATE_MODULES",
        ])
    elif target_id == "TARGET-X86_64-LINUX":
        blockers.append("PENDING_EXTERNAL_X86_RUNNER")
    else:
        blockers.append("PENDING_EXTERNAL_I686_TOOLCHAIN")
    trace_sha = digest_value(trace_plan)
    oracle_sha = digest_value(oracle_plan)
    cell = {
        "cell_id": cell_id,
        "cell_role": "PROTECTED_STRUCTURAL",
        "operation_gate_id": "CONTROL-" + control_id,
        "performance_unit_id": "NOT_APPLICABLE_CONTROL",
        "contract_id": "NOT_APPLICABLE_CONTROL",
        "member_contract_id": control_id,
        "outcome_id": "STRUCTURAL_IDENTITY",
        "policy_variant_id": "COMMON_ALL_BRANCHES",
        "derivation_reason_sha256": digest_value(control["equality_rule"]),
        "shape_id": shape_id,
        "payload_id": payload_id,
        "payload_code": payload_code(payload_id),
        "target_id": target_id,
        "layout_id": TARGET_LAYOUT[target_id],
        "owner_branch_ids": ",".join(branch_ids),
        "algorithm_id": "ALG-CONTROL-" + control_id,
        "rust_reference_route_id": (
            "REF-PROTECTED-BASELINE"
            if control_id in {"B-FIX", "B-P2"}
            else "REF-RUST-API-INVENTORY"
        ),
        "rust_floor_upper_ratio": "EXACT_STRUCTURAL_EQUALITY",
        "initial_len": "0",
        "initial_capacity": "0",
        "request_count": "0",
        "operation_count": "1",
        "logical_bytes": "0",
        "allocator_id": (
            "ALLOC-PROTECTED-HISTORICAL"
            if control_id in {"B-FIX", "B-P2"}
            else "ALLOC-COMMON-COUNTED-SYSTEM-V1"
        ),
        "growth_policy_id": "GROW-NONE",
        "failure_policy_id": "FP-PROTECTED-IDENTITY",
        "primary_endpoint_id": "NONE",
        "descriptive_endpoint_ids": (
            "END-CODE-UNION-BYTES,END-STRUCTURAL-COUNTERS"
        ),
        "structural_gate_ids": gate,
        "counter_policy_id": (
            "COUNTERS-AARCH64-DARWIN"
            if target_id == "TARGET-AARCH64-DARWIN"
            else "COUNTERS-X86_64-LINUX"
            if target_id == "TARGET-X86_64-LINUX"
            else "COUNTERS-NOT-APPLICABLE-I686"
        ),
        "warmup_id": "WARMUP-NONE-STRUCTURAL",
        "repetition_id": "REP-STRUCTURAL-2",
        "facts_policy_id": "FACTS-STRUCTURAL",
        "generator_id": "GEN-PROTECTED-V3",
        "trace_sha256": trace_sha,
        "oracle_sha256": oracle_sha,
        "status": PENDING,
        "blocker_ids": ",".join(sorted(set(blockers))),
        "candidate_execution_authorized": "NO",
    }
    descriptor = {
        "schema": "xlang-dense-protected-input-v3",
        "cell_id": cell_id,
        "trace_plan": trace_plan,
        "trace_sha256": trace_sha,
        "oracle_plan": oracle_plan,
        "oracle_sha256": oracle_sha,
        "seeds": {},
        "candidate_execution_authorized": False,
    }
    return cell, descriptor


def make_arithmetic_cell(
    target_id: str,
    branches: list[dict[str, str]],
) -> tuple[dict[str, str], dict[str, Any]]:
    bits = TARGET_BITS[target_id]
    shape_id = f"ARITHMETIC-{bits}-LAST-FIRST"
    branch_ids = sorted(
        row["branch_id"] for row in branches
        if row["branch_class"] == "ACTIVE_POWER_BRANCH"
        and (
            target_id == "TARGET-I686-STRUCTURAL"
            or target_id in row["required_target_ids"].split(",")
        )
    )
    cell_id = "DPERF-" + digest_value(
        ["ARITHMETIC", target_id, bits]
    )[:20].upper()
    last_valid = (1 << bits) - 2
    trace_plan = {
        "schema": "xlang-dense-arithmetic-trace-plan-v3",
        "cell_id": cell_id,
        "target_id": target_id,
        "layout_id": TARGET_LAYOUT[target_id],
        "pointer_bits": bits,
        "tuples": [
            {
                "name": "last_valid_add",
                "len": last_valid,
                "request": 1,
                "expected": (1 << bits) - 1,
            },
            {
                "name": "first_invalid_add",
                "len": last_valid,
                "request": 2,
                "expected": "CHECKED_FAILURE_BEFORE_ALLOCATION",
            },
        ],
        "candidate_execution_authorized": False,
    }
    oracle_plan = {
        "schema": "xlang-dense-arithmetic-oracle-plan-v3",
        "cell_id": cell_id,
        "pass_rule": (
            "Exact last-valid result and exact first failed premise; zero "
            "allocator calls and zero payload access."
        ),
        "candidate_execution_authorized": False,
    }
    trace_sha = digest_value(trace_plan)
    oracle_sha = digest_value(oracle_plan)
    blockers = [
        "PENDING_EXTERNAL_OWNER_AUTHORIZATION",
        "PENDING_EXTERNAL_REPOSITORY_BASELINE",
    ]
    if target_id == "TARGET-I686-STRUCTURAL":
        blockers.append("PENDING_EXTERNAL_I686_TOOLCHAIN")
    elif target_id == "TARGET-X86_64-LINUX":
        blockers.append("PENDING_EXTERNAL_X86_RUNNER")
    else:
        blockers.append("PENDING_EXTERNAL_AARCH64_CANDIDATE_MODULES")
    cell = {
        "cell_id": cell_id,
        "cell_role": "ARITHMETIC_STRUCTURAL",
        "operation_gate_id": "CONTROL-ARITHMETIC",
        "performance_unit_id": "NOT_APPLICABLE_CONTROL",
        "contract_id": "NOT_APPLICABLE_CONTROL",
        "member_contract_id": "DENSE-CAPACITY-ARITHMETIC",
        "outcome_id": "LAST_VALID_AND_FIRST_INVALID",
        "policy_variant_id": "COMMON_ALL_BRANCHES",
        "derivation_reason_sha256": digest_value(oracle_plan["pass_rule"]),
        "shape_id": shape_id,
        "payload_id": "P-U8",
        "payload_code": "U8",
        "target_id": target_id,
        "layout_id": TARGET_LAYOUT[target_id],
        "owner_branch_ids": ",".join(branch_ids),
        "algorithm_id": "ALG-CONTROL-ARITHMETIC",
        "rust_reference_route_id": "REF-RUST-RAWVEC",
        "rust_floor_upper_ratio": "EXACT_STRUCTURAL_EQUALITY",
        "initial_len": str(last_valid),
        "initial_capacity": str((1 << bits) - 1),
        "request_count": "2",
        "operation_count": "2",
        "logical_bytes": "0",
        "allocator_id": "ALLOC-NONE-ZST",
        "growth_policy_id": "GROW-NONE",
        "failure_policy_id": "FP-FUNCTIONAL-ONLY",
        "primary_endpoint_id": "NONE",
        "descriptive_endpoint_ids": "END-STRUCTURAL-COUNTERS",
        "structural_gate_ids": "SG-ARITHMETIC-32-64",
        "counter_policy_id": (
            "COUNTERS-AARCH64-DARWIN"
            if target_id == "TARGET-AARCH64-DARWIN"
            else "COUNTERS-X86_64-LINUX"
            if target_id == "TARGET-X86_64-LINUX"
            else "COUNTERS-NOT-APPLICABLE-I686"
        ),
        "warmup_id": "WARMUP-NONE-STRUCTURAL",
        "repetition_id": "REP-STRUCTURAL-2",
        "facts_policy_id": "FACTS-STRUCTURAL",
        "generator_id": "GEN-PROTECTED-V3",
        "trace_sha256": trace_sha,
        "oracle_sha256": oracle_sha,
        "status": PENDING,
        "blocker_ids": ",".join(sorted(blockers)),
        "candidate_execution_authorized": "NO",
    }
    descriptor = {
        "schema": "xlang-dense-protected-input-v3",
        "cell_id": cell_id,
        "trace_plan": trace_plan,
        "trace_sha256": trace_sha,
        "oracle_plan": oracle_plan,
        "oracle_sha256": oracle_sha,
        "seeds": {},
        "candidate_execution_authorized": False,
    }
    return cell, descriptor


def build_matrix(
    contract_rows: list[dict[str, str]],
    dispositions: list[dict[str, str]],
    gates: list[dict[str, str]],
    branches: list[dict[str, str]],
) -> tuple[list[dict[str, str]], list[dict[str, Any]]]:
    source_by_contract = {row["contract_id"]: row for row in contract_rows}
    disposition_by_contract = {
        row["contract_id"]: row for row in dispositions
    }
    gate_by_id = {row["operation_gate_id"]: row for row in gates}
    matrix: list[dict[str, str]] = []
    descriptors: list[dict[str, Any]] = []

    def add(cell_and_descriptor: tuple[dict[str, str], dict[str, Any]]) -> None:
        cell, descriptor = cell_and_descriptor
        matrix.append(cell)
        descriptors.append(descriptor)

    for disposition in dispositions:
        if disposition["disposition"] != "TIMED_PRIMARY":
            continue
        source = source_by_contract[disposition["contract_id"]]
        gate = gate_by_id[disposition["representative_operation_gate_id"]]
        payload_id = default_payload_for(source)
        for shape_id in gate["required_shape_ids"].split(","):
            for target_id in NATIVE_TARGETS:
                add(make_descriptor_and_cell(
                    "PRIMARY_OPERATION",
                    gate,
                    disposition,
                    source,
                    shape_id,
                    payload_id,
                    target_id,
                    branches,
                ))

    for payload_id, member in PAYLOAD_WITNESS_MEMBERS.items():
        disposition = choose_primary_disposition(dispositions, member)
        source = source_by_contract[disposition["contract_id"]]
        gate = gate_by_id[disposition["representative_operation_gate_id"]]
        shape_id = "PAYLOAD-" + payload_code(payload_id)
        for target_id in NATIVE_TARGETS:
            add(make_descriptor_and_cell(
                "PAYLOAD_SEPARATOR_PRIMARY",
                gate,
                disposition,
                source,
                shape_id,
                payload_id,
                target_id,
                branches,
            ))

    boundary_disposition = choose_primary_disposition(
        dispositions, "DENSE-RESERVE"
    )
    boundary_source = source_by_contract[boundary_disposition["contract_id"]]
    boundary_gate = gate_by_id[
        boundary_disposition["representative_operation_gate_id"]
    ]
    for boundary in (64, 4096, 262144, 4194304, 67108864):
        for target_id in NATIVE_TARGETS:
            add(make_descriptor_and_cell(
                "BYTE_BOUNDARY_PRIMARY",
                boundary_gate,
                boundary_disposition,
                boundary_source,
                f"BYTE-{boundary}",
                "P-U8",
                target_id,
                branches,
            ))

    scoped_gate = gate_by_id["OPG-OD4-SCOPED-CONSUME"]
    scoped_source = {
        "contract_id": "OD4-POLICY-SCOPED-CONSUME",
        "member_contract_id": "PROTOCOL-OD4-SCOPED-CONSUME",
        "outcome_id": "OD4.SCOPED.NORMAL",
        "policy_variant_id": "OD-4-EAGER-AND-SCOPED-CONSUME",
        "profile_id": "PROTOCOL_SYNTHETIC",
        "trigger": (
            "Exact range/predicate and direct consumer are valid and nonescaping."
        ),
        "pre_state": "One ValidDense owner and exact retained consumer state.",
        "commitment_point": "Move selected owner immediately before direct call.",
        "post_state": (
            "One repaired ValidDense owner plus exact retained consumer result."
        ),
        "result_owners": "Repaired BASE and declared consumer result owners.",
        "allocation_disposition": (
            "No removed-result allocation; base capacity follows exact input."
        ),
        "behavior_call_count_order_effects": (
            "Exactly once per visited selected owner in increasing source order."
        ),
        "state_equation": (
            "Every selected owner is consumed once; every retained owner appears "
            "once in repaired BASE before normal return."
        ),
        "resource_ceiling": (
            "O(n) direct pass, O(1) auxiliary container state, zero mandatory "
            "removed-result allocation."
        ),
    }
    scoped_disposition = {
        "performance_unit_id": "PERFUNIT-OD4-SCOPED",
        "source_contract_sha256": FROZEN_OD4_SHA256,
        "exact_reason": scoped_gate["derivation_reason"],
    }
    for shape_id in ("SCOPED-CONTINUE", "SCOPED-EARLY-STOP"):
        for target_id in NATIVE_TARGETS:
            add(make_descriptor_and_cell(
                "OD4_SCOPED_PRIMARY",
                scoped_gate,
                scoped_disposition,
                scoped_source,
                shape_id,
                "P-BEHAVIOR",
                target_id,
                branches,
                (
                    "PENDING_EXTERNAL_OD4_SCOPED_CONTRACT",
                    "PENDING_EXTERNAL_OD4_CANDIDATE_ARTIFACTS",
                ),
            ))

    controls = {row["control_id"]: row for row in CONTROLS}
    for control_id in ("B-FIX", "B-P2", "H-FLATSET", "W-SMALL", "W-GAP"):
        for target_id in ALL_TARGETS:
            add(make_control_cell(controls[control_id], target_id, branches))

    for target_id in ALL_TARGETS:
        add(make_arithmetic_cell(target_id, branches))

    cell_ids = [row["cell_id"] for row in matrix]
    if len(cell_ids) != len(set(cell_ids)):
        duplicates = [
            cell_id for cell_id, count in Counter(cell_ids).items()
            if count > 1
        ]
        raise ProtocolError(f"duplicate performance cell IDs: {duplicates[:5]}")
    trace_ids = [row["trace_sha256"] for row in matrix]
    oracle_ids = [row["oracle_sha256"] for row in matrix]
    if len(trace_ids) != len(set(trace_ids)):
        raise ProtocolError("performance trace hashes are not unique")
    if len(oracle_ids) != len(set(oracle_ids)):
        raise ProtocolError("performance oracle hashes are not unique")
    by_gate: dict[str, list[str]] = defaultdict(list)
    for row in matrix:
        if row["primary_endpoint_id"] == "END-RAW-TRACE-NS":
            by_gate[row["operation_gate_id"]].append(row["cell_id"])
    for gate in gates:
        ids = sorted(by_gate[gate["operation_gate_id"]])
        if not ids:
            raise ProtocolError(
                f"operation gate has no primary cells: {gate['operation_gate_id']}"
            )
        gate["primary_cell_ids"] = ",".join(ids)
        gate["primary_cell_set_sha256"] = digest_value(ids)
    matrix.sort(key=lambda row: row["cell_id"])
    descriptors.sort(key=lambda row: row["cell_id"])
    return matrix, descriptors


def finalize_owner_branches(
    branches: list[dict[str, str]],
    matrix: list[dict[str, str]],
) -> None:
    for branch in branches:
        if branch["branch_class"] != "ACTIVE_POWER_BRANCH":
            continue
        ids = sorted(
            row["cell_id"] for row in matrix
            if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            and branch["branch_id"] in row["owner_branch_ids"].split(",")
        )
        branch["primary_cell_count"] = str(len(ids))
        branch["primary_cell_ids_sha256"] = digest_value(ids)


def generated_input_rows(
    descriptors: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    return [
        {
            "schema": "xlang-dense-performance-generated-input-v3",
            "cell_id": row["cell_id"],
            "descriptor_sha256": digest_value(row),
            "trace_sha256": row["trace_sha256"],
            "oracle_sha256": row["oracle_sha256"],
            "seeds": row["seeds"],
            "candidate_execution_authorized": False,
        }
        for row in descriptors
    ]


WILLIAMS_NUMERIC = [
    [1, 2, 6, 3, 5, 4],
    [2, 3, 1, 4, 6, 5],
    [3, 4, 2, 5, 1, 6],
    [4, 5, 3, 6, 2, 1],
    [5, 6, 4, 1, 3, 2],
    [6, 1, 5, 2, 4, 3],
]


BENEFIT_ENDPOINT_ALTERNATIVES = (
    {
        "endpoint_id": "END-RAW-TRACE-NS",
        "acceptance_upper_ratio": "90/100",
        "injected_true_ratio": "85/100",
        "eligibility_rule": "Every exact primary timed cell.",
    },
    {
        "endpoint_id": "END-PEAK-ACQUIRED-BYTES",
        "acceptance_upper_ratio": "85/100",
        "injected_true_ratio": "80/100",
        "eligibility_rule": (
            "Every non-ZST primary cell whose frozen reference-only Rust "
            "peak-acquired-byte value is positive. A zero Rust value remains "
            "a structural equality case and is never an injected benefit."
        ),
    },
)


def exact_alternative_matrix(
    winner: str,
    endpoint_id: str,
) -> dict[str, Any]:
    specs = {
        row["endpoint_id"]: row for row in BENEFIT_ENDPOINT_ALTERNATIVES
    }
    if endpoint_id not in specs:
        raise ProtocolError(f"unknown benefit endpoint: {endpoint_id}")
    spec = specs[endpoint_id]
    true_ratio = spec["injected_true_ratio"]
    numerator, denominator = true_ratio.split("/")
    reciprocal_ratio = f"{denominator}/{numerator}"
    means = {
        treatment: true_ratio if treatment == winner else "1/1"
        for treatment in TREATMENTS
    }
    pairwise: dict[str, dict[str, str]] = {}
    for left in TREATMENTS:
        pairwise[left] = {}
        for right in TREATMENTS:
            if left == right:
                value = "1/1"
            elif left == winner:
                value = true_ratio
            elif right == winner:
                value = reciprocal_ratio
            else:
                value = "1/1"
            pairwise[left][right] = value
    return {
        "winner_id": winner,
        "benefit_endpoint_id": endpoint_id,
        "acceptance_upper_ratio": spec["acceptance_upper_ratio"],
        "injected_true_ratio": true_ratio,
        "eligibility_rule": spec["eligibility_rule"],
        "benefit_cell_mean_ratios_to_rust": means,
        "benefit_cell_pairwise_ratio_matrix": pairwise,
        "all_other_primary_cell_pairwise_ratio": "1/1",
        "interpretation": (
            f"For {endpoint_id}, the named winner has one exact eligible true "
            f"benefit cell at ratio {true_ratio} against Rust and every other "
            "candidate; all other cells are true ratio 1/1. The injected true "
            f"ratio is strictly below the frozen acceptance upper ratio "
            f"{spec['acceptance_upper_ratio']}."
        ),
    }


def benefit_hypothesis_ids(
    branch_id: str,
    matrix: list[dict[str, str]],
) -> list[str]:
    cells = [
        row for row in matrix
        if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
        and branch_id in row["owner_branch_ids"].split(",")
    ]
    result: list[str] = []
    for left in CANDIDATES:
        for right in CANDIDATES:
            if left == right:
                continue
            for cell in cells:
                result.append(
                    f"{left}>{right}|{cell['cell_id']}|END-RAW-TRACE-NS"
                )
                if cell["payload_id"] != "P-ZST-AFFINE":
                    result.append(
                        f"{left}>{right}|{cell['cell_id']}|"
                        "END-PEAK-ACQUIRED-BYTES"
                    )
    return sorted(result)


def scheduled_mixture_tail_protocol() -> dict[str, Any]:
    return {
        "schema": "xlang-dense-scheduled-mixture-tail-v1",
        "nuisance_coordinate_count": 30,
        "nuisance_coordinates": "six Williams rows times five layout salts",
        "allowed_block_counts": [60, 90, 120],
        "equal_repetitions_per_coordinate": {"60": 2, "90": 3, "120": 4},
        "null": (
            "For one exact cell, endpoint, and directed pair, conditional on the "
            "frozen assignment manifest let p_(c,h) be each scheduled slot's strict-"
            "success probability over r cycles and 30 row-by-salt coordinates. H0 "
            "is sum(p_(c,h))/(30*r)<=1/2. It is not a claim that every slot or "
            "nuisance coordinate has probability <=1/2."
        ),
        "independence_condition": (
            "Conditional on the fixed assignment and admitted machine state, the "
            "n scheduled fresh-child block signs are independent Bernoulli trials; "
            "their probabilities may differ by nuisance coordinate."
        ),
        "exact_p_value": (
            "Let s be strict successes. If 2*s<=n, p=1/1. Otherwise p_num="
            "sum(comb(n,j),j=s..n), p_den=2^n, reduced by gcd. This is the exact "
            "worst-case upper-tail p-value over independent heterogeneous Bernoulli "
            "trials with sum(p_i)<=n/2."
        ),
        "extremal_authority": (
            "Wassily Hoeffding, On the Distribution of the Number of Successes "
            "in Independent Trials, Annals of Mathematical Statistics 27 (1956), "
            "713-721, DOI 10.1214/aoms/1177728178, Theorem 4 and its extremal "
            "corollary."
        ),
        "extremal_verifier_rule": (
            "For n=60,90,120 enumerate every Hoeffding extremal vector with a "
            "ones, b equal rational probabilities p=(n/2-a)/b, and remaining "
            "zeros. Verify the supremum is 1 for 2*s<=n and the Binomial(n,1/2) "
            "tail otherwise."
        ),
        "p_value_arithmetic": (
            "Use unbounded nonnegative integers and reduced rationals for all "
            "combinations, tails, gcds, Holm thresholds, and cross-products."
        ),
    }


def benefit_testing_protocol() -> dict[str, Any]:
    tail = scheduled_mixture_tail_protocol()
    return {
        "family_alpha": "1/200",
        "family_membership": (
            "For the owner-selected branch, include every ID returned by "
            "benefit_hypothesis_ids exactly once. Rust is deliberately absent: "
            "Rust is the nonselectable NI floor, while benefit hypotheses establish "
            "dominance only among the five selectable candidates. Non-ZST "
            "structural-zero memory IDs remain with p=1; ZST cells are timing-only."
        ),
        "family_size_range": [8120, 16280],
        "observational_unit": (
            "One scheduled complete six-treatment block supplies one raw paired "
            "C/D integer comparison for one exact cell and endpoint."
        ),
        "endpoint_nulls": [
            {
                "endpoint_id": "END-RAW-TRACE-NS",
                "threshold_ratio": "9/10",
                "strict_success_cross_product": "10*C.elapsed_ns < 9*D.elapsed_ns",
                "null": "scheduled-mixture strict-success probability <=1/2",
            },
            {
                "endpoint_id": "END-PEAK-ACQUIRED-BYTES",
                "threshold_ratio": "17/20",
                "strict_success_cross_product": (
                    "C.peak_acquired_bytes=0 and D.peak_acquired_bytes>0, or both "
                    "positive and 20*C.peak_acquired_bytes < 17*D.peak_acquired_bytes"
                ),
                "null": "scheduled-mixture strict-success probability <=1/2",
            },
        ],
        "scheduled_mixture_tail_protocol_sha256": digest_value(tail),
        "one_sided_p_value": tail["exact_p_value"],
        "zero_reference_memory_rule": (
            "If all reference-pilot memory values are zero, candidate structural "
            "admission requires zero and each ordered memory-benefit ID receives "
            "p=1/1. Mixed zero/positive reference memory makes the branch infeasible."
        ),
        "tie_and_unusable_rule": (
            "Cross-product equality, malformed input, or any scheduled candidate "
            "slot that does not satisfy strict_success counts as failure and cannot "
            "reduce n, be replaced, or trigger an extra observation."
        ),
        "statistic": "s is the count of strict successes among fixed n slots.",
        "block_validity_rule": (
            "n is exactly 60, 90, or 120 with equal repetitions of all thirty "
            "nuisance coordinates. Every slot contributes success or failure."
        ),
        "descriptive_ratio_interval": (
            "Sort exact raw ratios by integer cross-products and report the reduced-"
            "rational central interval [r_(n/2),r_(n/2+1)]. If any fixed slot is "
            "unusable, report UNAVAILABLE plus the exact unusable-slot count; never "
            "sort a reduced-n subset. It is descriptive only."
        ),
        "minimum_block_count": 60,
        "minimum_attainable_p_value": "1/1152921504606846976",
        "maximum_family_size": 16280,
        "most_stringent_holm_threshold": "1/3256000",
        "holm_order": (
            "Sort by ascending exact rational p-value; break equal-p ties by "
            "bytewise ASCII hypothesis_id."
        ),
        "holm_step_down": (
            "For sorted rank i=1..m, reject while p_i <= (1/200)/(m-i+1) by "
            "unbounded-integer cross-products. Stop at first nonrejection."
        ),
        "inference_engine_rule": (
            "Inference and power use no fitted nuisance adjustment, logarithm, "
            "floating-point threshold, Monte Carlo draw, or approximate p-value. "
            "Power deterministically enumerates the explicit empirical resampling law."
        ),
    }


def endpoint_response_protocol() -> list[dict[str, str]]:
    return [
        {
            "endpoint_id": "END-RAW-TRACE-NS",
            "raw_field": "elapsed_ns",
            "eligibility": "Every admitted row; elapsed_ns must be positive.",
            "transform": "raw positive elapsed_ns integer",
            "zero_or_mixed_rule": "Any nonpositive value invalidates the block.",
        },
        {
            "endpoint_id": "END-PEAK-ACQUIRED-BYTES",
            "raw_field": "allocator_counters.peak_acquired_bytes",
            "eligibility": (
                "Non-ZST cell/target classified POSITIVE_REFERENCE_MEMORY from "
                "Rust pseudo-treatment rows."
            ),
            "transform": "raw nonnegative peak_acquired_bytes integer",
            "zero_or_mixed_rule": (
                "All zero is STRUCTURAL_ZERO_MEMORY and has no memory response; "
                "a zero/positive mixture invalidates the branch; all positive "
                "values enter exact integer cross-products."
            ),
        },
    ]


def noninferiority_testing_protocol() -> dict[str, Any]:
    tail = scheduled_mixture_tail_protocol()
    return {
        "family_alpha": "1/200",
        "directed_claim_count": 25,
        "per_directed_claim_alpha": "1/5000",
        "endpoint_scope": ["END-RAW-TRACE-NS"],
        "threshold_ratio": "51/50",
        "null": (
            "For one directed treatment pair, H0 is that at least one required "
            "exact timing cell has scheduled-mixture strict-success probability "
            "<=1/2. The IUT is across semantic cells, never nuisance coordinates."
        ),
        "strict_cell_success": "50*C.elapsed_ns < 51*D.elapsed_ns",
        "scheduled_mixture_tail_protocol_sha256": digest_value(tail),
        "point_estimator": (
            "Sort raw ratios by integer cross-products and report the exact reduced-"
            "rational central interval [r_(n/2),r_(n/2+1)]. Any unusable slot makes "
            "the interval UNAVAILABLE with exact count; selection still uses fixed-n s."
        ),
        "cell_statistic": (
            "For fixed n, s is the number of scheduled block slots satisfying "
            "strict_cell_success. Threshold equality is failure."
        ),
        "cell_p_value": (
            "Use the globally valid piecewise worst-case scheduled-mixture p-value "
            "and compare it with 1/5000 using unbounded integers."
        ),
        "critical_success_counts": {"60": 44, "90": 63, "120": 80},
        "directed_iut_rule": (
            "A directed claim passes only if every required exact timing cell "
            "has cell p-value <=1/5000. There is no cell pooling, averaging, "
            "or fresh alpha allocation."
        ),
        "missing_nonfinite_rule": (
            "Every selected scheduled slot must be valid. A missing, malformed, "
            "incomplete, nonpositive, or out-of-range integer timing observation sets the affected "
            "cell p-value to 1/1 and fails every directed IUT claim that requires "
            "that comparison. It cannot reduce n, be replaced, or trigger an "
            "extra observation."
        ),
        "confidence_interpretation": (
            "Cell rejection is the exact scheduled-mixture tail decision at "
            "1/5000; all required semantic cells must reject."
        ),
        "stochastic_resampling_rule": (
            "No bootstrap, Monte Carlo p-value, RNG, quantile rank, or empirical "
            "resolution enters noninferiority inference."
        ),
        "memory_rule": (
            "Memory does not participate in noninferiority. It is an exact "
            "structural admission and global-benefit endpoint only."
        ),
    }


def randomization_protocol() -> dict[str, Any]:
    pilot_seed = bytes(range(32))
    candidate_seed = bytes(range(32, 64))
    pilot_fields = [
        ("seed", "BYTES", pilot_seed),
        ("owner_branch_id", "ASCII", b"BR-TEST"),
        ("registry_sha256", "ASCII", b"0" * 64),
    ]
    candidate_fields = [
        ("seed", "BYTES", candidate_seed),
        ("owner_branch_id", "ASCII", b"BR-TEST"),
        ("selected_block_count", "ASCII", b"60"),
        ("power_result_sha256", "ASCII", b"1" * 64),
        ("candidate_freeze_b_sha256", "ASCII", b"2" * 64),
        ("matrix_sha256", "ASCII", b"3" * 64),
    ]
    pilot_message = encode_randomization_message("reference-root", pilot_fields)
    candidate_message = encode_randomization_message(
        "candidate-root", candidate_fields
    )
    candidate_root = hashlib.sha256(candidate_message).digest()
    mapping_message = encode_randomization_message(
        "permutation-manifest-row",
        [
            ("root", "BYTES", candidate_root),
            ("cell_id", "ASCII", b"CELL-TEST"),
            ("target_id", "ASCII", b"TARGET-TEST"),
            ("complete_cycle_index", "ASCII", b"1"),
            ("layout_salt_id", "ASCII", b"SALT-1"),
            ("draw_index", "ASCII", b"7"),
            ("rejection_count", "ASCII", b"0"),
            ("permutation_rank", "ASCII", b"17"),
        ],
    )
    block_order_vectors = []
    for williams_row_id in ("WILLIAMS-1", "WILLIAMS-2", "WILLIAMS-3"):
        block_fields = [
            ("owner_branch_id", "ASCII", b"BR-TEST"),
            ("cell_id", "ASCII", b"CELL-TEST"),
            ("target_id", "ASCII", b"TARGET-TEST"),
            ("complete_cycle_index", "ASCII", b"1"),
            ("layout_salt_id", "ASCII", b"SALT-1"),
            ("williams_row_id", "ASCII", williams_row_id.encode("ascii")),
        ]
        identity_message = encode_randomization_message(
            "block-identity", block_fields
        )
        block_id = "BLOCK-" + hashlib.sha256(identity_message).hexdigest().upper()
        message = encode_randomization_message(
            "block-order",
            [
                ("root", "BYTES", candidate_root),
                *block_fields,
                ("block_id", "ASCII", block_id.encode("ascii")),
            ],
        )
        block_order_vectors.append({
            "block_id": block_id,
            "owner_branch_id": "BR-TEST",
            "cell_id": "CELL-TEST",
            "target_id": "TARGET-TEST",
            "complete_cycle_index": "1",
            "layout_salt_id": "SALT-1",
            "williams_row_id": williams_row_id,
            "block_identity_message_hex": identity_message.hex(),
            "block_identity_sha256": hashlib.sha256(identity_message).hexdigest(),
            "message_hex": message.hex(),
            "key_sha256": hashlib.sha256(message).hexdigest(),
        })
    block_order_vectors.sort(
        key=lambda row: (row["key_sha256"], row["block_id"])
    )
    for rank, row in enumerate(block_order_vectors, start=1):
        row["global_execution_rank"] = rank
    return {
        "schema": "xlang-dense-randomization-custody-v2",
        "status": "BLOCKED_RANDOMIZATION_CUSTODY_ABSENT",
        "hash_and_encoding": (
            "XLANG-DENSE-RANDOMIZATION-V2 NUL magic; u64be domain length and ASCII "
            "domain; u64be field count; for each ordered field, u64be ASCII-name "
            "length, name, one-byte type A or B, u64be value length, and value. "
            "ASCII values are strict 0x20..0x7e; BYTES values are unmodified bytes."
        ),
        "seed_custody": (
            "An independent custodian generates two independent uniform 32-byte "
            "seeds. Commit SHA256(raw seed) for the reference seed before the pilot "
            "and release it only to the isolated reference runner. Commit the "
            "candidate seed before Candidate Freeze B and release it only to the "
            "isolated candidate runner after Freeze B. Both stay hidden from all "
            "candidate authors; commitments, releases, and receipts are distinct."
        ),
        "uniform_rank_sampling_protocol": (
            "For each stage, sort canonical (branch,cell,target,cycle,salt) mapping "
            "keys bytewise ASCII. From the independently seeded frozen CSPRNG consume "
            "16-bit big-endian words in transcript order; reject x>=65520 and accept "
            "rank=x mod720. Rank indexes bytewise-ASCII lexicographic permutations. "
            "Record draw index, rejection count, and rank. Any CSPRNG/transcript "
            "failure is PROTOCOL_INFEASIBLE; never redraw a committed seed."
        ),
        "reference_pilot_root": (
            "Hash the typed grammar whose domain string is reference-root with the "
            "raw reference seed, branch, and performance-registry-summary hash."
        ),
        "candidate_root_binding": (
            "After power selection and Candidate Freeze B, hash the typed grammar "
            "with raw seed, selected branch, selected block count, exact power-"
            "result hash, Candidate Freeze B hash, and matrix hash."
        ),
        "global_block_order": (
            "Create all blocks first. A canonical block identity is the typed "
            "block-identity message with ordered ASCII fields owner_branch_id, "
            "cell_id, target_id, complete_cycle_index, layout_salt_id, and "
            "williams_row_id. Its unique block_id is uppercase ASCII BLOCK- plus "
            "the SHA-256 hex digest of that complete identity message; duplicate "
            "identities or IDs invalidate the manifest. The typed block-order "
            "message has ordered fields root as BYTES, then those six ASCII "
            "identity fields, then block_id as ASCII. Sort by its SHA-256 digest "
            "and break a digest tie by bytewise ASCII block_id."
        ),
        "within_block_order": (
            "Execute the six mapped treatments in the frozen Williams-row order; "
            "do not shuffle positions independently."
        ),
        "mapping_scope": (
            "Use one manifest permutation per exact cell, target, complete-cycle "
            "index, and layout salt in both pilot and candidate stages. Hold it fixed "
            "across that salt's six Williams rows; derive the next rank only from the "
            "stage's sorted key stream. Pilot supports are indexed by realized numeric "
            "symbol, never by a static pseudo-to-symbol assumption."
        ),
        "randomness_interpretation": (
            "Operational ranks are computationally pseudorandom under the frozen "
            "CSPRNG and hidden-seed custody. Q/P/M power is an exact finite calculation "
            "under the preregistered ideal independent-uniform rank planning model; "
            "it is not a proof that a deterministic CSPRNG transcript is independent."
        ),
        "assignment_manifest": (
            "Before execution, commit each stage's full canonical permutation-rank "
            "manifest and canonical assignment JSON Lines with row ID, campaign "
            "domain, branch, seed commitment, CSPRNG identity and transcript hash, "
            "root hash, cell, target, cycle, salt, "
            "Williams row, global execution rank, rejection_count, "
            "permutation rank, period, predecessor, treatment or pseudo ID, "
            "descriptor hash, and row hash."
        ),
        "raw_row_binding": (
            "Every raw row carries randomization_commitment_sha256, "
            "assignment_manifest_sha256, and assignment_row_id; exact lookup, "
            "treatment, predecessor, period, and execution rank must match."
        ),
        "failure_rule": (
            "A missing commitment, early disclosure, duplicate/missing assignment, "
            "manifest mismatch, order mismatch, or custody-log gap invalidates the "
            "owner branch. No seed rotation, rerandomization, or replacement run "
            "is permitted after observation."
        ),
        "power_mapping_reduction": (
            "For any marginal directed C/D event, each per-salt/cycle exact-uniform "
            "720-mapping average reduces to the 30 ordered numeric-symbol pairs; "
            "the remaining 4! labels cancel. The power DP keeps the same mapping "
            "across six rows but integrates mappings independently by salt and cycle."
        ),
        "test_vectors": {
            "reference_seed_hex": pilot_seed.hex(),
            "reference_seed_commitment_sha256": hashlib.sha256(pilot_seed).hexdigest(),
            "candidate_seed_hex": candidate_seed.hex(),
            "candidate_seed_commitment_sha256": hashlib.sha256(candidate_seed).hexdigest(),
            "reference_root_message_hex": pilot_message.hex(),
            "reference_root_message_sha256": hashlib.sha256(pilot_message).hexdigest(),
            "candidate_root_message_hex": candidate_message.hex(),
            "candidate_root_message_sha256": candidate_root.hex(),
            "permutation_manifest_row_message_hex": mapping_message.hex(),
            "permutation_manifest_row_message_sha256": hashlib.sha256(mapping_message).hexdigest(),
            "block_order_rows": block_order_vectors,
        },
    }


def encode_randomization_message(
    domain: str,
    fields: list[tuple[str, str, bytes]],
) -> bytes:
    def u64be(value: int) -> bytes:
        return value.to_bytes(8, "big")

    domain_bytes = domain.encode("ascii")
    result = bytearray(b"XLANG-DENSE-RANDOMIZATION-V2\x00")
    result.extend(u64be(len(domain_bytes)))
    result.extend(domain_bytes)
    result.extend(u64be(len(fields)))
    for name, kind, value in fields:
        name_bytes = name.encode("ascii")
        if kind == "ASCII":
            if any(byte < 0x20 or byte > 0x7E for byte in value):
                raise ProtocolError("randomization ASCII field is not printable")
            tag = b"A"
        elif kind == "BYTES":
            tag = b"B"
        else:
            raise ProtocolError("unknown randomization field type")
        result.extend(u64be(len(name_bytes)))
        result.extend(name_bytes)
        result.extend(tag)
        result.extend(u64be(len(value)))
        result.extend(value)
    return bytes(result)


def power_task_manifest_protocol() -> dict[str, Any]:
    return {
        "schema": "xlang-dense-power-task-manifest-v1",
        "identity_fields": [
            "owner_branch_id",
            "true_winner_id",
            "benefit_endpoint_id",
            "benefit_cell_id",
            "target_id",
            "selected_block_count",
            "injected_alternative_sha256",
            "selected_family_sha256",
            "power_engine_protocol_sha256",
        ],
        "common_artifact_binding_fields": [
            "reference_pilot_raw_sha256",
            "reference_assignment_manifest_sha256",
            "timing_whole_cycle_support_sha256",
            "memory_eligibility_ledger_sha256",
            "power_sign_table_key_manifest_sha256",
            "randomization_protocol_sha256",
        ],
        "endpoint_artifact_binding_fields": {
            "END-RAW-TRACE-NS": [
                "required_benefit_f_ids_sha256",
                "timing_reverse_ni_bound_ledger_sha256",
            ],
            "END-PEAK-ACQUIRED-BYTES": [
                "memory_whole_cycle_support_sha256",
                "benefit_partition_sha256",
                "benefit_category_sum_sha256",
            ],
        },
        "power_task_id_rule": (
            "power_task_id is the lowercase SHA-256 hex digest of the canonical "
            "ASCII JSON object containing every identity field, every common "
            "artifact-binding field, and exactly the fields registered for that "
            "task endpoint. Canonical JSON sorts object keys bytewise ASCII, uses "
            "no insignificant whitespace, and excludes only power_task_id and "
            "row_sha256. No applicable binding may be PENDING or omitted, and an "
            "endpoint-inapplicable binding is forbidden."
        ),
        "maximum_identity_domain_rule": (
            "Before the reference pilot, freeze the exact maximum identity domain: "
            "for each active branch, five true winners times each branch timing "
            "cell for the timing endpoint plus each non-ZST memory cell for the "
            "memory endpoint, crossed with n=60,90,120."
        ),
        "active_domain_rule": (
            "After reference memory eligibility freezes, remove memory identities "
            "outside the positive-reference subset M, join every artifact binding, "
            "derive unique power_task_id values, sort them bytewise ASCII, and "
            "commit the complete manifest before candidate construction."
        ),
        "event_parent_rule": (
            "Every failure-event ID starts with its unique power_task_id. Local "
            "event keys are unique only inside that parent task; task scope prevents "
            "collisions across branch, winner, endpoint, cell, target, or n."
        ),
        "event_stream_hash_rule": (
            "Stream canonical ASCII event JSON Lines in power_task_id then local "
            "event-ID order through incremental SHA-256; the final ledger records "
            "the digest, total row count, and exact per-event-class counts. A full "
            "materialized event table is not required."
        ),
    }


def maximum_power_task_identity_rows(
    branch_id: str,
    matrix: list[dict[str, str]],
    power_engine_protocol_sha256: str,
) -> list[dict[str, Any]]:
    family_sha256 = digest_value(benefit_hypothesis_ids(branch_id, matrix))
    cells = [
        row for row in matrix
        if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
        and branch_id in row["owner_branch_ids"].split(",")
    ]
    rows: list[dict[str, Any]] = []
    for winner in CANDIDATES:
        for endpoint_id in (
            "END-RAW-TRACE-NS", "END-PEAK-ACQUIRED-BYTES",
        ):
            alternative = exact_alternative_matrix(winner, endpoint_id)
            for cell in cells:
                if (
                    endpoint_id == "END-PEAK-ACQUIRED-BYTES"
                    and cell["payload_id"] == "P-ZST-AFFINE"
                ):
                    continue
                injected_alternative_sha256 = digest_value({
                    "alternative_matrix": alternative,
                    "benefit_cell_id": cell["cell_id"],
                    "target_id": cell["target_id"],
                })
                for block_count in (60, 90, 120):
                    rows.append({
                        "owner_branch_id": branch_id,
                        "true_winner_id": winner,
                        "benefit_endpoint_id": endpoint_id,
                        "benefit_cell_id": cell["cell_id"],
                        "target_id": cell["target_id"],
                        "selected_block_count": block_count,
                        "injected_alternative_sha256":
                            injected_alternative_sha256,
                        "selected_family_sha256": family_sha256,
                        "power_engine_protocol_sha256":
                            power_engine_protocol_sha256,
                    })
    identity_fields = power_task_manifest_protocol()["identity_fields"]
    return sorted(
        rows,
        key=lambda row: tuple(str(row[field]) for field in identity_fields),
    )


def power_failure_event_ledger_schema() -> list[dict[str, Any]]:
    return [
        {
            "event_class": "WINNER_OUTGOING_NI_CELL_FAILURE",
            "endpoint_tasks": [
                "END-RAW-TRACE-NS", "END-PEAK-ACQUIRED-BYTES",
            ],
            "key_fields": [
                "power_task_id", "winner_id", "right_id", "timing_cell_id",
            ],
            "domain": (
                "right_id is Rust plus the four other candidates; timing_cell_id "
                "is every selected-branch primary timing cell"
            ),
            "exact_count_formula": "5*T",
            "event": "the required cell decision fails",
        },
        {
            "event_class": "WINNER_REQUIRED_BENEFIT_FAILURE",
            "endpoint_tasks": [
                "END-RAW-TRACE-NS", "END-PEAK-ACQUIRED-BYTES",
            ],
            "key_fields": [
                "power_task_id", "winner_id", "loser_id", "benefit_cell_id",
                "endpoint_id",
            ],
            "domain": "loser_id is each of the four other candidates",
            "exact_count_formula": "4",
            "event": "the injected benefit misses the first Holm threshold",
        },
        {
            "event_class": "TIMING_REVERSE_NI_ERRONEOUS_PASS",
            "endpoint_tasks": ["END-RAW-TRACE-NS"],
            "key_fields": ["power_task_id", "loser_id", "winner_id"],
            "domain": "loser_id is each of the four other candidates",
            "exact_count_formula": "4",
            "event": (
                "the reverse directed NI IUT passes; bound by the minimum exact "
                "pass probability over required cells, ASCII cell ID breaking ties"
            ),
        },
        {
            "event_class": "MEMORY_COMPLEMENT_BENEFIT_REJECTION_BOUND",
            "endpoint_tasks": ["END-PEAK-ACQUIRED-BYTES"],
            "key_fields": [
                "power_task_id", "benefit_partition_sha256",
                "category_sum_sha256",
            ],
            "domain": (
                "one cached exact-Q/P/M aggregate over N, the full benefit family "
                "minus the four injected winner-over-loser IDs F"
            ),
            "exact_count_formula": "1",
            "logical_subevent_exact_count_formula": "m-4",
            "event": (
                "If Holm rejects any N member, the first N rank is at most five, so "
                "its score is <=1/(200*(m-4)). Sum exact clustered marginal "
                "probabilities for N at that threshold by hashed cached categories "
                "and clip at one. No nominal FWER probability is imported."
            ),
        },
        {
            "event_class": "PROTOCOL_INVALIDITY",
            "endpoint_tasks": [
                "END-RAW-TRACE-NS", "END-PEAK-ACQUIRED-BYTES",
            ],
            "key_fields": ["power_task_id", "preflight_ledger_sha256"],
            "domain": "one composite deterministic preflight event",
            "exact_count_formula": "1",
            "event": (
                "bound is 0 only after every structural, schema, custody, timeout, "
                "support, engine, and manifest preflight passes; otherwise no power result"
            ),
        },
    ]


def benefit_partition_category_protocol() -> dict[str, Any]:
    return {
        "schema": "xlang-dense-benefit-partition-categories-v1",
        "symbols": {
            "T": "all selected-branch timing cells",
            "L": "all selected-branch non-ZST memory cells",
            "M": "positive-reference memory subset",
            "Z": "L-M structural-zero memory subset",
            "q": "the injected positive-reference memory cell",
        },
        "complement_categories": [
            {
                "category_id": "BASELINE_TIMING",
                "identity_count": "20*T",
                "marginal": "p_bt(cell)",
            },
            {
                "category_id": "OTHER_POSITIVE_MEMORY_BASELINE",
                "identity_count": "20*(M-1)",
                "marginal": "p_bm(cell)",
            },
            {
                "category_id": "INJECTED_CELL_BASELINE",
                "identity_count": "12",
                "marginal": "p_bm(q)",
            },
            {
                "category_id": "INJECTED_CELL_REVERSE",
                "identity_count": "4",
                "marginal": "p_rm(q)",
            },
            {
                "category_id": "STRUCTURAL_ZERO_MEMORY",
                "identity_count": "20*Z",
                "marginal": "0",
            },
        ],
        "complement_identity_count_formula": "20*(T+L)-4=m-4",
        "probability_sum_terms": [
            {"domain": "T", "marginal": "p_bt(cell)", "coefficient": 20},
            {"domain": "M", "marginal": "p_bm(cell)", "coefficient": 20},
            {"domain": "q", "marginal": "p_bm(q)", "coefficient": -8},
            {"domain": "q", "marginal": "p_rm(q)", "coefficient": 4},
        ],
        "probability_sum_formula": (
            "20*sum_T(p_bt)+20*sum_M(p_bm)-8*p_bm(q)+4*p_rm(q)"
        ),
        "cache_scope": (
            "Compute once per branch, winner, memory endpoint, injected q, and "
            "target; cache the exact partition across n=60,90,120, then compute "
            "and cache a separate exact marginal sum for each n after joining "
            "that n's laws."
        ),
    }


def power_law_key_manifest_protocol() -> dict[str, Any]:
    return {
        "schema": "xlang-dense-power-law-key-manifest-v1",
        "key_count_formula": "5*T+3*M",
        "key_identity_fields": [
            "branch_id",
            "cell_id",
            "target_id",
            "endpoint_id",
            "law_key_class_id",
        ],
        "required_binding_fields": [
            "orientation",
            "left_pilot_multiplier",
            "right_pilot_multiplier",
            "relative_injected_ratio",
            "acceptance_ratio",
            "strict_success_cross_product",
            "pilot_cycle_support_sha256",
            "q_polynomial_ledger_sha256",
            "p_polynomial_ledger_sha256",
            "m_polynomial_sha256",
            "law_n60_sha256",
            "law_n90_sha256",
            "law_n120_sha256",
        ],
        "key_classes": [
            {
                "law_key_class_id": "TIMING_NI_BASE",
                "endpoint_id": "END-RAW-TRACE-NS",
                "decision_family": "NONINFERIORITY",
                "orientation": "BASE",
                "left_pilot_multiplier": "1/1",
                "right_pilot_multiplier": "1/1",
                "relative_injected_ratio": "1/1",
                "acceptance_ratio": "51/50",
                "strict_success_cross_product": (
                    "50*scaled_left_elapsed_ns < 51*scaled_right_elapsed_ns"
                ),
                "count_formula": "T",
            },
            {
                "law_key_class_id": "TIMING_NI_FORWARD",
                "endpoint_id": "END-RAW-TRACE-NS",
                "decision_family": "NONINFERIORITY",
                "orientation": "INJECTED_WINNER_OVER_BASELINE",
                "left_pilot_multiplier": "17/20",
                "right_pilot_multiplier": "1/1",
                "relative_injected_ratio": "17/20",
                "acceptance_ratio": "51/50",
                "strict_success_cross_product": (
                    "50*scaled_left_elapsed_ns < 51*scaled_right_elapsed_ns"
                ),
                "count_formula": "T",
            },
            {
                "law_key_class_id": "TIMING_NI_REVERSE",
                "endpoint_id": "END-RAW-TRACE-NS",
                "decision_family": "NONINFERIORITY",
                "orientation": "BASELINE_OVER_INJECTED_WINNER",
                "left_pilot_multiplier": "1/1",
                "right_pilot_multiplier": "17/20",
                "relative_injected_ratio": "20/17",
                "acceptance_ratio": "51/50",
                "strict_success_cross_product": (
                    "50*scaled_left_elapsed_ns < 51*scaled_right_elapsed_ns"
                ),
                "count_formula": "T",
            },
            {
                "law_key_class_id": "TIMING_BENEFIT_FORWARD",
                "endpoint_id": "END-RAW-TRACE-NS",
                "decision_family": "BENEFIT",
                "orientation": "INJECTED_WINNER_OVER_BASELINE",
                "left_pilot_multiplier": "17/20",
                "right_pilot_multiplier": "1/1",
                "relative_injected_ratio": "17/20",
                "acceptance_ratio": "9/10",
                "strict_success_cross_product": (
                    "10*scaled_left_elapsed_ns < 9*scaled_right_elapsed_ns"
                ),
                "count_formula": "T",
            },
            {
                "law_key_class_id": "TIMING_BENEFIT_BASE",
                "endpoint_id": "END-RAW-TRACE-NS",
                "decision_family": "BENEFIT",
                "orientation": "BASE",
                "left_pilot_multiplier": "1/1",
                "right_pilot_multiplier": "1/1",
                "relative_injected_ratio": "1/1",
                "acceptance_ratio": "9/10",
                "strict_success_cross_product": (
                    "10*scaled_left_elapsed_ns < 9*scaled_right_elapsed_ns"
                ),
                "count_formula": "T",
            },
            {
                "law_key_class_id": "MEMORY_BENEFIT_BASE",
                "endpoint_id": "END-PEAK-ACQUIRED-BYTES",
                "decision_family": "BENEFIT",
                "orientation": "BASE",
                "left_pilot_multiplier": "1/1",
                "right_pilot_multiplier": "1/1",
                "relative_injected_ratio": "1/1",
                "acceptance_ratio": "17/20",
                "strict_success_cross_product": (
                    "20*scaled_left_peak_acquired_bytes < "
                    "17*scaled_right_peak_acquired_bytes"
                ),
                "count_formula": "M",
            },
            {
                "law_key_class_id": "MEMORY_BENEFIT_FORWARD",
                "endpoint_id": "END-PEAK-ACQUIRED-BYTES",
                "decision_family": "BENEFIT",
                "orientation": "INJECTED_WINNER_OVER_BASELINE",
                "left_pilot_multiplier": "4/5",
                "right_pilot_multiplier": "1/1",
                "relative_injected_ratio": "4/5",
                "acceptance_ratio": "17/20",
                "strict_success_cross_product": (
                    "20*scaled_left_peak_acquired_bytes < "
                    "17*scaled_right_peak_acquired_bytes"
                ),
                "count_formula": "M",
            },
            {
                "law_key_class_id": "MEMORY_BENEFIT_REVERSE",
                "endpoint_id": "END-PEAK-ACQUIRED-BYTES",
                "decision_family": "BENEFIT",
                "orientation": "BASELINE_OVER_INJECTED_WINNER",
                "left_pilot_multiplier": "1/1",
                "right_pilot_multiplier": "4/5",
                "relative_injected_ratio": "5/4",
                "acceptance_ratio": "17/20",
                "strict_success_cross_product": (
                    "20*scaled_left_peak_acquired_bytes < "
                    "17*scaled_right_peak_acquired_bytes"
                ),
                "count_formula": "M",
            },
        ],
        "sort_and_hash_rule": (
            "Sort complete rows bytewise ASCII by key_identity_fields. Each row "
            "hash is SHA-256 of its canonical ASCII JSON object excluding only "
            "row_sha256; the manifest hash covers the ordered complete rows."
        ),
        "support_and_polynomial_join_rule": (
            "Every key binds one frozen cell, target, endpoint, multiplier "
            "orientation, strict cross-product grammar, pilot-cycle support hash, "
            "Q and P polynomial-ledger hashes, M polynomial hash, and the exact "
            "n=60,90,120 law hashes. Missing, duplicate, or mismatched joins make "
            "the power protocol infeasible."
        ),
        "prepilot_identity_domain_rule": (
            "Before the reference pilot, freeze and hash the exact maximum identity "
            "domain with all selected-branch timing cells and all non-ZST memory "
            "cells. L is the complete non-ZST candidate memory-eligible upper-bound "
            "cell set before the pilot; the domain has 5*T+3*L rows."
        ),
        "active_domain_rule": (
            "After the frozen reference memory-eligibility ledger identifies the "
            "positive subset M, retain all five timing classes and exactly the "
            "three memory classes for M. The resulting 5*T+3*M complete binding "
            "rows and all support/Q/P/M/law hashes are frozen before candidate "
            "construction; structural-zero memory rows are absent."
        ),
    }


def maximum_power_law_key_identity_rows(
    branch_id: str,
    matrix: list[dict[str, str]],
) -> list[dict[str, str]]:
    protocol = power_law_key_manifest_protocol()
    timing_classes = [
        row for row in protocol["key_classes"]
        if row["endpoint_id"] == "END-RAW-TRACE-NS"
    ]
    memory_classes = [
        row for row in protocol["key_classes"]
        if row["endpoint_id"] == "END-PEAK-ACQUIRED-BYTES"
    ]
    rows: list[dict[str, str]] = []
    for cell in matrix:
        if (
            cell["primary_endpoint_id"] != "END-RAW-TRACE-NS"
            or branch_id not in cell["owner_branch_ids"].split(",")
        ):
            continue
        classes = list(timing_classes)
        if cell["payload_id"] != "P-ZST-AFFINE":
            classes.extend(memory_classes)
        for law_class in classes:
            rows.append({
                "branch_id": branch_id,
                "cell_id": cell["cell_id"],
                "target_id": cell["target_id"],
                "endpoint_id": law_class["endpoint_id"],
                "law_key_class_id": law_class["law_key_class_id"],
                "orientation": law_class["orientation"],
                "left_pilot_multiplier": law_class["left_pilot_multiplier"],
                "right_pilot_multiplier": law_class["right_pilot_multiplier"],
                "relative_injected_ratio": law_class[
                    "relative_injected_ratio"
                ],
                "acceptance_ratio": law_class["acceptance_ratio"],
                "strict_success_cross_product": law_class[
                    "strict_success_cross_product"
                ],
            })
    return sorted(
        rows,
        key=lambda row: tuple(
            row[field] for field in protocol["key_identity_fields"]
        ),
    )


def power_resource_limits() -> dict[str, Any]:
    return {
        "maximum_primary_timing_cells": 408,
        "maximum_positive_reference_memory_cells": 406,
        "maximum_sign_table_keys": 3258,
        "sign_table_key_formula": "5*T+3*M",
        "maximum_sign_table_rows": 1954800,
        "sign_table_row_formula": (
            "(5*T+3*M)*30 ordered pairs*4 pilot cycles*5 salts"
        ),
        "maximum_raw_integer_sign_comparisons": 11728800,
        "maximum_bigint_polynomial_operations_per_law": 15983,
        "bigint_polynomial_operation_decomposition": (
            "600 Q coefficient adds + 3640 P coefficient multiply/add operations "
            "+ 124 M coefficient adds + 11346 sequential cached M^2, M^3, M^4 "
            "coefficient multiply/add operations + 273 tail adds = 15983. Build "
            "M^2 once, M^3=M^2*M, and M^4=M^3*M; independent rebuild is forbidden."
        ),
        "maximum_bigint_polynomial_coefficient_operations": 52072614,
        "maximum_qpm_and_n_law_coefficient_ledger_hash_updates": 1850544,
        "maximum_memory_task_partition_identity_inspections": 33048400,
        "maximum_sign_table_row_hashes": 1954800,
        "maximum_power_task_manifest_rows": 12210,
        "maximum_alternatives_per_block_count": 4070,
        "maximum_alternative_block_tasks": 12210,
        "maximum_streamed_failure_event_terms": 25000020,
        "maximum_failure_event_identity_serializations_and_hashes": 25000020,
        "maximum_cached_probability_lookups": 25000020,
        "maximum_rational_union_bound_additions": 25000020,
        "derived_minimum_counted_operations": 175667428,
        "counted_operation_headroom": 24332572,
        "maximum_counted_primitive_operations": 200000000,
        "maximum_resident_bytes": "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "maximum_output_artifact_bytes": "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "maximum_monotonic_wall_time_ns": "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "counted_operation_rule": (
            "Count every raw sign comparison, bigint polynomial coefficient "
            "operation, Q/P/M/n-law coefficient-ledger hash update, F/N identity "
            "inspection fused with its partition-hash update, sign-row hash, task-manifest join "
            "and row hash, failure-event canonicalization and incremental hash "
            "update, cached probability lookup, and rational union-bound addition. "
            "Stop before an operation that would exceed the combinatorial ceiling."
        ),
        "failure_rule": (
            "Any exceeded ceiling, signal, nonzero exit, incomplete task key, or "
            "hash mismatch returns PROTOCOL_INFEASIBLE before construction."
        ),
    }


def execution_timeout_protocol() -> dict[str, Any]:
    return {
        "clock": "CLOCK_MONOTONIC_RAW nanoseconds",
        "per_child_timeout_ns": "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "per_six_treatment_block_timeout_ns": "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "reference_pilot_campaign_timeout_ns": "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "candidate_primary_campaign_timeout_ns": "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        "successful_exit": "normal exit status 0 plus complete schema-valid row",
        "reference_failure": (
            "Timeout, signal, nonzero exit, missing row, or malformed row makes the "
            "selected branch PROTOCOL_INFEASIBLE before candidate construction."
        ),
        "candidate_failure": (
            "Timeout, signal, nonzero exit, missing row, or malformed row is retained "
            "as the fixed slot's unfavorable sign failure and also applies every "
            "registered correctness/structural failure rule."
        ),
        "retry_rule": "No retry, replacement, extension, timeout reset, or extra slot.",
    }


def statistics_payload(
    matrix: list[dict[str, str]],
    branches: list[dict[str, str]],
) -> dict[str, Any]:
    treatment_sequences = [
        [TREATMENTS[index - 1] for index in row]
        for row in WILLIAMS_NUMERIC
    ]
    alpha_control = {
        "global_family_total_alpha": "0.01",
        "noninferiority_family_alpha": "0.005",
        "global_benefit_family_alpha": "0.005",
        "split_identity": "0.01=0.005+0.005",
        "noninferiority_directed_claim_count": 25,
        "noninferiority_claims": (
            "Five candidate-over-Rust claims plus twenty ordered "
            "candidate-over-candidate claims."
        ),
        "noninferiority_method": (
            "Bonferroni over the one 25-claim family at 0.005; each directed "
            "claim is an intersection-union across every required target and "
            "primary END-RAW-TRACE-NS cell using the raw equal-weight full "
            "Williams-by-five-salt scheduled-mixture decision. Memory does not enter NI."
        ),
        "noninferiority_per_directed_claim_alpha": "0.0002",
        "benefit_method": (
            "One global Holm step-down family at alpha 0.005 containing every "
            "registered ordered candidate-pair by exact cell by registered "
            "benefit endpoint hypothesis for the selected owner branch."
        ),
        "forbidden_reset": (
            "No candidate pair, endpoint, target, operation, payload, size, or "
            "workload shape receives a fresh alpha allocation."
        ),
    }
    benefit_testing = benefit_testing_protocol()
    benefit_testing_sha256 = digest_value(benefit_testing)
    noninferiority_testing = noninferiority_testing_protocol()
    noninferiority_testing_sha256 = digest_value(noninferiority_testing)
    randomization = randomization_protocol()
    randomization_sha256 = digest_value(randomization)
    tail_protocol = scheduled_mixture_tail_protocol()
    event_ledger_schema = power_failure_event_ledger_schema()
    benefit_partition_categories = benefit_partition_category_protocol()
    benefit_partition_categories_sha256 = digest_value(
        benefit_partition_categories
    )
    law_key_manifest_protocol = power_law_key_manifest_protocol()
    law_key_manifest_protocol_sha256 = digest_value(law_key_manifest_protocol)
    task_manifest_protocol = power_task_manifest_protocol()
    task_manifest_protocol_sha256 = digest_value(task_manifest_protocol)
    resource_limits = power_resource_limits()
    power_engine_protocol_authority = {
        "schema": "xlang-dense-power-engine-protocol-authority-v1",
        "method": "EXACT_REFERENCE_EMPIRICAL_DP_NO_MONTE_CARLO",
        "scheduled_mixture_tail_protocol_sha256": digest_value(tail_protocol),
        "benefit_testing_protocol_sha256": benefit_testing_sha256,
        "noninferiority_testing_protocol_sha256":
            noninferiority_testing_sha256,
        "randomization_protocol_sha256": randomization_sha256,
        "law_key_manifest_protocol_sha256": law_key_manifest_protocol_sha256,
        "power_task_manifest_protocol_sha256": task_manifest_protocol_sha256,
        "failure_event_ledger_schema_sha256": digest_value(event_ledger_schema),
        "benefit_partition_category_protocol_sha256":
            benefit_partition_categories_sha256,
        "resource_limits_sha256": digest_value(resource_limits),
    }
    power_engine_protocol_sha256 = digest_value(power_engine_protocol_authority)
    branch_plans = []
    for branch in branches:
        if branch["branch_class"] != "ACTIVE_POWER_BRANCH":
            continue
        cells = sorted(
            row["cell_id"] for row in matrix
            if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            and branch["branch_id"] in row["owner_branch_ids"].split(",")
        )
        memory_cells = sorted(
            row["cell_id"] for row in matrix
            if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            and row["payload_id"] != "P-ZST-AFFINE"
            and branch["branch_id"] in row["owner_branch_ids"].split(",")
        )
        benefits = benefit_hypothesis_ids(branch["branch_id"], matrix)
        maximum_law_key_identity_rows = maximum_power_law_key_identity_rows(
            branch["branch_id"], matrix
        )
        maximum_task_identity_rows = maximum_power_task_identity_rows(
            branch["branch_id"], matrix, power_engine_protocol_sha256
        )
        alternatives_per_n = 5 * (len(cells) + len(memory_cells))
        timing_event_count = 5 * len(cells) + 9
        memory_event_count = (
            5 * len(cells) + 6
        )
        event_terms_per_n = (
            5 * len(cells) * timing_event_count
            + 5 * len(memory_cells) * memory_event_count
        )
        branch_plans.append({
            "power_plan_id": branch["power_plan_id"],
            "branch_id": branch["branch_id"],
            "od1_option_id": branch["od1_option_id"],
            "od2_option_id": branch["od2_option_id"],
            "od3_option_id": branch["od3_option_id"],
            "primary_cell_count": len(cells),
            "primary_cell_ids_sha256": digest_value(cells),
            "positive_reference_memory_cell_max_count": len(memory_cells),
            "positive_reference_memory_cell_ids_sha256": digest_value(memory_cells),
            "global_benefit_hypothesis_count": len(benefits),
            "global_benefit_hypothesis_ids_sha256": digest_value(benefits),
            "reference_pilot_complete_block_count": 120,
            "reference_pilot_crossed_cycle_count": 4,
            "reference_pilot_observation_count_per_cell_target": 720,
            "reference_pilot_raw_sha256": PENDING,
            "memory_eligibility_ledger_sha256": PENDING,
            "timing_whole_cycle_support_sha256": PENDING,
            "memory_whole_cycle_support_sha256": PENDING,
            "descriptive_nuisance_summary_sha256": PENDING,
            "power_law_key_manifest_protocol_sha256":
                law_key_manifest_protocol_sha256,
            "maximum_power_law_key_identity_count":
                len(maximum_law_key_identity_rows),
            "maximum_power_law_key_identity_domain_sha256":
                digest_value(maximum_law_key_identity_rows),
            "power_sign_table_key_manifest_sha256": PENDING,
            "power_failure_event_ledger_sha256": PENDING,
            "power_benefit_partition_ledger_sha256": PENDING,
            "benefit_partition_category_protocol_sha256":
                benefit_partition_categories_sha256,
            "power_task_manifest_protocol_sha256": task_manifest_protocol_sha256,
            "power_engine_protocol_sha256": power_engine_protocol_sha256,
            "maximum_power_task_identity_count": len(maximum_task_identity_rows),
            "maximum_power_task_identity_domain_sha256":
                digest_value(maximum_task_identity_rows),
            "power_task_manifest_sha256": PENDING,
            "power_engine_source_sha256": PENDING,
            "power_engine_binary_sha256": PENDING,
            "power_engine_compiler_and_flags_sha256": PENDING,
            "power_engine_resource_result_sha256": PENDING,
            "benefit_testing_protocol_sha256": benefit_testing_sha256,
            "noninferiority_testing_protocol_sha256":
                noninferiority_testing_sha256,
            "randomization_protocol_sha256": randomization_sha256,
            "reference_assignment_manifest_sha256": PENDING,
            "candidate_assignment_manifest_sha256": PENDING,
            "maximum_alternatives_per_block_count": alternatives_per_n,
            "maximum_alternative_block_tasks": 3 * alternatives_per_n,
            "failure_event_terms_per_block_count": event_terms_per_n,
            "maximum_streamed_failure_event_terms": 3 * event_terms_per_n,
            "selected_block_count": PENDING,
            "power_state": "BLOCKED_REFERENCE_ONLY_PILOT_ABSENT",
        })
    stage_protocol = stage_prerequisite_protocol()
    construction_required_by_branch = {
        branch_id: branch_protocol["pipeline"][
            "CANDIDATE_CONSTRUCTION"
        ]["cumulative_blocker_ids"]
        for branch_id, branch_protocol in
        stage_protocol["per_owner_branch"].items()
    }
    return {
        "schema": "xlang-dense-performance-statistics-v5",
        "status": "BLOCKED_REFERENCE_ONLY_PILOT_ABSENT",
        "candidate_construction_authorized": False,
        "stage_prerequisite_protocol": stage_protocol,
        "owner_branch_design": {
            "active_branch_count": 8,
            "active_factorization": "OD-1(2) x OD-2(2) x OD-3(2)",
            "common_conditions": {
                "OD-0": "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE",
                "OD-4": "OD-4-EAGER-AND-SCOPED-CONSUME",
                "OD-5": NO_CROSSOVER,
            },
            "blocked_or_reopen_required": [
                "OD-0-SEPARATE-PREREQUISITE-LOCKS",
                "OD-4-EAGER-ONLY",
                "OD-4-PROMOTE-LAZY",
                "OD-5-ENUMERATED-CROSSOVER",
            ],
            "selection_rule": (
                "The owner selects exactly one branch before any candidate "
                "construction or observation. Branch results are never pooled."
            ),
        },
        "randomization_and_custody": randomization,
        "primary_design": {
            "candidate_count_k": 5,
            "facts_on_treatments": list(TREATMENTS),
            "treatment_count": 6,
            "williams_period_length": 6,
            "numeric_sequences": WILLIAMS_NUMERIC,
            "treatment_sequences": treatment_sequences,
            "carryover_balance_rule": (
                "Every treatment occurs once in every period position and "
                "every ordered distinct predecessor-successor pair occurs once "
                "across the six rows."
            ),
            "layout_salts": [
                {
                    "salt_id": f"SALT-{index + 1}",
                    "retained_inert_bytes": 64 * (index + 1),
                }
                for index in range(5)
            ],
            "salt_rule": (
                "The inert owner is outside the timer and subject counters, "
                "has identical bytes and lifetime in all six treatments, and "
                "cannot change the logical input."
            ),
            "one_block": (
                "One complete six-treatment Williams row for one cell and salt; "
                "each observation is a fresh child."
            ),
            "one_crossed_cycle": (
                "Six Williams rows times five salts equals 30 complete blocks "
                "and 180 fresh-child observations per cell."
            ),
            "allowed_block_counts": [60, 90, 120],
            "allowed_complete_cycles": [2, 3, 4],
            "selection_rule": (
                "Choose the smallest allowed count whose exact conservative "
                "reference-empirical decision-power lower bound is at least 9/10. "
                "If none passes, return PROTOCOL_INFEASIBLE; no "
                "extra blocks are added after candidate construction."
            ),
        },
        "reference_only_pilot": {
            "pseudo_treatments": [
                {
                    "pseudo_id": f"RUST-PSEUDO-{index + 1}",
                    "source_id": "RUST-1.97",
                    "source_sha256": "MUST_EQUAL_ALL_SIX",
                    "compiler_sha256": RUSTC_SHA256,
                    "executable_sha256": "MUST_EQUAL_ALL_SIX",
                    "allocator_adapter_sha256": "MUST_EQUAL_ALL_SIX",
                    "environment_sha256": "MUST_EQUAL_ALL_SIX",
                }
                for index in range(6)
            ],
            "canonical_pseudo_ids": [
                f"RUST-PSEUDO-{index + 1}" for index in range(6)
            ],
            "identity_rule": (
                "All six pseudo-treatments are byte-identical Rust source, "
                "compiler, flags, executable, adapter, input, and environment. "
                "The committed pilot permutation manifest assigns canonical pseudo "
                "IDs to numeric symbols separately by cycle and salt; power indexes "
                "raw supports by those realized symbols."
            ),
            "schedule": (
                "Use the exact Williams rows and five salts above; pseudo labels "
                "occupy the six treatment positions exactly as candidate labels "
                "will later occupy them."
            ),
            "fixed_complete_blocks_per_cell_target": 120,
            "fixed_crossed_cycles_per_cell_target": 4,
            "fixed_observations_per_cell_target": 720,
            "fixed_count_identity": (
                "Four crossed cycles times six Williams rows times five salts is "
                "120 complete blocks; six pseudo treatments per block is 720 "
                "fresh-child observations for every required cell/target."
            ),
            "stopping_rule": (
                "Run exactly four complete crossed cycles. No pilot result, "
                "power estimate, or observed response may adapt, extend, replace, "
                "or rerun a block. Any timeout, signal, nonzero exit, absent row, or "
                "invalid scheduled row makes the owner branch "
                "PROTOCOL_INFEASIBLE before candidate construction."
            ),
            "candidate_count_separation": (
                "The fixed 120-block reference pilot is not the later candidate "
                "block count. Only after this pilot is frozen may power choose "
                "candidate count 60, 90, or 120."
            ),
            "candidate_data_rule": (
                "No candidate source, build, timing, structural, or held-out "
                "result may enter support construction or power calculation."
            ),
            "raw_schema": (
                "One canonical JSON row per pseudo observation with branch, "
                "cell, target, salt, cycle, Williams row, period, predecessor, "
                "pseudo ID, all identity hashes, raw elapsed_ns, exact allocator "
                "counters including peak_acquired_bytes, machine probes, "
                "completion, and row hash."
            ),
            "memory_eligibility_freeze": (
                "Before power fitting, classify each non-ZST cell/target from "
                "all admitted Rust pseudo rows: all positive is "
                "POSITIVE_REFERENCE_MEMORY, all zero is STRUCTURAL_ZERO_MEMORY, "
                "and any zero/positive mixture invalidates that owner branch. "
                "Freeze the complete classification ledger and SHA-256 before "
                "candidate construction."
            ),
        },
        "inferential_estimand": {
            "endpoint_responses": endpoint_response_protocol(),
            "claim_scope": (
                "Each semantic cell is a separate gate. Within that cell the "
                "estimand is the equal-weight mixture over the complete six-row "
                "Williams by five-salt schedule, repeated two, three, or four times."
            ),
            "raw_integer_rule": (
                "Inference uses raw elapsed-nanosecond and allocator-byte integers "
                "only. No log, fitted period effect, predecessor effect, residual, "
                "covariance, or nuisance adjustment enters inference or power."
            ),
            "scheduled_mixture_tail_protocol": tail_protocol,
        },
        "descriptive_nuisance_diagnostics": {
            "scope": (
                "Report raw values grouped by period, predecessor treatment, "
                "Williams row, salt, and cycle with exact counts and hashes."
            ),
            "statistics": (
                "Integer minima, maxima, sums, and exact reduced-rational means; "
                "no fitted coefficient, standard error, or covariance estimate."
            ),
            "selection_use": (
                "DESCRIPTIVE_ONLY: never subtracted, fit, pooled, shrunk, or used "
                "by NI, benefit, power, dominance, or selection."
            ),
        },
        "benefit_testing": benefit_testing,
        "noninferiority_testing": noninferiority_testing,
        "power_calculation": {
            "method": "EXACT_REFERENCE_EMPIRICAL_DP_NO_MONTE_CARLO",
            "benefit_testing_protocol_sha256": benefit_testing_sha256,
            "noninferiority_testing_protocol_sha256":
                noninferiority_testing_sha256,
            "reference_model": (
                "For each endpoint and exact cell retain four complete raw integer "
                "crossed-cycle supports. Each support contains all thirty blocks "
                "across six Williams rows and five salts; no scalar residual or "
                "nuisance adjustment exists."
            ),
            "mapping_integration": (
                "For each cell, target, candidate cycle, and salt, the hidden mapping "
                "is independently derived and fixed across that salt's six Williams "
                "rows. Each marginal averages exactly over 30 ordered numeric-symbol "
                "pairs; the remaining 4! assignments cancel."
            ),
            "whole_cycle_success_table": (
                "For each on-demand key, pilot cycle j, salt s, and ordered pair "
                "(a,b), evaluate the six Williams rows and store u[j,s,a,b] in 0..6. "
                "Hash the exact key domain and every stored count."
            ),
            "injected_integer_ratios": (
                "Timing winner values are multiplied by exact 17/20; memory winner "
                "values by exact 4/5; all other means are 1/1. Compare scaled "
                "integers by unbounded cross-products without rounding or logs."
            ),
            "exact_pass_probability": (
                "For each j,s form Q[j,s](z)=sum(z^u[j,s,a,b],a!=b) over 30 pairs; "
                "form P[j](z)=product(Q[j,s](z),s=1..5), preserving one common pilot "
                "cycle across salts; form M(z)=sum(P[j](z),j=1..4). For r=n/30 in "
                "{2,3,4}, M(z)^r/(4*30^5)^r is the exact success-count law. Sum "
                "coefficients meeting the registered decision critical and reduce."
            ),
            "cluster_dependence_rule": (
                "Within one candidate cycle, a single pilot cycle j is common to all "
                "five salts while mappings are independent by salt and fixed across "
                "six rows. Candidate cycles draw independently from M. The final "
                "union bound assumes no dependence across distinct events."
            ),
            "winner_qualification_bound": (
                "Compute all 25 directed NI claims, but candidate C's qualifying "
                "status uses only C/Rust and C/D for the four other candidates, "
                "plus C's four outgoing benefit relations. For the injected true "
                "winner, sum failure probabilities for every cell in those five "
                "outgoing NI IUT claims and the four injected benefit hypotheses. "
                "For a timing task also sum four reverse loser/winner NI erroneous-"
                "pass bounds, each the minimum required-cell pass probability with "
                "its bound cell recorded. For a memory task, add the one exact "
                "cached Q/P/M union bound that any complement N benefit has score "
                "<=1/(200*(m-4)). Add the single "
                "protocol-invalidity term. Clip one "
                "minus the exhaustive sum to [0,1]."
            ),
            "failure_event_ledger_schema": event_ledger_schema,
            "benefit_partition_category_protocol": benefit_partition_categories,
            "law_key_manifest_protocol": law_key_manifest_protocol,
            "power_task_manifest_protocol": task_manifest_protocol,
            "power_engine_protocol_authority": power_engine_protocol_authority,
            "power_engine_protocol_sha256": power_engine_protocol_sha256,
            "failure_event_uniqueness": (
                "Canonical event_id is SHA-256 of canonical ASCII JSON containing "
                "the unique parent power_task_id, event_class, and all remaining "
                "listed local key fields. Generate every task-scoped exact domain, "
                "sort bytewise ASCII inside task and then by power_task_id, reject "
                "any duplicate or omission, and stream each event exactly once."
            ),
            "benefit_partition_ledger": (
                "Once per memory alternative, join the full family IDs to exactly "
                "four injected "
                "winner-over-loser IDs F and their exact complement N, proving "
                "|F|=4, |N|=m-4, disjointness, union equality, IDs, and hashes. "
                "Let T be timing cells, L all non-ZST family memory cells, M the "
                "positive-reference subset, Z=L-M, and q the injected q in M. N is "
                "20*T baseline timing, 20*(M-1) other-positive baseline memory, "
                "12 baseline IDs at q, four reverse IDs at q, and 20*Z ZERO IDs; "
                "the total is 20*(T+L)-4=m-4. Cache probability sum "
                "20*sum_T(p_bt)+20*sum_M(p_bm)-8*p_bm(q)+4*p_rm(q). ZERO has "
                "score 1 and probability 0. Cache the exact F/N partition and "
                "category sum across n=60,90,120; category IDs, counts, and hashes "
                "must join."
            ),
            "holm_power_rule": (
                "For each of the four injected winner-over-loser benefits, use "
                "local threshold (1/200)/m, where m is the exact selected-branch "
                "family size. If all four pass that most stringent threshold, "
                "the exact Holm algorithm rejects them regardless of other p-value "
                "ordering. If any N member is rejected, the first N has rank <=5 "
                "and therefore score <=1/(200*(m-4)); the clustered Q/P/M marginal "
                "sum bounds that event directly."
            ),
            "selection_event": (
                "Construct the qualifying set S exactly as final analysis does. "
                "The conservative event proves the injected winner's five outgoing "
                "NI claims and four outgoing benefits. Timing tasks exclude each "
                "loser through its reverse NI failure; memory tasks exclude losers "
                "through absence of any complement N benefit rejection. Reverse NI "
                "is never conjoined with the winner's qualification. Success is "
                "|S|=1 with the true winner."
            ),
            "bound_interpretation": (
                "This is a conservative simultaneous lower bound on reference-"
                "empirical selection power. It is not an exact joint probability "
                "and makes no independence claim across cells, targets, endpoints, "
                "candidate pairs, event classes, or mapping marginals. Exact 17/20 "
                "and 4/5 multiplicative shifts are applied to frozen Rust pilot "
                "supports; the result is conditional planning power, not a guarantee "
                "about unknown candidate variance, tails, or real-world power, and "
                "candidate data never updates it."
            ),
            "injected_alternative_matrices": [
                exact_alternative_matrix(candidate, spec["endpoint_id"])
                for spec in BENEFIT_ENDPOINT_ALTERNATIVES
                for candidate in CANDIDATES
            ],
            "true_winner_alternatives": list(CANDIDATES),
            "benefit_endpoint_alternatives": [
                dict(spec) for spec in BENEFIT_ENDPOINT_ALTERNATIVES
            ],
            "benefit_cell_alternative": (
                "For each true winner and endpoint, enumerate every eligible "
                "exact cell/endpoint hypothesis as the sole benefit location. "
                "Timing injects 85/100 under acceptance 90/100; positive-reference "
                "memory injects 80/100 under acceptance 85/100. All other ratios "
                "are exactly 1/1."
            ),
            "allowed_candidate_block_counts": [60, 90, 120],
            "allowed_candidate_complete_cycles": [2, 3, 4],
            "resource_limits": resource_limits,
            "executable_algorithm_bound": (
                "Construct only the on-demand 5*T+3*M sign-table key domain, retain "
                "4*5*30 per-salt/pilot-cycle ordered-pair counts per key, cache exact marginal event "
                "probabilities, and stream the exhaustive event ledgers for at most "
                "12,210 alternative/block tasks. Literal hypothesis-by-alternative "
                "table expansion is forbidden."
            ),
            "exact_arithmetic": (
                "All critical tails, empirical-power probabilities, union sums, "
                "Holm thresholds, clipping, and 9/10 comparison use reduced "
                "unbounded-integer rationals; binary floating point is forbidden."
            ),
            "structural_condition": (
                "Every soundness, common-substrate, B-FIX, B-P2, H-FLATSET, "
                "W-SMALL, W-GAP, layout, ZST, and exact structural gate is "
                "conditioned PASS. A structural failure makes selection false."
            ),
            "worst_case_power": (
                "For each selected branch and candidate block count, take the "
                "minimum exact simultaneous lower bound over five true winners, "
                "both endpoints, every eligible benefit cell, and every target. "
                "Each marginal already integrates the exact hidden mapping and "
                "whole-cycle empirical support."
            ),
            "power_floor": "9/10",
            "selection_rule": (
                "Choose the smallest of 60, 90, and 120 whose worst-case exact "
                "simultaneous lower bound is at least 9/10. If none passes, return "
                "PROTOCOL_INFEASIBLE before candidate construction; no extra "
                "pilot or candidate observations are allowed."
            ),
            "engine_and_resource_gate": (
                "Before the reference pilot, freeze power-engine source, binary, "
                "compiler, flags, exact task-manifest schema, arithmetic tests, "
                "the exact resource_limits object, instrumented operation counters, "
                "and artifact hashes. Any ceiling or manifest failure returns "
                "PROTOCOL_INFEASIBLE; no adjacent engine, extra budget, approximation, "
                "partial result, or retry may replace it."
            ),
            "alpha_control": alpha_control,
        },
        "endpoint_ratio_rules": {
            "ordered_ratio_rule": (
                "Evaluate C/D directly from raw nonnegative integers by the endpoint's "
                "registered strict cross-product; never invert a rounded result."
            ),
            "memory_zero_rule": (
                "reference=0 requires candidate=0 as structural admission; 0/0 "
                "is equality ratio 1 and never benefit; candidate>0/reference=0 "
                "fails; candidate=0/reference>0 is exact benefit; positive/positive "
                "uses exact integer cross-products. ZST allocator bytes must be "
                "0/0 structural equality and never enter benefit."
            ),
        },
        "multiplicity": alpha_control,
        "selection": {
            "structural_admission": (
                "All required soundness, substrate, protected, witness, layout, "
                "allocator, ZST, and exact operation structural gates pass."
            ),
            "rust_floor": (
                "For every selected target and every exact primary operation/"
                "shape/payload/size cell, the directed candidate/Rust scheduled-"
                "mixture NI decision passes at 1/5000. No semantic cell is pooled."
            ),
            "dominance": (
                "C dominates D only if every exact C/D primary cell is "
                "noninferior and at least one registered benefit hypothesis for "
                "C over D is rejected by the one global Holm family and meets "
                "the frozen 0.90 time or 0.85 memory threshold."
            ),
            "unique_survivor": (
                "Select C only if C passes Rust and dominates each other "
                "structurally admitted candidate."
            ),
            "qualifying_candidate_rule": {
                "structural_gate": "all registered structural admissions pass",
                "outgoing_ni_claims": (
                    "exactly five: C/Rust and C/D for each of the four D!=C"
                ),
                "outgoing_benefit_relations": (
                    "exactly four: at least one Holm-rejected registered benefit "
                    "for C/D for each of the four D!=C"
                ),
                "all_25_role": (
                    "Compute all 25 directed NI claims, but never require all 25 "
                    "simultaneously for one candidate's qualification."
                ),
            },
            "no_selection": (
                "Form the qualifying set S from candidates that pass structural "
                "admission, every Rust floor, and the complete dominance rule. "
                "Select only when |S|=1. If |S| != 1, including |S|=0 or any "
                "cardinality 2 through 5, return NO-SELECTION. No score, weight, "
                "aggregate, or crossover changes this cardinality rule."
            ),
            "no_pooling": (
                "Operations, sort shapes, edit indices, retain 10/50/90, "
                "clone_from length relations, splice position/replacement "
                "relations, traversal modes, payloads, sizes, and targets are "
                "independent intersection gates and are never pooled."
            ),
        },
        "descriptive_latency": {
            "endpoints": [
                "END-TRACE-LATENCY-P50", "END-TRACE-LATENCY-P95",
                "END-TRACE-LATENCY-P99", "END-OP-LATENCY-P50",
                "END-OP-LATENCY-P95", "END-OP-LATENCY-P99",
            ],
            "quantile": (
                "Nearest-rank ceil(q*n), clamped to [1,n], on raw valid rows "
                "inside one exact cell and treatment."
            ),
            "selection_use": "DESCRIPTIVE_ONLY",
        },
        "hardware_counters": {
            "endpoint": "END-HARDWARE-COUNTERS",
            "availability_policy": (
                "Use the exact target counter registry. Each unavailable event "
                "is NOT_AVAILABLE with probe hash and reason, never zero or proxy."
            ),
            "selection_use": "DESCRIPTIVE_ONLY",
        },
        "facts_off_diagnostic": {
            "role": (
                "Separate post-primary randomized campaign excluded from Williams "
                "blocks, k, global families, dominance, and selection."
            ),
            "order_rule": (
                "It begins only after primary raw rows, manifests, machine-state "
                "artifacts, and hashes are closed. Derive its post-primary root from "
                "the already committed candidate seed, Candidate Freeze B hash, and "
                "closed primary-manifest hash; freeze a separate derived-root "
                "commitment, manifest, global rank, and raw schema. No third seed or "
                "seed rotation is allowed, and it cannot affect primary runs."
            ),
            "semantic_gate": (
                "Source and caller semantics must be exact; mismatch rejects."
            ),
        },
        "raw_sample_schemas": {
            "format": "Canonical ASCII JSON Lines, append-only, one fresh child per row",
            "common_randomization_fields": [
                "randomization_commitment_sha256", "assignment_manifest_sha256",
                "assignment_row_id", "global_execution_rank",
            ],
            "reference_pilot_required_fields": [
                "schema_version", "campaign_id", "owner_branch_id", "cell_id",
                "target_id", "layout_salt_id", "complete_cycle_index",
                "williams_row_id", "block_id", "period_position",
                "numeric_williams_symbol", "predecessor_numeric_symbol",
                "pseudo_treatment_id", "descriptor_sha256", "trace_sha256",
                "oracle_sha256", "source_sha256", "executable_sha256",
                "toolchain_bundle_sha256", "allocator_adapter_sha256",
                "machine_state_sha256", "power_state", "thermal_state",
                "randomization_commitment_sha256", "assignment_manifest_sha256",
                "assignment_row_id", "global_execution_rank", "monotonic_start_ns",
                "monotonic_end_ns", "elapsed_ns", "process_exit_status",
                "termination_signal", "operation_count", "result_digest",
                "allocator_counters", "hardware_counter_rows", "complete",
                "invalidation_reason", "row_sha256",
            ],
            "candidate_primary_required_fields": [
                "schema_version", "campaign_id", "owner_branch_id",
                "candidate_freeze_b_sha256", "selected_block_count",
                "power_result_sha256", "cell_id", "target_id", "layout_salt_id",
                "complete_cycle_index", "williams_row_id", "block_id",
                "period_position", "numeric_williams_symbol",
                "predecessor_numeric_symbol", "predecessor_treatment_id",
                "treatment_id", "facts_mode", "descriptor_sha256", "trace_sha256",
                "oracle_sha256", "source_sha256", "executable_sha256",
                "toolchain_bundle_sha256", "allocator_adapter_sha256",
                "common_substrate_sha256", "machine_state_sha256", "power_state",
                "thermal_state", "randomization_commitment_sha256",
                "assignment_manifest_sha256", "assignment_row_id",
                "global_execution_rank", "monotonic_start_ns", "monotonic_end_ns",
                "elapsed_ns", "process_exit_status", "termination_signal",
                "operation_count", "result_digest", "allocator_counters",
                "hardware_counter_rows", "structural_report_sha256",
                "fact_report_sha256", "complete", "invalidation_reason", "row_sha256",
            ],
            "facts_off_required_fields": [
                "schema_version", "campaign_id", "owner_branch_id",
                "candidate_freeze_b_sha256", "cell_id", "target_id",
                "treatment_id", "facts_mode", "diagnostic_pair_id",
                "randomization_commitment_sha256", "assignment_manifest_sha256",
                "assignment_row_id", "global_execution_rank", "descriptor_sha256",
                "source_sha256", "executable_sha256", "machine_state_sha256",
                "monotonic_start_ns", "monotonic_end_ns", "elapsed_ns",
                "process_exit_status", "termination_signal", "fact_report_sha256",
                "complete", "invalidation_reason", "row_sha256",
            ],
            "manifest_join_rule": (
                "Every row joins exactly one manifest row by assignment_row_id and "
                "must match commitment, manifest hash, global rank, campaign, branch, "
                "cell, target, cycle, salt, row, period, predecessor, and either the "
                "candidate treatment ID or reference pseudo ID."
            ),
            "unusable_row_sentinel_rule": (
                "The supervising runner writes a schema-valid row for timeout, signal, "
                "or nonzero exit: complete=false; elapsed_ns, operation_count, "
                "result_digest, allocator_counters, and hardware_counter_rows are JSON "
                "null; process_exit_status is integer or null; termination_signal is "
                "integer or null; invalidation_reason is exactly TIMEOUT, SIGNAL, "
                "NONZERO_EXIT, MALFORMED_OUTPUT, or MISSING_CHILD_OUTPUT. If an entire "
                "row is absent, the manifest join creates a MISSING_RAW_ROW audit "
                "sentinel. Every such fixed slot is an inferential failure."
            ),
            "row_identity": "SHA256 canonical row bytes excluding only row_sha256.",
        },
        "execution_timeouts": execution_timeout_protocol(),
        "branch_power_plans": branch_plans,
        "construction_protocol": {
            "status": "UNRESOLVED_OPERATIONAL_BLOCKERS",
            "arbitrary_resource_default_forbidden": True,
            "required_blocker_ids": sorted({
                blocker_id
                for blocker_ids in construction_required_by_branch.values()
                for blocker_id in blocker_ids
            }),
            "required_blocker_ids_by_owner_branch":
                construction_required_by_branch,
            "first_candidate_prompt_gate": (
                "Every applicable cumulative CANDIDATE_CONSTRUCTION blocker, "
                "including a feasible frozen reference pilot and exact common "
                "repository baseline, resolves before the first candidate prompt."
            ),
            "mechanism_failure_rule": (
                "Once exact equal resources and repair rules are owner-frozen, "
                "an arm that cannot build or pass within them is a mechanism "
                "result and fails. No adjacent "
                "mechanism, author, service, extra prompt, time, or tuning budget "
                "may substitute."
            ),
        },
    }


def control_algorithm_rows() -> list[dict[str, str]]:
    rows = []
    for control in CONTROLS:
        if control["control_id"] not in {
            "B-FIX", "B-P2", "H-FLATSET", "W-SMALL", "W-GAP"
        }:
            continue
        control_id = control["control_id"]
        rows.append({
            "algorithm_id": "ALG-CONTROL-" + control_id,
            "operation_gate_id": "CONTROL-" + control_id,
            "member_contract_id": control_id,
            "contract_ids": "NOT_APPLICABLE_CONTROL",
            "exact_trigger_and_prestate": control["source_authority"],
            "exact_commit_and_poststate": control["equality_rule"],
            "behavior_schedule": "Exact protected or witness oracle",
            "allocation_and_capacity_rule": control["layout_oracle"],
            "resource_ceiling": control["structural_oracle"],
            "rust_reference_route_id": (
                "REF-PROTECTED-BASELINE"
                if control_id in {"B-FIX", "B-P2"}
                else "REF-RUST-API-INVENTORY"
            ),
            "member_declaration_sha256": control["source_sha256"],
            "status": "FROZEN_PROTOCOL",
        })
    rows.append({
        "algorithm_id": "ALG-CONTROL-ARITHMETIC",
        "operation_gate_id": "CONTROL-ARITHMETIC",
        "member_contract_id": "DENSE-CAPACITY-ARITHMETIC",
        "contract_ids": "NOT_APPLICABLE_CONTROL",
        "exact_trigger_and_prestate": (
            "Exact target pointer width and last-valid/first-invalid tuples"
        ),
        "exact_commit_and_poststate": (
            "No commit, allocation, or payload access; exact checked result"
        ),
        "behavior_schedule": "No behavior",
        "allocation_and_capacity_rule": "No allocation",
        "resource_ceiling": "Constant checked arithmetic only",
        "rust_reference_route_id": "REF-RUST-RAWVEC",
        "member_declaration_sha256": RAWVEC_SHA256,
        "status": "FROZEN_PROTOCOL",
    })
    return rows


def table_rows(
    dispositions: list[dict[str, str]],
    gates: list[dict[str, str]],
    branches: list[dict[str, str]],
    algorithms: list[dict[str, str]],
    matrix: list[dict[str, str]],
) -> dict[str, list[dict[str, str]]]:
    return {
        "dispositions": dispositions,
        "operation_gates": gates,
        "owner_branches": branches,
        "common_substrate": common_substrate_rows(),
        "algorithms": algorithms + control_algorithm_rows(),
        "references": REFERENCE_ROUTES,
        "payloads": payload_rows(),
        "targets": TARGETS,
        "layouts": LAYOUTS,
        "allocators": ALLOCATORS,
        "growth": GROWTH,
        "endpoints": ENDPOINTS,
        "counter_policies": COUNTER_POLICIES,
        "structural": structural_rows(),
        "controls": CONTROLS,
        "failures": FAILURES,
        "warmups": WARMUPS,
        "repetitions": REPETITIONS,
        "facts": FACTS,
        "generators": GENERATORS,
        "schedules": schedule_rows(),
        "distributions": distribution_rows(),
        "blockers": blocker_rows(),
        "matrix": matrix,
    }


def artifact_record(path: Path) -> dict[str, Any]:
    data = path.read_bytes()
    return {
        "path": path.name,
        "bytes": len(data),
        "sha256": digest_bytes(data),
    }


def render_protocol_report(
    dispositions: list[dict[str, str]],
    gates: list[dict[str, str]],
    branches: list[dict[str, str]],
    matrix: list[dict[str, str]],
    pre_report_records: list[dict[str, Any]],
) -> str:
    disposition_counts = Counter(
        row["disposition"] for row in dispositions
    )
    role_counts = Counter(row["cell_role"] for row in matrix)
    active_branches = [
        row for row in branches
        if row["branch_class"] == "ACTIVE_POWER_BRANCH"
    ]
    blocked_branches = [
        row for row in branches
        if row["branch_class"] == "BLOCKED_OR_REOPEN_REQUIRED"
    ]
    timed = [
        row for row in matrix
        if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
    ]
    payload_witnesses = Counter(
        row["payload_code"] for row in matrix
        if row["cell_role"] == "PAYLOAD_SEPARATOR_PRIMARY"
    )
    stage_protocol = stage_prerequisite_protocol()
    pilot_direct_counts = sorted({
        len(branch["pipeline"]["REFERENCE_PILOT"]["direct_blocker_ids"])
        for branch in stage_protocol["per_owner_branch"].values()
    })
    construction_cumulative_counts = sorted({
        len(branch["pipeline"]["CANDIDATE_CONSTRUCTION"][
            "cumulative_blocker_ids"
        ])
        for branch in stage_protocol["per_owner_branch"].values()
    })
    lines = [
        "# Dense Family Lock A performance protocol",
        "",
        "Status: research-only frozen protocol; candidate construction, "
        "Candidate Freeze B, pilot execution, scoring, held-out access, and "
        "production work are not authorized.",
        "",
        "## Frozen input boundary",
        "",
        f"- Exact contract registry: {len(dispositions)} rows, SHA-256 "
        f"{FROZEN_EXACT_CONTRACT_SHA256}.",
        f"- Contract generator SHA-256: {FROZEN_CONTRACT_SOURCE_SHA256}.",
        f"- Candidate operations SHA-256: {FROZEN_CANDIDATE_OPERATIONS_SHA256}.",
        f"- Candidate bindings SHA-256: {FROZEN_CANDIDATE_BINDINGS_SHA256}.",
        f"- Candidate lifecycle SHA-256: {FROZEN_CANDIDATE_LIFECYCLE_SHA256}.",
        f"- Common substrate SHA-256: {FROZEN_COMMON_SUBSTRATE_SHA256}.",
        f"- OD4 policy SHA-256: {FROZEN_OD4_SHA256}.",
        "",
        "Any input-byte change blocks regeneration until this protocol is "
        "reviewed and explicitly repinned.",
        "",
        "## Exact coverage",
        "",
        f"- Exact dispositions: {dict(sorted(disposition_counts.items()))}.",
        f"- Standalone operation gates: {len(gates)} including the scoped-consume "
        "protocol gate; every executable exact member has an independent "
        "same-shape Rust-floor gate.",
        f"- Matrix cells: {len(matrix)} total, {len(timed)} primary timed; "
        f"roles {dict(sorted(role_counts.items()))}.",
        f"- Timed payload separators: {dict(sorted(payload_witnesses.items()))}.",
        "- Sort shapes, edit indices, retain 10/50/90, clone_from length "
        "relations, splice range/replacement relations, traversal modes, byte "
        "boundaries, payloads, targets, and exact operations are never pooled.",
        "- B-FIX, B-P2, H-FLATSET, W-SMALL, and W-GAP each have AArch64, "
        "x86-64, and i686 structural cells.",
        "- ZST operation latency is timed where registered; all allocator call "
        "and byte fields are exact structural zero and can never supply benefit.",
        "",
        "## Owner branches and common substrate",
        "",
        f"- Active power branches: {len(active_branches)} = OD1(2) x OD2(2) x "
        "OD3(2), all under OD0 common substrate, OD4 eager plus scoped consume, "
        "and OD5 no crossover.",
        f"- Blocked or reopen-required alternatives: {len(blocked_branches)} "
        "(OD0 separate, OD4 eager-only, OD4 persistent lazy, OD5 crossover).",
        "- All five arms bind the same sealing, generic-call, reborrow, "
        "result-provenance, checked allocator, affine interval, owning cursor, "
        "and cost hashes. A private allocator or cursor rejects the arm.",
        "",
        "## Statistics and endpoints",
        "",
        "- One global alpha family totals 0.01: 0.005 for the 25 directed "
        "noninferiority claims and 0.005 for one global Holm benefit family. "
        "No candidate pair or cell receives a fresh alpha allocation.",
        "- Every selected target and exact primary cell must independently have "
        "the directed candidate/Rust raw scheduled-mixture NI decision pass at "
        "1/5000; exact integer cross-products are used and no aggregate masks an "
        "operation.",
        "- Power uses six byte-identical Rust pseudo-treatments in the exact "
        "Williams and five-salt schedule, retains four raw whole-cycle supports, "
        "integrates the exact hidden treatment mapping, injects exact 17/20 timing "
        "or 4/5 memory alternatives, and computes a conservative union-bound "
        "selection-power lower bound by exact finite DP.",
        "- NI and benefit use the globally valid scheduled-mixture tail p-value: "
        "p=1 when 2*s<=n and the exact Binomial(n,1/2) upper tail otherwise. "
        "Benefit then uses one global exact-rational Holm step-down family. "
        "Threshold ties and unusable fixed slots count as failures.",
        "- Period, predecessor, row, salt, and cycle summaries are descriptive "
        "only; no fitted adjustment, logarithm, residual, or covariance enters "
        "inference, power, dominance, or selection.",
        "- Trace and per-operation p50/p95/p99 plus target hardware counters are "
        "descriptive. Missing counters are explicit NOT_AVAILABLE evidence, "
        "never zero or imputed.",
        "",
        "## Construction boundary",
        "",
        f"- Explicit operational blockers: {len(BLOCKERS)}.",
        "- `earliest_blocked_stage` is cumulative across REFERENCE_PILOT, "
        "CANDIDATE_CONSTRUCTION, and CANDIDATE_FREEZE_B; descriptive counter "
        "reporting is a nonselection side stage.",
        f"- Per-branch reference-pilot prerequisite counts: {pilot_direct_counts}; "
        f"cumulative construction-gate counts: {construction_cumulative_counts}. "
        "The extra prerequisite is the x86 runner in dual-native branches.",
        "- The reference pilot must close as feasible before the first candidate "
        "prompt; its row names the complete per-branch prerequisite manifest, "
        "not a two-blocker shortcut.",
        "- Author, service, disclosure, custody, equal resources, and repair "
        "rules are not yet defensibly frozen; no arbitrary default is supplied.",
        "- After a later exact freeze, inability to build or pass within the "
        "frozen equal protocol is a mechanism result. It does not permit another "
        "author, service, mechanism, resource extension, or tuning round.",
        "",
        "## Generated artifact hashes",
        "",
    ]
    for record in sorted(pre_report_records, key=lambda row: row["path"]):
        lines.append(
            f"- {record['path']}: {record['bytes']} bytes, SHA-256 "
            f"{record['sha256']}."
        )
    lines.extend([
        "",
        "The registry summary adds the report hash and source-authority hashes. "
        "Independent hostile review remains required before any authorization.",
        "",
    ])
    return "\n".join(lines)


def source_authority_records() -> list[dict[str, Any]]:
    paths = [
        EXACT_CONTRACT_REGISTRY,
        CONTRACT_SOURCE,
        HERE / "DENSE-CANDIDATE-OPERATION-REGISTRY.tsv",
        HERE / "DENSE-CANDIDATE-CONTRACT-BINDINGS.tsv",
        HERE / "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv",
        HERE / "DENSE-COMMON-SUBSTRATE-REGISTRY.tsv",
        HERE / "DENSE-OD4-POLICY-REGISTRY.tsv",
        HERE / "dense_performance_registry.py",
        HERE / "verify_dense_performance.py",
        HERE / "generate_dense_performance_inputs.py",
    ]
    return [artifact_record(path) for path in paths]


def main() -> None:
    contract_rows = assert_frozen_contract_inputs()
    authority, dispositions = derive_dispositions(contract_rows)
    branches = owner_branch_base_rows()
    gates, algorithms = build_operation_gates_and_algorithms(
        contract_rows, dispositions
    )
    matrix, descriptors = build_matrix(
        contract_rows, dispositions, gates, branches
    )
    finalize_owner_branches(branches, matrix)
    inputs = generated_input_rows(descriptors)
    statistics = statistics_payload(matrix, branches)
    tables = table_rows(
        dispositions, gates, branches, algorithms, matrix
    )

    for key, rows in tables.items():
        write_tsv(HERE / OUTPUTS[key], SCHEMAS[key], rows)
    write_jsonl(HERE / OUTPUTS["descriptors"], descriptors)
    write_jsonl(HERE / OUTPUTS["inputs"], inputs)
    with (HERE / OUTPUTS["statistics"]).open(
        "w", encoding="utf-8", newline=""
    ) as handle:
        handle.write(json.dumps(statistics, indent=2, sort_keys=True) + "\n")

    pre_report_names = [
        OUTPUTS[key] for key in tables
    ] + [
        OUTPUTS["descriptors"],
        OUTPUTS["inputs"],
        OUTPUTS["statistics"],
    ]
    pre_report_records = [
        artifact_record(HERE / name)
        for name in sorted(pre_report_names)
    ]
    report = render_protocol_report(
        dispositions, gates, branches, matrix, pre_report_records
    )
    (HERE / OUTPUTS["protocol"]).write_text(
        report, encoding="ascii"
    )
    artifact_records = pre_report_records + [
        artifact_record(HERE / OUTPUTS["protocol"])
    ]
    summary = {
        "schema": "xlang-dense-performance-registry-summary-v5",
        "status": "RESEARCH_PROTOCOL_FROZEN_CONSTRUCTION_BLOCKED",
        "candidate_construction_authorized": False,
        "frozen_inputs": {
            "dense_contract_registry.py": FROZEN_CONTRACT_SOURCE_SHA256,
            "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv":
                FROZEN_EXACT_CONTRACT_SHA256,
            "DENSE-CANDIDATE-OPERATION-REGISTRY.tsv":
                FROZEN_CANDIDATE_OPERATIONS_SHA256,
            "DENSE-CANDIDATE-CONTRACT-BINDINGS.tsv":
                FROZEN_CANDIDATE_BINDINGS_SHA256,
            "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv":
                FROZEN_CANDIDATE_LIFECYCLE_SHA256,
            "DENSE-COMMON-SUBSTRATE-REGISTRY.tsv":
                FROZEN_COMMON_SUBSTRATE_SHA256,
            "DENSE-OD4-POLICY-REGISTRY.tsv": FROZEN_OD4_SHA256,
        },
        "exact_contract_count": len(dispositions),
        "exact_member_count": len({
            row["member_contract_id"] for row in dispositions
        }),
        "exact_cluster_member_unit_count": len({
            (row["cluster_id"], row["member_contract_id"])
            for row in dispositions
        }),
        "disposition_counts": dict(sorted(Counter(
            row["disposition"] for row in dispositions
        ).items())),
        "derivation_kind_counts": dict(sorted(Counter(
            row["derivation_kind"] for row in dispositions
        ).items())),
        "operation_gate_count": len(gates),
        "active_owner_branch_count": sum(
            row["branch_class"] == "ACTIVE_POWER_BRANCH"
            for row in branches
        ),
        "blocked_or_reopen_branch_count": sum(
            row["branch_class"] == "BLOCKED_OR_REOPEN_REQUIRED"
            for row in branches
        ),
        "matrix_cell_count": len(matrix),
        "timed_primary_cell_count": sum(
            row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            for row in matrix
        ),
        "cell_role_counts": dict(sorted(Counter(
            row["cell_role"] for row in matrix
        ).items())),
        "target_cell_counts": dict(sorted(Counter(
            row["target_id"] for row in matrix
        ).items())),
        "payload_separator_counts": dict(sorted(Counter(
            row["payload_code"] for row in matrix
            if row["cell_role"] == "PAYLOAD_SEPARATOR_PRIMARY"
        ).items())),
        "descriptive_latency_endpoint_count": 6,
        "explicit_blocker_count": len(BLOCKERS),
        "artifact_count_excluding_summary": len(artifact_records),
        "artifacts": sorted(artifact_records, key=lambda row: row["path"]),
        "source_authorities": source_authority_records(),
        "schemas": SCHEMAS,
        "supersession_scope": (
            "These v5 protocol/statistics artifacts replace the v4 staging map, "
            "the rejected Cartesian matrix, and "
            "the incomplete v2 sparse protocol. No earlier DENSE-PERFORMANCE "
            "output is live authority."
        ),
    }
    with (HERE / OUTPUTS["summary"]).open(
        "w", encoding="utf-8", newline=""
    ) as handle:
        handle.write(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(
        "Dense performance protocol v5: "
        f"{len(dispositions)} exact derivations, {len(gates)} operation gates, "
        f"{len(matrix)} cells, {len(descriptors)} descriptors, "
        f"{len(BLOCKERS)} explicit blockers"
    )


if __name__ == "__main__":
    main()
