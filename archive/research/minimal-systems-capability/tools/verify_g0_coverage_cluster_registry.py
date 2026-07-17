#!/usr/bin/env python3
"""Independently verify the non-importable G0 coverage-cluster policy."""

from __future__ import annotations

import csv
import hashlib
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "G0-COVERAGE-CLUSTER-REGISTRY.tsv"
CENSUS = ROOT / "RUST-DATA-CONTRACT-CENSUS.tsv"
MATRIX = ROOT / "DERIVATION-MATRIX.tsv"
BUILDER = ROOT / "tools" / "build_g0_coverage_cluster_registry.py"

FIELDS = [
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
EXPECTED_FILE_SHA256 = "5140fe32d4529e28452f5e7515fe4a7d8c90c57e1303cde00fcbf4eff196af2d"
EXPECTED_CLUSTER_ID_SHA256 = "6419d6d8d5b69af94f00070f7ed680c8a450565e0fbbd76f10445c497abe35a1"
EXPECTED_SOURCE_PAIR_SHA256 = "46f41cacdaef4851ee5c2159d2881e37891275ad536db32520e195ead1573f62"
EXPECTED_POLICY_SHA256 = "8cf38f58f6865b970d40c38637a553c7b27df5b262a5b6ffab4bcc26af2c0670"
EXPECTED_POLICY = {
    "semantic_class": "G0_COVERAGE_CLUSTER",
    "importability": "NON_IMPORTABLE",
    "refinement_policy": "FAMILY_LOCK_COMPLETE_EVIDENCE_TO_MEMBER_OUTCOME",
    "evidence_universe_policy": "ALL_SAFE_D10_UNSAFE_IMPL_AND_HELPER_KEYS",
    "allowed_evidence_dispositions": (
        "REFINED_IN_LOCK;PREDECESSOR_PROVED;EXCLUDED_BLOCKS_CLAIM"
    ),
    "prohibited_direct_uses": (
        "FAMILY_CLOSURE_UNIT;MEMBER_CONTRACT;OUTCOME_CONTRACT;"
        "CAPABILITY_INHERITANCE;COST_INHERITANCE;FAMILY_LOCK_IMPORT;"
        "CANDIDATE_CONSTRUCTION;SCORED_EXPERIMENT;FAMILY_E;FAMILY_P"
    ),
    "policy_version": "xlang-g0-coverage-cluster-policy-v1",
}


def fail(message: str) -> None:
    raise SystemExit(f"G0 coverage-cluster registry: FAIL: {message}")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_tsv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        fields = list(reader.fieldnames or [])
        rows = list(reader)
    if not fields or any(None in row for row in rows):
        fail(f"{path.name} is malformed")
    return fields, rows


def row_sha256(fields: list[str], row: dict[str, str]) -> str:
    return sha256(("\t".join(row[field] for field in fields) + "\n").encode())


def digest_lines(lines: list[str]) -> str:
    return sha256("".join(f"{line}\n" for line in lines).encode())


def main() -> None:
    result = subprocess.run(
        [sys.executable, "-B", str(BUILDER), "--check"],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.returncode != 0:
        fail(result.stdout.strip() or "builder check failed")
    if sha256(REGISTRY.read_bytes()) != EXPECTED_FILE_SHA256:
        fail("registry bytes changed without independent verifier review")

    fields, rows = read_tsv(REGISTRY)
    if fields != FIELDS:
        fail("registry schema changed")
    if len(rows) != 276:
        fail(f"expected 276 rows, found {len(rows)}")
    if [row["cluster_ordinal"] for row in rows] != [str(i) for i in range(1, 277)]:
        fail("cluster ordinals are not the exact 1..276 sequence")
    cluster_ids = [row["cluster_id"] for row in rows]
    if len(cluster_ids) != len(set(cluster_ids)):
        fail("cluster IDs are not unique")
    if digest_lines(cluster_ids) != EXPECTED_CLUSTER_ID_SHA256:
        fail("ordered cluster-ID set changed")
    if any(
        any(row[field] != value for field, value in EXPECTED_POLICY.items())
        for row in rows
    ):
        fail("a cluster lost the exact non-importable policy")
    policy_payload = "\n".join(EXPECTED_POLICY.values()) + "\n"
    if sha256(policy_payload.encode()) != EXPECTED_POLICY_SHA256:
        fail("independent policy pin changed")

    census_fields, census_rows = read_tsv(CENSUS)
    matrix_fields, matrix_rows = read_tsv(MATRIX)
    if [row["contract_id"] for row in census_rows] != cluster_ids:
        fail("census order differs from the cluster registry")
    if [row["contract_id"] for row in matrix_rows] != cluster_ids:
        fail("derivation-matrix order differs from the cluster registry")
    for cluster, census, matrix in zip(rows, census_rows, matrix_rows):
        if cluster["family"] != census["family"]:
            fail(f"family label drifted for {cluster['cluster_id']}")
        if cluster["census_row_sha256"] != row_sha256(census_fields, census):
            fail(f"census-row digest drifted for {cluster['cluster_id']}")
        if cluster["derivation_row_sha256"] != row_sha256(matrix_fields, matrix):
            fail(f"matrix-row digest drifted for {cluster['cluster_id']}")
    source_pairs = [
        f"{row['census_row_sha256']}\t{row['derivation_row_sha256']}" for row in rows
    ]
    if digest_lines(source_pairs) != EXPECTED_SOURCE_PAIR_SHA256:
        fail("ordered source-row digest set changed")

    prohibited = set(EXPECTED_POLICY["prohibited_direct_uses"].split(";"))
    required_prohibitions = {
        "FAMILY_CLOSURE_UNIT",
        "MEMBER_CONTRACT",
        "OUTCOME_CONTRACT",
        "CAPABILITY_INHERITANCE",
        "COST_INHERITANCE",
        "FAMILY_LOCK_IMPORT",
        "CANDIDATE_CONSTRUCTION",
        "SCORED_EXPERIMENT",
        "FAMILY_E",
        "FAMILY_P",
    }
    if prohibited != required_prohibitions:
        fail("prohibited-use set is incomplete")

    print(
        "G0 coverage-cluster registry: PASS — 276 exact rows independently "
        "pin the non-importable cluster policy and source digests"
    )


if __name__ == "__main__":
    main()
