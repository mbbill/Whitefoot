#!/usr/bin/env python3
"""Verify the bounded Candidate C Hashbrown Stage 1 audit."""

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
MATRIX = BASE / "CANDIDATE-C-HASHBROWN-AUDIT.tsv"
REPORT = BASE / "CANDIDATE-C-HASHBROWN-AUDIT.md"

FIELDS = (
    "project",
    "revision",
    "source_identity",
    "operation",
    "observable_contract",
    "reference_representation",
    "live_state",
    "owner_traffic",
    "provenance",
    "failure_cleanup",
    "fact_behavior",
    "c_route",
    "c0_dependency",
    "reference_events",
    "forced_delta",
    "evidence_state",
    "falsifier",
    "evidence_reference",
)
OPERATIONS = {
    "lookup",
    "vacant insertion",
    "replacement",
    "removal",
    "rehash",
}
STATES = {
    "ROUTED",
    "COMPOSED",
    "FAMILY-GAP",
    "COMPOSITION-GAP",
    "C0-GAP",
    "TAXED",
    "OPTIMIZER-GAP",
    "UNKNOWN",
}
REVISION = "c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d"
SOURCE_HASHES = {
    "src/raw.rs": "0c8ad353ba95817e72b0a8fea48fa2599099ea3def374f254ab6402a9c468d22",
    "src/map.rs": "b79497ce537ffc5ed4f8f3399434b9216c01e7927fdc434fee190e9e9ce2abb0",
}


def fail(message: str) -> None:
    raise ValueError(message)


def main() -> int:
    with MATRIX.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != FIELDS:
            fail(f"matrix schema mismatch: {reader.fieldnames}")
        rows = list(reader)

    if not rows or len(rows) > 25:
        fail(f"expected 1..25 rows, found {len(rows)}")
    for index, row in enumerate(rows, 1):
        missing = [field for field in FIELDS if not (row.get(field) or "").strip()]
        if missing:
            fail(f"row {index} has empty fields: {missing}")
        if row["project"] != "hashbrown":
            fail(f"row {index} uses unexpected project {row['project']!r}")
        if row["revision"] != REVISION:
            fail(f"row {index} uses unexpected revision {row['revision']!r}")
        if row["operation"] not in OPERATIONS:
            fail(f"row {index} uses unexpected operation {row['operation']!r}")
        if row["evidence_state"] not in STATES:
            fail(f"row {index} uses unexpected state {row['evidence_state']!r}")
        identity = row["source_identity"]
        if "@" not in identity:
            fail(f"row {index} has malformed source identity")
        path, digest = identity.split("@", 1)
        if SOURCE_HASHES.get(path) != digest:
            fail(f"row {index} has unpinned or unexpected source identity {identity!r}")

    counts = Counter(row["operation"] for row in rows)
    if set(counts) != OPERATIONS:
        fail(f"operation coverage mismatch: {sorted(counts)}")
    if any(counts[operation] > 6 for operation in OPERATIONS):
        fail(f"an operation exceeds the bounded six-row allowance: {dict(counts)}")

    report = REPORT.read_text(encoding="utf-8")
    required = (
        "Gate 1 disposition: `C-REVISE`.",
        "mandatory Gate 1 stop",
        REVISION,
        "The matrix contains 18 outcome/transition rows",
        "Stage 2 is not authorized.",
        "This report does not apply any repair.",
        "No forced initialization or zeroing identified.",
        "no unrelated-shape metadata tax identified.",
        "There is no generated-code or measured-performance evidence.",
    )
    for phrase in required:
        if phrase not in report:
            fail(f"report omits {phrase!r}")
    for forbidden in (
        "Gate 1 disposition: `C-SURVIVES`.",
        "Gate 1 disposition: `C-FAILS`.",
    ):
        if forbidden in report:
            fail(f"report contains conflicting disposition {forbidden!r}")
    for path, digest in SOURCE_HASHES.items():
        if path not in report or digest not in report:
            fail(f"report omits source pin {path}@{digest}")

    state_counts = Counter(row["evidence_state"] for row in rows)
    print(
        "Candidate C Hashbrown audit: "
        f"{len(rows)} rows, 5 operations, C-REVISE, states={dict(state_counts)}"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"Candidate C Hashbrown audit verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
