#!/usr/bin/env python3
"""Verify the generated fail-closed G0 cluster family-routing registry."""

from __future__ import annotations

import collections
import csv
import hashlib
import io
import re

import build_g0_cluster_family_routing as build


EXPECTED_OUTPUT_SHA256 = "f56f589580f1a8c25f718fc2efc1cd968f1dac8fb5a4fac51484736060a1308d"
EXPECTED_CLUSTER_ORDER_SHA256 = "6419d6d8d5b69af94f00070f7ed680c8a450565e0fbbd76f10445c497abe35a1"
EXPECTED_ROUTE_SHA256 = "4f60b91a943a3b70f7cd23f8635ad88576d5b6cfcd5999ce54c7464660763582"
EXPECTED_TRAIT_SHA256 = "0053f2306de587e9a863a36f2b13798d9a8dc8899bc47fa1358bf6fe3ab72b7f"
EXPECTED_EVIDENCE_SHA256 = "bdaaea64c757f0f792188945f60fba7fba28bc3ac8c3e0bdcb0f7d1a64a44e68"
EXPECTED_DAG_SHA256 = "3fbda78ef73cae77e0a62e6d033b1f4d3223f1b1711acb382dfd2851e1ebc602"
EXPECTED_COMBINED_DAG_SHA256 = "49d73a238959f6cd10cb1695beb35b93f9ffe00a481f40903db8580f4bdc84f3"
EXPECTED_COMBINED_DAG_EDGE_COUNT = 53
EXPECTED_AGGREGATE_DIGESTS = {
    "route_state": "464dc9eaf75a8ee4bf8b270e7c8d2ba9fce1e78bc4f76038034739362d5f2a24",
    "route_category_id": "0165dcd36c2a717c191f0b18d9b493ed339d46ec341858c996a654382f57fb9a",
    "assignment_rationale_id": "f6c96f4ce5dfb9321359bb49b2785f6ba3ccc421529ee9fa30b4249dd3617a8b",
    "primary_refinement_owner_or_gate_stage": "354cf39cdc46f6aca6abe4efefdd9cab39b0a0c52ab694b6716902fe692337f4",
}
EXPECTED_COUNTS = {
    "route_state": {"ACTIVE": 243, "BOUNDARY": 13, "DELEGATED": 1, "SCOPED_LATER": 19},
    "route_category_id": {
        "ROUTE-ACTIVE-CROSS-FAMILY-GATE": 15,
        "ROUTE-ACTIVE-FAMILY-COMPOUND": 1,
        "ROUTE-ACTIVE-FAMILY-DIRECT": 213,
        "ROUTE-ACTIVE-TRAIT-FAMILY": 10,
        "ROUTE-ACTIVE-TRAIT-GATE": 4,
        "ROUTE-BOUNDARY-FAMILY": 4,
        "ROUTE-BOUNDARY-REJECTION-GATE": 9,
        "ROUTE-DELEGATED-GATE": 1,
        "ROUTE-SCOPED-LATER-FAMILY": 19,
    },
    "assignment_rationale_id": {
        "ASSIGN-BOXED-INIT-SPLIT": 1,
        "ASSIGN-DELEGATED-ALLOCATION": 1,
        "ASSIGN-DENSE-FOUNDATION": 48,
        "ASSIGN-EXACT-TRAIT-TOPOLOGY": 10,
        "ASSIGN-GENERIC-BOX": 1,
        "ASSIGN-LINKED-COMPOSITION": 9,
        "ASSIGN-OPERATION-GATE-COMPOSITION": 10,
        "ASSIGN-OUTER-TOPOLOGY-DIRECT": 113,
        "ASSIGN-RAW-BOUNDARY": 13,
        "ASSIGN-SCOPED-LATER-WRAPPER": 19,
        "ASSIGN-SEQUENCE-BACKED-HEAP": 9,
        "ASSIGN-TEXT-OVER-DENSE": 42,
    },
    "primary_refinement_owner_or_gate_stage": {
        "F-ABI": 3,
        "F-ALLOC": 1,
        "F-DENSE": 54,
        "F-DEQUE": 18,
        "F-DYNAMIC-BORROW": 5,
        "F-HEAP": 9,
        "F-ITERATION": 65,
        "F-ORDERED": 17,
        "F-PIN-ADDRESS": 3,
        "F-RECURSIVE": 2,
        "F-SHARED": 8,
        "F-SPARSE": 17,
        "F-TEXT": 42,
        "F-TYPE-IDENTITY": 2,
        "F-UNICODE": 1,
        "GATE-BULK-CONSTRUCTION-CROSS-FAMILY": 2,
        "GATE-CONVERSION-CROSS-FAMILY": 1,
        "GATE-FAMILY-ALLOCATION-ERROR": 1,
        "GATE-INDEX-CROSS-FAMILY": 1,
        "GATE-KEYED-ENTRY-CROSS-FAMILY": 3,
        "GATE-LINKED-COMPOSITION": 9,
        "GATE-RAW-SPELLING-REJECTION": 9,
        "GATE-SET-CROSS-FAMILY": 3,
    },
}
EXPECTED_DIMENSION_COUNTS = {
    "DIM-ACCESS": 209,
    "DIM-BEHAVIOR": 147,
    "DIM-FAILURE": 240,
    "DIM-OWNERSHIP": 190,
    "DIM-RESOURCE-LIFETIME": 140,
    "DIM-STORED-BORROW": 166,
}
EXPECTED_DELEGATED_OWNER_CONTRACT_IDS = (
    "SEQ-TRY-RESERVE-01",
    "DEQUE-RESERVE-01",
    "HEAP-RESERVE-01",
    "HMAP-RESERVE-01",
    "HSET-RESERVE-01",
    "STRING-RESERVE-01",
)
EXPECTED_APPLICABILITY_POLICY_PREFIX = (
    "LOCK_AUDIT_DOMAIN_INCLUDES_CLUSTER_IFF_PRIMARY_REFINEMENT_OWNER_OR_GATE_"
    "STAGE_EQUALS_LOCK_TARGET_OR_LOCK_TARGET_IS_IN_IMPLICATED_OR_REOPENING_"
    "FAMILY_OR_GATE_SET;CLUSTER_INCLUSION_NEVER_CONFERS_EVIDENCE_CHILD_"
    "APPLICABILITY;EVIDENCE_CHILD_APPLICABLE_IFF_LOCK_TARGET_IS_IN_"
    "INDEPENDENTLY_DERIVED_A_OF_E;EACH_APPLICABLE_EVIDENCE_TARGET_PAIR_"
    "REQUIRES_EXACT_MEMBER_OUTCOME_OR_CLAIM_BLOCKING_EXCLUSION;NON_APPLICABLE_"
    "TARGET_FORBIDS_REFINEMENT_PREDECESSOR_PROOF_OR_EXCLUSION;"
)


