#!/usr/bin/env python3
"""Verify the combined G0 family and gate predecessor DAG."""

from __future__ import annotations

import csv
import hashlib
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
FAMILY_REQUIREMENTS = ROOT / "G0-FAMILY-REQUIREMENT-REGISTRY.tsv"
CLUSTER_ROUTING = ROOT / "G0-CLUSTER-FAMILY-ROUTING.tsv"
TRAIT_TOPOLOGY = ROOT / "G0-TRAIT-IMPL-TOPOLOGY-ROUTING.tsv"
EVIDENCE_UNIVERSE = ROOT / "G0-COVERAGE-EVIDENCE-UNIVERSE.tsv"

EXPECTED_ROW_COUNTS = {
    "family_requirement": 49,
    "cluster_route": 276,
    "trait_topology": 334,
}
EXPECTED_ORIGIN_EDGE_COUNTS = {
    "cluster_route": 208,
    "family_requirement": 23,
    "trait_topology": 222,
}
EXPECTED_ORIGIN_EDGE_SHA256 = (
    "82dcaa2e167007bfd173acf2459cd383fbce01e3a5501a1dc29f3a85a916fd3b"
)
EXPECTED_UNION_EDGE_COUNT = 53
EXPECTED_UNION_EDGE_SHA256 = (
    "3d4d6882515a4218ec710fa47ca4ceb28f2c4653202197d1036cc1064437bf98"
)
EXPECTED_CHILD_GATE_RELATION_COUNT = 97
EXPECTED_CHILD_GATE_RELATION_SHA256 = (
    "1720fd28e161be9132480da27a8b5232906de8c21ed3b12eceab313df6aaa9ec"
)
EXPECTED_CHILD_GATE_UNION_EDGE_COUNT = 17
EXPECTED_CHILD_GATE_UNION_EDGE_SHA256 = (
    "eaf96d2dabe5c503925cbe5b7b987eb4d57f541455fc0da1e2485d485bd625e3"
)


def fail(message: str) -> None:
    raise SystemExit(f"G0 combined dependency DAG verification failed: {message}")


def read_tsv(path: Path, required_fields: set[str]) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = set(reader.fieldnames or [])
        rows = list(reader)
    if not required_fields <= fields:
        fail(f"{path.name} lacks fields {sorted(required_fields - fields)}")
    if any(None in row for row in rows):
        fail(f"{path.name} has extra TSV columns")
    return rows


def digest_lines(lines: list[str]) -> str:
    payload = "".join(f"{line}\n" for line in lines).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def typed_tokens(value: str, *, source: str) -> list[str]:
    if value == "NONE":
        return []
    tokens = value.split(",")
    if any(not token or not token.startswith(("F-", "GATE-")) for token in tokens):
        fail(f"{source} has an untyped predecessor list: {value}")
    if len(tokens) != len(set(tokens)):
        fail(f"{source} has duplicate predecessor IDs: {value}")
    return tokens


def require_node(value: str, *, source: str) -> str:
    if not value.startswith(("F-", "GATE-")):
        fail(f"{source} has a non-family/non-gate primary node: {value}")
    return value


