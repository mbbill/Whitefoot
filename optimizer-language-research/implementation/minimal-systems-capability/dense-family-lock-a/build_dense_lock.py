#!/usr/bin/env python3
"""Build the research-only Dense Family Lock A closure manifest."""

from __future__ import annotations

import hashlib
import json
import csv
import re
import subprocess
from pathlib import Path


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
MANIFEST = HERE / "DENSE-LOCK-ARTIFACT-MANIFEST.json"
SUMMARY = HERE / "DENSE-LOCK-BUILD-SUMMARY.json"
WHOLE_LOCK_REVIEW = HERE / "DENSE-WHOLE-LOCK-HOSTILE-REVIEW-PASS.md"

STATUS = "OWNER_REVIEW_READY_RESEARCH_ONLY"
LOCK_ID = "F-DENSE-LOCK-A"
FAMILY_NAME = "dense unique-owner"
LOCK_REVISION = "F-DENSE-LOCK-A-R5"
LOCK_FROZEN_AT = "2026-07-15T01:11:35-07:00"
LOCK_TIMEZONE = "America/Los_Angeles"
INTEGRATING_AUTHOR_TASK_ID = "/root"
WHOLE_LOCK_REVIEWER_TASK_ID = "/root/whole_lock_hostile"
AUTHORIZATION_COMMIT = "c4ca5437fc90f3ce833fb026f2e794f4f758d011"
AUTHORIZATION_PATH = "optimizer-language-research/notes/user-directives.md"
AUTHORIZATION_HEADING = "## D13 (2026-07-14): Draft the dense unique-owner Family Lock A\n"
AUTHORIZATION_DIRECTIVE_SHA256 = "3a209c195f575408a65d1a81b9e3e01b4c95dd406b589fab801c64b3ed29c64c"
UNRESOLVED_DECISIONS = ["OD-0", "OD-1", "OD-2", "OD-3", "OD-4", "OD-5"]
CALLER_VISIBLE_FAMILY_CLAIM = (
    "Ordinary no-unsafe xlang libraries can implement the frozen sequential, "
    "unique-owner dense-prefix contracts for region-free, borrow-free, "
    "non-address-sensitive affine payloads with the frozen ownership, failure, "
    "destruction, asymptotic, structural, and target-local performance bounds, "
    "while protected B-FIX and B-P2 paths remain unchanged."
)
EXPLICITLY_EXCLUDED_CLAIMS = [
    "borrow-bearing or region-bearing payloads",
    "nonlocal lifetime storage",
    "shared ownership",
    "dynamic borrowing",
    "concurrency",
    "pinning or address-sensitive values",
    "custom allocators",
    "FFI resources",
    "async cancellation",
    "panic unwind, exception handling, or recoverable cleanup after traps",
    "complete text semantics",
    "target intrinsics",
    "platforms other than the selected exact native targets",
    "universal platform or architecture claims",
    "cyclic collection",
    "complete standard library",
    "iteration-family W-PIPE",
]
G0_CORE_COMMIT = "a4de0eb70c345dcd198b11f435a5538ccc863113"
G0_CORE_GATE_HEADING = (
    "G0-Core capability accounting is complete; mechanisms remain unselected "
    "(2026-07-14)"
)
DECISION_GATES_PATH = "optimizer-language-research/implementation/decision-gates.md"
G0_CORE_MANIFEST_PATH = (
    "optimizer-language-research/implementation/minimal-systems-capability/"
    "G0-CORE-ARTIFACT-MANIFEST.json"
)
G0_CORE_MANIFEST = REPO / G0_CORE_MANIFEST_PATH
G0_CORE_MANIFEST_SHA256 = (
    "f0eced756688affef1732a133c43fb39ab6fc672334dca27b26129ddb5123719"
)
COVERAGE_INDEPENDENCE_COMMIT = "32c01e188ba55f652700cf8547187fe462302f0b"
COVERAGE_INDEPENDENCE_GATE_HEADING = (
    "D13-R3 closes the exact dense coverage authority (2026-07-14)"
)
COVERAGE_INDEPENDENCE_GATE_SHA256 = (
    "08b5322c032d878f1f2ac2055095d6c14d71756320d5a9ef3ddd8121232e7be2"
)

CURRENT_CONTROLS = (
    "AGENTS.md",
    "CLAUDE.md",
    "THE-PLAN.md",
    "optimizer-language-research/notes/user-directives.md",
    "mcts_mem/xlang.md",
    "mcts_mem/xlang/data-model.md",
    "mcts_mem/xlang/ownership.md",
    "mcts_mem/xlang/fact-channels.md",
)

COVERAGE_REVIEW_FILES = (
    "dense_literal_registry.py",
    "dense_coverage_closed_registry.py",
    "dense_coverage_authority.py",
    "test_dense_coverage_authority.py",
    "DENSE-LOCAL-DECLARATIVE-INPUT-AUTHORITY.tsv",
    "DENSE-FROZEN-G0-INPUT-AUTHORITY.tsv",
    "DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv",
    "DENSE-EVIDENCE-TARGET-AUTHORITY.tsv",
    "DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv",
    "DENSE-OVERLAY-BRANCH-AUTHORITY.tsv",
    "DENSE-ROLE-UNIT-AUTHORITY.tsv",
    "DENSE-CAPABILITY-UNIT-AUTHORITY.tsv",
)

