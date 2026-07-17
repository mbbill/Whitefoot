#!/usr/bin/env python3
"""Verify the exact G0 B/M/W/H/O requirement and canary universe."""

from __future__ import annotations

import csv
import hashlib
import subprocess
import sys
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "G0-FAMILY-REQUIREMENT-REGISTRY.tsv"
BUILDER = ROOT / "tools" / "build_g0_family_requirement_registry.py"
VOCABULARY = ROOT / "G0-FAMILY-GATE-VOCABULARY.md"

EXPECTED_ROWS = 49
EXPECTED_FILE_SHA256 = "6e53a8dbbe24045c22448141022459b6bcaa200fa181018baf15c824437bc1a6"
EXPECTED_VOCABULARY_SHA256 = (
    "9174a2aa7c51b79f36b7ca4280072dea431ba3eeb1b67b58d873ba6b49ff0fad"
)
EXPECTED_ID_SET_SHA256 = "337721e580681c1b71c45b71e77d80fea09d78e92fca42b7f77928ea87cfefbd"
EXPECTED_ROLES = {"B": 4, "H": 7, "M": 18, "O": 5, "W": 15}
EXPECTED_STATUS = {
    "E": 1,
    "OPTIONAL": 4,
    "P": 1,
    "PROTECTED": 2,
    "REQUIRED": 12,
    "U": 2,
    "X": 27,
}
EXPECTED_MATRIX_CANARIES = {
    "C-ARENA": 1,
    "C-COMPACT": 1,
    "C-DEQUE": 1,
    "C-FIX": 2,
    "C-GRAPH": 3,
    "C-HASH": 1,
    "C-HEAP": 1,
    "C-ITER": 1,
    "C-LAZY-DRAIN": 1,
    "C-ORDER": 1,
    "C-P2": 1,
    "C-POOL": 1,
    "C-RECUR": 1,
    "C-SEQ": 9,
    "C-SMALL": 1,
    "C-SORT": 1,
    "C-TEXT": 1,
    "H-IPQ": 1,
    "H-LRU": 1,
    "H-STORE": 1,
}
EXPECTED_WITNESS_SUBJECTS = {
    "B-FIX",
    "B-P2",
    "W-POOL",
    "W-ARENA",
    "W-SMALL",
    "W-RECUR",
    "W-GRAPH",
    "W-ECS",
    "W-GAP",
    "W-PIPE",
    "H-FLATSET",
    "H-STORE",
    "H-LRU",
    "H-IPQ",
    "O-SLAB",
    "O-ROPE-UNIQUE",
    "O-INTRUSIVE",
    "O-LAZY-DRAIN",
}
EXPECTED_OWNERS = {
    "ALL-FAMILY-LOCKS": 4,
    "F-ARENA": 2,
    "F-DENSE": 19,
    "F-DEQUE": 1,
    "F-HEAP": 3,
    "F-IDENTITY": 10,
    "F-ITERATION": 1,
    "F-ORDERED": 1,
    "F-PIN-ADDRESS": 1,
    "F-RECURSIVE": 2,
    "F-SPARSE": 3,
    "F-TEXT": 1,
    "GATE-ROPE-POST-DENSE-ORDERED": 1,
}
EXPECTED_PREDECESSORS = {
    "F-DENSE": 12,
    "F-DENSE,F-IDENTITY,F-RECURSIVE": 1,
    "F-DENSE,F-ORDERED": 1,
    "F-DENSE,F-SPARSE": 2,
    "F-SPARSE": 4,
    "NONE": 29,
}
EXPECTED_REBIND_POLICIES = {
    "EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY": 2,
    "EVERY_CANDIDATE_EXECUTES": 4,
    "OPTIONAL_PROMOTION_ONLY_IMPLICATED_REOPEN": 5,
    "OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN": 38,
}
EXPECTED_C_FAIL_CONTRACT = (
    "every owning canary repeated through early `Result`, checked capacity "
    "failure, partial construction, and each recoverable allocation point"
)
EXPECTED_C_FAIL_SOURCE_SHA256 = (
    "19f02393da541de5fb44b5aec311af516ac8caddf53abbb3af49a92f6c34903d"
)
EXPECTED_CANONICAL_MANDATORY_PREDECESSORS = {
    "ALL-FAMILY-LOCKS": set(),
    "F-DENSE": set(),
    "F-DEQUE": set(),
    "F-ORDERED": {"F-DENSE", "F-IDENTITY", "F-RECURSIVE"},
    "F-RECURSIVE": set(),
    "F-PIN-ADDRESS": set(),
    "F-ARENA": {"F-DENSE"},
    "F-SPARSE": {"F-DENSE"},
    "F-TEXT": {"F-DENSE"},
    "F-ITERATION": {"F-DENSE"},
    "F-HEAP": {"F-DENSE", "F-SPARSE"},
    "F-IDENTITY": {"F-DENSE", "F-SPARSE"},
}


