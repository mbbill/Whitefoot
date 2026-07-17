#!/usr/bin/env python3
"""Verify candidate-set inventory and gap-crosswalk bookkeeping."""

from __future__ import annotations

import csv
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "optimizer-language-research/implementation/minimal-systems-capability"
CANDIDATES = BASE / "PERFORMANCE-FIRST-CANDIDATE-SETS.md"
GAPS = BASE / "PERFORMANCE-EXPRESSIVENESS-GAP-LEDGER.tsv"


def fail(message: str) -> None:
    raise ValueError(message)


def main() -> int:
    text = CANDIDATES.read_text(encoding="utf-8")
    expected_items = {
        *(f"C0-{i}" for i in range(1, 12)),
        *(f"A-{i}" for i in range(1, 11)),
        *(f"B-{i}" for i in range(1, 12)),
        *(f"C-{i}" for i in range(1, 13)),
    }

    definitions: dict[str, int] = {}
    for line in text.splitlines():
        match = re.match(r"\| (C0|A|B|C)-(\d+) \|", line)
        if match:
            item = f"{match.group(1)}-{match.group(2)}"
            definitions[item] = definitions.get(item, 0) + 1
            if (
                not any(marker in line for marker in ("Delete", "Remove", "Without"))
                and not item.startswith("C0-")
            ):
                fail(f"{item}: candidate item omits a deletion consequence")

    missing_items = sorted(expected_items - definitions.keys())
    extra_items = sorted(definitions.keys() - expected_items)
    duplicates = sorted(item for item, count in definitions.items() if count != 1)
    if missing_items or extra_items or duplicates:
        fail(
            f"item inventory mismatch: missing={missing_items}, extra={extra_items}, "
            f"duplicates={duplicates}"
        )

    with GAPS.open(newline="", encoding="utf-8") as handle:
        gap_ids = [row["gap_id"] for row in csv.DictReader(handle, delimiter="\t")]

    crosswalk: dict[str, list[str]] = {}
    for line in text.splitlines():
        if not line.startswith("| PF-G"):
            continue
        columns = [column.strip() for column in line.strip().strip("|").split("|")]
        if len(columns) != 4:
            fail(f"malformed crosswalk row: {line}")
        gap_id = columns[0].split()[0]
        if gap_id in crosswalk:
            fail(f"duplicate crosswalk row {gap_id}")
        crosswalk[gap_id] = columns[1:]
        for candidate_index, route in enumerate(columns[1:], start=1):
            if not route:
                fail(f"{gap_id}: empty candidate {candidate_index} route")
            refs = set(re.findall(r"(?:C0|A|B|C)-\d+", route))
            if not refs:
                fail(f"{gap_id}: candidate {candidate_index} route has no item reference")
            unknown = sorted(refs - expected_items)
            if unknown:
                fail(f"{gap_id}: candidate {candidate_index} uses unknown items {unknown}")

    if list(crosswalk) != gap_ids:
        fail(
            "crosswalk must cover the gap ledger in exact order: "
            f"expected={gap_ids}, actual={list(crosswalk)}"
        )

    for phrase in (
        "no candidate is selected",
        "paper route",
        "deferred table",
        "do not enumerate platform rows",
        "None is complete as a production design",
    ):
        if phrase not in text:
            fail(f"missing status safeguard: {phrase!r}")

    print(
        "performance candidate sets: 3 candidates, "
        f"{len(expected_items)} capability items, {len(crosswalk)}/15 gap routes each"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"performance candidate verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
