#!/usr/bin/env python3
"""Verify the bounded Candidate B architecture comparison and design gate."""

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
PLAN = BASE / "CANDIDATE-B-ELEGANT-DESIGN-PLAN.md"
SOURCE_REPORT = BASE / "CANDIDATE-B-MULTIPROJECT-AUDIT.md"
SOURCE_MATRIX = BASE / "CANDIDATE-B-MULTIPROJECT-AUDIT.tsv"
REPORT = BASE / "CANDIDATE-B-ELEGANT-DESIGN.md"
MATRIX = BASE / "CANDIDATE-B-ELEGANT-DESIGN-MATRIX.tsv"

FIELDS = (
    "candidate",
    "operation",
    "route_state",
    "capability_composition",
    "performance_blocker_removed",
    "owner_cleanup",
    "provenance",
    "concurrency",
    "runtime_state",
    "structural_delta",
    "extension_effect",
    "unresolved",
    "falsifier",
)

CANDIDATES = ("B-FORMS", "B-STRATA", "B-GRAPHS")
OPERATIONS = (
    "H-LOOKUP",
    "H-INSERT",
    "H-REPLACE",
    "H-REMOVE",
    "H-REHASH",
    "M-ALLOC",
    "M-LOCAL-FREE",
    "M-REMOTE-FREE",
    "S-INSERT-SPLIT",
    "S-DELETE-BALANCE",
    "S-ROLLBACK",
    "X-PROTECTED-LOAD",
    "X-RETIRE",
    "X-COLLECT",
)
STATES = {
    "CLOSED",
    "OPEN",
    "TAXED",
    "CONVERGES-C",
    "CONVERGES-A",
    "UNKNOWN",
}

EXPECTED_STATES = {
    "B-FORMS": {operation: "OPEN" for operation in OPERATIONS},
    "B-STRATA": {
        "H-LOOKUP": "CLOSED",
        "H-INSERT": "CLOSED",
        "H-REPLACE": "CLOSED",
        "H-REMOVE": "CLOSED",
        "H-REHASH": "CLOSED",
        "M-ALLOC": "CLOSED",
        "M-LOCAL-FREE": "OPEN",
        "M-REMOTE-FREE": "OPEN",
        "S-INSERT-SPLIT": "OPEN",
        "S-DELETE-BALANCE": "OPEN",
        "S-ROLLBACK": "OPEN",
        "X-PROTECTED-LOAD": "OPEN",
        "X-RETIRE": "OPEN",
        "X-COLLECT": "OPEN",
    },
    "B-GRAPHS": {
        "H-LOOKUP": "CLOSED",
        "H-INSERT": "CLOSED",
        "H-REPLACE": "CLOSED",
        "H-REMOVE": "CLOSED",
        "H-REHASH": "CLOSED",
        "M-ALLOC": "CLOSED",
        "M-LOCAL-FREE": "OPEN",
        "M-REMOTE-FREE": "OPEN",
        "S-INSERT-SPLIT": "OPEN",
        "S-DELETE-BALANCE": "OPEN",
        "S-ROLLBACK": "OPEN",
        "X-PROTECTED-LOAD": "OPEN",
        "X-RETIRE": "OPEN",
        "X-COLLECT": "OPEN",
    },
}


def fail(message: str) -> None:
    raise ValueError(message)


