#!/usr/bin/env python3
"""Build the fail-closed G0 coverage-cluster evidence-universe registry."""

from __future__ import annotations

import argparse
import collections
import csv
import hashlib
import io
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CLUSTER_REGISTRY = ROOT / "G0-COVERAGE-CLUSTER-REGISTRY.tsv"
CENSUS = ROOT / "RUST-DATA-CONTRACT-CENSUS.tsv"
SAFE_SURFACE = ROOT / "RUST-DATA-SURFACE-MAP.tsv"
D10_SURFACE = ROOT / "RUST-D10-SURFACE-MAP.tsv"
UNSAFE_EVIDENCE = ROOT / "RUST-DATA-UNSAFE-EVIDENCE-MAP.tsv"
TRAIT_IMPL = ROOT / "RUST-1.97.0-TRAIT-IMPL-CROSSWALK.tsv"
TRAIT_IMPL_TOPOLOGY = ROOT / "G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv"
VOCABULARY = ROOT / "G0-FAMILY-GATE-VOCABULARY.md"
OUTPUT = ROOT / "G0-COVERAGE-EVIDENCE-UNIVERSE.tsv"

CLUSTER_REGISTRY_FIELDS = [
    "cluster_ordinal",
    "cluster_id",
    "family",
    "semantic_class",
    "importability",
    "refinement_policy",
    "evidence_universe_policy",
    "allowed_evidence_dispositions",
    "prohibited_direct_uses",
    "census_row_sha256",
    "derivation_row_sha256",
    "policy_version",
]
CENSUS_FIELDS = [
    "contract_id",
    "family",
    "rust_surfaces",
    "pre_state",
    "input_ownership",
    "post_state_result",
    "invalidation",
    "failure_drop_abandonment",
    "complexity",
    "layout_identity_order",
    "behavior_parameter",
    "implementation_privilege_evidence",
    "xlang_current_status",
    "required_obligations",
    "source_refs",
]
SAFE_SURFACE_FIELDS = [
    "canonical_key",
    "item_path",
    "member_name",
    "source_path",
    "primary_contract_id",
    "markers",
]
D10_SURFACE_FIELDS = [
    "canonical_key",
    "representative_path",
    "member_name",
    "route_kind",
    "route_id",
    "route_reason",
]
UNSAFE_EVIDENCE_FIELDS = [
    "canonical_key",
    "representative_surface_crate",
    "representative_module_path",
    "representative_item_path",
    "member_kind",
    "member_name",
    "source_path",
    "representative_docs_path",
    "seed_surface_paths",
    "selected_seed_rendering_count",
    "canonical_rendering_count",
    "stability",
    "caller_safety",
    "evidence_cluster_id",
    "evidence_disposition",
    "markers",
]
TRAIT_IMPL_FIELDS = [
    "impl_key",
    "selection_family",
    "owning_contract_ids",
    "trait_path",
    "trait_application",
    "implementer",
    "impl_signature",
    "associated_bindings",
    "required_method_shapes",
    "stability",
    "stable_since",
    "stable_surface_reachable",
    "stable_surface_note",
    "ownership_shape",
    "rustdoc_identity",
    "rustdoc_aliases",
    "source_identity",
    "source_snippet_sha256",
]

FIELDS = [
    "relation_ordinal",
    "evidence_identity",
    "cluster_id",
    "evidence_kind",
    "evidence_key",
    "source_artifact",
    "source_selector",
    "selected_source_value",
    "selected_source_value_sha256",
    "source_row_sha256",
    "evidence_granularity",
    "materialization_policy",
    "applicability",
    "applicability_authority",
    "applicability_primary_refinement_family_or_gate",
    "applicability_required_predecessor_family_ids",
    "applicability_required_predecessor_gate_stage_ids",
    "applicability_additional_operation_gate_stage_ids",
    "applicability_additional_operation_gate_child_specific_immediate_predecessor_family_or_gate_ids",
    "applicability_implicated_or_reopening_family_ids",
    "applicability_implicated_or_reopening_gate_stage_ids",
    "applicability_authority_row_sha256",
    "terminal_disposition_policy",
    "allowed_terminal_dispositions",
    "cluster_relation_count",
    "cluster_relation_sha256",
    "kind_relation_count",
    "kind_relation_sha256",
    "policy_version",
]

