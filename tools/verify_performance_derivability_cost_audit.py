#!/usr/bin/env python3
"""Verify witness and held-out derivability/cost audit bookkeeping."""

from __future__ import annotations

import csv
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "optimizer-language-research/implementation/minimal-systems-capability"
MATRIX = BASE / "PERFORMANCE-FIRST-DERIVABILITY-COST-AUDIT.tsv"
REPORT = BASE / "PERFORMANCE-FIRST-DERIVABILITY-COST-AUDIT.md"
WITNESSES = BASE / "WITNESS-REGISTRY.md"

FIELDS = [
    "case_id",
    "contract_and_budget",
    "candidate_a_route",
    "candidate_a_structural_account",
    "candidate_a_status",
    "candidate_b_route",
    "candidate_b_structural_account",
    "candidate_b_status",
    "candidate_c_route",
    "candidate_c_structural_account",
    "candidate_c_status",
    "unresolved_or_falsifier",
]

CASES = [
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
]

STATUSES = {
    "CONTROL_CLAIM_UNTESTED",
    "ROUTED_UNPROVED",
    "ROUTED_DEFINITION_RISK",
    "HELDOUT_PAPER_ROUTE",
    "HELDOUT_DEFINITION_GAP",
    "HELDOUT_DEFINITION_RISK",
    "HELDOUT_CONDITIONAL_PREDECESSORS",
}


def fail(message: str) -> None:
    raise ValueError(message)


def main() -> int:
    with MATRIX.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        rows = list(reader)
        if list(reader.fieldnames or []) != FIELDS:
            fail("unexpected audit schema")

    if [row["case_id"] for row in rows] != CASES:
        fail("audit cases must match the closed ordered case set")

    witness_text = WITNESSES.read_text(encoding="utf-8")
    for row in rows:
        case_id = row["case_id"]
        if not re.search(rf"\b{re.escape(case_id)}\b", witness_text):
            fail(f"{case_id}: absent from witness registry")
        for field in FIELDS:
            if not row[field].strip():
                fail(f"{case_id}: empty {field}")
        for candidate in ("a", "b", "c"):
            status = row[f"candidate_{candidate}_status"]
            if status not in STATUSES:
                fail(f"{case_id}/{candidate}: unknown status {status}")
            if "PASS" in status:
                fail(f"{case_id}/{candidate}: paper audit must not claim PASS")
            if len(row[f"candidate_{candidate}_route"].split()) < 8:
                fail(f"{case_id}/{candidate}: route is too short")
            if len(row[f"candidate_{candidate}_structural_account"].split()) < 8:
                fail(f"{case_id}/{candidate}: structural account is too short")
        if len(row["unresolved_or_falsifier"].split()) < 8:
            fail(f"{case_id}: unresolved/falsifier is too short")

    if not any(row["candidate_b_status"] == "HELDOUT_DEFINITION_GAP" for row in rows):
        fail("audit must preserve Candidate B's held-out definition gap")

    report = REPORT.read_text(encoding="utf-8")
    for phrase in (
        "This is not a blind held-out validation",
        "no PASS status",
        "H-STORE exposes B's most important definition gap",
        "W-ARENA exposes a shared provenance gap",
        "no candidate was modified after opening the held-out contracts",
        "No repair, model, prototype, witness construction, held-out execution, or benchmark is authorized",
    ):
        if phrase not in report:
            fail(f"audit report omits safeguard or finding {phrase!r}")

    print("performance derivability audit: 14 cases, 42 A/B/C routes, no paper PASS claims")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"performance derivability audit failed: {error}", file=sys.stderr)
        raise SystemExit(1)
