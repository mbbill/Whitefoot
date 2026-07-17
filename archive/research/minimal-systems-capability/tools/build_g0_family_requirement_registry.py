#!/usr/bin/env python3
"""Build the fail-closed G0 B/M/W/H/O family-requirement registry."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RESEARCH = ROOT.parent / "general-purpose-data-structure-capability-RESEARCH.md"
WITNESS = ROOT / "WITNESS-REGISTRY.md"
VOCABULARY = ROOT / "G0-FAMILY-GATE-VOCABULARY.md"
OUTPUT = ROOT / "G0-FAMILY-REQUIREMENT-REGISTRY.tsv"

HEADER = [
    "obligation_id",
    "record_kind",
    "registry_subject_id",
    "workload_or_operation",
    "role",
    "primary_canary_id",
    "required_crosscut_canary_ids",
    "c_fail_applicability",
    "crosscut_canary_contract",
    "crosscut_canary_source_sha256",
    "closure_owner_or_gate_stage",
    "required_predecessor_family_ids",
    "implicated_family_ids",
    "implicated_rebind_policy",
    "required_efficient_property",
    "g0_status",
    "current_workaround_or_blocker",
    "closure_policy",
    "owner_lock_allowed_dispositions",
    "implicated_rebind_allowed_dispositions",
    "canary_contract",
    "linked_registry_ids",
    "source_identity",
    "source_row_sha256",
    "canary_source_sha256",
    "vocabulary_authority_sha256",
]

ROLE_POLICY = {
    "B": "PROTECTED_NO_REGRESSION_IN_EVERY_FAMILY_LOCK",
    "M": "BLOCKS_ASSIGNED_FAMILY_AND_COMPLETE_FLOOR",
    "W": "BLOCKS_ASSIGNED_FAMILY_AND_COMPLETE_FLOOR",
    "H": "BLOCKS_ASSIGNED_FAMILY_AND_COMPLETE_FLOOR",
    "O": "BLOCKS_ONLY_IF_PROMOTED_OR_REQUIRED_BY_A_MANDATORY_CONTRACT",
}


def owner_lock_allowed_dispositions(role: str) -> str:
    if role == "B":
        return "PROTECTED_CONTROL"
    dispositions = ["REQUIRED_IN_LOCK", "EXCLUDED_BLOCKS_FAMILY_AND_FLOOR"]
    if role == "O":
        dispositions.append("OPTIONAL_NOT_PROMOTED")
    return ",".join(dispositions)


def implicated_rebind_allowed_dispositions(role: str, rebind_policy: str) -> str:
    if role == "B":
        return "PROTECTED_CONTROL"
    if rebind_policy == "EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY":
        return (
            "PREDECESSOR_REUSE_AND_LOCAL_REBIND_PROVED,"
            "EXCLUDED_BLOCKS_FAMILY_AND_FLOOR"
        )
    return "NOT_APPLICABLE_REOPENING_ONLY"

# This is the bounded G0 dependency assignment frozen by Sections 6, 10, and 11
# of the source report. A changed Section 4 workload fails closed until this map
# is deliberately reviewed and extended.
FAMILY_BY_WORKLOAD = {
    "Fixed buffer of Copy scalars": "ALL-FAMILY-LOCKS",
    "Fixed AoS record buffer": "F-DENSE",
    "Unknown-length append": "F-DENSE",
    "Append affine value": "F-DENSE",
    "Grow/shrink contiguous sequence": "F-DENSE",
    "Pop affine value": "F-DENSE",
    "Ordered insert/remove and unordered `swap_remove`": "F-DENSE",
    "Swap two dynamic elements": "F-DENSE",
    "Clear/truncate": "F-DENSE",
    "Deep clone and bulk move-append": "F-DENSE",
    "Stable retain and eager drain/splice": "F-DENSE",
    "Lazy drain cursor": "F-DENSE",
    "Generic unstable and stable sort": "F-DENSE",
    "Stack adapter": "F-DENSE",
    "FIFO queue/deque": "F-DEQUE",
    "Priority queue": "F-HEAP",
    "Hash map/set": "F-SPARSE",
    "Ordered map/set": "F-ORDERED",
    "Append-only AST/DAG/graph": "ALL-FAMILY-LOCKS",
    "Recyclable stable pool": "F-IDENTITY",
    "Frozen graph": "F-DENSE,F-IDENTITY",
    "Dynamic graph with deletion": "F-DENSE,F-IDENTITY",
    "Singly owned recursive list": "F-RECURSIVE",
    "Doubly linked/cyclic list": "F-DENSE,F-IDENTITY",
    "Homogeneous bump arena/slab": "F-DENSE,F-ARENA",
    "Inline-to-heap small sequence": "F-DENSE",
    "Unseen storage-bearing structure": "F-SPARSE",
    "LRU cache": "F-SPARSE,F-IDENTITY",
    "Indexed priority queue": "F-DENSE,F-SPARSE,F-HEAP",
    "Bytes and UTF-8 text builder": "F-DENSE,F-TEXT",
    "Borrowed, uniq, and owning iteration": (
        "F-DENSE,F-DEQUE,F-SPARSE,F-ORDERED,F-HEAP,F-IDENTITY,"
        "F-RECURSIVE,F-ARENA,F-TEXT"
    ),
}

OWNER_BY_WORKLOAD = {
    "Fixed buffer of Copy scalars": "ALL-FAMILY-LOCKS",
    "Fixed AoS record buffer": "F-DENSE",
    "Unknown-length append": "F-DENSE",
    "Append affine value": "F-DENSE",
    "Grow/shrink contiguous sequence": "F-DENSE",
    "Pop affine value": "F-DENSE",
    "Ordered insert/remove and unordered `swap_remove`": "F-DENSE",
    "Swap two dynamic elements": "F-DENSE",
    "Clear/truncate": "F-DENSE",
    "Deep clone and bulk move-append": "F-DENSE",
    "Stable retain and eager drain/splice": "F-DENSE",
    "Lazy drain cursor": "F-DENSE",
    "Generic unstable and stable sort": "F-DENSE",
    "Stack adapter": "F-DENSE",
    "FIFO queue/deque": "F-DEQUE",
    "Priority queue": "F-HEAP",
    "Hash map/set": "F-SPARSE",
    "Ordered map/set": "F-ORDERED",
    "Append-only AST/DAG/graph": "ALL-FAMILY-LOCKS",
    "Recyclable stable pool": "F-IDENTITY",
    "Frozen graph": "F-IDENTITY",
    "Dynamic graph with deletion": "F-IDENTITY",
    "Singly owned recursive list": "F-RECURSIVE",
    "Doubly linked/cyclic list": "F-IDENTITY",
    "Homogeneous bump arena/slab": "F-ARENA",
    "Inline-to-heap small sequence": "F-DENSE",
    "Unseen storage-bearing structure": "F-SPARSE",
    "LRU cache": "F-IDENTITY",
    "Indexed priority queue": "F-HEAP",
    "Bytes and UTF-8 text builder": "F-TEXT",
    "Borrowed, uniq, and owning iteration": "F-DENSE",
}

PREDECESSORS_BY_WORKLOAD = {
    **{workload: "NONE" for workload in FAMILY_BY_WORKLOAD},
    "Frozen graph": "F-DENSE",
    "Dynamic graph with deletion": "F-DENSE",
    "Doubly linked/cyclic list": "F-DENSE",
    "Homogeneous bump arena/slab": "F-DENSE",
    "Ordered map/set": "F-DENSE,F-IDENTITY,F-RECURSIVE",
    "Priority queue": "F-DENSE",
    "Recyclable stable pool": "F-SPARSE",
    "Unseen storage-bearing structure": "F-DENSE",
    "LRU cache": "F-SPARSE",
    "Indexed priority queue": "F-DENSE,F-SPARSE",
    "Bytes and UTF-8 text builder": "F-DENSE",
}

LINKED_REGISTRY_IDS = {
    "Fixed buffer of Copy scalars": "B-FIX",
    "Fixed AoS record buffer": "H-FLATSET",
    "Unknown-length append": "H-FLATSET",
    "Append affine value": "H-FLATSET",
    "Grow/shrink contiguous sequence": "H-FLATSET",
    "Pop affine value": "H-FLATSET",
    "Ordered insert/remove and unordered `swap_remove`": "H-FLATSET",
    "Clear/truncate": "H-FLATSET",
    "Stable retain and eager drain/splice": "H-FLATSET",
    "Lazy drain cursor": "O-LAZY-DRAIN",
    "Priority queue": "H-IPQ",
    "Hash map/set": "H-STORE",
    "Append-only AST/DAG/graph": "B-P2",
    "Recyclable stable pool": "W-POOL",
    "Frozen graph": "W-GRAPH",
    "Dynamic graph with deletion": "W-GRAPH",
    "Singly owned recursive list": "W-RECUR",
    "Doubly linked/cyclic list": "W-GRAPH",
    "Homogeneous bump arena/slab": "W-ARENA",
    "Inline-to-heap small sequence": "W-SMALL",
    "Unseen storage-bearing structure": "H-STORE",
    "LRU cache": "H-LRU",
    "Indexed priority queue": "H-IPQ",
    "Borrowed, uniq, and owning iteration": "B-P2,W-PIPE",
}

SUPPLEMENTAL_ROLE = {
    "B-FIX": "B",
    "B-P2": "B",
    "W-POOL": "W",
    "W-ARENA": "W",
    "W-SMALL": "W",
    "W-RECUR": "W",
    "W-GRAPH": "W",
    "W-ECS": "W",
    "W-GAP": "W",
    "W-PIPE": "W",
    "H-FLATSET": "H",
    "H-STORE": "H",
    "H-LRU": "H",
    "H-IPQ": "H",
    "O-SLAB": "O",
    "O-ROPE-UNIQUE": "O",
    "O-INTRUSIVE": "O",
    "O-LAZY-DRAIN": "O",
}

SUPPLEMENTAL_FAMILIES = {
    "B-FIX": "ALL-FAMILY-LOCKS",
    "B-P2": "ALL-FAMILY-LOCKS",
    "W-POOL": "F-IDENTITY",
    "W-ARENA": "F-DENSE,F-ARENA",
    "W-SMALL": "F-DENSE",
    "W-RECUR": "F-RECURSIVE",
    "W-GRAPH": "F-DENSE,F-IDENTITY",
    "W-ECS": "F-DENSE,F-IDENTITY",
    "W-GAP": "F-DENSE",
    "W-PIPE": (
        "F-DENSE,F-DEQUE,F-SPARSE,F-ORDERED,F-HEAP,F-IDENTITY,F-ITERATION,"
        "F-RECURSIVE,F-ARENA,F-TEXT"
    ),
    "H-FLATSET": "F-DENSE",
    "H-STORE": "F-SPARSE",
    "H-LRU": "F-SPARSE,F-IDENTITY",
    "H-IPQ": "F-DENSE,F-SPARSE,F-HEAP",
    "O-SLAB": "F-IDENTITY",
    "O-ROPE-UNIQUE": "F-DENSE,F-ORDERED",
    "O-INTRUSIVE": "F-IDENTITY,F-PIN-ADDRESS",
    "O-LAZY-DRAIN": "F-DENSE",
}

SUPPLEMENTAL_OWNER = {
    "B-FIX": "ALL-FAMILY-LOCKS",
    "B-P2": "ALL-FAMILY-LOCKS",
    "W-POOL": "F-IDENTITY",
    "W-ARENA": "F-ARENA",
    "W-SMALL": "F-DENSE",
    "W-RECUR": "F-RECURSIVE",
    "W-GRAPH": "F-IDENTITY",
    "W-ECS": "F-IDENTITY",
    "W-GAP": "F-DENSE",
    "W-PIPE": "F-ITERATION",
    "H-FLATSET": "F-DENSE",
    "H-STORE": "F-SPARSE",
    "H-LRU": "F-IDENTITY",
    "H-IPQ": "F-HEAP",
    "O-SLAB": "F-IDENTITY",
    "O-ROPE-UNIQUE": "GATE-ROPE-POST-DENSE-ORDERED",
    "O-INTRUSIVE": "F-PIN-ADDRESS",
    "O-LAZY-DRAIN": "F-DENSE",
}

SUPPLEMENTAL_PREDECESSORS = {
    **{subject: "NONE" for subject in SUPPLEMENTAL_ROLE},
    "W-ARENA": "F-DENSE",
    "W-POOL": "F-SPARSE",
    "W-GRAPH": "F-DENSE",
    "W-ECS": "F-DENSE",
    "W-PIPE": "F-DENSE",
    "H-STORE": "F-DENSE",
    "H-LRU": "F-SPARSE",
    "H-IPQ": "F-DENSE,F-SPARSE",
    "O-ROPE-UNIQUE": "F-DENSE,F-ORDERED",
}

OWNING_CANARY_IDS = {
    "C-FIX",
    "C-SEQ",
    "C-COMPACT",
    "C-LAZY-DRAIN",
    "C-SORT",
    "C-DEQUE",
    "C-HASH",
    "C-ORDER",
    "C-HEAP",
    "C-P2",
    "C-POOL",
    "C-GRAPH",
    "C-RECUR",
    "C-ARENA",
    "C-SMALL",
    "C-TEXT",
    "C-ITER",
    *SUPPLEMENTAL_ROLE.keys(),
}


def digest(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def parse_assignment_authority() -> tuple[dict[str, dict[str, str]], str]:
    if not VOCABULARY.is_file():
        raise SystemExit(f"missing assignment authority: {VOCABULARY.name}")
    text = VOCABULARY.read_text(encoding="utf-8")
    start_marker = "<!-- G0_REQUIREMENT_ASSIGNMENTS_BEGIN -->"
    end_marker = "<!-- G0_REQUIREMENT_ASSIGNMENTS_END -->"
    start = text.find(start_marker)
    end = text.find(end_marker, start + len(start_marker))
    if start < 0 or end < 0:
        raise SystemExit("family/gate assignment-authority markers are missing")
    section = text[start + len(start_marker) : end].strip().splitlines()
    expected_header = (
        "| Requirement subject | Owner or gate | Required predecessors | "
        "Implicated families | Rebind policy |"
    )
    if len(section) < 2 or section[0] != expected_header or not re.fullmatch(
        r"\|(?:---\|){5}", section[1].replace(" ", "")
    ):
        raise SystemExit("family/gate assignment-authority table header changed")

    records: dict[str, dict[str, str]] = {}
    allowed_rebind = {
        "EVERY_CANDIDATE_EXECUTES",
        "OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN",
        "OPTIONAL_PROMOTION_ONLY_IMPLICATED_REOPEN",
        "EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY",
    }
    for line in section[2:]:
        if not line.startswith("|"):
            raise SystemExit("assignment-authority table contains non-row content")
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != 5:
            raise SystemExit(f"malformed assignment-authority row: {line}")
        subject, owner, predecessors, implicated, rebind = cells
        if not subject or subject in records:
            raise SystemExit(f"duplicate or empty assignment subject: {subject!r}")
        if not (
            owner == "ALL-FAMILY-LOCKS"
            or re.fullmatch(r"F-[A-Z0-9-]+", owner)
            or re.fullmatch(r"GATE-[A-Z0-9-]+", owner)
        ):
            raise SystemExit(f"untyped assignment owner for {subject}: {owner}")
        predecessor_ids = [] if predecessors == "NONE" else predecessors.split(",")
        if any(not re.fullmatch(r"F-[A-Z0-9-]+", token) for token in predecessor_ids):
            raise SystemExit(f"non-family predecessor for {subject}: {predecessors}")
        implicated_ids = implicated.split(",")
        if any(
            token != "ALL-FAMILY-LOCKS"
            and not re.fullmatch(r"F-[A-Z0-9-]+", token)
            for token in implicated_ids
        ):
            raise SystemExit(f"non-family implication for {subject}: {implicated}")
        if rebind not in allowed_rebind:
            raise SystemExit(f"unknown rebind policy for {subject}: {rebind}")
        if owner.startswith("F-") and owner not in implicated_ids:
            raise SystemExit(f"owner absent from implicated set for {subject}")
        if owner == "ALL-FAMILY-LOCKS" and (
            predecessors != "NONE"
            or implicated != "ALL-FAMILY-LOCKS"
            or rebind != "EVERY_CANDIDATE_EXECUTES"
        ):
            raise SystemExit(f"protected-control assignment is malformed for {subject}")
        records[subject] = {
            "owner": owner,
            "predecessors": predecessors,
            "implicated": implicated,
            "rebind": rebind,
        }
    return records, digest(text)


def implicated_rebind_policy(role: str, subject: str) -> str:
    if role == "B":
        return "EVERY_CANDIDATE_EXECUTES"
    if subject in {"Borrowed, uniq, and owning iteration", "W-PIPE"}:
        return "EACH_IMPLICATED_FAMILY_REBINDS_EXACT_TOPOLOGY"
    if role == "O":
        return "OPTIONAL_PROMOTION_ONLY_IMPLICATED_REOPEN"
    return "OWNER_OR_GATE_CLOSES_IMPLICATED_REOPEN"


def parse_table(text: str) -> list[dict[str, str]]:
    lines = text.splitlines()
    header = (
        "| Workload or operation | Role | Canary | Required efficient property | "
        "Status | Current workaround or blocker |"
    )
    try:
        start = lines.index(header)
    except ValueError as exc:
        raise SystemExit("G0 family requirement source table header is missing") from exc

    rows: list[dict[str, str]] = []
    for line_number, line in enumerate(lines[start + 2 :], start + 3):
        if not line.startswith("|"):
            break
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != 6:
            raise SystemExit(
                f"malformed G0 family requirement row at source line {line_number}"
            )
        workload, role, canary, required, status, blocker = cells
        rows.append(
            {
                "workload": workload,
                "role": role,
                "canary": canary,
                "required": required,
                "status": status,
                "blocker": blocker,
                "source_identity": (
                    "general-purpose-data-structure-capability-RESEARCH.md:"
                    f"section-4:line-{line_number}"
                ),
                "source_row_sha256": digest("\t".join(cells)),
            }
        )
    if len(rows) != 31:
        raise SystemExit(f"expected 31 G0 family requirement rows, found {len(rows)}")
    return rows


def parse_canaries(text: str) -> dict[str, tuple[str, str]]:
    start_marker = "- **C-FIX:**"
    end_marker = "\n\nH-STORE, H-LRU, and H-IPQ"
    start = text.find(start_marker)
    end = text.find(end_marker, start)
    if start < 0 or end < 0:
        raise SystemExit("G0 role-mapped canary section is missing")
    section = text[start:end]
    records: dict[str, tuple[str, str]] = {}
    current_id = ""
    current_lines: list[str] = []

    def finish() -> None:
        nonlocal current_id, current_lines
        if not current_id:
            return
        raw = "\n".join(current_lines)
        contract = re.sub(r"\s+", " ", raw).strip().rstrip(";.")
        records[current_id] = (contract, digest(raw))
        current_id = ""
        current_lines = []

    for line in section.splitlines():
        match = re.match(r"^- \*\*(C-[A-Z0-9-]+):\*\*\s*(.*)$", line)
        if match:
            finish()
            current_id = match.group(1)
            current_lines = [match.group(2)]
        elif current_id:
            current_lines.append(line.strip())
    finish()
    expected = {
        "C-FIX",
        "C-SEQ",
        "C-COMPACT",
        "C-LAZY-DRAIN",
        "C-SORT",
        "C-DEQUE",
        "C-HASH",
        "C-ORDER",
        "C-HEAP",
        "C-P2",
        "C-POOL",
        "C-GRAPH",
        "C-RECUR",
        "C-ARENA",
        "C-SMALL",
        "C-TEXT",
        "C-ITER",
        "C-FAIL",
    }
    if set(records) != expected:
        raise SystemExit(
            "G0 canary identity set changed: "
            f"missing={sorted(expected - set(records))}, "
            f"extra={sorted(set(records) - expected)}"
        )
    return records


def parse_markdown_table(
    text: str, header: str, expected_cells: int
) -> list[dict[str, object]]:
    lines = text.splitlines()
    try:
        start = lines.index(header)
    except ValueError as exc:
        raise SystemExit(f"witness-registry table header is missing: {header}") from exc
    rows: list[dict[str, object]] = []
    for line_number, line in enumerate(lines[start + 2 :], start + 3):
        if not line.startswith("|"):
            break
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != expected_cells:
            raise SystemExit(
                f"malformed witness-registry row at source line {line_number}"
            )
        rows.append({"cells": cells, "line_number": line_number, "raw": line})
    return rows


def parse_optional_witnesses(text: str) -> dict[str, dict[str, str]]:
    start = text.find("### 3.3 Visible controls and optional compositions")
    end = text.find("\n## 4. Exact held-out contracts", start)
    if start < 0 or end < 0:
        raise SystemExit("optional witness section is missing")
    records: dict[str, dict[str, str]] = {}
    current_id = ""
    current_lines: list[str] = []

    def finish() -> None:
        nonlocal current_id, current_lines
        if not current_id:
            return
        raw = "\n".join(current_lines)
        records[current_id] = {
            "contract": re.sub(r"\s+", " ", raw).strip(),
            "raw": raw,
        }
        current_id = ""
        current_lines = []

    for line in text[start:end].splitlines():
        match = re.match(r"^- \*\*(O-[A-Z0-9-]+):\*\*\s*(.*)$", line)
        if match:
            finish()
            current_id = match.group(1)
            current_lines = [match.group(2)]
        elif current_id:
            current_lines.append(line.strip())
    finish()
    return records


def parse_witness_supplement(
    c_fail_contract: str,
    c_fail_source_sha256: str,
    assignments: dict[str, dict[str, str]],
) -> list[dict[str, str]]:
    text = WITNESS.read_text(encoding="utf-8")
    records: dict[str, dict[str, str]] = {}

    baseline_rows = parse_markdown_table(
        text, "| ID | Contract | Protection |", 3
    )
    for row in baseline_rows:
        subject, contract, protection = row["cells"]
        raw = str(row["raw"])
        records[subject] = {
            "contract": f"{contract} Protection: {protection}",
            "blocker": protection,
            "source_identity": (
                "WITNESS-REGISTRY.md:section-2:line-" + str(row["line_number"])
            ),
            "source_sha256": digest(raw),
        }

    visible_rows = parse_markdown_table(
        text,
        "| ID | Role | Frozen observable contract | Separating purpose | Capability dependency budget |",
        5,
    )
    for row in visible_rows:
        subject, role, contract, purpose, budget = row["cells"]
        if role != "W":
            raise SystemExit(f"visible witness {subject} has unexpected role {role}")
        raw = str(row["raw"])
        records[subject] = {
            "contract": f"{contract} Dependency budget: {budget}",
            "blocker": purpose,
            "source_identity": (
                "WITNESS-REGISTRY.md:section-3:line-" + str(row["line_number"])
            ),
            "source_sha256": digest(raw),
        }

    resource_rows = parse_markdown_table(
        text,
        "| ID | Persistent and retained memory | Peak/transient memory | Backing allocations | Separating traffic or code ceiling |",
        5,
    )
    resources: dict[str, str] = {}
    resource_hashes: dict[str, str] = {}
    for row in resource_rows:
        subject, persistent, peak, allocations, traffic = row["cells"]
        resources[subject] = (
            f"Persistent: {persistent} Peak/transient: {peak} "
            f"Allocations: {allocations} Traffic/code: {traffic}"
        )
        resource_hashes[subject] = digest(str(row["raw"]))

    for subject in ("H-FLATSET", "H-STORE", "H-LRU", "H-IPQ"):
        heading = f"### {subject} —"
        start = text.find(heading)
        if start < 0:
            raise SystemExit(f"held-out witness section is missing: {subject}")
        body_start = text.find("\n", start) + 1
        next_heading = text.find("\n### H-", body_start)
        section_end = next_heading if next_heading >= 0 else text.find("\n## 5.", body_start)
        raw_section = text[start:section_end]
        contract = re.sub(r"\s+", " ", raw_section).strip()
        if subject not in resources:
            raise SystemExit(f"held-out resource envelope is missing: {subject}")
        records[subject] = {
            "contract": f"{contract} Resource envelope: {resources[subject]}",
            "blocker": "Training-excluded held-out witness; omission blocks its assigned family and the complete floor.",
            "source_identity": f"WITNESS-REGISTRY.md:section-4:{subject}",
            "source_sha256": digest(raw_section + "\n" + resource_hashes[subject]),
        }

    optional = parse_optional_witnesses(text)
    for subject, values in optional.items():
        records[subject] = {
            "contract": values["contract"],
            "blocker": "Optional unless promoted or required by a mandatory contract.",
            "source_identity": f"WITNESS-REGISTRY.md:section-3.3:{subject}",
            "source_sha256": digest(values["raw"]),
        }

    expected = set(SUPPLEMENTAL_ROLE)
    if not (
        expected
        == set(SUPPLEMENTAL_FAMILIES)
        == set(SUPPLEMENTAL_OWNER)
        == set(SUPPLEMENTAL_PREDECESSORS)
    ):
        raise SystemExit("supplemental family ownership/predecessor map is incomplete")
    if set(records) != expected:
        raise SystemExit(
            "witness-registry role universe changed: "
            f"missing={sorted(expected - set(records))}, "
            f"extra={sorted(set(records) - expected)}"
        )

    rows: list[dict[str, str]] = []
    for subject in SUPPLEMENTAL_ROLE:
        record = records[subject]
        assignment = assignments[subject]
        role = SUPPLEMENTAL_ROLE[subject]
        closure_policy = ROLE_POLICY[role]
        rows.append(
            {
                "record_kind": "WITNESS_REGISTRY",
                "registry_subject_id": subject,
                "workload": subject,
                "role": role,
                "canary": subject,
                "crosscut": "C-FAIL" if subject in OWNING_CANARY_IDS else "NONE",
                "c_fail_applicability": (
                    "REQUIRED_OWNING_CANARY"
                    if subject in OWNING_CANARY_IDS
                    else "NOT_APPLICABLE_NONOWNING_CANARY"
                ),
                "crosscut_canary_contract": (
                    c_fail_contract if subject in OWNING_CANARY_IDS else "NONE"
                ),
                "crosscut_canary_source_sha256": (
                    c_fail_source_sha256
                    if subject in OWNING_CANARY_IDS
                    else "NONE"
                ),
                "closure_owner_or_gate_stage": assignment["owner"],
                "required_predecessor_family_ids": assignment["predecessors"],
                "implicated_family_ids": assignment["implicated"],
                "implicated_rebind_policy": assignment["rebind"],
                "required": record["contract"],
                "status": "PROTECTED" if role == "B" else ("OPTIONAL" if role == "O" else "REQUIRED"),
                "blocker": record["blocker"],
                "closure_policy": closure_policy,
                "owner_allowed": owner_lock_allowed_dispositions(role),
                "rebind_allowed": implicated_rebind_allowed_dispositions(
                    role, assignment["rebind"]
                ),
                "canary_contract": record["contract"],
                "linked_registry_ids": subject,
                "source_identity": record["source_identity"],
                "source_row_sha256": record["source_sha256"],
                "canary_source_sha256": record["source_sha256"],
            }
        )
    return rows


def build_rows() -> list[dict[str, str]]:
    text = RESEARCH.read_text(encoding="utf-8")
    source_rows = parse_table(text)
    canaries = parse_canaries(text)
    assignments, vocabulary_authority_sha256 = parse_assignment_authority()
    source_workloads = {row["workload"] for row in source_rows}
    mapping_workloads = (
        set(FAMILY_BY_WORKLOAD)
        & set(OWNER_BY_WORKLOAD)
        & set(PREDECESSORS_BY_WORKLOAD)
    )
    if source_workloads != mapping_workloads or not (
        set(FAMILY_BY_WORKLOAD)
        == set(OWNER_BY_WORKLOAD)
        == set(PREDECESSORS_BY_WORKLOAD)
    ):
        raise SystemExit(
            "G0 family ownership/predecessor assignment is incomplete: "
            f"missing={sorted(source_workloads - mapping_workloads)}, "
            f"stale={sorted(mapping_workloads - source_workloads)}"
        )
    expected_assignment_subjects = source_workloads | set(SUPPLEMENTAL_ROLE)
    if set(assignments) != expected_assignment_subjects:
        raise SystemExit(
            "assignment-authority subject set changed: "
            f"missing={sorted(expected_assignment_subjects - set(assignments))}, "
            f"extra={sorted(set(assignments) - expected_assignment_subjects)}"
        )
    for workload in source_workloads:
        expected = {
            "owner": OWNER_BY_WORKLOAD[workload],
            "predecessors": PREDECESSORS_BY_WORKLOAD[workload],
            "implicated": FAMILY_BY_WORKLOAD[workload],
            "rebind": implicated_rebind_policy(
                next(row["role"] for row in source_rows if row["workload"] == workload),
                workload,
            ),
        }
        if assignments[workload] != expected:
            raise SystemExit(f"assignment authority disagrees with reviewed pin for {workload}")
    for subject in SUPPLEMENTAL_ROLE:
        expected = {
            "owner": SUPPLEMENTAL_OWNER[subject],
            "predecessors": SUPPLEMENTAL_PREDECESSORS[subject],
            "implicated": SUPPLEMENTAL_FAMILIES[subject],
            "rebind": implicated_rebind_policy(SUPPLEMENTAL_ROLE[subject], subject),
        }
        if assignments[subject] != expected:
            raise SystemExit(f"assignment authority disagrees with reviewed pin for {subject}")

    rows: list[dict[str, str]] = []
    seen_ids: set[str] = set()
    for source in source_rows:
        assignment = assignments[source["workload"]]
        role = source["role"]
        canary = source["canary"]
        if role not in ROLE_POLICY:
            raise SystemExit(f"unknown G0 role {role!r} for {source['workload']!r}")
        if canary not in canaries and canary not in SUPPLEMENTAL_ROLE:
            raise SystemExit(
                f"unknown G0 canary {canary!r} for {source['workload']!r}"
            )
        closure_policy = ROLE_POLICY[role]
        canonical_identity = "\t".join(
            [source["workload"], role, canary, source["required"]]
        )
        obligation_id = "G0-ROLE-" + digest(canonical_identity)[:16].upper()
        if obligation_id in seen_ids:
            raise SystemExit(f"duplicate generated obligation ID {obligation_id}")
        seen_ids.add(obligation_id)
        if canary in canaries:
            canary_contract, canary_hash = canaries[canary]
        else:
            canary_contract = source["required"]
            canary_hash = source["source_row_sha256"]
        rows.append(
            {
                "obligation_id": obligation_id,
                "record_kind": "CAPABILITY_MATRIX",
                "registry_subject_id": obligation_id,
                "workload_or_operation": source["workload"],
                "role": role,
                "primary_canary_id": canary,
                "required_crosscut_canary_ids": (
                    "C-FAIL" if canary in OWNING_CANARY_IDS else "NONE"
                ),
                "c_fail_applicability": (
                    "REQUIRED_OWNING_CANARY"
                    if canary in OWNING_CANARY_IDS
                    else "NOT_APPLICABLE_NONOWNING_CANARY"
                ),
                "crosscut_canary_contract": (
                    canaries["C-FAIL"][0]
                    if canary in OWNING_CANARY_IDS
                    else "NONE"
                ),
                "crosscut_canary_source_sha256": (
                    canaries["C-FAIL"][1]
                    if canary in OWNING_CANARY_IDS
                    else "NONE"
                ),
                "closure_owner_or_gate_stage": assignment["owner"],
                "required_predecessor_family_ids": assignment["predecessors"],
                "implicated_family_ids": assignment["implicated"],
                "implicated_rebind_policy": assignment["rebind"],
                "required_efficient_property": source["required"],
                "g0_status": source["status"],
                "current_workaround_or_blocker": source["blocker"],
                "closure_policy": closure_policy,
                "owner_lock_allowed_dispositions": (
                    owner_lock_allowed_dispositions(role)
                ),
                "implicated_rebind_allowed_dispositions": (
                    implicated_rebind_allowed_dispositions(
                        role, assignment["rebind"]
                    )
                ),
                "canary_contract": canary_contract,
                "linked_registry_ids": LINKED_REGISTRY_IDS.get(
                    source["workload"], "NONE"
                ),
                "source_identity": source["source_identity"],
                "source_row_sha256": source["source_row_sha256"],
                "canary_source_sha256": canary_hash,
                "vocabulary_authority_sha256": vocabulary_authority_sha256,
            }
        )
    for source in parse_witness_supplement(
        canaries["C-FAIL"][0],
        canaries["C-FAIL"][1],
        assignments,
    ):
        canonical_identity = "\t".join(
            [
                source["record_kind"],
                source["registry_subject_id"],
                source["role"],
                source["required"],
            ]
        )
        obligation_id = "G0-WITNESS-" + digest(canonical_identity)[:16].upper()
        if obligation_id in seen_ids:
            raise SystemExit(f"duplicate generated obligation ID {obligation_id}")
        seen_ids.add(obligation_id)
        rows.append(
            {
                "obligation_id": obligation_id,
                "record_kind": source["record_kind"],
                "registry_subject_id": source["registry_subject_id"],
                "workload_or_operation": source["workload"],
                "role": source["role"],
                "primary_canary_id": source["canary"],
                "required_crosscut_canary_ids": source["crosscut"],
                "c_fail_applicability": source["c_fail_applicability"],
                "crosscut_canary_contract": source["crosscut_canary_contract"],
                "crosscut_canary_source_sha256": source[
                    "crosscut_canary_source_sha256"
                ],
                "closure_owner_or_gate_stage": source[
                    "closure_owner_or_gate_stage"
                ],
                "required_predecessor_family_ids": source[
                    "required_predecessor_family_ids"
                ],
                "implicated_family_ids": source["implicated_family_ids"],
                "implicated_rebind_policy": source["implicated_rebind_policy"],
                "required_efficient_property": source["required"],
                "g0_status": source["status"],
                "current_workaround_or_blocker": source["blocker"],
                "closure_policy": source["closure_policy"],
                "owner_lock_allowed_dispositions": source["owner_allowed"],
                "implicated_rebind_allowed_dispositions": source[
                    "rebind_allowed"
                ],
                "canary_contract": source["canary_contract"],
                "linked_registry_ids": source["linked_registry_ids"],
                "source_identity": source["source_identity"],
                "source_row_sha256": source["source_row_sha256"],
                "canary_source_sha256": source["canary_source_sha256"],
                "vocabulary_authority_sha256": vocabulary_authority_sha256,
            }
        )
    return rows


def render(rows: list[dict[str, str]]) -> str:
    buffer = io.StringIO(newline="")
    writer = csv.DictWriter(
        buffer,
        fieldnames=HEADER,
        delimiter="\t",
        lineterminator="\n",
    )
    writer.writeheader()
    writer.writerows(rows)
    return buffer.getvalue()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail unless the checked-in registry matches deterministic output",
    )
    args = parser.parse_args()
    expected = render(build_rows())
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="utf-8") != expected:
            raise SystemExit(f"stale generated artifact: {OUTPUT.relative_to(ROOT)}")
        print("G0 family requirement registry: PASS")
        return
    OUTPUT.write_text(expected, encoding="utf-8")
    print(f"wrote {OUTPUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
