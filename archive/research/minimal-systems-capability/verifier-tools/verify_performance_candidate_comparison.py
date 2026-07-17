#!/usr/bin/env python3
"""Verify the uniform candidate-comparison matrix."""

from __future__ import annotations

import csv
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "optimizer-language-research/implementation/minimal-systems-capability"
MATRIX = BASE / "PERFORMANCE-FIRST-CANDIDATE-COMPARISON.tsv"
REPORT = BASE / "PERFORMANCE-FIRST-CANDIDATE-COMPARISON.md"

FIELDS = [
    "dimension",
    "why_it_matters",
    "candidate_a_proof_indexed",
    "candidate_b_bounded_kernel",
    "candidate_c_family_specialized",
    "evidence_status",
    "decision_rule_or_falsifier",
]

DIMENSIONS = [
    "performance_gap_coverage",
    "native_representation_and_zero_cost",
    "weaker_shape_no_tax",
    "semantic_and_rule_count",
    "compositionality",
    "checker_difficulty",
    "compiler_and_backend_special_cases",
    "runtime_metadata",
    "failure_and_cleanup",
    "optimizer_facts_and_check_elision",
    "ai_writing_and_repair_stability",
    "specification_and_teaching_size",
    "no_standard_library_use",
    "cross_target_portability",
    "future_extension_control",
    "safety_and_hostile_review_burden",
    "compile_time_and_code_size",
]


def fail(message: str) -> None:
    raise ValueError(message)


def main() -> int:
    with MATRIX.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        rows = list(reader)
        if list(reader.fieldnames or []) != FIELDS:
            fail("unexpected comparison schema")

    if [row["dimension"] for row in rows] != DIMENSIONS:
        fail("comparison dimensions must match the closed ordered list")

    for row in rows:
        dimension = row["dimension"]
        for field in FIELDS:
            if not row[field].strip():
                fail(f"{dimension}: empty {field}")
        for candidate in FIELDS[2:5]:
            cell = row[candidate]
            for marker in ("Advantage:", "Liability:", "Trade-off:"):
                if marker not in cell:
                    fail(f"{dimension}/{candidate}: missing {marker}")
        if not any(
            marker in row["evidence_status"].lower()
            for marker in ("paper", "measured", "unmeasured", "untested", "unknown", "no candidate", "no ai", "no draft", "unenumerated", "witness", "ownership", "current")
        ):
            fail(f"{dimension}: evidence class is not explicit")
        if len(row["decision_rule_or_falsifier"].split()) < 8:
            fail(f"{dimension}: decision rule is too short")

    report = REPORT.read_text(encoding="utf-8")
    for phrase in (
        "There is no evidence-backed winner yet.",
        "Candidate A",
        "Candidate B",
        "Candidate C",
        "A versus B",
        "B versus C",
        "A versus C",
        "not a recommendation",
        "No candidate execution or benchmark is authorized.",
    ):
        if phrase not in report:
            fail(f"comparison report omits safeguard or section {phrase!r}")

    print(f"performance candidate comparison: {len(rows)} dimensions, A/B/C pros-cons complete")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"performance candidate comparison failed: {error}", file=sys.stderr)
        raise SystemExit(1)