SOUNDNESS_REVIEW_FILES = (
    "dense_owner_decisions.py",
    "dense_contract_registry.py",
    "dense_soundness_oracle.py",
    "dense_meta5.py",
    "dense_literal_registry.py",
    "dense_coverage_closed_registry.py",
    "DENSE-EVIDENCE-TARGET-AUTHORITY.tsv",
    "DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv",
    "DENSE-OVERLAY-BRANCH-AUTHORITY.tsv",
    "DENSE-CAPABILITY-UNIT-AUTHORITY.tsv",
    "DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv",
    "DENSE-ROLE-UNIT-AUTHORITY.tsv",
    "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
    "DENSE-OWNER-ROLE-REGISTRY.tsv",
    "DENSE-COMMON-SUBSTRATE-REGISTRY.tsv",
    "DENSE-STORED-BORROW-ROUTE-REGISTRY.tsv",
    "DENSE-OD4-POLICY-REGISTRY.tsv",
    "DENSE-OD1-POLICY-REGISTRY.tsv",
    "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv",
    "DENSE-CANDIDATE-OPERATION-REGISTRY.tsv",
    "DENSE-CANDIDATE-CONTRACT-BINDINGS.tsv",
    "DENSE-CANDIDATE-DISTINCTION-REGISTRY.tsv",
    "DENSE-ZST-POLICY-REGISTRY.tsv",
    "DENSE-FACT-CHANNEL-REGISTRY.tsv",
    "DENSE-SYNTHETIC-UNIT-REGISTRY.tsv",
    "DENSE-MATHEMATICAL-SOUNDNESS-TRACES.jsonl",
    "DENSE-SOUNDNESS-PROTOCOL-MANIFEST.json",
)

REQUIRED_CLOSURE_FILES = (
    "DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md",
    "DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md",
    "DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md",
    "DENSE-E01-DISPOSITION-AUTHORITY.tsv",
    "DENSE-E01-MEMBER-CLASSIFICATION.tsv",
    "DENSE-UNIQUE-OWNER-FAMILY-LOCK-A.md",
    "DENSE-PERFORMANCE-REGISTRY-SUMMARY.json",
    "DENSE-SOUNDNESS-PROTOCOL-MANIFEST.json",
)

