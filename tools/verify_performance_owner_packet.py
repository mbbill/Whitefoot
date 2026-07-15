#!/usr/bin/env python3
"""Verify hostile review, validation requests, and owner decision packet."""

from __future__ import annotations

import csv
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "optimizer-language-research/implementation/minimal-systems-capability"
REVIEW = BASE / "PERFORMANCE-FIRST-HOSTILE-REVIEW.md"
REQUESTS = BASE / "PERFORMANCE-FIRST-VALIDATION-REQUESTS.tsv"
PACKET = BASE / "PERFORMANCE-FIRST-OWNER-DECISION-PACKET.md"

REQUEST_FIELDS = [
    "request_id",
    "class",
    "objective",
    "scope_and_artifacts",
    "prerequisites",
    "pass_condition",
    "fail_condition",
    "explicit_exclusions",
    "owner_options",
]

REQUESTS_EXPECTED = [
    ("VR-0", "semantic_definition"),
    ("VR-1", "safety_model"),
    ("VR-2", "structural_cost"),
    ("VR-3", "machine_event_model"),
    ("VR-4", "performance_measurement"),
    ("VR-5", "ai_stability"),
]


def fail(message: str) -> None:
    raise ValueError(message)


def main() -> int:
    review = REVIEW.read_text(encoding="utf-8")
    attacks = re.findall(r"^\| (HR-\d{2}) \|", review, flags=re.MULTILINE)
    expected_attacks = [f"HR-{i:02d}" for i in range(1, 23)]
    if attacks != expected_attacks:
        fail(f"hostile attack inventory mismatch: {attacks}")
    for phrase in (
        "no candidate is selectable as a language design",
        "There is no winner and no capability set should be selected.",
        "Candidate B — repair-and-validation priority; do not select",
        "Exact D-2, P-1, concurrency, external, target, and final-image obligations remain fail-closed.",
    ):
        if phrase not in review:
            fail(f"hostile review omits verdict or safeguard {phrase!r}")

    with REQUESTS.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        rows = list(reader)
        if list(reader.fieldnames or []) != REQUEST_FIELDS:
            fail("unexpected validation-request schema")
    if [(row["request_id"], row["class"]) for row in rows] != REQUESTS_EXPECTED:
        fail("validation requests must match the closed ordered inventory")
    for row in rows:
        request_id = row["request_id"]
        for field in REQUEST_FIELDS:
            if not row[field].strip():
                fail(f"{request_id}: empty {field}")
        if "No " not in row["explicit_exclusions"]:
            fail(f"{request_id}: exclusions must be explicit")
        if "Authorize none" not in row["owner_options"]:
            fail(f"{request_id}: owner must have a no-authorization option")
        if len(row["pass_condition"].split()) < 10 or len(row["fail_condition"].split()) < 10:
            fail(f"{request_id}: pass/fail condition is too short")

    packet = PACKET.read_text(encoding="utf-8")
    for phrase in (
        "There is no evidence-backed winner.",
        "recommended research hypothesis",
        "Decision 1 — performance gap definition",
        "Decision 2 — candidate retention",
        "Decision 3 — current conclusion",
        "Decision 4 — validation authorization",
        "Decision 5 — production progression",
        "Recommended: authorize VR-0 only.",
        "Until then, “no winner” is the accurate result.",
        "This packet authorizes no syntax or keyword",
    ):
        if phrase not in packet:
            fail(f"owner packet omits decision or safeguard {phrase!r}")
    for request_id, _ in REQUESTS_EXPECTED:
        if request_id not in packet:
            fail(f"owner packet omits {request_id}")

    print("performance owner packet: 22 hostile attacks, 6 separate requests, 5 owner decisions")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"performance owner packet verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
