#!/usr/bin/env python3
"""Verify the exact fail-closed G0 coverage evidence universe."""

from __future__ import annotations

import collections
import csv
import hashlib
import re
from pathlib import Path

from build_g0_coverage_evidence_universe import (
    ALLOWED_TERMINAL_DISPOSITIONS,
    CENSUS,
    CENSUS_FIELDS,
    CLUSTER_REGISTRY,
    CLUSTER_REGISTRY_FIELDS,
    D10_SURFACE,
    D10_SURFACE_FIELDS,
    DIRECT_APPLICABILITY,
    DIRECT_APPLICABILITY_AUTHORITY,
    EXACT_KEY_GRANULARITY,
    EXACT_MATERIALIZATION_POLICY,
    EXPECTED_KIND_COUNTS,
    FIELDS,
    IMPL_APPLICABILITY,
    IMPL_APPLICABILITY_AUTHORITY,
    OUTPUT,
    POLICY_VERSION,
    SAFE_SURFACE,
    SAFE_SURFACE_FIELDS,
    SELECTOR_APPLICABILITY,
    SELECTOR_APPLICABILITY_AUTHORITY,
    SELECTOR_GRANULARITY,
    SELECTOR_KINDS,
    SELECTOR_MATERIALIZATION_POLICY,
    TERMINAL_POLICY,
    TRAIT_IMPL,
    TRAIT_IMPL_FIELDS,
    TRAIT_IMPL_TOPOLOGY,
    UNRESOLVED_APPLICABILITY_TARGET,
    UNSAFE_EVIDENCE,
    UNSAFE_EVIDENCE_FIELDS,
    build_rows,
    evidence_identity,
    render,
    row_sha256,
    value_sha256,
)


ROOT = Path(__file__).resolve().parent.parent
VOCABULARY = ROOT / "G0-FAMILY-GATE-VOCABULARY.md"
CLUSTER_ROUTING = ROOT / "G0-CLUSTER-FAMILY-ROUTING.tsv"

EXPECTED_OUTPUT_SHA256 = "521fbbe8d49bbe95f2b4d6d7c46122d74443291e6546db48e16569f506e72eff"
EXPECTED_ALL_IDENTITY_SHA256 = (
    "9e9b5012b4c712a045883eb31ca2836e9d58c5128c455d1a2310d91e064e7d0c"
)
EXPECTED_CLUSTER_SUMMARY_SHA256 = (
    "25a27b11065b9c02dbd959c838e66d99349e73c4ccf991dd5725564f50212e13"
)
EXPECTED_KIND_IDENTITY_SHA256 = {
    "STABLE_SAFE_SURFACE": (
        "61904777ba44c7ef738e4ac637b9ec14965e6018a2e5e449c68868654956a9eb"
    ),
    "D10_CONTRACT_ROUTE": (
        "7067fc2e1136890c47eda536e7b647da0e6ff9b7aad9fa27faff9b88cf374fde"
    ),
    "D10_REDUNDANT_SURFACE_ROUTE": (
        "2eae3755f4a50578fbedf57a090db6a26f82c2e144dd468467b1e02bf297713c"
    ),
    "STABLE_UNSAFE_EVIDENCE": (
        "b1e2c42acf600330945469ccb88e13c5f6a33fb3b4961628f16fa37524994aaa"
    ),
    "CONCRETE_TRAIT_IMPL": (
        "1fe947f3f694a7671e9ec0ba288577fc91e7aaf81957d8efda8453eb73ba833d"
    ),
    "CLUSTER_RUST_SURFACES_SELECTOR": (
        "0c412f72d82d73aa77f0a033219606baf1c32b88ac63dc0c1c9af5177bb4b330"
    ),
    "CLUSTER_IMPLEMENTATION_PRIVILEGE_SELECTOR": (
        "8cca9d357d3dcdd58346a9fc187d545580428fe7835bdb07c24215b6acc756b8"
    ),
    "CLUSTER_SOURCE_REFS_SELECTOR": (
        "5be94c7a50be99f1d2028e205b49b011f1a8477b60251c9a49bb2a00d2a49ef1"
    ),
}
EXPECTED_KIND_KEYSET_SHA256 = {
    "STABLE_SAFE_SURFACE": (
        "1efbfb6a06913ac6fca541c099b693714de69e0fdefb97ba6fc4205096ff8b52"
    ),
    "D10_CONTRACT_ROUTE": (
        "a5afaec1b037367cbc5813a825a8bfa9c17dce18dbe96f973a813d9d4e3d2dca"
    ),
    "D10_REDUNDANT_SURFACE_ROUTE": (
        "dba9452e7a00774e1572c3b8bd08683967bc2901f8c7431967806cef91853cbc"
    ),
    "STABLE_UNSAFE_EVIDENCE": (
        "e93833338e17d3d7592b2efa0c4c2bbe23b4c0b36c9eacbd2973a1197e125b22"
    ),
    "CONCRETE_TRAIT_IMPL": (
        "1e7e3dc5fc345be2e991614494d1e4a43c9d5b97acecb68f72d288b33b9b1532"
    ),
    "CLUSTER_RUST_SURFACES_SELECTOR": (
        "1413a7c5dac877316f72402113b571660c32ef607963e6b0502f741304e8fb35"
    ),
    "CLUSTER_IMPLEMENTATION_PRIVILEGE_SELECTOR": (
        "135d18461155546fde74ca2036d719e5e52a4e61a12b4d65490920571551c5f8"
    ),
    "CLUSTER_SOURCE_REFS_SELECTOR": (
        "a3d395ade1bb570505d686e25036c0fdfca62091bf53041f7e878d3483e43cab"
    ),
}