def main() -> None:
    requirements = read_tsv(
        FAMILY_REQUIREMENTS,
        {
            "obligation_id",
            "role",
            "closure_owner_or_gate_stage",
            "required_predecessor_family_ids",
        },
    )
    routes = read_tsv(
        CLUSTER_ROUTING,
        {
            "cluster_id",
            "primary_refinement_owner_or_gate_stage",
            "required_predecessor_family_ids",
            "required_predecessor_gate_stage_ids",
        },
    )
    topology = read_tsv(
        TRAIT_TOPOLOGY,
        {
            "impl_key",
            "primary_refinement_family_or_gate",
            "required_predecessor_family_ids",
            "required_predecessor_gate_stage_ids",
        },
    )
    actual_row_counts = {
        "family_requirement": len(requirements),
        "cluster_route": len(routes),
        "trait_topology": len(topology),
    }
    if actual_row_counts != EXPECTED_ROW_COUNTS:
        fail(f"source row counts changed: {actual_row_counts}")

    origin_edges: set[tuple[str, str, str, str]] = set()
    cluster_route_edges: set[tuple[str, str, str]] = set()

    def add_edges(origin: str, key: str, primary: str, values: list[str]) -> None:
        node = require_node(primary, source=f"{origin}:{key}")
        for predecessor in values:
            edge = (origin, key, node, predecessor)
            if edge in origin_edges:
                fail(f"duplicate origin edge: {edge}")
            origin_edges.add(edge)

    for row in requirements:
        if row["role"] == "O":
            continue
        primary = row["closure_owner_or_gate_stage"]
        predecessors = typed_tokens(
            row["required_predecessor_family_ids"],
            source=f"family_requirement:{row['obligation_id']}",
        )
        if primary == "ALL-FAMILY-LOCKS":
            if predecessors:
                fail("an all-family protected row has a predecessor")
            continue
        add_edges("family_requirement", row["obligation_id"], primary, predecessors)

    for row in routes:
        source = f"cluster_route:{row['cluster_id']}"
        predecessors = typed_tokens(
            row["required_predecessor_family_ids"], source=source
        ) + typed_tokens(row["required_predecessor_gate_stage_ids"], source=source)
        add_edges(
            "cluster_route",
            row["cluster_id"],
            row["primary_refinement_owner_or_gate_stage"],
            predecessors,
        )
        cluster_route_edges.update(
            (
                row["cluster_id"],
                row["primary_refinement_owner_or_gate_stage"],
                predecessor,
            )
            for predecessor in predecessors
        )

    for row in topology:
        source = f"trait_topology:{row['impl_key']}"
        predecessors = typed_tokens(
            row["required_predecessor_family_ids"], source=source
        ) + typed_tokens(row["required_predecessor_gate_stage_ids"], source=source)
        add_edges(
            "trait_topology",
            row["impl_key"],
            row["primary_refinement_family_or_gate"],
            predecessors,
        )

    evidence = read_tsv(
        EVIDENCE_UNIVERSE,
        {
            "evidence_identity",
            "cluster_id",
            "evidence_kind",
            "applicability_primary_refinement_family_or_gate",
            "applicability_additional_operation_gate_stage_ids",
            "applicability_additional_operation_gate_child_specific_immediate_predecessor_family_or_gate_ids",
        },
    )
    if len(evidence) != 1961:
        fail(f"evidence universe row count changed: {len(evidence)}")
    child_gate_lines: list[str] = []
    child_gate_union_edges: set[tuple[str, str]] = set()
    for row in evidence:
        if row["evidence_kind"] != "CONCRETE_TRAIT_IMPL":
            continue
        gate = row["applicability_additional_operation_gate_stage_ids"]
        predecessor = row[
            "applicability_additional_operation_gate_child_specific_immediate_"
            "predecessor_family_or_gate_ids"
        ]
        if gate == "NONE" or predecessor == "NONE":
            if gate != predecessor:
                fail("a concrete impl has a half-populated child operation-gate edge")
            continue
        if not gate.startswith("GATE-") or not predecessor.startswith(("F-", "GATE-")):
            fail("a child operation-gate edge is not typed")
        if gate == predecessor:
            fail("a child operation-gate edge is a self-cycle")
        if row["applicability_primary_refinement_family_or_gate"] != predecessor:
            fail("a child operation-gate predecessor is not the exact topology primary")
        child_edge = (row["cluster_id"], gate, predecessor)
        if child_edge not in cluster_route_edges:
            fail(
                "a child-specific operation-gate edge is not a subset of the full "
                "cluster-route gate DAG"
            )
        child_gate_lines.append(
            f"{row['evidence_identity']}\t{row['cluster_id']}\t{gate}\t{predecessor}"
        )
        child_gate_union_edges.add((gate, predecessor))
    if len(child_gate_lines) != EXPECTED_CHILD_GATE_RELATION_COUNT:
        fail(f"child operation-gate relation count changed: {len(child_gate_lines)}")
    child_relation_digest = digest_lines(child_gate_lines)
    if child_relation_digest != EXPECTED_CHILD_GATE_RELATION_SHA256:
        fail(f"ordered child operation-gate relation changed: {child_relation_digest}")
    if len(child_gate_union_edges) != EXPECTED_CHILD_GATE_UNION_EDGE_COUNT:
        fail(
            "child operation-gate union edge count changed: "
            f"{len(child_gate_union_edges)}"
        )
    child_union_digest = digest_lines(
        sorted(f"{gate}\t{predecessor}" for gate, predecessor in child_gate_union_edges)
    )
    if child_union_digest != EXPECTED_CHILD_GATE_UNION_EDGE_SHA256:
        fail(f"child operation-gate union edge set changed: {child_union_digest}")

    origin_counts = dict(Counter(edge[0] for edge in origin_edges))
    if origin_counts != EXPECTED_ORIGIN_EDGE_COUNTS:
        fail(f"origin edge counts changed: {origin_counts}")
    ordered_origin_lines = sorted("\t".join(edge) for edge in origin_edges)
    origin_digest = digest_lines(ordered_origin_lines)
    if origin_digest != EXPECTED_ORIGIN_EDGE_SHA256:
        fail(f"ordered source-edge relation changed: {origin_digest}")

    union_edges = {(primary, predecessor) for _, _, primary, predecessor in origin_edges}
    if len(union_edges) != EXPECTED_UNION_EDGE_COUNT:
        fail(f"union edge count changed: {len(union_edges)}")
    ordered_union_lines = sorted(f"{primary}\t{predecessor}" for primary, predecessor in union_edges)
    union_digest = digest_lines(ordered_union_lines)
    if union_digest != EXPECTED_UNION_EDGE_SHA256:
        fail(f"ordered union edge set changed: {union_digest}")
    for primary, predecessor in union_edges:
        if primary == predecessor:
            fail(f"self-cycle at {primary}")

    graph: dict[str, set[str]] = defaultdict(set)
    nodes: set[str] = set()
    for primary, predecessor in union_edges:
        graph[primary].add(predecessor)
        nodes.update((primary, predecessor))
    visiting: list[str] = []
    visited: set[str] = set()

    def visit(node: str) -> None:
        if node in visiting:
            cycle = visiting[visiting.index(node) :] + [node]
            fail(f"combined dependency cycle: {' -> '.join(cycle)}")
        if node in visited:
            return
        visiting.append(node)
        for predecessor in sorted(graph[node]):
            visit(predecessor)
        visiting.pop()
        visited.add(node)

    for node in sorted(nodes):
        visit(node)

    print(
        "G0 combined dependency DAG verification: PASS — "
        f"{len(origin_edges)} source edges collapse to {len(union_edges)} "
        "acyclic family/gate dependencies; all 97 child-specific operation-gate "
        "edges are typed subsets of the full cluster-route gate DAG"
    )


if __name__ == "__main__":
    main()
