#!/usr/bin/env python3
"""Verify the finite performance-expressiveness gap ledger."""

from __future__ import annotations

import csv
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "optimizer-language-research/implementation/minimal-systems-capability"
LEDGER = BASE / "PERFORMANCE-EXPRESSIVENESS-GAP-LEDGER.tsv"
CAPABILITIES = BASE / "CAPABILITY-OBLIGATION-REGISTRY.tsv"
DOMAINS = BASE / "SYSTEMS-DOMAIN-LEDGER.md"
WITNESSES = BASE / "WITNESS-REGISTRY.md"
SPEC = ROOT / "spec/kernel-spec-v0.6.md"
REPORT = BASE / "PERFORMANCE-EXPRESSIVENESS-GAP-REPORT.md"

FIELDS = [
    "gap_id",
    "gap_name",
    "target_contract",
    "current_expression_limit",
    "forced_machine_cost_or_shape",
    "correctness_cleanup_and_fact_obligations",
    "protected_cost_budget",
    "evidence_refs",
    "evidence_strength",
    "falsifier",
    "frontier_capability_refs",
    "scope_status",
]

STRENGTHS = {
    "mechanism_and_heldout",
    "mechanism_and_structural_witnesses",
    "measured_plus_mechanism",
    "mechanism_and_domain_boundary",
    "deferred_domain_boundary",
    "domain_boundary_only",
    "research_statement_and_domain_boundary",
    "measured_narrow_channels_plus_research_statement",
}

STATUSES = {"candidate_required", "candidate_required_deferred_evidence"}
ESTABLISHED_OR_CONTROL = {"established", "established_narrow", "protected"}
SPEC_RULES = {"TYPE-2", "OWN-1", "OWN-7", "OP-9"}
MEASUREMENTS = {
    "SCOPED-ALIAS-2026-07-09": ROOT / "experiments/scoped-alias-channel/RESULTS.md",
    "CHECKED-LAW-2026-07-09": ROOT / "experiments/checked-law-channel/RESULTS.md",
    "BOUNDS-PROOF-2026-07-10": ROOT / "experiments/port-study/base64/RESULTS.md",
    "INTERPRETER-DISPATCH-CARDED-2026-07-10": ROOT / "mcts_mem/xlang/fact-channels.md",
}


def fail(message: str) -> None:
    raise ValueError(message)


def read_tsv(path: Path) -> tuple[list[str], list[dict[str, str]]]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        rows = list(reader)
        return list(reader.fieldnames or []), rows


def main() -> int:
    header, rows = read_tsv(LEDGER)
    if header != FIELDS:
        fail(f"unexpected ledger schema: {header}")
    if [row["gap_id"] for row in rows] != [f"PF-G{i:02d}" for i in range(1, 16)]:
        fail("gap IDs must be the closed ordered range PF-G01 through PF-G15")

    cap_header, cap_rows = read_tsv(CAPABILITIES)
    required_cap_fields = {"capability_id", "current_xlang_status"}
    if not required_cap_fields.issubset(cap_header):
        fail("capability registry schema is missing required fields")
    capability_status = {
        row["capability_id"]: row["current_xlang_status"] for row in cap_rows
    }

    domain_text = DOMAINS.read_text(encoding="utf-8")
    witness_text = WITNESSES.read_text(encoding="utf-8")
    spec_text = SPEC.read_text(encoding="utf-8")
    report_text = REPORT.read_text(encoding="utf-8")
    routed: set[str] = set()

    for row in rows:
        gap_id = row["gap_id"]
        for field in FIELDS:
            if not row[field].strip():
                fail(f"{gap_id}: empty {field}")
        if row["evidence_strength"] not in STRENGTHS:
            fail(f"{gap_id}: unknown evidence strength")
        if row["scope_status"] not in STATUSES:
            fail(f"{gap_id}: unknown scope status")
        if len(row["forced_machine_cost_or_shape"].split()) < 5:
            fail(f"{gap_id}: forced-cost account is too short")
        if len(row["protected_cost_budget"].split()) < 5:
            fail(f"{gap_id}: protected budget is too short")
        if not row["falsifier"].startswith(("A ", "Current ", "Existing ")):
            fail(f"{gap_id}: falsifier must state an observable disproof route")

        refs = row["evidence_refs"].split(";")
        if len(refs) < 2:
            fail(f"{gap_id}: at least two evidence references are required")
        for ref in refs:
            kind, separator, value = ref.partition(":")
            if not separator or not value:
                fail(f"{gap_id}: malformed evidence reference {ref!r}")
            if kind == "CAP":
                if value not in capability_status:
                    fail(f"{gap_id}: unknown capability {value}")
            elif kind == "W":
                if not re.search(rf"\b{re.escape(value)}\b", witness_text):
                    fail(f"{gap_id}: unknown witness {value}")
            elif kind == "DOMAIN":
                if not re.fullmatch(r"D(?:0[1-9]|1[0-9]|2[0-6])", value):
                    fail(f"{gap_id}: invalid domain ID {value}")
                if not re.search(rf"\b{re.escape(value)}\b", domain_text):
                    fail(f"{gap_id}: absent domain ID {value}")
            elif kind == "SPEC":
                if value not in SPEC_RULES or f"[{value}]" not in spec_text:
                    fail(f"{gap_id}: unknown spec anchor {value}")
            elif kind == "MEASURE":
                path = MEASUREMENTS.get(value)
                if path is None or not path.is_file():
                    fail(f"{gap_id}: unknown measurement or statement {value}")
            else:
                fail(f"{gap_id}: unknown evidence kind {kind}")

        frontier = row["frontier_capability_refs"]
        if frontier != "-":
            for capability in frontier.split(";"):
                if capability not in capability_status:
                    fail(f"{gap_id}: unknown frontier capability {capability}")
                routed.add(capability)

    expected_frontier = {
        capability
        for capability, status in capability_status.items()
        if status not in ESTABLISHED_OR_CONTROL
    }
    missing = sorted(expected_frontier - routed)
    if missing:
        fail(f"unrouted non-established capabilities: {', '.join(missing)}")

    extra = sorted(routed - expected_frontier)
    if extra:
        fail(f"established controls incorrectly treated as frontier: {', '.join(extra)}")

    for control in ("NT-FIXED", "NT-P2"):
        if control not in report_text:
            fail(f"report omits protected control {control}")

    print(
        "performance gap ledger: "
        f"{len(rows)} gaps, {len(routed)}/{len(expected_frontier)} frontier capabilities routed, "
        "domain references validated, 2 protected controls present"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ValueError as error:
        print(f"performance gap ledger verification failed: {error}", file=sys.stderr)
        raise SystemExit(1)
