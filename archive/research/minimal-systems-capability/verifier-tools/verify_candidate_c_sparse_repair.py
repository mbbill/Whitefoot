#!/usr/bin/env python3
"""Verify the bounded Candidate C sparse-repair comparison."""

from __future__ import annotations

import csv
import sys
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASE = (
    ROOT
    / "optimizer-language-research"
    / "implementation"
    / "minimal-systems-capability"
)
MATRIX = BASE / "CANDIDATE-C-SPARSE-REPAIR-MATRIX.tsv"
REPORT = BASE / "CANDIDATE-C-SPARSE-REPAIR-CANDIDATES.md"

FIELDS = (
    "candidate",
    "operation",
    "route_state",
    "exact_route",
    "required_state",
    "owner_traffic",
    "provenance",
    "failure_cleanup",
    "fact_behavior",
    "c0_dependency",
    "reference_events",
    "forced_delta",
    "safety_status",
    "boundedness",
    "growth_convergence_risk",
    "unresolved",
    "falsifier",
)
CANDIDATES = {"SR-CLOSED", "SR-PROFILE", "SR-ORTHOGONAL"}
OPERATIONS = {
    "lookup",
    "vacant insertion",
    "replacement",
    "removal",
    "rehash",
}
STATES = {
    "CLOSED",
    "OPEN",
    "TAXED",
    "CONVERGES-B",
    "CONVERGES-A",
    "UNKNOWN",
}


def fail(message: str) -> None:
    raise ValueError(message)


def main() -> int:
    with MATRIX.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != FIELDS:
            fail(f"matrix schema mismatch: {reader.fieldnames}")
        rows = list(reader)

    if len(rows) != 15:
        fail(f"expected exactly 15 rows, found {len(rows)}")
    pairs: set[tuple[str, str]] = set()
    for index, row in enumerate(rows, 1):
        missing = [field for field in FIELDS if not (row.get(field) or "").strip()]
        if missing:
            fail(f"row {index} has empty fields: {missing}")
        candidate = row["candidate"]
        operation = row["operation"]
        state = row["route_state"]
        if candidate not in CANDIDATES:
            fail(f"row {index} uses unexpected candidate {candidate!r}")
        if operation not in OPERATIONS:
            fail(f"row {index} uses unexpected operation {operation!r}")
        if state not in STATES:
            fail(f"row {index} uses unexpected route state {state!r}")
        pair = (candidate, operation)
        if pair in pairs:
            fail(f"duplicate candidate-operation pair {pair}")
        pairs.add(pair)

    expected_pairs = {
        (candidate, operation)
        for candidate in CANDIDATES
        for operation in OPERATIONS
    }
    if pairs != expected_pairs:
        fail(f"candidate-operation coverage mismatch: {sorted(expected_pairs - pairs)}")

    by_candidate = {
        candidate: Counter(
            row["route_state"] for row in rows if row["candidate"] == candidate
        )
        for candidate in sorted(CANDIDATES)
    }
    if by_candidate["SR-CLOSED"] != {"CLOSED": 5}:
        fail(f"SR-CLOSED state mismatch: {by_candidate['SR-CLOSED']}")
    if by_candidate["SR-PROFILE"] != {"CLOSED": 5}:
        fail(f"SR-PROFILE state mismatch: {by_candidate['SR-PROFILE']}")
    if by_candidate["SR-ORTHOGONAL"] != {"CONVERGES-B": 5}:
        fail(f"SR-ORTHOGONAL state mismatch: {by_candidate['SR-ORTHOGONAL']}")

    report = REPORT.read_text(encoding="utf-8")
    required = (
        "Sparse Repair Gate disposition: `SPARSE-SELECT: SR-PROFILE`.",
        "mandatory Sparse Repair Gate stop",
        "GROUP-MATCH-1",
        "ROOT-ALLOC-1",
        "SPARSE-AUTOMATON-1",
        "INSERT-1",
        "REPLACE-1",
        "REMOVE-1",
        "RELOCATE-1",
        "REHASH-1",
        "All five matrix rows are therefore `CONVERGES-B`.",
        "Candidate C v0 is unchanged.",
        "Work stops at this gate.",
        "This is structural paper accounting only.",
    )
    for phrase in required:
        if phrase not in report:
            fail(f"report omits {phrase!r}")
    for forbidden in (
        "Sparse Repair Gate disposition: `SPARSE-REVISE`.",
        "Sparse Repair Gate disposition: `SPARSE-NONE`.",
    ):
        if forbidden in report:
            fail(f"report contains conflicting disposition {forbidden!r}")

    print(
        "Candidate C sparse repair: 15 rows, 3 candidates, "
        "SPARSE-SELECT: SR-PROFILE, "
        f"states={{{', '.join(f'{key}: {dict(value)}' for key, value in by_candidate.items())}}}"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"Candidate C sparse-repair verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