EXPECTED_OPERATION_GATE_ASSIGNMENT_AUTHORITY = {
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

ADDITIONAL_OPERATION_GATE_FIELD = (
    "applicability_additional_operation_gate_stage_ids"
)
CHILD_SPECIFIC_IMMEDIATE_PREDECESSOR_FIELD = (
    "applicability_additional_operation_gate_child_specific_immediate_"
    "predecessor_family_or_gate_ids"
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"G0 coverage evidence-universe verification failed: {message}")


def read_tsv(path: Path, expected_fields: list[str]) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    require(fields == expected_fields, f"{path.name} schema changed")
    require(all(None not in row for row in rows), f"{path.name} has extra columns")
    return rows


def read_tsv_with_fields(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    require(bool(fields), f"{path.name} has no schema")
    require(all(None not in row for row in rows), f"{path.name} has extra columns")
    return fields, rows


def digest_lines(values: list[str]) -> str:
    return hashlib.sha256("".join(f"{value}\n" for value in values).encode("utf-8")).hexdigest()


def keyset_digest(values: set[str]) -> str:
    return digest_lines(sorted(values))


def sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def parse_operation_gate_assignment_authority() -> tuple[
    dict[str, tuple[str, str, str, str]], dict[str, str]
]:
    require(VOCABULARY.is_file(), f"missing {VOCABULARY.name}")
    text = VOCABULARY.read_text(encoding="utf-8")
    begin_marker = "<!-- G0_TRAIT_OPERATION_GATE_ASSIGNMENT_AUTHORITY_BEGIN -->"
    end_marker = "<!-- G0_TRAIT_OPERATION_GATE_ASSIGNMENT_AUTHORITY_END -->"
    require(
        text.count(begin_marker) == 1 and text.count(end_marker) == 1,
        "trait operation-gate authority markers are missing or duplicated",
    )
    body = text.split(begin_marker, 1)[1].split(end_marker, 1)[0]
    table_lines = [line for line in body.splitlines() if line.startswith("|")]
    require(len(table_lines) == 6, "trait operation-gate authority is not four rows")
    authority: dict[str, tuple[str, str, str, str]] = {}
    row_hashes: dict[str, str] = {}
    for line in table_lines[2:]:
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        require(len(cells) == 5, "trait operation-gate authority schema changed")
        cells = [
            cell[1:-1] if cell.startswith("`") and cell.endswith("`") else cell
            for cell in cells
        ]
        cluster_id = cells[0]
        require(cluster_id not in authority, "duplicate operation-gate authority row")
        authority[cluster_id] = (cells[1], cells[2], cells[3], cells[4])
        row_hashes[cluster_id] = sha256_text("\t".join(cells) + "\n")
    require(
        authority == EXPECTED_OPERATION_GATE_ASSIGNMENT_AUTHORITY,
        "trait operation-gate authority differs from the independent four-row pin",
    )
    return authority, row_hashes


def closed_negative_operation_gate_assignment_sha256(cluster_id: str) -> str:
    return sha256_text(
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
    return sha256_text(
        f"cluster_id={cluster_id}\n"
        f"impl_key={impl_key}\n"
        f"topology_row_sha256={topology_row_sha256}\n"
        f"operation_gate_assignment_row_sha256={assignment_row_sha256}\n"
        f"additional_operation_gate_stage_ids={additional_operation_gate_stage_ids}\n"
        "child_specific_immediate_predecessor_family_or_gate_ids="
        f"{child_specific_immediate_predecessor_family_or_gate_ids}\n"
    )


def parse_ids(value: str) -> set[str]:
    return set() if value == "NONE" else set(value.split(","))


def main() -> None:
    require(OUTPUT.is_file(), f"missing {OUTPUT.name}")
    output_bytes = OUTPUT.read_bytes()
    require(
        hashlib.sha256(output_bytes).hexdigest() == EXPECTED_OUTPUT_SHA256,
        "registry bytes differ from the pinned G0 evidence universe",
    )
    rows = read_tsv(OUTPUT, FIELDS)
    require(len(rows) == 1961, f"expected 1,961 relations, got {len(rows)}")
    require(
        all(all(row[field] for field in FIELDS) for row in rows),
        "registry has an empty required field",
    )
    require(
        render(build_rows()) == output_bytes.decode("utf-8"),
        "checked-in registry is not the deterministic source derivation",
    )
    require(
        [row["relation_ordinal"] for row in rows]
        == [str(index) for index in range(1, 1962)],
        "relation ordinals are not the exact 1..1961 sequence",
    )

    cluster_rows = read_tsv(CLUSTER_REGISTRY, CLUSTER_REGISTRY_FIELDS)
    census_rows = read_tsv(CENSUS, CENSUS_FIELDS)
    safe_rows = read_tsv(SAFE_SURFACE, SAFE_SURFACE_FIELDS)
    d10_rows = read_tsv(D10_SURFACE, D10_SURFACE_FIELDS)
    unsafe_rows = read_tsv(UNSAFE_EVIDENCE, UNSAFE_EVIDENCE_FIELDS)
    impl_rows = read_tsv(TRAIT_IMPL, TRAIT_IMPL_FIELDS)
    cluster_ids = [row["cluster_id"] for row in cluster_rows]
    cluster_set = set(cluster_ids)
    require(len(cluster_ids) == 276 and len(cluster_set) == 276, "cluster set changed")
    require(
        {row["cluster_id"] for row in rows} == cluster_set,
        "registry cluster set does not exactly equal the G0 cluster registry",
    )

    relation_keys = [
        (
            row["cluster_id"],
            row["evidence_kind"],
            row["evidence_key"],
            row["source_selector"],
        )
        for row in rows
    ]
    identities = [row["evidence_identity"] for row in rows]
    require(len(relation_keys) == len(set(relation_keys)), "duplicate evidence relation")
    require(len(identities) == len(set(identities)), "duplicate evidence identity")
    require(
        all(re.fullmatch(r"[0-9a-f]{64}", identity) for identity in identities),
        "invalid evidence identity",
    )
    require(
        all(
            row["selected_source_value_sha256"]
            == value_sha256(row["selected_source_value"])
            for row in rows
        ),
        "selected source-value digest mismatch",
    )
    require(
        all(row["evidence_identity"] == evidence_identity(row) for row in rows),
        "evidence identity does not match its canonical relation tuple",
    )
    require(
        all(
            row["terminal_disposition_policy"] == TERMINAL_POLICY
            and row["allowed_terminal_dispositions"]
            == ALLOWED_TERMINAL_DISPOSITIONS
            and row["policy_version"] == POLICY_VERSION
            for row in rows
        ),
        "Family Lock terminal-disposition policy drifted",
    )
    require(
        all(
            re.fullmatch(
                r"[0-9a-f]{64}", row["applicability_authority_row_sha256"]
            )
            for row in rows
        ),
        "an applicability-authority digest is invalid",
    )
    require(
        all(
            "APPLICABLE_WHEN_CLUSTER_IMPLICATED" not in value
            for row in rows
            for value in row.values()
        ),
        "a cluster-wide evidence-applicability shortcut survived",
    )
    require(
        digest_lines(identities) == EXPECTED_ALL_IDENTITY_SHA256,
        "whole ordered evidence-identity set changed",
    )

    by_kind: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    by_cluster: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for row in rows:
        by_kind[row["evidence_kind"]].append(row)
        by_cluster[row["cluster_id"]].append(row)
    require(
        {kind: len(kind_rows) for kind, kind_rows in by_kind.items()}
        == EXPECTED_KIND_COUNTS,
        "evidence-kind counts changed",
    )
    for kind, kind_rows in by_kind.items():
        identity_digest = digest_lines(
            [row["evidence_identity"] for row in kind_rows]
        )
        require(
            identity_digest == EXPECTED_KIND_IDENTITY_SHA256[kind],
            f"{kind} ordered evidence-identity set changed",
        )
        require(
            keyset_digest({row["evidence_key"] for row in kind_rows})
            == EXPECTED_KIND_KEYSET_SHA256[kind],
            f"{kind} evidence-key set changed",
        )
        require(
            all(
                row["kind_relation_count"] == str(len(kind_rows))
                and row["kind_relation_sha256"] == identity_digest
                for row in kind_rows
            ),
            f"{kind} repeated count or digest is stale",
        )

    cluster_summaries = []
    for cluster_id in cluster_ids:
        cluster_evidence = by_cluster[cluster_id]
        cluster_digest = digest_lines(
            [row["evidence_identity"] for row in cluster_evidence]
        )
        require(
            all(
                row["cluster_relation_count"] == str(len(cluster_evidence))
                and row["cluster_relation_sha256"] == cluster_digest
                for row in cluster_evidence
            ),
            f"{cluster_id} repeated count or ordered digest is stale",
        )
        cluster_summaries.append(
            f"{cluster_id}\t{len(cluster_evidence)}\t{cluster_digest}"
        )
    require(
        digest_lines(cluster_summaries) == EXPECTED_CLUSTER_SUMMARY_SHA256,
        "ordered per-cluster count/digest summary changed",
    )

    safe_output = [row for row in rows if row["evidence_kind"] == "STABLE_SAFE_SURFACE"]
    require(
        collections.Counter(
            (row["cluster_id"], row["evidence_key"]) for row in safe_output
        )
        == collections.Counter(
            (row["primary_contract_id"], row["canonical_key"]) for row in safe_rows
        ),
        "stable-safe source-key relation set is not exact",
    )
    require(
        all(
                row["source_artifact"] == SAFE_SURFACE.name
            and row["source_selector"] == f"canonical_key={row['evidence_key']}"
            and row["selected_source_value"] == row["evidence_key"]
            and row["evidence_granularity"] == EXACT_KEY_GRANULARITY
            and row["materialization_policy"] == EXACT_MATERIALIZATION_POLICY
            for row in safe_output
        ),
        "stable-safe provenance or granularity is not exact",
    )

    d10_kind = {
        "contract": "D10_CONTRACT_ROUTE",
        "redundant_surface": "D10_REDUNDANT_SURFACE_ROUTE",
    }
    d10_output = [row for row in rows if row["evidence_kind"] in set(d10_kind.values())]
    require(
        collections.Counter(
            (row["evidence_kind"], row["cluster_id"], row["evidence_key"])
            for row in d10_output
        )
        == collections.Counter(
            (d10_kind[row["route_kind"]], row["route_id"], row["canonical_key"])
            for row in d10_rows
        ),
        "D10 contract and redundant source-key relation sets are not exact",
    )
    require(
        len(d10_output) == 175
        and len({row["evidence_key"] for row in d10_output}) == 175,
        "a D10 key disappeared or was duplicated",
    )

    unsafe_output = [
        row for row in rows if row["evidence_kind"] == "STABLE_UNSAFE_EVIDENCE"
    ]
    require(
        collections.Counter(
            (row["cluster_id"], row["evidence_key"]) for row in unsafe_output
        )
        == collections.Counter(
            (row["evidence_cluster_id"], row["canonical_key"])
            for row in unsafe_rows
        ),
        "stable-unsafe source-key relation set is not exact",
    )

    impl_output = [row for row in rows if row["evidence_kind"] == "CONCRETE_TRAIT_IMPL"]
    topology_fields, topology_rows = read_tsv_with_fields(TRAIT_IMPL_TOPOLOGY)
    topology_required_fields = {
        "impl_key",
        "primary_refinement_family_or_gate",
        "required_predecessor_family_ids",
        "required_predecessor_gate_stage_ids",
        "implicated_or_reopening_family_ids",
        "implicated_or_reopening_gate_stage_ids",
        "source_row_sha256",
    }
    require(
        topology_required_fields <= set(topology_fields),
        "trait-implementation topology routing lacks required fields",
    )
    topology_by_key = {row["impl_key"]: row for row in topology_rows}
    impl_by_key = {row["impl_key"]: row for row in impl_rows}
    require(
        len(topology_rows) == len(topology_by_key) == 334,
        "trait-implementation topology routing is not 334 unique exact keys",
    )
    require(
        set(topology_by_key) == set(impl_by_key),
        "trait-implementation topology and crosswalk key sets differ",
    )
    require(
        all(
            topology_by_key[impl_key]["source_row_sha256"]
            == row_sha256(TRAIT_IMPL_FIELDS, impl_by_key[impl_key])
            for impl_key in impl_by_key
        ),
        "trait-implementation topology source-row binding is stale",
    )
    (
        operation_gate_assignments,
        operation_gate_assignment_row_hashes,
    ) = parse_operation_gate_assignment_authority()
    route_fields, route_rows = read_tsv_with_fields(CLUSTER_ROUTING)
    required_route_fields = {
        "cluster_id",
        "primary_refinement_owner_or_gate_stage",
        "required_predecessor_family_ids",
        "required_predecessor_gate_stage_ids",
        "route_category_id",
        "trait_impl_relation_count",
    }
    require(
        required_route_fields <= set(route_fields),
        "cluster routing lacks operation-gate cross-check fields",
    )
    route_by_cluster = {row["cluster_id"]: row for row in route_rows}
    require(
        len(route_rows) == len(route_by_cluster) == 276,
        "cluster routing is not 276 unique exact rows",
    )
    routed_trait_gate_clusters = {
        row["cluster_id"]
        for row in route_rows
        if row["route_category_id"] == "ROUTE-ACTIVE-TRAIT-GATE"
    }
    require(
        routed_trait_gate_clusters == set(operation_gate_assignments),
        "four-row operation-gate authority is not bidirectionally equal to the "
        "active trait-gate cluster routes",
    )
    for cluster_id, assignment in operation_gate_assignments.items():
        gate, predecessor_policy, relation_count, _ = assignment
        route = route_by_cluster[cluster_id]
        require(
            predecessor_policy == "EXACT_TOPOLOGY_PRIMARY"
            and route["primary_refinement_owner_or_gate_stage"] == gate
            and route["trait_impl_relation_count"] == relation_count,
            f"{cluster_id} operation-gate authority disagrees with cluster routing",
        )
    expected_operation_gate_relation_counts = collections.Counter(
        owner_id
        for row in impl_rows
        for owner_id in row["owning_contract_ids"].split(",")
        if owner_id in operation_gate_assignments
    )
    require(
        expected_operation_gate_relation_counts
        == collections.Counter(
            {
                cluster_id: int(assignment[2])
                for cluster_id, assignment in operation_gate_assignments.items()
            }
        ),
        "crosswalk owners do not independently derive the pinned 22/21/14/40 "
        "operation-gate relation counts",
    )
    expected_impl_relations = collections.Counter(
        (owner_id, row["impl_key"])
        for row in impl_rows
        for owner_id in row["owning_contract_ids"].split(",")
    )
    require(
        collections.Counter(
            (row["cluster_id"], row["evidence_key"]) for row in impl_output
        )
        == expected_impl_relations,
        "concrete trait-implementation relation set is not exact",
    )
    require(
        len(impl_output) == 378
        and len({row["evidence_key"] for row in impl_output}) == 334,
        "334 concrete impl keys did not expand to exactly 378 owner relations",
    )
    observed_operation_gate_counts: collections.Counter[str] = collections.Counter()
    target_cardinality_counts: collections.Counter[int] = collections.Counter()
    for row in impl_output:
        topology = topology_by_key[row["evidence_key"]]
        topology_primary = topology["primary_refinement_family_or_gate"]
        topology_row_digest = row_sha256(topology_fields, topology)
        assignment = operation_gate_assignments.get(row["cluster_id"])
        if assignment is None:
            expected_operation_gate = "NONE"
            expected_child_predecessor = "NONE"
            assignment_row_digest = (
                closed_negative_operation_gate_assignment_sha256(row["cluster_id"])
            )
        else:
            expected_operation_gate = assignment[0]
            expected_child_predecessor = topology_primary
            assignment_row_digest = operation_gate_assignment_row_hashes[
                row["cluster_id"]
            ]
            observed_operation_gate_counts[row["cluster_id"]] += 1
        expected_composite_digest = (
            composite_impl_applicability_authority_sha256(
                cluster_id=row["cluster_id"],
                impl_key=row["evidence_key"],
                topology_row_sha256=topology_row_digest,
                assignment_row_sha256=assignment_row_digest,
                additional_operation_gate_stage_ids=expected_operation_gate,
                child_specific_immediate_predecessor_family_or_gate_ids=(
                    expected_child_predecessor
                ),
            )
        )
        require(
            row["applicability"] == IMPL_APPLICABILITY
            and row["applicability_authority"] == IMPL_APPLICABILITY_AUTHORITY
            and row["applicability_primary_refinement_family_or_gate"]
            == topology_primary
            and row["applicability_required_predecessor_family_ids"]
            == topology["required_predecessor_family_ids"]
            and row["applicability_required_predecessor_gate_stage_ids"]
            == topology["required_predecessor_gate_stage_ids"]
            and row[ADDITIONAL_OPERATION_GATE_FIELD]
            == expected_operation_gate
            and row[CHILD_SPECIFIC_IMMEDIATE_PREDECESSOR_FIELD]
            == expected_child_predecessor
            and row["applicability_implicated_or_reopening_family_ids"]
            == topology["implicated_or_reopening_family_ids"]
            and row["applicability_implicated_or_reopening_gate_stage_ids"]
            == topology["implicated_or_reopening_gate_stage_ids"]
            and row["applicability_authority_row_sha256"]
            == expected_composite_digest,
            "concrete trait-implementation applicability is not the exact "
            "topology-plus-closed-operation-gate join",
        )
        topology_predecessors = parse_ids(
            row["applicability_required_predecessor_family_ids"]
        ) | parse_ids(row["applicability_required_predecessor_gate_stage_ids"])
        applicability_targets = {topology_primary}
        if expected_operation_gate != "NONE":
            applicability_targets.add(expected_operation_gate)
            require(
                expected_operation_gate != topology_primary
                and expected_operation_gate not in topology_predecessors,
                "an operation-gate target collapsed into the topology target or "
                "topology predecessor set",
            )
            route = route_by_cluster[row["cluster_id"]]
            full_route_predecessors = parse_ids(
                route["required_predecessor_family_ids"]
            ) | parse_ids(route["required_predecessor_gate_stage_ids"])
            require(
                topology_primary in full_route_predecessors,
                "a child-specific operation-gate edge is absent from the full "
                "cluster-route gate predecessor DAG",
            )
        require(
            applicability_targets.isdisjoint(topology_predecessors),
            "a topology predecessor was promoted into exact A(e)",
        )
        target_cardinality_counts[len(applicability_targets)] += 1
    require(
        observed_operation_gate_counts == expected_operation_gate_relation_counts,
        "observed operation-gate applicability does not preserve 22/21/14/40",
    )
    require(
        target_cardinality_counts == {1: 281, 2: 97},
        "exact concrete-implementation A(e) cardinality is not 281 single-target "
        "and 97 distinct two-target relations",
    )
    step_keys = {
        row["impl_key"] for row in impl_rows if row["selection_family"] == "RANGE_STEP"
    }
    require(
        len(step_keys) == 22
        and sum(row["evidence_key"] in step_keys for row in impl_output) == 66,
        "22 Step keys did not expand to all three range clusters",
    )

    census_by_id = {row["contract_id"]: row for row in census_rows}
    for field, kind in SELECTOR_KINDS.items():
        selector_rows = by_kind[kind]
        expected_selectors = {
            (
                census_row["contract_id"],
                f"{census_row['contract_id']}|{field}|{value_sha256(census_row[field])}",
                census_row[field],
                row_sha256(CENSUS_FIELDS, census_row),
            )
            for census_row in census_rows
        }
        actual_selectors = {
            (
                row["cluster_id"],
                row["evidence_key"],
                row["selected_source_value"],
                row["source_row_sha256"],
            )
            for row in selector_rows
        }
        require(
            actual_selectors == expected_selectors and len(actual_selectors) == 276,
            f"{field} selector set does not bind all 276 exact census rows",
        )
        require(
            all(
                row["source_artifact"] == CENSUS.name
                and row["source_selector"]
                == f"contract_id={row['cluster_id']};field={field}"
                and row["evidence_granularity"] == SELECTOR_GRANULARITY
                and row["materialization_policy"]
                == SELECTOR_MATERIALIZATION_POLICY
                and row["selected_source_value"]
                == census_by_id[row["cluster_id"]][field]
                for row in selector_rows
            ),
            f"{field} selector provenance or future-materialization state drifted",
        )

    exact_key_kinds = set(EXPECTED_KIND_COUNTS) - set(SELECTOR_KINDS.values())
    require(
        all(
            row["evidence_granularity"] == EXACT_KEY_GRANULARITY
            and row["materialization_policy"] == EXACT_MATERIALIZATION_POLICY
            for kind in exact_key_kinds
            for row in by_kind[kind]
        ),
        "an exact source key was downgraded to a coarse selector",
    )
    direct_kinds = exact_key_kinds - {"CONCRETE_TRAIT_IMPL"}
    unresolved_fields = [
        "applicability_primary_refinement_family_or_gate",
        "applicability_required_predecessor_family_ids",
        "applicability_required_predecessor_gate_stage_ids",
        ADDITIONAL_OPERATION_GATE_FIELD,
        CHILD_SPECIFIC_IMMEDIATE_PREDECESSOR_FIELD,
        "applicability_implicated_or_reopening_family_ids",
        "applicability_implicated_or_reopening_gate_stage_ids",
    ]
    require(
        all(
            row["applicability"] == DIRECT_APPLICABILITY
            and row["applicability_authority"]
            == DIRECT_APPLICABILITY_AUTHORITY
            and all(
                row[field] == UNRESOLVED_APPLICABILITY_TARGET
                for field in unresolved_fields
            )
            and row["applicability_authority_row_sha256"]
            == value_sha256(DIRECT_APPLICABILITY_AUTHORITY)
            for kind in direct_kinds
            for row in by_kind[kind]
        ),
        "an exact non-impl child bypasses independent applicability audit",
    )
    require(
        all(
            row["applicability"] == SELECTOR_APPLICABILITY
            and row["applicability_authority"]
            == SELECTOR_APPLICABILITY_AUTHORITY
            and all(
                row[field] == UNRESOLVED_APPLICABILITY_TARGET
                for field in unresolved_fields
            )
            and row["applicability_authority_row_sha256"]
            == value_sha256(SELECTOR_APPLICABILITY_AUTHORITY)
            for kind in SELECTOR_KINDS.values()
            for row in by_kind[kind]
        ),
        "a selector bypasses exhaustive expansion and independent child routing",
    )
    print(
        "G0 coverage evidence-universe verification: PASS — 1,961 relations; "
        "every one of 276 clusters has mechanically derived ordered counts and "
        "digests; all 1,133 exact external evidence relations and 828 explicit "
        "future-materialization selectors are fail-closed"
    )


if __name__ == "__main__":
    main()