def fail(message: str) -> None:
    raise SystemExit(f"G0 family requirement registry: FAIL: {message}")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    result = subprocess.run(
        [sys.executable, "-B", str(BUILDER), "--check"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        fail(result.stderr.strip() or result.stdout.strip() or "builder check failed")

    data = REGISTRY.read_bytes()
    if sha256_bytes(data) != EXPECTED_FILE_SHA256:
        fail("registry byte digest changed without verifier review")
    with REGISTRY.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if len(rows) != EXPECTED_ROWS:
        fail(f"expected {EXPECTED_ROWS} rows, found {len(rows)}")

    ids = [row["obligation_id"] for row in rows]
    if len(ids) != len(set(ids)):
        fail("duplicate obligation ID")
    if sha256_bytes("\n".join(ids).encode("utf-8")) != EXPECTED_ID_SET_SHA256:
        fail("ordered obligation-ID set changed without verifier review")
    if dict(Counter(row["role"] for row in rows)) != EXPECTED_ROLES:
        fail("B/M/W/H/O role partition changed")
    if dict(Counter(row["g0_status"] for row in rows)) != EXPECTED_STATUS:
        fail("G0 status partition changed")
    record_kinds = Counter(row["record_kind"] for row in rows)
    if dict(record_kinds) != {"CAPABILITY_MATRIX": 31, "WITNESS_REGISTRY": 18}:
        fail("source-registry partition changed")
    matrix_rows = [row for row in rows if row["record_kind"] == "CAPABILITY_MATRIX"]
    witness_rows = [row for row in rows if row["record_kind"] == "WITNESS_REGISTRY"]
    if dict(Counter(row["primary_canary_id"] for row in matrix_rows)) != EXPECTED_MATRIX_CANARIES:
        fail("Section-4 primary canary universe changed")
    if {row["registry_subject_id"] for row in witness_rows} != EXPECTED_WITNESS_SUBJECTS:
        fail("witness-registry B/W/H/O universe changed")
    if dict(Counter(row["closure_owner_or_gate_stage"] for row in rows)) != EXPECTED_OWNERS:
        fail("closure-owner/gate-stage partition changed")
    if dict(Counter(row["required_predecessor_family_ids"] for row in rows)) != EXPECTED_PREDECESSORS:
        fail("predecessor-family partition changed")
    if dict(Counter(row["implicated_rebind_policy"] for row in rows)) != EXPECTED_REBIND_POLICIES:
        fail("implicated-family rebind policy changed")
    vocabulary_digest = sha256_bytes(VOCABULARY.read_bytes())
    if vocabulary_digest != EXPECTED_VOCABULARY_SHA256:
        fail("family/gate vocabulary bytes changed without verifier review")
    if {
        row["vocabulary_authority_sha256"] for row in rows
    } != {EXPECTED_VOCABULARY_SHA256}:
        fail("registry does not bind the exact vocabulary authority")

    owner_allowed_terminal = {
        "REQUIRED_IN_LOCK",
        "PROTECTED_CONTROL",
        "EXCLUDED_BLOCKS_FAMILY_AND_FLOOR",
        "OPTIONAL_NOT_PROMOTED",
    }
    rebind_allowed_terminal = {
        "PREDECESSOR_REUSE_AND_LOCAL_REBIND_PROVED",
        "PROTECTED_CONTROL",
        "EXCLUDED_BLOCKS_FAMILY_AND_FLOOR",
        "NOT_APPLICABLE_REOPENING_ONLY",
    }
    for row in rows:
        role = row["role"]
        owner_dispositions = set(
            row["owner_lock_allowed_dispositions"].split(",")
        )
        rebind_dispositions = set(
            row["implicated_rebind_allowed_dispositions"].split(",")
        )
        if not owner_dispositions or not owner_dispositions <= owner_allowed_terminal:
            fail(f"illegal owner-lock disposition set for {row['obligation_id']}")
        if not rebind_dispositions or not rebind_dispositions <= rebind_allowed_terminal:
            fail(f"illegal implicated-rebind disposition set for {row['obligation_id']}")
        if row["required_crosscut_canary_ids"] not in {"C-FAIL", "NONE"}:
            fail(f"unknown cross-cut canary state for {row['obligation_id']}")
        if role in {"M", "W", "H"} and (
            "OPTIONAL_NOT_PROMOTED" in owner_dispositions
        ):
            fail(f"mandatory role can be silently omitted: {row['obligation_id']}")
        if role != "O" and (
            "OPTIONAL_NOT_PROMOTED" in owner_dispositions
        ):
            fail(f"non-O role has optional terminal state: {row['obligation_id']}")
        if role != "B" and (
            "EXCLUDED_BLOCKS_FAMILY_AND_FLOOR" not in owner_dispositions
        ):
            fail(f"exclusion does not block closure for {row['obligation_id']}")
        if role != "B":
            expected_owner = {
                "REQUIRED_IN_LOCK",
                "EXCLUDED_BLOCKS_FAMILY_AND_FLOOR",
            }
            expected_rebind = {"NOT_APPLICABLE_REOPENING_ONLY"}
            if (
                row["implicated_rebind_policy"]
                == "EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY"
            ):
                expected_rebind = {
                    "PREDECESSOR_REUSE_AND_LOCAL_REBIND_PROVED",
                    "EXCLUDED_BLOCKS_FAMILY_AND_FLOOR",
                }
            if role == "O":
                expected_owner.add("OPTIONAL_NOT_PROMOTED")
            if owner_dispositions != expected_owner:
                fail(f"owner-lock disposition equation changed: {row['obligation_id']}")
            if rebind_dispositions != expected_rebind:
                fail(f"implicated-rebind disposition equation changed: {row['obligation_id']}")
        for field in (
            "closure_owner_or_gate_stage",
            "required_predecessor_family_ids",
            "implicated_family_ids",
        ):
            if not row[field]:
                fail(f"missing {field} for {row['obligation_id']}")
        if not row["primary_canary_id"] or not row["canary_contract"]:
            fail(f"missing canary identity or coarse contract for {row['obligation_id']}")
        for field in ("source_row_sha256", "canary_source_sha256"):
            value = row[field]
            if len(value) != 64 or any(ch not in "0123456789abcdef" for ch in value):
                fail(f"invalid {field} for {row['obligation_id']}")
        if role in {"B", "W", "H"} and row["record_kind"] == "CAPABILITY_MATRIX":
            if row["linked_registry_ids"] == "NONE":
                fail(f"B/W/H operation lacks exact witness binding: {row['obligation_id']}")
        if role == "B":
            if row["closure_owner_or_gate_stage"] != "ALL-FAMILY-LOCKS":
                fail(f"protected baseline is not required in every lock: {row['obligation_id']}")
            if row["closure_policy"] != "PROTECTED_NO_REGRESSION_IN_EVERY_FAMILY_LOCK":
                fail(f"protected baseline lost its no-regression policy: {row['obligation_id']}")
            if owner_dispositions != {"PROTECTED_CONTROL"} or rebind_dispositions != {
                "PROTECTED_CONTROL"
            }:
                fail(f"protected baseline can bypass per-candidate execution: {row['obligation_id']}")
        if row["record_kind"] == "WITNESS_REGISTRY":
            if row["linked_registry_ids"] != row["registry_subject_id"]:
                fail(f"witness row lost self-identity binding: {row['obligation_id']}")
        if "POST-" in row["closure_owner_or_gate_stage"]:
            if row["required_predecessor_family_ids"] == "NONE":
                fail(f"post-adoption gate lacks exact predecessors: {row['obligation_id']}")
            if row["closure_owner_or_gate_stage"] in row["required_predecessor_family_ids"].split(","):
                fail(f"post-adoption witness can close its predecessor: {row['obligation_id']}")

    traversal_rows = [
        row
        for row in rows
        if row["workload_or_operation"]
        in {"Borrowed, uniq, and owning iteration", "W-PIPE"}
    ]
    if len(traversal_rows) != 2 or any(
        row["implicated_rebind_policy"]
        != "EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY"
        for row in traversal_rows
    ):
        fail("topology-specific traversal can inherit dense evidence")
    matrix_traversal = next(
        row
        for row in traversal_rows
        if row["workload_or_operation"] == "Borrowed, uniq, and owning iteration"
    )
    pipe = next(row for row in traversal_rows if row["workload_or_operation"] == "W-PIPE")
    if matrix_traversal["closure_owner_or_gate_stage"] != "F-DENSE":
        fail("dense C-ITER obligation is not owned by the dense lock")
    if pipe["closure_owner_or_gate_stage"] != "F-ITERATION" or pipe[
        "required_predecessor_family_ids"
    ] != "F-DENSE":
        fail("W-PIPE is not staged after dense traversal")

    for row in rows:
        applicability = row["c_fail_applicability"]
        crosscut = row["required_crosscut_canary_ids"]
        if applicability == "REQUIRED_OWNING_CANARY" and crosscut != "C-FAIL":
            fail(f"owning canary lost C-FAIL: {row['obligation_id']}")
        if applicability == "REQUIRED_OWNING_CANARY" and (
            row["crosscut_canary_contract"] != EXPECTED_C_FAIL_CONTRACT
            or row["crosscut_canary_source_sha256"]
            != EXPECTED_C_FAIL_SOURCE_SHA256
        ):
            fail(f"C-FAIL contract or source identity drifted: {row['obligation_id']}")
        if applicability == "NOT_APPLICABLE_NONOWNING_CANARY" and crosscut != "NONE":
            fail(f"non-owning canary gained C-FAIL: {row['obligation_id']}")
        if applicability == "NOT_APPLICABLE_NONOWNING_CANARY" and (
            row["crosscut_canary_contract"] != "NONE"
            or row["crosscut_canary_source_sha256"] != "NONE"
        ):
            fail(f"non-owning canary retained a C-FAIL contract: {row['obligation_id']}")
        if applicability not in {
            "REQUIRED_OWNING_CANARY",
            "NOT_APPLICABLE_NONOWNING_CANARY",
        }:
            fail(f"unknown C-FAIL applicability: {row['obligation_id']}")

    ordinary_owners = {
        row["closure_owner_or_gate_stage"]
        for row in rows
        if row["closure_owner_or_gate_stage"].startswith("F-")
    }
    used_families = set()
    for row in rows:
        used_families.update(
            token
            for token in row["required_predecessor_family_ids"].split(",")
            if token.startswith("F-")
        )
        used_families.update(
            token
            for token in row["implicated_family_ids"].split(",")
            if token.startswith("F-")
        )
        owner = row["closure_owner_or_gate_stage"]
        if owner.startswith("F-") and owner not in row["implicated_family_ids"].split(","):
            fail(f"ordinary owner is absent from its implicated set: {row['obligation_id']}")
    if used_families - ordinary_owners:
        fail(f"orphan family IDs lack an owner row: {sorted(used_families - ordinary_owners)}")

    # A Lock's canonical predecessor set is the union of its applicable
    # non-optional owned rows, not a value copied onto every row. Optional rows
    # join only after promotion or a mandatory dependency makes them required.
    owner_universe = {
        row["closure_owner_or_gate_stage"]
        for row in rows
        if row["closure_owner_or_gate_stage"] == "ALL-FAMILY-LOCKS"
        or row["closure_owner_or_gate_stage"].startswith("F-")
    }
    canonical_predecessors = {owner: set() for owner in owner_universe}
    for row in rows:
        owner = row["closure_owner_or_gate_stage"]
        if owner not in canonical_predecessors or row["role"] == "O":
            continue
        if row["required_predecessor_family_ids"] != "NONE":
            canonical_predecessors[owner].update(
                row["required_predecessor_family_ids"].split(",")
            )
    if canonical_predecessors != EXPECTED_CANONICAL_MANDATORY_PREDECESSORS:
        fail(f"canonical mandatory predecessor DAG changed: {canonical_predecessors}")
    for owner, predecessors in canonical_predecessors.items():
        if owner in predecessors:
            fail(f"canonical predecessor DAG contains a self-cycle at {owner}")
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(owner: str) -> None:
        if owner in visiting:
            fail(f"canonical predecessor DAG contains a cycle at {owner}")
        if owner in visited:
            return
        visiting.add(owner)
        for predecessor in canonical_predecessors.get(owner, set()):
            visit(predecessor)
        visiting.remove(owner)
        visited.add(owner)

    for owner in canonical_predecessors:
        visit(owner)

    print(
        "G0 family requirement registry: PASS "
        "(49 obligations; B4/M18/W15/H7/O5; exact Section-4 plus witness-registry union)"
    )


if __name__ == "__main__":
    main()