OBSOLETE_DRAFT_FILES = (
    "dense_lock_model.py",
    "DENSE-CANDIDATE-SET.tsv",
    "DENSE-CAPABILITY-DISPOSITIONS.tsv",
    "DENSE-CLUSTER-AUDIT.tsv",
    "DENSE-DELEGATED-BRANCHES.tsv",
    "DENSE-EVIDENCE-APPLICABILITY.tsv",
    "DENSE-MEMBER-OUTCOME-LEDGER.tsv",
    "DENSE-META5-LEDGER.tsv",
    "DENSE-PAYLOAD-DISPOSITIONS.tsv",
    "DENSE-ROLE-DISPOSITIONS.tsv",
    "DENSE-SELECTOR-CHILDREN.tsv",
    "DENSE-SELECTOR-SUMMARY.tsv",
    "DENSE-SOUNDNESS-CASES.jsonl",
    "DENSE-SOUNDNESS-SCENARIOS.tsv",
    "generate_dense_soundness_cases.py",
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def artifact_record(path: Path, display_path: str) -> dict[str, object]:
    data = path.read_bytes()
    return {
        "path": display_path,
        "bytes": len(data),
        "sha256": sha256(data),
    }


def read_tsv(name: str) -> list[dict[str, str]]:
    with (HERE / name).open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def extract_authorization_directive(data: bytes) -> bytes:
    marker = AUTHORIZATION_HEADING.encode("ascii")
    try:
        start = data.index(marker)
    except ValueError as error:
        raise SystemExit("D13 authorization heading is absent") from error
    end = data.find(b"\n## ", start + len(marker))
    return data[start:] if end < 0 else data[start:end + 1]


def extract_gate_entry(data: bytes, heading: str) -> bytes:
    marker = f"## {heading}\n".encode("ascii")
    try:
        start = data.index(marker)
    except ValueError as error:
        raise SystemExit(f"decision-gate entry is absent: {heading}") from error
    end = data.find(b"\n## ", start + len(marker))
    return data[start:] if end < 0 else data[start:end + 1]


def authorization_record() -> dict[str, object]:
    subprocess.run(
        ["git", "cat-file", "-e", f"{AUTHORIZATION_COMMIT}^{{commit}}"],
        cwd=REPO,
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if subprocess.run(
        ["git", "merge-base", "--is-ancestor", AUTHORIZATION_COMMIT, "HEAD"],
        cwd=REPO,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode != 0:
        raise SystemExit("D13 authorization commit is not an ancestor of HEAD")
    committed = subprocess.check_output(
        ["git", "show", f"{AUTHORIZATION_COMMIT}:{AUTHORIZATION_PATH}"],
        cwd=REPO,
    )
    directive = extract_authorization_directive(committed)
    if sha256(directive) != AUTHORIZATION_DIRECTIVE_SHA256:
        raise SystemExit("D13 authorization directive bytes changed")
    current = (REPO / AUTHORIZATION_PATH).read_bytes()
    if extract_authorization_directive(current) != directive:
        raise SystemExit("current D13 directive differs from its authorizing commit")
    return {
        "commit": AUTHORIZATION_COMMIT,
        "directive_path": AUTHORIZATION_PATH,
        "directive_bytes": len(directive),
        "directive_sha256": AUTHORIZATION_DIRECTIVE_SHA256,
        "ancestor_of_head": True,
    }


def g0_core_record() -> dict[str, object]:
    subprocess.run(
        ["git", "cat-file", "-e", f"{G0_CORE_COMMIT}^{{commit}}"],
        cwd=REPO,
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if subprocess.run(
        ["git", "merge-base", "--is-ancestor", G0_CORE_COMMIT, "HEAD"],
        cwd=REPO,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode != 0:
        raise SystemExit("G0-Core closing commit is not an ancestor of HEAD")
    committed_gates = subprocess.check_output(
        ["git", "show", f"{G0_CORE_COMMIT}:{DECISION_GATES_PATH}"],
        cwd=REPO,
    ).decode("utf-8")
    current_gates = (REPO / DECISION_GATES_PATH).read_text(encoding="utf-8")
    heading = f"## {G0_CORE_GATE_HEADING}\n"
    if committed_gates.count(heading) != 1 or current_gates.count(heading) != 1:
        raise SystemExit("G0-Core closing gate heading is absent or duplicated")
    committed_manifest = subprocess.check_output(
        ["git", "show", f"{G0_CORE_COMMIT}:{G0_CORE_MANIFEST_PATH}"],
        cwd=REPO,
    )
    if (
        sha256(committed_manifest) != G0_CORE_MANIFEST_SHA256
        or sha256(G0_CORE_MANIFEST.read_bytes()) != G0_CORE_MANIFEST_SHA256
    ):
        raise SystemExit("G0-Core frozen artifact manifest changed")
    return {
        "status": "COMPLETE_MECHANISMS_UNSELECTED",
        "closing_commit": G0_CORE_COMMIT,
        "decision_gate_heading": G0_CORE_GATE_HEADING,
        "artifact_manifest_path": G0_CORE_MANIFEST_PATH,
        "artifact_manifest_sha256": G0_CORE_MANIFEST_SHA256,
        "relationship_to_family_lock": (
            "FROZEN_RESEARCH_INPUT_NOT_A_COMPLETED_FAMILY_PREDECESSOR"
        ),
    }


def coverage_independence_record() -> dict[str, object]:
    subprocess.run(
        ["git", "cat-file", "-e", f"{COVERAGE_INDEPENDENCE_COMMIT}^{{commit}}"],
        cwd=REPO,
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if subprocess.run(
        [
            "git",
            "merge-base",
            "--is-ancestor",
            COVERAGE_INDEPENDENCE_COMMIT,
            "HEAD",
        ],
        cwd=REPO,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode != 0:
        raise SystemExit("coverage independence commit is not an ancestor of HEAD")
    committed_gates = subprocess.check_output(
        [
            "git",
            "show",
            f"{COVERAGE_INDEPENDENCE_COMMIT}:{DECISION_GATES_PATH}",
        ],
        cwd=REPO,
    )
    current_gates = (REPO / DECISION_GATES_PATH).read_bytes()
    committed_entry = extract_gate_entry(
        committed_gates, COVERAGE_INDEPENDENCE_GATE_HEADING
    )
    current_entry = extract_gate_entry(
        current_gates, COVERAGE_INDEPENDENCE_GATE_HEADING
    )
    if (
        committed_entry.rstrip(b"\n") != current_entry.rstrip(b"\n")
        or sha256(committed_entry) != COVERAGE_INDEPENDENCE_GATE_SHA256
    ):
        raise SystemExit("coverage independence gate evidence changed")
    return {
        "commit": COVERAGE_INDEPENDENCE_COMMIT,
        "decision_gate_heading": COVERAGE_INDEPENDENCE_GATE_HEADING,
        "decision_gate_entry_sha256": COVERAGE_INDEPENDENCE_GATE_SHA256,
        "attested_property": "INDEPENDENT_G0_ONLY_ORACLE_AND_REVIEW",
    }


def review_hash_table(report: str) -> dict[str, str]:
    result: dict[str, str] = {}
    pattern = re.compile(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|$", re.MULTILINE)
    for name, digest in pattern.findall(report):
        if name in result:
            raise SystemExit(f"duplicate hostile-review hash row: {name}")
        result[name] = digest
    return result


def performance_review_files() -> tuple[str, ...]:
    summary = json.loads(
        (HERE / "DENSE-PERFORMANCE-REGISTRY-SUMMARY.json").read_text(
            encoding="utf-8"
        )
    )
    names = {"DENSE-PERFORMANCE-REGISTRY-SUMMARY.json"}
    names.update(record["path"] for record in summary["artifacts"])
    names.update(
        record["path"] for record in summary["source_authorities"]
        if (HERE / record["path"]).is_file()
    )
    return tuple(sorted(names))


def review_record(
    report_name: str,
    expected_status: str,
    required_current_files: tuple[str, ...],
) -> dict[str, object]:
    path = HERE / report_name
    report = path.read_text(encoding="utf-8")
    statuses = re.findall(r"(?:^| )Status: `([^`]+)`\.", report, re.MULTILINE)
    results = re.findall(r"^Result: `([^`]+)`\.", report, re.MULTILINE)
    if statuses != [expected_status] or results != [expected_status]:
        raise SystemExit(f"hostile-review status/result is not exact: {report_name}")
    if re.search(r"(?:^|\n)(?:Status|Result): .*FAIL", report):
        raise SystemExit(f"hostile review contains a failing verdict: {report_name}")
    reviewed = review_hash_table(report)
    missing = [name for name in required_current_files if name not in reviewed]
    if missing:
        raise SystemExit(
            f"hostile review omits current byte(s) in {report_name}: "
            + ", ".join(missing)
        )
    mismatches = [
        name for name in required_current_files
        if reviewed[name] != sha256((HERE / name).read_bytes())
    ]
    if mismatches:
        raise SystemExit(
            f"hostile review pins stale byte(s) in {report_name}: "
            + ", ".join(mismatches)
        )
    reviewed_subset = {
        name: reviewed[name] for name in sorted(required_current_files)
    }
    return {
        "status": expected_status,
        "review_path": report_name,
        "review_sha256": sha256(path.read_bytes()),
        "reviewed_current_artifact_count": len(reviewed_subset),
        "reviewed_current_artifact_map_sha256": sha256(
            (json.dumps(reviewed_subset, sort_keys=True, separators=(",", ":")) + "\n").encode("ascii")
        ),
    }


def validated_review_layers() -> dict[str, object]:
    return {
        "coverage": review_record(
            "DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md",
            "PASS_EXACT_BYTES_COVERAGE_ONLY",
            COVERAGE_REVIEW_FILES,
        ),
        "contract_soundness": review_record(
            "DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md",
            "PASS_EXACT_BYTES_CONTRACT_SOUNDNESS_ONLY",
            SOUNDNESS_REVIEW_FILES,
        ),
        "performance_protocol": review_record(
            "DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md",
            "PASS_EXACT_BYTES_PERFORMANCE_PROTOCOL_ONLY",
            performance_review_files(),
        ),
        "whole_lock": {
            "status": "EXTERNAL_EXACT_HASH_REVIEW_REQUIRED",
            "review_path": WHOLE_LOCK_REVIEW.name,
        },
    }


def review_identity_records(reviews: dict[str, object]) -> dict[str, object]:
    historical = (
        ("coverage", "COVERAGE-REVIEW-R3", "independent coverage and provenance reviewer"),
        (
            "contract_soundness",
            "CONTRACT-SOUNDNESS-REVIEW-R1",
            "independent contract and mathematical soundness reviewer",
        ),
        (
            "performance_protocol",
            "PERFORMANCE-REVIEW-V5",
            "independent performance and statistics reviewer",
        ),
    )
    records: dict[str, object] = {
        "integrating_author": {
            "task_identity": INTEGRATING_AUTHOR_TASK_ID,
            "role": "research lock integrator and artifact author",
        },
        "historical_identity_rule": (
            "Coverage and contract-soundness personal/task identities were not "
            "retained; their durable identity is the exact review artifact and "
            "hash. The performance reviewer task identity is retained in its report."
        ),
        "final_whole_lock_reviewer": {
            "task_identity": WHOLE_LOCK_REVIEWER_TASK_ID,
            "role": "independent final exact-byte whole-lock reviewer",
            "reviewed_bytes_edited_by_reviewer": False,
            "report_hash_location": "external report; excluded to avoid a hash cycle",
        },
    }
    for layer, prefix, role in historical:
        review = reviews[layer]
        digest = review["review_sha256"]
        records[layer] = {
            "durable_identity": f"{prefix}@{digest}",
            "role": role,
            "review_path": review["review_path"],
            "review_sha256": digest,
            "independent_review": True,
            "reviewed_bytes_edited_by_reviewer": (
                "NOT_RETAINED" if layer == "coverage" else False
            ),
            "no_edit_attestation": (
                "NOT_RETAINED_IN_DURABLE_COVERAGE_REPORT"
                if layer == "coverage"
                else "PRESENT_IN_EXACT_REVIEW_REPORT"
            ),
            "historical_personal_or_task_identity_retained": (
                layer == "performance_protocol"
            ),
        }
        if layer == "coverage":
            records[layer]["independent_review_evidence"] = (
                coverage_independence_record()
            )
        else:
            records[layer]["independent_review_evidence"] = {
                "review_path": review["review_path"],
                "review_sha256": digest,
            }
        if layer == "performance_protocol":
            records[layer]["task_identity"] = "/root/repair_soundness_protocol"
    return records


def claim_summary() -> dict[str, object]:
    targets = read_tsv("DENSE-EVIDENCE-TARGET-AUTHORITY.tsv")
    selectors = read_tsv("DENSE-SELECTOR-EXPANSION-AUTHORITY.tsv")
    evidence_members = read_tsv("DENSE-EVIDENCE-MEMBER-AUTHORITY.tsv")
    overlays = read_tsv("DENSE-OVERLAY-BRANCH-AUTHORITY.tsv")
    roles = read_tsv("DENSE-ROLE-UNIT-AUTHORITY.tsv")
    capabilities = read_tsv("DENSE-CAPABILITY-UNIT-AUTHORITY.tsv")
    soundness = json.loads(
        (HERE / "DENSE-SOUNDNESS-PROTOCOL-MANIFEST.json").read_text(
            encoding="utf-8"
        )
    )
    performance = json.loads(
        (HERE / "DENSE-PERFORMANCE-REGISTRY-SUMMARY.json").read_text(
            encoding="utf-8"
        )
    )
    e0_members = read_tsv("DENSE-E01-MEMBER-CLASSIFICATION.tsv")
    e0_classes: dict[str, int] = {}
    for row in e0_members:
        relation = row["e01_relation_class"]
        e0_classes[relation] = e0_classes.get(relation, 0) + 1
    return {
        "coverage": {
            "audit_cluster_count": len({row["cluster_id"] for row in targets}),
            "selector_child_count": len(selectors),
            "target_terminal_count": len(targets),
            "evidence_member_binding_count": len(evidence_members),
            "overlay_binding_count": len(overlays),
            "role_binding_count": len(roles),
            "capability_binding_count": len(capabilities),
            "direct_evidence_identity_count": len({
                row["subject_identity"] for row in targets
                if row["subject_kind"] == "DIRECT_EVIDENCE"
            }),
        },
        "contract_soundness": {
            "candidate_count": len(read_tsv("DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv")),
            "contract_count": soundness["validation_counts"]["contracts"],
            "candidate_binding_count": soundness["validation_counts"]["candidate_bindings"],
            "trace_count": soundness["trace_artifact"]["case_count"],
            "registered_mutation_count": soundness["mutation_test_count"],
            "hostile_case_count": soundness["validation_counts"]["hostile_cases"],
        },
        "performance": {
            "operation_gate_count": performance["operation_gate_count"],
            "matrix_cell_count": performance["matrix_cell_count"],
            "timed_primary_cell_count": performance["timed_primary_cell_count"],
            "explicit_blocker_count": performance["explicit_blocker_count"],
            "active_owner_branch_count": performance["active_owner_branch_count"],
            "disposition_counts": performance["disposition_counts"],
        },
        "e0_1_disposition_count": len(read_tsv("DENSE-E01-DISPOSITION-AUTHORITY.tsv")),
        "e0_1_member_classification_count": len(e0_members),
        "e0_1_member_classification_counts": e0_classes,
        "unresolved_owner_decision_count": len(UNRESOLVED_DECISIONS),
    }


def exact_path_map(paths: tuple[str, ...]) -> dict[str, object]:
    unique_paths = tuple(sorted(set(paths)))
    missing = [name for name in unique_paths if not (HERE / name).is_file()]
    if missing:
        raise SystemExit("artifact-class path is absent: " + ", ".join(missing))
    digests = {
        name: sha256((HERE / name).read_bytes()) for name in unique_paths
    }
    encoded = (
        json.dumps(digests, sort_keys=True, separators=(",", ":")) + "\n"
    ).encode("ascii")
    return {
        "paths": list(unique_paths),
        "sha256_by_path": digests,
        "path_sha256_map_sha256": sha256(encoded),
    }


def artifact_class_records(reviews: dict[str, object]) -> list[dict[str, object]]:
    contract_tables = (
        "DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv",
        "DENSE-OWNER-ROLE-REGISTRY.tsv",
        "DENSE-COMMON-SUBSTRATE-REGISTRY.tsv",
        "DENSE-STORED-BORROW-ROUTE-REGISTRY.tsv",
        "DENSE-OD4-POLICY-REGISTRY.tsv",
        "DENSE-OD1-POLICY-REGISTRY.tsv",
        "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv",
        "DENSE-CANDIDATE-OPERATION-REGISTRY.tsv",
        "DENSE-CANDIDATE-CONTRACT-BINDINGS.tsv",
        "DENSE-CANDIDATE-DISTINCTION-REGISTRY.tsv",
        "DENSE-ZST-POLICY-REGISTRY.tsv",
        "DENSE-FACT-CHANNEL-REGISTRY.tsv",
        "DENSE-SYNTHETIC-UNIT-REGISTRY.tsv",
    )
    soundness_tools = (
        "dense_contract_registry.py",
        "dense_soundness_oracle.py",
        "DENSE-MATHEMATICAL-SOUNDNESS-TRACES.jsonl",
        "DENSE-SOUNDNESS-PROTOCOL-MANIFEST.json",
    )
    performance_paths = performance_review_files()
    classes = [
        {
            "artifact_class": "INSTANTIATED_FAMILY_LOCK_A",
            **exact_path_map(("DENSE-UNIQUE-OWNER-FAMILY-LOCK-A.md",)),
            "producer": INTEGRATING_AUTHOR_TASK_ID,
            "reviewer": WHOLE_LOCK_REVIEWER_TASK_ID,
            "status": "EXTERNAL_EXACT_HASH_WHOLE_LOCK_REVIEW_REQUIRED",
        },
        {
            "artifact_class": "FAIL_CLOSED_LOCK_VALIDATOR",
            **exact_path_map(("build_dense_lock.py", "verify_dense_lock.py")),
            "producer": INTEGRATING_AUTHOR_TASK_ID,
            "reviewer": WHOLE_LOCK_REVIEWER_TASK_ID,
            "status": "EXTERNAL_EXACT_HASH_WHOLE_LOCK_REVIEW_REQUIRED",
        },
        {
            "artifact_class": "CONTRACT_TABLES",
            **exact_path_map(contract_tables),
            "producer": (
                "dense_contract_registry.py@"
                + sha256((HERE / "dense_contract_registry.py").read_bytes())
            ),
            "reviewer": (
                "CONTRACT-SOUNDNESS-REVIEW-R1@"
                + reviews["contract_soundness"]["review_sha256"]
            ),
            "status": "PASS_EXACT_BYTES_CONTRACT_SOUNDNESS_ONLY",
        },
        {
            "artifact_class": "SOUNDNESS_FIXTURES_AND_GENERATORS",
            **exact_path_map(soundness_tools),
            "producer": (
                "dense_contract_registry.py@"
                + sha256((HERE / "dense_contract_registry.py").read_bytes())
                + ";dense_soundness_oracle.py@"
                + sha256((HERE / "dense_soundness_oracle.py").read_bytes())
            ),
            "reviewer": (
                "CONTRACT-SOUNDNESS-REVIEW-R1@"
                + reviews["contract_soundness"]["review_sha256"]
            ),
            "status": "PASS_EXACT_BYTES_CONTRACT_SOUNDNESS_ONLY",
        },
        {
            "artifact_class": "PERFORMANCE_PROTOCOL_AND_TOOLS",
            **exact_path_map(performance_paths),
            "producer": (
                "dense_performance_registry.py@"
                + sha256((HERE / "dense_performance_registry.py").read_bytes())
                + ";generate_dense_performance_inputs.py@"
                + sha256(
                    (HERE / "generate_dense_performance_inputs.py").read_bytes()
                )
            ),
            "reviewer": (
                "PERFORMANCE-REVIEW-V5@"
                + reviews["performance_protocol"]["review_sha256"]
            ),
            "status": "PASS_EXACT_BYTES_PERFORMANCE_PROTOCOL_ONLY",
        },
        {
            "artifact_class": "META5_LEDGER",
            **exact_path_map(("dense_meta5.py",)),
            "producer": (
                "dense_meta5.py@" + sha256((HERE / "dense_meta5.py").read_bytes())
            ),
            "reviewer": (
                "CONTRACT-SOUNDNESS-REVIEW-R1@"
                + reviews["contract_soundness"]["review_sha256"]
            ),
            "status": "PASS_RESEARCH_DELTAS_UNSELECTED_NO_PRODUCTION_CHANGE",
        },
        {
            "artifact_class": "HELD_OUT_CUSTODY_RECORD",
            **exact_path_map(("DENSE-PERFORMANCE-BLOCKERS.tsv",)),
            "producer": "PENDING_EXTERNAL_CUSTODIAN",
            "reviewer": "PENDING_EXTERNAL_INDEPENDENT_CUSTODY_AUDITOR",
            "status": "BLOCKED_PENDING_EXTERNAL_H_FLATSET_CUSTODY",
            "blocker_id": "PENDING_EXTERNAL_H_FLATSET_CUSTODY",
            "hidden_source_or_hash_claimed": False,
        },
    ]
    return classes


def owner_authorization_record() -> dict[str, object]:
    return {
        "activities_authorized_after_owner_approval": (
            "SIX_OWNER_RESEARCH_PROTOCOL_DECISIONS_ONLY"
        ),
        "activities_still_prohibited": [
            "reference pilot",
            "candidate construction",
            "Candidate Freeze B",
            "candidate-primary execution",
            "scored or held-out execution",
            "held-out access",
            "candidate selection or scoring",
            "language or specification change",
            "language or specification decision",
            "compiler implementation",
            "production implementation",
            "production adoption",
            "E0.1 restart",
            "xlc migration",
            "default teaching",
            "complete-floor claim",
            "whole systems-language claim",
        ],
        "candidate_freeze_b_approval_requirement": (
            "Separate explicit owner authorization after candidate construction, "
            "all applicable cumulative CANDIDATE_FREEZE_B blockers, and a fresh "
            "independent exact-hash construction review close."
        ),
        "scored_and_held_out_approval_requirement": (
            "Separate explicit owner authorization after Candidate Freeze B pins "
            "every candidate hash and an independent custody audit freezes held-out "
            "source, tests, traces, disclosure, access, leak, and rotation records."
        ),
        "family_closure_criterion": (
            "One owner-selected branch must close all 303 exact contracts, every "
            "applicable mandatory M/W/H role and protected B control, mathematical "
            "proof, structural and measured Rust-floor gates, fact-channel reports, "
            "and independent exact-hash reviews; the selection rule must return one "
            "candidate rather than NO-SELECTION."
        ),
        "production_adoption_gate": "SEPARATE_OWNER_DECISION_AFTER_FAMILY_CLOSURE",
        "specification_or_compiler_gate": "SEPARATE_OWNER_DECISION",
        "xlc_migration_gate": "SEPARATE_OWNER_DECISION_AND_WORKLOAD_EVIDENCE",
        "default_teaching_gate": (
            "SEPARATE_BENCHMARK_BLIND_WRITER_EVIDENCE_AND_OWNER_DECISION"
        ),
        "complete_floor_claim": "PROHIBITED_UNTIL_ALL_MANDATORY_FAMILIES_CLOSE",
        "whole_systems_language_claim": (
            "PROHIBITED_UNTIL_ALL_OWNER_REQUIRED_DEFERRED_DOMAINS_CLOSE"
        ),
    }


def completion_record() -> list[dict[str, object]]:
    statistics = json.loads(
        (HERE / "DENSE-PERFORMANCE-STATISTICS.json").read_text(encoding="utf-8")
    )
    stages = statistics["stage_prerequisite_protocol"]["per_owner_branch"]
    construction_blockers = sorted({
        blocker_id
        for branch in stages.values()
        for blocker_id in branch["pipeline"]["CANDIDATE_CONSTRUCTION"][
            "cumulative_blocker_ids"
        ]
    })
    freeze_b_blockers = sorted({
        blocker_id
        for branch in stages.values()
        for blocker_id in branch["pipeline"]["CANDIDATE_FREEZE_B"][
            "cumulative_blocker_ids"
        ]
    })
    return [
        {
            "completion_item": "ALL_FIELD_MARKERS_RESOLVED",
            "status": "PASS_RESEARCH_INSTANTIATION",
            "evidence_paths": ["DENSE-UNIQUE-OWNER-FAMILY-LOCK-A.md", "build_dense_lock.py"],
            "blocker_ids": [],
        },
        {
            "completion_item": "CONTRACT_CAPABILITY_WITNESS_CLOSURE",
            "status": "PASS_RESEARCH_FREEZE_FAMILY_CLOSURE_NOT_CLAIMED",
            "evidence_paths": ["DENSE-EXACT-MEMBER-OUTCOME-REGISTRY.tsv", "DENSE-ROLE-UNIT-AUTHORITY.tsv"],
            "blocker_ids": [
                "PENDING_EXTERNAL_H_FLATSET_CUSTODY",
                "PENDING_EXTERNAL_W_GAP_FIXTURE",
                "PENDING_EXTERNAL_W_SMALL_FIXTURE",
            ],
        },
        {
            "completion_item": "SOUNDNESS_FIXTURE_FREEZE",
            "status": "PASS_EXACT_BYTES_CONTRACT_SOUNDNESS_ONLY",
            "evidence_paths": ["DENSE-SOUNDNESS-PROTOCOL-MANIFEST.json", "DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md"],
            "blocker_ids": [],
        },
        {
            "completion_item": "CONSTRUCTION_CORRECTION_FREEZE",
            "status": "BLOCKED_BEFORE_FIRST_CANDIDATE_PROMPT",
            "evidence_paths": ["DENSE-PERFORMANCE-STATISTICS.json", "DENSE-PERFORMANCE-BLOCKERS.tsv"],
            "blocker_ids": construction_blockers,
        },
        {
            "completion_item": "PERFORMANCE_AND_SELECTION_FREEZE",
            "status": "PASS_PROTOCOL_ONLY_EXECUTION_AND_SELECTION_BLOCKED",
            "evidence_paths": ["DENSE-PERFORMANCE-PROTOCOL.md", "DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md"],
            "blocker_ids": freeze_b_blockers,
        },
        {
            "completion_item": "B_FIX_B_P2_NO_TAX_ORACLE_FREEZE",
            "status": "PASS_PROTOCOL_ONLY_CANDIDATE_COMPARISON_BLOCKED",
            "evidence_paths": ["DENSE-PERFORMANCE-CONTROLS.tsv", "DENSE-PERFORMANCE-STRUCTURAL-GATES.tsv"],
            "blocker_ids": freeze_b_blockers,
        },
        {
            "completion_item": "HELD_OUT_CUSTODY_FREEZE",
            "status": "BLOCKED_NO_HIDDEN_ARTIFACT_CLAIMED",
            "evidence_paths": ["DENSE-PERFORMANCE-BLOCKERS.tsv"],
            "blocker_ids": ["PENDING_EXTERNAL_H_FLATSET_CUSTODY"],
        },
        {
            "completion_item": "META5_CLOSURE",
            "status": "PASS_RESEARCH_DELTAS_UNSELECTED_PRODUCTION_ARTIFACTS_BLOCKED",
            "evidence_paths": ["dense_meta5.py", "DENSE-CANDIDATE-LIFECYCLE-REGISTRY.tsv"],
            "blocker_ids": ["PENDING_EXTERNAL_OD4_CANDIDATE_ARTIFACTS"],
        },
        {
            "completion_item": "E0_1_DISPOSITION",
            "status": "PASS_13_INPUTS_93_MEMBERS_NO_RESTART",
            "evidence_paths": ["DENSE-E01-DISPOSITION-AUTHORITY.tsv", "DENSE-E01-MEMBER-CLASSIFICATION.tsv"],
            "blocker_ids": [],
        },
        {
            "completion_item": "INDEPENDENT_HOSTILE_REVIEWS",
            "status": "LAYER_PASSES_EXTERNAL_WHOLE_LOCK_REVIEW_REQUIRED",
            "evidence_paths": ["DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md", "DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md", "DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md"],
            "blocker_ids": [],
        },
        {
            "completion_item": "REPOSITORY_VERIFICATION",
            "status": "REQUIRED_ON_FINAL_BYTES_BEFORE_AND_AFTER_EXTERNAL_REVIEW",
            "evidence_paths": ["verify_dense_lock.py"],
            "commands": ["make check", "make -C compiler check", "git diff --check"],
            "blocker_ids": [],
        },
        {
            "completion_item": "OWNER_AUTHORIZATION",
            "status": "BLOCKED_SIX_OWNER_DECISIONS_UNRESOLVED",
            "evidence_paths": ["dense_owner_decisions.py"],
            "blocker_ids": ["PENDING_EXTERNAL_OWNER_BRANCH_SELECTION"],
        },
        {
            "completion_item": "DURABILITY",
            "status": "EXTERNAL_FINAL_COMMIT_AND_DECISION_GATE_REQUIRED",
            "evidence_paths": [],
            "blocker_ids": [],
        },
    ]


def canonical_directory_files() -> list[Path]:
    # The external whole-lock review pins this manifest, so it cannot itself be
    # inside the manifest without creating a hash cycle.
    excluded = {MANIFEST.name, SUMMARY.name, WHOLE_LOCK_REVIEW.name}
    return sorted(
        path
        for path in HERE.iterdir()
        if path.is_file() and path.name not in excluded
    )


def assert_closure_boundary() -> None:
    missing = [name for name in REQUIRED_CLOSURE_FILES if not (HERE / name).is_file()]
    if missing:
        raise SystemExit("missing closure artifact(s): " + ", ".join(missing))
    obsolete = [name for name in OBSOLETE_DRAFT_FILES if (HERE / name).exists()]
    if obsolete:
        raise SystemExit("obsolete draft artifact(s) remain: " + ", ".join(obsolete))
    if (REPO / "AGENTS.md").read_bytes() != (REPO / "CLAUDE.md").read_bytes():
        raise SystemExit("AGENTS.md and CLAUDE.md are not byte-identical")


def build_manifest() -> dict[str, object]:
    assert_closure_boundary()
    authorization = authorization_record()
    reviews = validated_review_layers()
    review_identities = review_identity_records(reviews)
    claims = claim_summary()
    artifact_classes = artifact_class_records(reviews)
    authorization_boundary = owner_authorization_record()
    completion = completion_record()
    artifacts = [
        artifact_record(path, path.name)
        for path in canonical_directory_files()
    ]
    controls = [
        artifact_record(REPO / relative, relative)
        for relative in CURRENT_CONTROLS
    ]
    return {
        "schema": "xlang-dense-family-lock-a-closure-manifest-v4",
        "lock_id": LOCK_ID,
        "family_name": FAMILY_NAME,
        "lock_revision": LOCK_REVISION,
        "status": STATUS,
        "frozen_at": LOCK_FROZEN_AT,
        "timezone": LOCK_TIMEZONE,
        "integrating_author_task_identity": INTEGRATING_AUTHOR_TASK_ID,
        "review_identities": review_identities,
        "artifact_class_records": artifact_classes,
        "required_predecessor_locks": [],
        "predecessor_lock_basis": (
            "This is the first owning-storage Family Lock. G0-Core is a frozen "
            "research input, not a completed family predecessor."
        ),
        "current_g0_core_status": g0_core_record(),
        "caller_visible_family_claim": CALLER_VISIBLE_FAMILY_CLAIM,
        "explicitly_excluded_claims": EXPLICITLY_EXCLUDED_CLAIMS,
        "cross_family_state_or_fact_exposure": "NONE",
        "cross_family_import_rule": (
            "Candidate partial states and optimizer facts are sealed and unselected. "
            "No later family may import dense capability until a separate production "
            "adoption decision closes it."
        ),
        "authorization_commit": AUTHORIZATION_COMMIT,
        "authorization": authorization,
        "candidate_construction_authorized": False,
        "reference_pilot_execution_authorized": False,
        "candidate_freeze_b_authorized": False,
        "candidate_execution_authorized": False,
        "held_out_access_authorized": False,
        "selection_or_scoring_authorized": False,
        "language_or_specification_change_authorized": False,
        "language_or_specification_decision_authorized": False,
        "compiler_implementation_authorized": False,
        "production_implementation_authorized": False,
        "production_adoption_authorized": False,
        "compiler_or_production_implementation_authorized": False,
        "e0_1_restart_authorized": False,
        "xlc_migration_authorized": False,
        "default_teaching_authorized": False,
        "unresolved_owner_decisions": UNRESOLVED_DECISIONS,
        "review_layers": reviews,
        "claim_summary": claims,
        "artifacts": artifacts,
        "current_controls": controls,
        "artifact_count": len(artifacts),
        "current_control_count": len(controls),
        "authority_boundary": (
            "This manifest closes a research dossier and executable preregistration "
            "for owner review only. It grants no candidate construction, observation, "
            "selection, language decision, compiler work, or production authority."
        ),
        "authorization_granted_by_approving_this_lock": (
            "SIX_OWNER_RESEARCH_PROTOCOL_DECISIONS_ONLY; no pilot, candidate "
            "construction, execution, language or specification decision, compiler "
            "implementation, or production adoption without a later separate "
            "authorization."
        ),
        "owner_authorization_record": authorization_boundary,
        "completion_record": completion,
    }


def manifest_bytes(manifest: dict[str, object]) -> bytes:
    return (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode("utf-8")


def build_summary(manifest: dict[str, object], encoded_manifest: bytes) -> dict[str, object]:
    return {
        "schema": "xlang-dense-family-lock-a-build-summary-v4",
        "lock_id": LOCK_ID,
        "family_name": FAMILY_NAME,
        "lock_revision": LOCK_REVISION,
        "status": STATUS,
        "frozen_at": LOCK_FROZEN_AT,
        "integrating_author_task_identity": INTEGRATING_AUTHOR_TASK_ID,
        "manifest_path": MANIFEST.name,
        "manifest_bytes": len(encoded_manifest),
        "manifest_sha256": sha256(encoded_manifest),
        "artifact_count": manifest["artifact_count"],
        "current_control_count": manifest["current_control_count"],
        "unresolved_owner_decisions": UNRESOLVED_DECISIONS,
        "candidate_construction_authorized": False,
        "reference_pilot_execution_authorized": False,
        "candidate_freeze_b_authorized": False,
        "candidate_execution_authorized": False,
        "held_out_access_authorized": False,
        "selection_or_scoring_authorized": False,
        "language_or_specification_change_authorized": False,
        "language_or_specification_decision_authorized": False,
        "compiler_implementation_authorized": False,
        "production_implementation_authorized": False,
        "production_adoption_authorized": False,
        "compiler_or_production_implementation_authorized": False,
        "e0_1_restart_authorized": False,
        "xlc_migration_authorized": False,
        "default_teaching_authorized": False,
        "required_predecessor_locks": [],
        "cross_family_state_or_fact_exposure": "NONE",
        "current_g0_core_status": manifest["current_g0_core_status"],
        "caller_visible_family_claim": manifest["caller_visible_family_claim"],
        "explicitly_excluded_claims": manifest["explicitly_excluded_claims"],
        "review_identities": manifest["review_identities"],
        "artifact_class_record_count": len(manifest["artifact_class_records"]),
        "completion_record": manifest["completion_record"],
        "owner_authorization_record": manifest["owner_authorization_record"],
        "authorization_granted_by_approving_this_lock": manifest[
            "authorization_granted_by_approving_this_lock"
        ],
        "claim_summary": manifest["claim_summary"],
        "review_layers": manifest["review_layers"],
        "next_action": "OWNER_REVIEW_AND_SIX_EXPLICIT_DECISIONS",
    }


def summary_bytes(summary: dict[str, object]) -> bytes:
    return (json.dumps(summary, indent=2, sort_keys=True) + "\n").encode("utf-8")


def main() -> None:
    manifest = build_manifest()
    encoded_manifest = manifest_bytes(manifest)
    encoded_summary = summary_bytes(build_summary(manifest, encoded_manifest))
    MANIFEST.write_bytes(encoded_manifest)
    SUMMARY.write_bytes(encoded_summary)
    print(
        "Dense Family Lock A closure manifest: "
        f"{manifest['artifact_count']} artifacts, "
        f"{manifest['current_control_count']} current controls, research-only"
    )


if __name__ == "__main__":
    main()