KIND_ORDER = {
    "STABLE_SAFE_SURFACE": 1,
    "D10_CONTRACT_ROUTE": 2,
    "D10_REDUNDANT_SURFACE_ROUTE": 3,
    "STABLE_UNSAFE_EVIDENCE": 4,
    "CONCRETE_TRAIT_IMPL": 5,
    "CLUSTER_RUST_SURFACES_SELECTOR": 6,
    "CLUSTER_IMPLEMENTATION_PRIVILEGE_SELECTOR": 7,
    "CLUSTER_SOURCE_REFS_SELECTOR": 8,
}
EXPECTED_KIND_COUNTS = {
    "STABLE_SAFE_SURFACE": 545,
    "D10_CONTRACT_ROUTE": 138,
    "D10_REDUNDANT_SURFACE_ROUTE": 37,
    "STABLE_UNSAFE_EVIDENCE": 35,
    "CONCRETE_TRAIT_IMPL": 378,
    "CLUSTER_RUST_SURFACES_SELECTOR": 276,
    "CLUSTER_IMPLEMENTATION_PRIVILEGE_SELECTOR": 276,
    "CLUSTER_SOURCE_REFS_SELECTOR": 276,
}
SELECTOR_KINDS = {
    "rust_surfaces": "CLUSTER_RUST_SURFACES_SELECTOR",
    "implementation_privilege_evidence": (
        "CLUSTER_IMPLEMENTATION_PRIVILEGE_SELECTOR"
    ),
    "source_refs": "CLUSTER_SOURCE_REFS_SELECTOR",
}

DIRECT_APPLICABILITY = (
    "INDEPENDENT_EXACT_CHILD_TO_FAMILY_OR_GATE_AUDIT_REQUIRED_BEFORE_DISPOSITION"
)
IMPL_APPLICABILITY = (
    "EXACT_IMPL_KEY_TOPOLOGY_PLUS_CLOSED_OWNING_CLUSTER_OPERATION_GATE_ROUTE_JOIN"
)
SELECTOR_APPLICABILITY = (
    "INDEPENDENT_EXHAUSTIVE_EXPANSION_THEN_EXACT_CHILD_ROUTE_REQUIRED"
)
DIRECT_APPLICABILITY_AUTHORITY = (
    "FAMILY_LOCK_INDEPENDENT_CHILD_APPLICABILITY_LEDGER"
)
SELECTOR_APPLICABILITY_AUTHORITY = (
    "FAMILY_LOCK_INDEPENDENT_SELECTOR_EXPANSION_AND_CHILD_APPLICABILITY_LEDGER"
)
IMPL_APPLICABILITY_AUTHORITY = (
    "G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv:impl_key+"
    "G0-FAMILY-GATE-VOCABULARY.md:"
    "G0_TRAIT_OPERATION_GATE_ASSIGNMENT_AUTHORITY:owning_cluster_id"
)
UNRESOLVED_APPLICABILITY_TARGET = (
    "UNRESOLVED_REQUIRES_INDEPENDENT_CHILD_APPLICABILITY_AUDIT"
)
TERMINAL_POLICY = (
    "EXACTLY_ONE_TERMINAL_PER_APPLICABLE_EVIDENCE_IDENTITY_AND_TARGET;"
    "EXCLUSION_FOR_ONE_TARGET_NEVER_ERASES_ANOTHER_TARGET"
)
ALLOWED_TERMINAL_DISPOSITIONS = (
    "REFINED_IN_LOCK;PREDECESSOR_PROVED;EXCLUDED_BLOCKS_CLAIM"
)
EXACT_KEY_GRANULARITY = "EXACT_SOURCE_KEY"
SELECTOR_GRANULARITY = (
    "EXACT_SOURCE_FIELD_SELECTOR_REQUIRES_FAMILY_LOCK_MEMBER_MATERIALIZATION"
)
EXACT_MATERIALIZATION_POLICY = "NONE_EXACT_SOURCE_KEY"
SELECTOR_MATERIALIZATION_POLICY = (
    "REFINED_REQUIRES_INDEPENDENT_EXHAUSTIVE_EXPANSION_OF_FROZEN_PARENT_BY_"
    "FROZEN_PARSER_OR_HOSTILE_REVIEWED_TOKEN_ANCHOR_LEDGER_AND_EXACT_CHILD_"
    "COUNT_SHA256_AND_EACH_CHILD_TERMINAL;PREDECESSOR_REQUIRES_THE_SAME_"
    "EXHAUSTIVE_CHILD_EQUALITY_AND_EXACT_PREDECESSOR_TERMINALS;EXCLUSION_"
    "BLOCKS_ALL_UNRESOLVED_CHILD_CLAIMS;SELF_SELECTED_CHILD_SET_OR_PARENT_"
    "ONLY_REFINEMENT_INVALID"
)
POLICY_VERSION = "xlang-g0-coverage-evidence-universe-v3"