def require_phrases(path: Path, phrases: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for phrase in phrases:
        if phrase not in text:
            fail(f"{path.relative_to(ROOT)} omits {phrase!r}")


def main() -> int:
    with MATRIX.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != FIELDS:
            fail(f"design-matrix schema mismatch: {reader.fieldnames}")
        rows = list(reader)

    expected_order = [
        (candidate, operation)
        for candidate in CANDIDATES
        for operation in OPERATIONS
    ]
    actual_order: list[tuple[str, str]] = []
    seen: set[tuple[str, str]] = set()

    if len(rows) != len(expected_order):
        fail(f"expected exactly 42 design rows, found {len(rows)}")

    for index, row in enumerate(rows, start=2):
        missing = [field for field in FIELDS if not (row.get(field) or "").strip()]
        if missing:
            fail(f"matrix line {index} has empty fields: {missing}")

        candidate = row["candidate"]
        operation = row["operation"]
        state = row["route_state"]
        if candidate not in CANDIDATES:
            fail(f"matrix line {index} has unknown candidate {candidate!r}")
        if operation not in OPERATIONS:
            fail(f"matrix line {index} has unknown operation {operation!r}")
        if state not in STATES:
            fail(f"matrix line {index} has unknown route state {state!r}")

        pair = (candidate, operation)
        if pair in seen:
            fail(f"duplicate candidate-operation pair {pair}")
        seen.add(pair)
        actual_order.append(pair)

        expected_state = EXPECTED_STATES[candidate][operation]
        if state != expected_state:
            fail(
                f"{candidate}/{operation} expected {expected_state}, found {state}"
            )

        if state == "CLOSED":
            unresolved = row["unresolved"].lower()
            if "remain open" in unresolved or "remains open" in unresolved:
                fail(f"closed row {candidate}/{operation} claims an open definition")
        else:
            if len(row["unresolved"].split()) < 4:
                fail(f"non-closed row {candidate}/{operation} lacks an exact blocker")

    if actual_order != expected_order:
        fail("design rows do not follow the frozen candidate/operation order")

    counts = {
        candidate: Counter(
            row["route_state"] for row in rows if row["candidate"] == candidate
        )
        for candidate in CANDIDATES
    }
    expected_counts = {
        "B-FORMS": Counter({"OPEN": 14}),
        "B-STRATA": Counter({"CLOSED": 6, "OPEN": 8}),
        "B-GRAPHS": Counter({"CLOSED": 6, "OPEN": 8}),
    }
    if counts != expected_counts:
        fail(f"route-state counts differ: {counts}")

    # A B-REVISE gate is fail-closed only when no candidate closes every row,
    # while at least one route remains credible enough that B-NONE is not forced.
    for candidate in CANDIDATES:
        if counts[candidate].get("CLOSED", 0) == len(OPERATIONS):
            fail(f"{candidate} closes all rows, conflicting with B-REVISE")
    if sum(counter.get("CLOSED", 0) for counter in counts.values()) == 0:
        fail("no route is closed, so the report's reason for rejecting B-NONE is absent")

    require_phrases(
        REPORT,
        (
            "Candidate B Design Gate disposition: `B-REVISE`.",
            "This is not a Hashbrown-only result.",
            "Why four projects are necessary",
            "Alternative 1: B-FORMS",
            "Alternative 2: B-STRATA",
            "Alternative 3: B-GRAPHS",
            "BS-1 PHYSICAL-PLACE",
            "BS-2 STRUCTURAL-LIVE",
            "BS-3 OWNER-TRANSITION",
            "BS-4 FOCUS-PROGRESS",
            "BS-5 ROOT-FOOTPRINT",
            "BS-6 EXECUTABLE-DISPOSITION",
            "BS-7 INVALIDATABLE-FACT",
            "BS-8 CONCURRENT-CUSTODY",
            "Exact 42-route result",
            "`B-FORMS` has fourteen open rows.",
            "`B-STRATA` has six closed and eight open rows.",
            "`B-GRAPHS` has six closed and eight open rows.",
            "refinement validation cannot create liveness or ownership.",
            "cold collect/extend/retry/null",
            "Structural-cost comparison",
            "Broader design comparison",
            "Pros and cons",
            "Hostile review of B-STRATA",
            "The exact disposition is `B-REVISE`.",
            "Retain `B-STRATA` only as the best-defined B revision hypothesis.",
            "Work stops at\nthis gate.",
            "No safety model, additional audit, language or specification change",
        ),
    )

    report_text = REPORT.read_text(encoding="utf-8")
    for operation in OPERATIONS:
        if f"\t{operation}\t" not in MATRIX.read_text(encoding="utf-8"):
            fail(f"design matrix omits {operation}")
    for conflicting in (
        "Candidate B Design Gate disposition: `B-SELECT`.",
        "Candidate B Design Gate disposition: `B-NONE`.",
        "The exact disposition is `B-SELECT`.",
        "The exact disposition is `B-NONE`.",
    ):
        if conflicting in report_text:
            fail(f"report contains conflicting gate {conflicting!r}")

    require_phrases(
        PLAN,
        (
            "Audit exactly fourteen operations.",
            "Derive exactly three alternatives before routing any operation",
            "Produce exactly 42 candidate-operation rows",
            "`B-REVISE`: B remains plausible but every alternative has at least one open",
        ),
    )
    require_phrases(
        SOURCE_REPORT,
        (
            "Phase 1 disposition: `SOURCE-AUDIT-COMPLETE`.",
            "Hashbrown: metadata is evidence, not ownership",
            "mimalloc: a place can change layout without acquiring a tag",
            "SQLite: safe failure does not always mean local rollback",
            "Crossbeam: access, retirement, and reclamation are three relations",
        ),
    )
    with SOURCE_MATRIX.open(encoding="utf-8", newline="") as handle:
        source_rows = list(csv.DictReader(handle, delimiter="\t"))
    if len(source_rows) != len(OPERATIONS):
        fail(f"source matrix must retain 14 rows, found {len(source_rows)}")
    source_operations = tuple(row.get("operation_id", "") for row in source_rows)
    if source_operations != OPERATIONS:
        fail("design operations differ from the frozen source-audit order")

    summary = "; ".join(
        f"{candidate}={dict(counts[candidate])}" for candidate in CANDIDATES
    )
    print(
        "Candidate B elegant design: 42/42 routes verified, "
        f"gate=B-REVISE, {summary}"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"Candidate B design verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
