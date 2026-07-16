#!/usr/bin/env python3
"""Verify the bounded Candidate B multiproject source audit."""

from __future__ import annotations

import csv
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASE = (
    ROOT
    / "optimizer-language-research"
    / "implementation"
    / "minimal-systems-capability"
)
MATRIX = BASE / "CANDIDATE-B-MULTIPROJECT-AUDIT.tsv"
REPORT = BASE / "CANDIDATE-B-MULTIPROJECT-AUDIT.md"
PLAN = BASE / "CANDIDATE-B-ELEGANT-DESIGN-PLAN.md"

COLUMNS = (
    "operation_id",
    "project",
    "version",
    "revision",
    "source_anchors",
    "representation",
    "admission",
    "owner_traffic",
    "failure_cleanup",
    "provenance",
    "concurrency",
    "reference_events",
    "forbidden_delta",
    "unresolved",
    "falsifier",
)

EXPECTED = (
    ("H-LOOKUP", "Hashbrown", "0.17.1", "c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d"),
    ("H-INSERT", "Hashbrown", "0.17.1", "c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d"),
    ("H-REPLACE", "Hashbrown", "0.17.1", "c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d"),
    ("H-REMOVE", "Hashbrown", "0.17.1", "c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d"),
    ("H-REHASH", "Hashbrown", "0.17.1", "c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d"),
    ("M-ALLOC", "mimalloc", "3.3.2", "source=30b2d9d89099bee08e9f67a1ffb3e12e7ba45227"),
    ("M-LOCAL-FREE", "mimalloc", "3.3.2", "source=30b2d9d89099bee08e9f67a1ffb3e12e7ba45227"),
    ("M-REMOTE-FREE", "mimalloc", "3.3.2", "source=30b2d9d89099bee08e9f67a1ffb3e12e7ba45227"),
    ("S-INSERT-SPLIT", "SQLite", "3.53.3", "fossil=d4c0e51e4aeb96955b99185ab9cde75c339e2c29c3f3f12428d364a10d782c62"),
    ("S-DELETE-BALANCE", "SQLite", "3.53.3", "fossil=d4c0e51e4aeb96955b99185ab9cde75c339e2c29c3f3f12428d364a10d782c62"),
    ("S-ROLLBACK", "SQLite", "3.53.3", "fossil=d4c0e51e4aeb96955b99185ab9cde75c339e2c29c3f3f12428d364a10d782c62"),
    ("X-PROTECTED-LOAD", "Crossbeam Epoch", "0.9.18", "9c3182abebb36bdc9446d75d4644190fef70fa01"),
    ("X-RETIRE", "Crossbeam Epoch", "0.9.18", "9c3182abebb36bdc9446d75d4644190fef70fa01"),
    ("X-COLLECT", "Crossbeam Epoch", "0.9.18", "9c3182abebb36bdc9446d75d4644190fef70fa01"),
)


def fail(message: str) -> None:
    raise ValueError(message)


def require_phrases(path: Path, phrases: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for phrase in phrases:
        if phrase not in text:
            fail(f"{path.relative_to(ROOT)} omits {phrase!r}")


def main() -> int:
    with MATRIX.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != COLUMNS:
            fail("source-audit columns differ from the frozen schema")
        rows = list(reader)

    if len(rows) != len(EXPECTED):
        fail(f"expected {len(EXPECTED)} operation rows, found {len(rows)}")

    for index, (row, expected) in enumerate(zip(rows, EXPECTED), start=1):
        operation, project, version, revision_fragment = expected
        for column in COLUMNS:
            value = row.get(column)
            if value is None or not value.strip():
                fail(f"row {index} {operation} has empty {column}")
        if row["operation_id"] != operation:
            fail(f"row {index} expected {operation}, found {row['operation_id']}")
        if row["project"] != project or row["version"] != version:
            fail(f"{operation} project or version differs from the frozen input")
        if revision_fragment not in row["revision"]:
            fail(f"{operation} omits frozen revision {revision_fragment}")
        if ":" not in row["source_anchors"]:
            fail(f"{operation} has no line-qualified source anchor")

    mimalloc_revisions = {
        row["revision"] for row in rows if row["project"] == "mimalloc"
    }
    if mimalloc_revisions != {
        "tag-object=5687270e7fbb15d494a46b0d048f978bad973e4f;"
        "source=30b2d9d89099bee08e9f67a1ffb3e12e7ba45227"
    }:
        fail("mimalloc tag object and dereferenced source identity are not exact")

    require_phrases(
        REPORT,
        (
            "Phase 1 disposition: `SOURCE-AUDIT-COMPLETE`.",
            "The fourteen concrete operations",
            "Hashbrown: metadata is evidence, not ownership",
            "mimalloc: a place can change layout without acquiring a tag",
            "SQLite: safe failure does not always mean local rollback",
            "Crossbeam: access, retirement, and reclamation are three relations",
            "Project-independent demands",
            "Four projects and fourteen operations are a bounded falsification set",
            "does not yet support `B-SELECT`, `B-REVISE`, or `B-NONE`",
        ),
    )
    report_text = REPORT.read_text(encoding="utf-8")
    for operation, _, _, _ in EXPECTED:
        if f"`{operation}`" not in report_text:
            fail(f"source-audit report omits operation {operation}")

    require_phrases(
        PLAN,
        (
            "Audit exactly fourteen operations.",
            "30b2d9d89099bee08e9f67a1ffb3e12e7ba45227",
            "Produce exactly 42 candidate-operation rows",
        ),
    )

    print("Candidate B multiproject source audit: 14/14 operations verified")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"Candidate B source-audit verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