OPERATION_GATE_ASSIGNMENT_AUTHORITY = {
    "TRAIT-EXTEND-01": (
        "GATE-BULK-CONSTRUCTION-CROSS-FAMILY",
        "EXACT_TOPOLOGY_PRIMARY",
        "22",
        "Extend semantics remain an independently applicable bulk-construction target after the exact implementer topology primary.",
    ),
    "TRAIT-COLLECT-01": (
        "GATE-BULK-CONSTRUCTION-CROSS-FAMILY",
        "EXACT_TOPOLOGY_PRIMARY",
        "21",
        "Collect semantics remain an independently applicable bulk-construction target after the exact implementer topology primary.",
    ),
    "TRAIT-INDEX-01": (
        "GATE-INDEX-CROSS-FAMILY",
        "EXACT_TOPOLOGY_PRIMARY",
        "14",
        "Index semantics remain an independently applicable cross-family target after the exact implementer topology primary.",
    ),
    "TRAIT-CONVERT-01": (
        "GATE-CONVERSION-CROSS-FAMILY",
        "EXACT_TOPOLOGY_PRIMARY",
        "40",
        "Conversion semantics remain an independently applicable cross-family target after the exact implementer topology primary.",
    ),
}


def fail(message: str) -> None:
    raise SystemExit(f"G0 coverage evidence-universe build failed: {message}")


def read_tsv(path: Path, expected_fields: list[str]) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    if fields != expected_fields:
        fail(f"{path.name} schema changed: {fields}")
    if any(None in row for row in rows):
        fail(f"{path.name} has extra TSV columns")
    if any(
        any("\r" in value or "\n" in value for value in row.values())
        for row in rows
    ):
        fail(f"{path.name} has an embedded newline")
    return rows