def fail(message: str) -> None:
    raise SystemExit(f"G0 cluster family-routing verification failed: {message}")


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def ids(text: str) -> set[str]:
    return set() if text == "NONE" else set(text.split(","))


def aggregate_digest(rows: list[dict[str, str]], field: str) -> str:
    grouped: dict[str, list[str]] = collections.defaultdict(list)
    for row in rows:
        grouped[row[field]].append(row["cluster_id"])
    parts = []
    for key in sorted(grouped):
        child_digest = sha256_text("".join(f"{value}\n" for value in grouped[key]))
        parts.append(f"{key}|{len(grouped[key])}|{child_digest}\n")
    return sha256_text("".join(parts))


def assert_acyclic(edges: set[tuple[str, str]]) -> None:
    graph: dict[str, set[str]] = collections.defaultdict(set)
    for predecessor, owner in edges:
        graph[predecessor].add(owner)
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(node: str) -> None:
        if node in visiting:
            fail(f"predecessor graph contains a cycle through {node}")
        if node in visited:
            return
        visiting.add(node)
        for successor in graph.get(node, set()):
            visit(successor)
        visiting.remove(node)
        visited.add(node)

    for node in set(graph) | {target for targets in graph.values() for target in targets}:
        visit(node)


def main() -> None:
    if not build.OUTPUT.is_file():
        fail(f"missing {build.OUTPUT.name}")
    actual = build.OUTPUT.read_text(encoding="utf-8")
    actual_sha256 = sha256_text(actual)
    if actual_sha256 != EXPECTED_OUTPUT_SHA256:
        fail("reviewed output hash changed")
    expected = build.render(build.build_rows())
    if actual != expected:
        fail("checked-in routing differs from deterministic regeneration")
    reader = csv.DictReader(io.StringIO(actual), delimiter="\t")
    if list(reader.fieldnames or []) != build.FIELDS:
        fail("routing schema changed")
    rows = list(reader)
    if len(rows) != 276:
        fail(f"routing has {len(rows)} rows, expected 276")
    if [row["cluster_ordinal"] for row in rows] != [str(i) for i in range(1, 277)]:
        fail("cluster ordinals are not the exact 1..276 sequence")
    census_ids = [
        row["contract_id"] for row in build.read_tsv(build.CENSUS, build.CENSUS_FIELDS)[1]
    ]
    cluster_ids = [row["cluster_id"] for row in rows]
    if cluster_ids != census_ids or len(set(cluster_ids)) != 276:
        fail("cluster identity set/order differs from the census")
    if sha256_text("".join(f"{value}\n" for value in cluster_ids)) != EXPECTED_CLUSTER_ORDER_SHA256:
        fail("reviewed cluster-order digest changed")
    by_cluster_id = {row["cluster_id"]: row for row in rows}
    align_row = by_cluster_id["RAW-UNSAFE-ALIGN-01"]
    if not (
        align_row["primary_refinement_owner_or_gate_stage"]
        == "GATE-RAW-SPELLING-REJECTION"
        and align_row["required_predecessor_family_ids"] == "NONE"
        and align_row["implicated_or_reopening_family_ids"] == "F-DENSE"
        and "DIM-ACCESS" in ids(align_row["required_crosscut_dimension_ids"])
        and "F-PIN-ADDRESS" not in "|".join(align_row.values())
    ):
        fail("RAW-UNSAFE-ALIGN-01 lost its dense borrowed-slice boundary route")
    _, payload_rows = build.read_tsv(build.PAYLOAD_SCOPE, build.PAYLOAD_SCOPE_FIELDS)
    allocation_scope = next(
        row for row in payload_rows if row["contract_id"] == "ALLOC-ERROR-01"
    )
    delegated_contracts = allocation_scope["scope_owner_contract_ids"].split(";")
    if tuple(delegated_contracts) != EXPECTED_DELEGATED_OWNER_CONTRACT_IDS:
        fail("allocation payload scope changed from six reviewed delegated owner contracts")
    allocation_route = by_cluster_id["ALLOC-ERROR-01"]
    if allocation_route["delegated_owner_contract_ids"] != ",".join(delegated_contracts):
        fail("delegated owner contract IDs differ from PAYLOAD-SCOPE-CLASSIFICATION")
    derived_delegated_families = {
        by_cluster_id[contract_id]["primary_refinement_owner_or_gate_stage"]
        for contract_id in delegated_contracts
    }
    if derived_delegated_families != ids(
        allocation_route["delegated_owner_family_ids"]
    ):
        fail("delegated owner families differ from the six exact owner cluster routes")
    route_identities = [row["route_identity"] for row in rows]
    if len(set(route_identities)) != 276 or any(
        not re.fullmatch(r"[0-9a-f]{64}", value) for value in route_identities
    ):
        fail("route identities are not 276 unique SHA-256 values")
    if any(
        not row["applicability_policy"].startswith(
            EXPECTED_APPLICABILITY_POLICY_PREFIX
        )
        for row in rows
    ):
        fail("cluster audit-domain policy lost independent A(e) applicability")

    for field, expected_counts in EXPECTED_COUNTS.items():
        actual_counts = dict(collections.Counter(row[field] for row in rows))
        if actual_counts != expected_counts:
            fail(f"reviewed {field} counts changed")
        if aggregate_digest(rows, field) != EXPECTED_AGGREGATE_DIGESTS[field]:
            fail(f"reviewed {field} membership digest changed")

    route_digest = sha256_text(
        "".join(
            f"{row['cluster_id']}|{row['primary_refinement_owner_or_gate_stage']}|"
            f"{row['required_predecessor_family_ids']}|{row['required_predecessor_gate_stage_ids']}|"
            f"{row['implicated_or_reopening_family_ids']}|{row['implicated_or_reopening_gate_stage_ids']}|"
            f"{row['delegated_owner_family_ids']}|{row['delegated_owner_contract_ids']}|"
            f"{row['required_crosscut_dimension_ids']}|"
            f"{row['route_category_id']}|{row['assignment_rationale_id']}|{row['route_state']}\n"
            for row in rows
        )
    )
    if route_digest != EXPECTED_ROUTE_SHA256:
        fail("reviewed ordered route-target digest changed")
    trait_digest = sha256_text(
        "".join(
            f"{row['cluster_id']}|{row['trait_impl_relation_count']}|"
            f"{row['trait_impl_key_sha256']}|{row['trait_impl_required_predecessor_family_ids']}|"
            f"{row['trait_impl_required_predecessor_gate_stage_ids']}|"
            f"{row['trait_impl_topology_family_ids']}|{row['trait_impl_topology_gate_stage_ids']}\n"
            for row in rows
        )
    )
    if trait_digest != EXPECTED_TRAIT_SHA256:
        fail("reviewed exact trait topology-union digest changed")
    if sum(int(row["trait_impl_relation_count"]) for row in rows) != 378:
        fail("trait-implementation relation total is not 378")
    evidence_digest = sha256_text(
        "".join(
            f"{row['cluster_id']}|{row['evidence_child_count']}|"
            f"{row['evidence_child_identity_sha256']}\n"
            for row in rows
        )
    )
    if evidence_digest != EXPECTED_EVIDENCE_SHA256:
        fail("reviewed exact evidence-child digest changed")
    if sum(int(row["evidence_child_count"]) for row in rows) != 1961:
        fail("evidence-child relation total is not 1961")

    families, gates, dimensions, _, _, _ = build.parse_vocabulary()
    dimension_counts: collections.Counter[str] = collections.Counter()
    edge_lines: list[str] = []
    graph_edges: set[tuple[str, str]] = set()
    for row in rows:
        owner = row["primary_refinement_owner_or_gate_stage"]
        predecessor_families = ids(row["required_predecessor_family_ids"])
        predecessor_gates = ids(row["required_predecessor_gate_stage_ids"])
        implicated_families = ids(row["implicated_or_reopening_family_ids"])
        implicated_gates = ids(row["implicated_or_reopening_gate_stage_ids"])
        delegated_owners = ids(row["delegated_owner_family_ids"])
        row_dimensions = ids(row["required_crosscut_dimension_ids"])
        if (
            owner not in families | gates
            or not predecessor_families <= families
            or not predecessor_gates <= gates
            or not implicated_families <= families
            or not implicated_gates <= gates
            or not delegated_owners <= families
            or not row_dimensions <= dimensions
        ):
            fail(f"{row['cluster_id']} uses an ID outside the typed vocabulary")
        if owner.startswith("F-") and owner not in implicated_families:
            fail(f"{row['cluster_id']} omits its primary family from applicability")
        if owner.startswith("GATE-") and owner not in implicated_gates:
            fail(f"{row['cluster_id']} omits its primary gate from applicability")
        family_overlap = predecessor_families & implicated_families
        gate_overlap = predecessor_gates & implicated_gates
        if row["route_category_id"] == "ROUTE-ACTIVE-TRAIT-GATE":
            if family_overlap != ids(row["trait_impl_topology_family_ids"]):
                fail(f"{row['cluster_id']} has an unexplained trait-gate family overlap")
            if gate_overlap != ids(row["trait_impl_topology_gate_stage_ids"]):
                fail(f"{row['cluster_id']} has an unexplained trait-gate overlap")
        elif family_overlap or gate_overlap:
            fail(f"{row['cluster_id']} conflates predecessor and applicability sets")
        if delegated_owners:
            if (
                row["route_state"] != "DELEGATED"
                or predecessor_families
                or implicated_families
                or row["delegated_owner_contract_ids"] == "NONE"
                or delegated_owners
                != {"F-DENSE", "F-DEQUE", "F-SPARSE", "F-HEAP", "F-TEXT"}
            ):
                fail("delegated allocation owners are not isolated from applicability")
        elif row["route_state"] == "DELEGATED":
            fail("delegated route has no delegated owner family set")
        elif row["delegated_owner_contract_ids"] != "NONE":
            fail(f"{row['cluster_id']} has delegated owner contracts outside delegation")
        for dimension in row_dimensions:
            dimension_counts[dimension] += 1
        for predecessor in predecessor_families:
            edge_lines.append(f"{row['cluster_id']}|F|{predecessor}|{owner}\n")
            graph_edges.add((predecessor, owner))
        for predecessor in predecessor_gates:
            edge_lines.append(f"{row['cluster_id']}|G|{predecessor}|{owner}\n")
            graph_edges.add((predecessor, owner))
    if dict(sorted(dimension_counts.items())) != EXPECTED_DIMENSION_COUNTS:
        fail("reviewed local-dimension counts changed")
    if sha256_text("".join(sorted(edge_lines))) != EXPECTED_DAG_SHA256:
        fail("reviewed predecessor family/gate graph digest changed")
    assert_acyclic(graph_edges)

    combined_edges = set(graph_edges)
    requirement_fields, requirement_rows = build.read_tsv(build.FAMILY_REQUIREMENTS)
    for required in ("role", "closure_owner_or_gate_stage", "required_predecessor_family_ids"):
        if required not in requirement_fields:
            fail(f"family-requirement registry lacks {required}")
    for requirement in requirement_rows:
        owner = requirement["closure_owner_or_gate_stage"]
        if requirement["role"] == "O" or owner not in families | gates:
            continue
        for predecessor in ids(requirement["required_predecessor_family_ids"]):
            if predecessor not in families:
                fail("family-requirement union DAG contains an untyped predecessor")
            combined_edges.add((predecessor, owner))

    topology_fields, topology_rows = build.read_tsv(build.TRAIT_IMPL_TOPOLOGY)
    for required in (
        "primary_refinement_family_or_gate",
        "required_predecessor_family_ids",
        "required_predecessor_gate_stage_ids",
    ):
        if required not in topology_fields:
            fail(f"trait topology routing lacks {required}")
    if len(topology_rows) != 334:
        fail("combined DAG input is not the exact 334-row topology routing")
    for topology in topology_rows:
        owner = topology["primary_refinement_family_or_gate"]
        if owner not in families | gates:
            fail("trait topology union DAG contains an untyped owner")
        for predecessor in ids(topology["required_predecessor_family_ids"]):
            if predecessor not in families:
                fail("trait topology union DAG contains an untyped family predecessor")
            combined_edges.add((predecessor, owner))
        for predecessor in ids(topology["required_predecessor_gate_stage_ids"]):
            if predecessor not in gates:
                fail("trait topology union DAG contains an untyped gate predecessor")
            combined_edges.add((predecessor, owner))
    if len(combined_edges) != EXPECTED_COMBINED_DAG_EDGE_COUNT:
        fail("reviewed combined Family Lock DAG edge count changed")
    combined_digest = sha256_text(
        "".join(f"{source}|{target}\n" for source, target in sorted(combined_edges))
    )
    if combined_digest != EXPECTED_COMBINED_DAG_SHA256:
        fail("reviewed combined requirement/cluster/topology DAG digest changed")
    assert_acyclic(combined_edges)

    print(
        "G0 cluster family routing verifier: PASS — 276 exact routes, 1,961 "
        "evidence relations, 378 exact trait relations, typed acyclic family/gate "
        f"dependencies, independent review pins (sha256={actual_sha256})"
    )


if __name__ == "__main__":
    main()