def read_tsv_with_fields(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    if not fields or any(None in row for row in rows):
        fail(f"{path.name} is malformed")
    if any(
        any("\r" in value or "\n" in value for value in row.values())
        for row in rows
    ):
        fail(f"{path.name} has an embedded newline")
    return fields, rows


def markdown_authority_rows(
    text: str, begin_marker: str, end_marker: str, column_count: int
) -> list[list[str]]:
    if text.count(begin_marker) != 1 or text.count(end_marker) != 1:
        fail(f"authority markers are missing or duplicated: {begin_marker}")
    body = text.split(begin_marker, 1)[1].split(end_marker, 1)[0]
    table_lines = [line for line in body.splitlines() if line.startswith("|")]
    if len(table_lines) < 3:
        fail(f"authority table is empty: {begin_marker}")
    rows: list[list[str]] = []
    for line in table_lines[2:]:
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != column_count:
            fail(f"authority table column count changed: {begin_marker}")
        rows.append(
            [
                cell[1:-1] if cell.startswith("`") and cell.endswith("`") else cell
                for cell in cells
            ]
        )
    return rows


def parse_operation_gate_assignment_authority() -> tuple[
    dict[str, tuple[str, str, str, str]], dict[str, str]
]:
    if not VOCABULARY.is_file():
        fail(f"missing {VOCABULARY.name}")
    rows = markdown_authority_rows(
        VOCABULARY.read_text(encoding="utf-8"),
        "<!-- G0_TRAIT_OPERATION_GATE_ASSIGNMENT_AUTHORITY_BEGIN -->",
        "<!-- G0_TRAIT_OPERATION_GATE_ASSIGNMENT_AUTHORITY_END -->",
        5,
    )
    authority: dict[str, tuple[str, str, str, str]] = {}
    row_hashes: dict[str, str] = {}
    for cells in rows:
        cluster_id = cells[0]
        if cluster_id in authority:
            fail(f"duplicate trait operation-gate authority row: {cluster_id}")
        authority[cluster_id] = (cells[1], cells[2], cells[3], cells[4])
        row_hashes[cluster_id] = value_sha256("\t".join(cells) + "\n")
    if authority != OPERATION_GATE_ASSIGNMENT_AUTHORITY:
        fail("trait operation-gate authority differs from the closed code table")
    if sum(int(row[2]) for row in authority.values()) != 97:
        fail("trait operation-gate authority no longer accounts for 97 relations")
    return authority, row_hashes


def row_sha256(fields: list[str], row: dict[str, str]) -> str:
    encoded = ("\t".join(row[field] for field in fields) + "\n").encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def value_sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def closed_negative_operation_gate_assignment_sha256(cluster_id: str) -> str:
    return value_sha256(
        "CLOSED_NEGATIVE_OPERATION_GATE_ASSIGNMENT\n"
        f"owning_cluster_id={cluster_id}\n"
        "additional_operation_gate_stage_ids=NONE\n"
        "child_specific_immediate_predecessor_family_or_gate_ids=NONE\n"
    )


def composite_impl_applicability_authority_sha256(
    *,
    cluster_id: str,
    impl_key: str,
    topology_row_sha256: str,
    assignment_row_sha256: str,
    additional_operation_gate_stage_ids: str,
    child_specific_immediate_predecessor_family_or_gate_ids: str,
) -> str:
    return value_sha256(
        f"cluster_id={cluster_id}\n"
        f"impl_key={impl_key}\n"
        f"topology_row_sha256={topology_row_sha256}\n"
        f"operation_gate_assignment_row_sha256={assignment_row_sha256}\n"
        f"additional_operation_gate_stage_ids={additional_operation_gate_stage_ids}\n"
        "child_specific_immediate_predecessor_family_or_gate_ids="
        f"{child_specific_immediate_predecessor_family_or_gate_ids}\n"
    )


def evidence_identity(row: dict[str, str]) -> str:
    identity_fields = [
        POLICY_VERSION,
        row["cluster_id"],
        row["evidence_kind"],
        row["evidence_key"],
        row["source_artifact"],
        row["source_selector"],
        row["selected_source_value_sha256"],
        row["source_row_sha256"],
        row["evidence_granularity"],
        row["materialization_policy"],
        row["applicability"],
        row["applicability_authority"],
        row["applicability_primary_refinement_family_or_gate"],
        row["applicability_required_predecessor_family_ids"],
        row["applicability_required_predecessor_gate_stage_ids"],
        row["applicability_additional_operation_gate_stage_ids"],
        row[
            "applicability_additional_operation_gate_child_specific_immediate_"
            "predecessor_family_or_gate_ids"
        ],
        row["applicability_implicated_or_reopening_family_ids"],
        row["applicability_implicated_or_reopening_gate_stage_ids"],
        row["applicability_authority_row_sha256"],
        TERMINAL_POLICY,
        ALLOWED_TERMINAL_DISPOSITIONS,
    ]
    return hashlib.sha256(("\n".join(identity_fields) + "\n").encode("utf-8")).hexdigest()


def add_relation(
    rows: list[dict[str, str]],
    *,
    cluster_id: str,
    evidence_kind: str,
    evidence_key: str,
    source_artifact: Path,
    source_selector: str,
    selected_source_value: str,
    source_fields: list[str],
    source_row: dict[str, str],
    evidence_granularity: str = EXACT_KEY_GRANULARITY,
    applicability: str = DIRECT_APPLICABILITY,
    applicability_authority: str = DIRECT_APPLICABILITY_AUTHORITY,
    applicability_primary: str = UNRESOLVED_APPLICABILITY_TARGET,
    applicability_predecessor_families: str = UNRESOLVED_APPLICABILITY_TARGET,
    applicability_predecessor_gates: str = UNRESOLVED_APPLICABILITY_TARGET,
    applicability_additional_operation_gates: str = (
        UNRESOLVED_APPLICABILITY_TARGET
    ),
    applicability_additional_operation_gate_child_specific_predecessors: str = (
        UNRESOLVED_APPLICABILITY_TARGET
    ),
    applicability_implicated_families: str = UNRESOLVED_APPLICABILITY_TARGET,
    applicability_implicated_gates: str = UNRESOLVED_APPLICABILITY_TARGET,
    applicability_authority_row_sha256: str | None = None,
) -> None:
    materialization_policy = (
        EXACT_MATERIALIZATION_POLICY
        if evidence_granularity == EXACT_KEY_GRANULARITY
        else SELECTOR_MATERIALIZATION_POLICY
    )
    relation = {
        "cluster_id": cluster_id,
        "evidence_kind": evidence_kind,
        "evidence_key": evidence_key,
        "source_artifact": source_artifact.name,
        "source_selector": source_selector,
        "selected_source_value": selected_source_value,
        "selected_source_value_sha256": value_sha256(selected_source_value),
        "source_row_sha256": row_sha256(source_fields, source_row),
        "evidence_granularity": evidence_granularity,
        "materialization_policy": materialization_policy,
        "applicability": applicability,
        "applicability_authority": applicability_authority,
        "applicability_primary_refinement_family_or_gate": applicability_primary,
        "applicability_required_predecessor_family_ids": (
            applicability_predecessor_families
        ),
        "applicability_required_predecessor_gate_stage_ids": (
            applicability_predecessor_gates
        ),
        "applicability_additional_operation_gate_stage_ids": (
            applicability_additional_operation_gates
        ),
        "applicability_additional_operation_gate_child_specific_immediate_predecessor_family_or_gate_ids": (
            applicability_additional_operation_gate_child_specific_predecessors
        ),
        "applicability_implicated_or_reopening_family_ids": (
            applicability_implicated_families
        ),
        "applicability_implicated_or_reopening_gate_stage_ids": (
            applicability_implicated_gates
        ),
        "applicability_authority_row_sha256": (
            applicability_authority_row_sha256
            if applicability_authority_row_sha256 is not None
            else value_sha256(applicability_authority)
        ),
        "terminal_disposition_policy": TERMINAL_POLICY,
        "allowed_terminal_dispositions": ALLOWED_TERMINAL_DISPOSITIONS,
        "policy_version": POLICY_VERSION,
    }
    relation["evidence_identity"] = evidence_identity(relation)
    rows.append(relation)


def digest_identities(rows: list[dict[str, str]]) -> str:
    payload = "".join(f"{row['evidence_identity']}\n" for row in rows).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def require_unique_source_keys(
    rows: list[dict[str, str]], key_field: str, artifact: Path
) -> None:
    keys = [row[key_field] for row in rows]
    if any(not key for key in keys):
        fail(f"{artifact.name} has an empty {key_field}")
    if len(keys) != len(set(keys)):
        fail(f"{artifact.name} has duplicate {key_field} values")


def build_rows() -> list[dict[str, str]]:
    cluster_rows = read_tsv(CLUSTER_REGISTRY, CLUSTER_REGISTRY_FIELDS)
    census_rows = read_tsv(CENSUS, CENSUS_FIELDS)
    safe_rows = read_tsv(SAFE_SURFACE, SAFE_SURFACE_FIELDS)
    d10_rows = read_tsv(D10_SURFACE, D10_SURFACE_FIELDS)
    unsafe_rows = read_tsv(UNSAFE_EVIDENCE, UNSAFE_EVIDENCE_FIELDS)
    impl_rows = read_tsv(TRAIT_IMPL, TRAIT_IMPL_FIELDS)
    topology_fields, topology_rows = read_tsv_with_fields(TRAIT_IMPL_TOPOLOGY)
    (
        operation_gate_assignments,
        operation_gate_assignment_row_hashes,
    ) = parse_operation_gate_assignment_authority()

    if len(cluster_rows) != 276:
        fail(f"cluster registry has {len(cluster_rows)} rows, expected 276")
    cluster_ids = [row["cluster_id"] for row in cluster_rows]
    if len(cluster_ids) != len(set(cluster_ids)):
        fail("cluster registry has duplicate cluster IDs")
    if [row["cluster_ordinal"] for row in cluster_rows] != [
        str(index) for index in range(1, 277)
    ]:
        fail("cluster registry ordinals are not the exact 1..276 sequence")
    if any(
        row["semantic_class"] != "G0_COVERAGE_CLUSTER"
        or row["importability"] != "NON_IMPORTABLE"
        or row["evidence_universe_policy"]
        != "ALL_SAFE_D10_UNSAFE_IMPL_AND_HELPER_KEYS"
        for row in cluster_rows
    ):
        fail("cluster registry does not carry the required non-importable evidence policy")
    cluster_set = set(cluster_ids)
    cluster_order = {cluster_id: index for index, cluster_id in enumerate(cluster_ids)}

    if [row["contract_id"] for row in census_rows] != cluster_ids:
        fail("contract census and cluster registry order differ")
    for cluster_row, census_row in zip(cluster_rows, census_rows):
        if cluster_row["census_row_sha256"] != row_sha256(CENSUS_FIELDS, census_row):
            fail(f"cluster registry census digest is stale for {census_row['contract_id']}")

    require_unique_source_keys(safe_rows, "canonical_key", SAFE_SURFACE)
    require_unique_source_keys(d10_rows, "canonical_key", D10_SURFACE)
    require_unique_source_keys(unsafe_rows, "canonical_key", UNSAFE_EVIDENCE)
    require_unique_source_keys(impl_rows, "impl_key", TRAIT_IMPL)
    if len(safe_rows) != 545:
        fail(f"safe surface map has {len(safe_rows)} rows, expected 545")
    if collections.Counter(row["route_kind"] for row in d10_rows) != {
        "contract": 138,
        "redundant_surface": 37,
    }:
        fail("D10 route partition changed from 138 contract / 37 redundant")
    if len(unsafe_rows) != 35:
        fail(f"unsafe evidence map has {len(unsafe_rows)} rows, expected 35")
    if len(impl_rows) != 334:
        fail(f"trait implementation crosswalk has {len(impl_rows)} rows, expected 334")
    topology_required_fields = {
        "impl_key",
        "primary_refinement_family_or_gate",
        "required_predecessor_family_ids",
        "required_predecessor_gate_stage_ids",
        "implicated_or_reopening_family_ids",
        "implicated_or_reopening_gate_stage_ids",
        "source_row_sha256",
    }
    if not topology_required_fields <= set(topology_fields):
        fail(
            "trait implementation topology routing lacks fields: "
            f"{sorted(topology_required_fields - set(topology_fields))}"
        )
    require_unique_source_keys(topology_rows, "impl_key", TRAIT_IMPL_TOPOLOGY)
    if len(topology_rows) != 334:
        fail(
            f"trait implementation topology routing has {len(topology_rows)} "
            "rows, expected 334"
        )
    topology_by_key = {row["impl_key"]: row for row in topology_rows}
    impl_by_key = {row["impl_key"]: row for row in impl_rows}
    if set(topology_by_key) != set(impl_by_key):
        fail("trait topology route key set differs from the concrete impl crosswalk")
    for impl_key, topology in topology_by_key.items():
        if topology["source_row_sha256"] != row_sha256(
            TRAIT_IMPL_FIELDS, impl_by_key[impl_key]
        ):
            fail(f"trait topology route source-row digest is stale for {impl_key}")

    relations: list[dict[str, str]] = []
    for row in safe_rows:
        cluster_id = row["primary_contract_id"]
        if cluster_id not in cluster_set:
            fail(f"safe key {row['canonical_key']} names unknown cluster {cluster_id}")
        if "stable_safe_seed" not in row["markers"].split(";"):
            fail(f"safe key {row['canonical_key']} lost stable_safe_seed marker")
        add_relation(
            relations,
            cluster_id=cluster_id,
            evidence_kind="STABLE_SAFE_SURFACE",
            evidence_key=row["canonical_key"],
            source_artifact=SAFE_SURFACE,
            source_selector=f"canonical_key={row['canonical_key']}",
            selected_source_value=row["canonical_key"],
            source_fields=SAFE_SURFACE_FIELDS,
            source_row=row,
        )

    for row in d10_rows:
        cluster_id = row["route_id"]
        if cluster_id not in cluster_set:
            fail(f"D10 key {row['canonical_key']} names unknown cluster {cluster_id}")
        evidence_kind = {
            "contract": "D10_CONTRACT_ROUTE",
            "redundant_surface": "D10_REDUNDANT_SURFACE_ROUTE",
        }[row["route_kind"]]
        add_relation(
            relations,
            cluster_id=cluster_id,
            evidence_kind=evidence_kind,
            evidence_key=row["canonical_key"],
            source_artifact=D10_SURFACE,
            source_selector=f"canonical_key={row['canonical_key']}",
            selected_source_value=row["canonical_key"],
            source_fields=D10_SURFACE_FIELDS,
            source_row=row,
        )

    for row in unsafe_rows:
        cluster_id = row["evidence_cluster_id"]
        if cluster_id not in cluster_set:
            fail(f"unsafe key {row['canonical_key']} names unknown cluster {cluster_id}")
        if (
            row["stability"] != "stable"
            or row["caller_safety"] != "unsafe"
            or row["evidence_disposition"] != "RAW_EVIDENCE_ONLY_NO_XLANG_SURFACE"
        ):
            fail(f"unsafe key {row['canonical_key']} is not stable evidence-only input")
        add_relation(
            relations,
            cluster_id=cluster_id,
            evidence_kind="STABLE_UNSAFE_EVIDENCE",
            evidence_key=row["canonical_key"],
            source_artifact=UNSAFE_EVIDENCE,
            source_selector=f"canonical_key={row['canonical_key']}",
            selected_source_value=row["canonical_key"],
            source_fields=UNSAFE_EVIDENCE_FIELDS,
            source_row=row,
        )

    impl_relation_count = 0
    step_row_count = 0
    operation_gate_relation_counts: collections.Counter[str] = collections.Counter()
    ungated_impl_relation_count = 0
    for row in impl_rows:
        topology = topology_by_key[row["impl_key"]]
        owner_ids = row["owning_contract_ids"].split(",")
        if any(not owner_id for owner_id in owner_ids) or len(owner_ids) != len(set(owner_ids)):
            fail(f"impl key {row['impl_key']} has invalid owning_contract_ids")
        if row["selection_family"] == "RANGE_STEP":
            step_row_count += 1
            if owner_ids != [
                "RANGE-ITER-HALFOPEN-01",
                "RANGE-ITER-FROM-01",
                "RANGE-ITER-INCLUSIVE-01",
            ]:
                fail(f"Step impl key {row['impl_key']} lost one of its three owners")
        elif len(owner_ids) != 1:
            fail(f"non-Step impl key {row['impl_key']} has multiple owners")
        for cluster_id in owner_ids:
            if cluster_id not in cluster_set:
                fail(f"impl key {row['impl_key']} names unknown cluster {cluster_id}")
            topology_primary = topology["primary_refinement_family_or_gate"]
            topology_row_digest = row_sha256(topology_fields, topology)
            assignment = operation_gate_assignments.get(cluster_id)
            if assignment is None:
                additional_operation_gate = "NONE"
                child_specific_immediate_predecessor = "NONE"
                assignment_row_digest = (
                    closed_negative_operation_gate_assignment_sha256(cluster_id)
                )
                ungated_impl_relation_count += 1
            else:
                (
                    additional_operation_gate,
                    immediate_predecessor_policy,
                    _,
                    _,
                ) = assignment
                if immediate_predecessor_policy != "EXACT_TOPOLOGY_PRIMARY":
                    fail(
                        f"operation-gate assignment for {cluster_id} lost the exact "
                        "topology-primary predecessor policy"
                    )
                child_specific_immediate_predecessor = topology_primary
                assignment_row_digest = operation_gate_assignment_row_hashes[
                    cluster_id
                ]
                if additional_operation_gate == topology_primary:
                    fail(
                        f"impl key {row['impl_key']} collapses its topology primary "
                        "and additional operation-gate targets"
                    )
                topology_predecessors = {
                    target
                    for field in (
                        "required_predecessor_family_ids",
                        "required_predecessor_gate_stage_ids",
                    )
                    for target in (
                        [] if topology[field] == "NONE" else topology[field].split(",")
                    )
                }
                if additional_operation_gate in topology_predecessors:
                    fail(
                        f"impl key {row['impl_key']} conflates an additional operation "
                        "gate with a topology predecessor"
                    )
                operation_gate_relation_counts[cluster_id] += 1
            composite_authority_digest = (
                composite_impl_applicability_authority_sha256(
                    cluster_id=cluster_id,
                    impl_key=row["impl_key"],
                    topology_row_sha256=topology_row_digest,
                    assignment_row_sha256=assignment_row_digest,
                    additional_operation_gate_stage_ids=(
                        additional_operation_gate
                    ),
                    child_specific_immediate_predecessor_family_or_gate_ids=(
                        child_specific_immediate_predecessor
                    ),
                )
            )
            add_relation(
                relations,
                cluster_id=cluster_id,
                evidence_kind="CONCRETE_TRAIT_IMPL",
                evidence_key=row["impl_key"],
                source_artifact=TRAIT_IMPL,
                source_selector=(
                    f"impl_key={row['impl_key']};owning_contract_id={cluster_id}"
                ),
                selected_source_value=row["impl_key"],
                source_fields=TRAIT_IMPL_FIELDS,
                source_row=row,
                applicability=IMPL_APPLICABILITY,
                applicability_authority=IMPL_APPLICABILITY_AUTHORITY,
                applicability_primary=topology_primary,
                applicability_predecessor_families=topology[
                    "required_predecessor_family_ids"
                ],
                applicability_predecessor_gates=topology[
                    "required_predecessor_gate_stage_ids"
                ],
                applicability_additional_operation_gates=(
                    additional_operation_gate
                ),
                applicability_additional_operation_gate_child_specific_predecessors=(
                    child_specific_immediate_predecessor
                ),
                applicability_implicated_families=topology[
                    "implicated_or_reopening_family_ids"
                ],
                applicability_implicated_gates=topology[
                    "implicated_or_reopening_gate_stage_ids"
                ],
                applicability_authority_row_sha256=composite_authority_digest,
            )
            impl_relation_count += 1
    if step_row_count != 22 or impl_relation_count != 378:
        fail(
            "trait implementation relation set changed from 312 single-owner plus "
            "22 three-owner Step rows"
        )
    expected_operation_gate_counts = {
        cluster_id: int(assignment[2])
        for cluster_id, assignment in operation_gate_assignments.items()
    }
    if dict(operation_gate_relation_counts) != expected_operation_gate_counts:
        fail(
            "exact operation-gate relation counts differ from the closed four-row "
            "authority"
        )
    if sum(operation_gate_relation_counts.values()) != 97:
        fail("operation-gate applicability does not cover exactly 97 impl relations")
    if ungated_impl_relation_count != 281:
        fail("closed-negative operation-gate assignment does not cover 281 relations")

    for row in census_rows:
        cluster_id = row["contract_id"]
        for field, evidence_kind in SELECTOR_KINDS.items():
            selected_value = row[field]
            if not selected_value:
                fail(f"{cluster_id} has an empty {field} selector")
            selected_digest = value_sha256(selected_value)
            add_relation(
                relations,
                cluster_id=cluster_id,
                evidence_kind=evidence_kind,
                evidence_key=f"{cluster_id}|{field}|{selected_digest}",
                source_artifact=CENSUS,
                source_selector=f"contract_id={cluster_id};field={field}",
                selected_source_value=selected_value,
                source_fields=CENSUS_FIELDS,
                source_row=row,
                evidence_granularity=SELECTOR_GRANULARITY,
                applicability=SELECTOR_APPLICABILITY,
                applicability_authority=SELECTOR_APPLICABILITY_AUTHORITY,
            )

    relations.sort(
        key=lambda row: (
            cluster_order[row["cluster_id"]],
            KIND_ORDER[row["evidence_kind"]],
            row["evidence_key"],
            row["source_selector"],
        )
    )
    relation_keys = [
        (
            row["cluster_id"],
            row["evidence_kind"],
            row["evidence_key"],
            row["source_selector"],
        )
        for row in relations
    ]
    if len(relation_keys) != len(set(relation_keys)):
        fail("duplicate evidence relation")
    identities = [row["evidence_identity"] for row in relations]
    if len(identities) != len(set(identities)):
        fail("duplicate evidence identity")
    if collections.Counter(row["evidence_kind"] for row in relations) != (
        EXPECTED_KIND_COUNTS
    ):
        fail("evidence-kind counts changed")
    if len(relations) != 1961:
        fail(f"built {len(relations)} evidence relations, expected 1961")

    by_cluster: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    by_kind: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for row in relations:
        by_cluster[row["cluster_id"]].append(row)
        by_kind[row["evidence_kind"]].append(row)
    if set(by_cluster) != cluster_set:
        fail("one or more G0 clusters have no evidence-universe relation")

    cluster_aggregates = {
        cluster_id: (str(len(rows)), digest_identities(rows))
        for cluster_id, rows in by_cluster.items()
    }
    kind_aggregates = {
        kind: (str(len(rows)), digest_identities(rows))
        for kind, rows in by_kind.items()
    }
    output_rows = []
    for ordinal, row in enumerate(relations, start=1):
        output_row = {"relation_ordinal": str(ordinal), **row}
        (
            output_row["cluster_relation_count"],
            output_row["cluster_relation_sha256"],
        ) = cluster_aggregates[row["cluster_id"]]
        (
            output_row["kind_relation_count"],
            output_row["kind_relation_sha256"],
        ) = kind_aggregates[row["evidence_kind"]]
        output_rows.append(output_row)
    return output_rows


def render(rows: list[dict[str, str]]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(
        output,
        fieldnames=FIELDS,
        delimiter="\t",
        lineterminator="\n",
    )
    writer.writeheader()
    writer.writerows(rows)
    return output.getvalue()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify that the checked-in registry exactly matches generated bytes",
    )
    arguments = parser.parse_args()
    expected = render(build_rows())
    if arguments.check:
        if not OUTPUT.is_file() or OUTPUT.read_text(encoding="utf-8") != expected:
            fail(f"{OUTPUT.name} is missing or stale")
    else:
        OUTPUT.write_text(expected, encoding="utf-8")
    print(
        "G0 coverage evidence universe: PASS — 1,961 exact relations cover "
        "276 clusters, including 545 stable-safe keys, all 175 D10 keys, "
        "35 stable-unsafe keys, 378 concrete-impl relations, and 828 "
        "source-field selectors"
    )


if __name__ == "__main__":
    main()
