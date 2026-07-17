#!/usr/bin/env python3
"""Fail-closed whole-dossier verifier for Dense Family Lock A."""

from __future__ import annotations

import csv
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

import build_dense_lock as closure


HERE = Path(__file__).resolve().parent


def fail(message: str) -> None:
    raise SystemExit("Dense Family Lock A verification failed: " + message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_local(script: str, *arguments: str) -> None:
    command = [sys.executable, "-B", str(HERE / script), *arguments]
    subprocess.run(command, cwd=str(closure.REPO), check=True)


def verify_manifest() -> dict[str, object]:
    if not closure.MANIFEST.is_file() or not closure.SUMMARY.is_file():
        fail("closure manifest or build summary is absent")
    manifest_data = closure.MANIFEST.read_bytes()
    summary_data = closure.SUMMARY.read_bytes()
    manifest = json.loads(manifest_data.decode("utf-8"))
    summary = json.loads(summary_data.decode("utf-8"))

    expected_manifest = closure.build_manifest()
    if manifest != expected_manifest:
        fail("closure manifest differs from current canonical artifacts")
    if manifest_data != closure.manifest_bytes(expected_manifest):
        fail("closure manifest is not canonical exact JSON bytes")
    expected_summary = closure.build_summary(manifest, manifest_data)
    if summary != expected_summary or summary_data != closure.summary_bytes(expected_summary):
        fail("build summary does not pin the exact closure manifest")

    forbidden_true = (
        "candidate_construction_authorized",
        "reference_pilot_execution_authorized",
        "candidate_freeze_b_authorized",
        "candidate_execution_authorized",
        "held_out_access_authorized",
        "selection_or_scoring_authorized",
        "language_or_specification_change_authorized",
        "language_or_specification_decision_authorized",
        "compiler_implementation_authorized",
        "production_implementation_authorized",
        "production_adoption_authorized",
        "compiler_or_production_implementation_authorized",
        "e0_1_restart_authorized",
        "xlc_migration_authorized",
        "default_teaching_authorized",
    )
    if any(manifest.get(field) is not False for field in forbidden_true):
        fail("manifest exceeds the research-only authorization boundary")
    if manifest.get("unresolved_owner_decisions") != closure.UNRESOLVED_DECISIONS:
        fail("owner-decision set is not exactly OD-0 through OD-5")
    if (
        manifest.get("lock_id") != closure.LOCK_ID
        or manifest.get("family_name") != closure.FAMILY_NAME
        or manifest.get("lock_revision") != closure.LOCK_REVISION
        or manifest.get("schema")
        != "xlang-dense-family-lock-a-closure-manifest-v4"
    ):
        fail("lock identity or revision changed")
    if (
        manifest.get("frozen_at") != closure.LOCK_FROZEN_AT
        or not re.fullmatch(
            r"2026-07-15T\d{2}:\d{2}:\d{2}-07:00",
            str(manifest.get("frozen_at", "")),
        )
        or manifest.get("timezone") != closure.LOCK_TIMEZONE
    ):
        fail("lock timestamp or timezone changed")
    if manifest.get("integrating_author_task_identity") != "/root":
        fail("integrating author identity changed")
    identities = manifest.get("review_identities", {})
    if (
        identities.get("integrating_author", {}).get("task_identity") != "/root"
        or identities.get("final_whole_lock_reviewer", {}).get("task_identity")
        != "/root/whole_lock_hostile"
    ):
        fail("current author or reviewer task identity is absent")
    for layer, prefix in (
        ("coverage", "COVERAGE-REVIEW-R3@"),
        ("contract_soundness", "CONTRACT-SOUNDNESS-REVIEW-R1@"),
        ("performance_protocol", "PERFORMANCE-REVIEW-V5@"),
    ):
        identity = identities.get(layer, {})
        review = manifest["review_layers"][layer]
        expected_no_edit = "NOT_RETAINED" if layer == "coverage" else False
        expected_attestation = (
            "NOT_RETAINED_IN_DURABLE_COVERAGE_REPORT"
            if layer == "coverage"
            else "PRESENT_IN_EXACT_REVIEW_REPORT"
        )
        expected_task_retained = layer == "performance_protocol"
        if (
            identity.get("durable_identity")
            != prefix + review["review_sha256"]
            or identity.get("review_sha256") != review["review_sha256"]
            or identity.get("independent_review") is not True
            or identity.get("reviewed_bytes_edited_by_reviewer")
            != expected_no_edit
            or identity.get("no_edit_attestation") != expected_attestation
            or identity.get("historical_personal_or_task_identity_retained")
            is not expected_task_retained
        ):
            fail(f"durable reviewer identity changed: {layer}")
        if (
            layer == "performance_protocol"
            and identity.get("task_identity") != "/root/repair_soundness_protocol"
        ):
            fail("performance reviewer task identity changed")
        evidence = identity.get("independent_review_evidence", {})
        if layer == "coverage":
            if (
                evidence.get("commit")
                != closure.COVERAGE_INDEPENDENCE_COMMIT
                or evidence.get("decision_gate_heading")
                != closure.COVERAGE_INDEPENDENCE_GATE_HEADING
                or evidence.get("decision_gate_entry_sha256")
                != closure.COVERAGE_INDEPENDENCE_GATE_SHA256
                or evidence.get("attested_property")
                != "INDEPENDENT_G0_ONLY_ORACLE_AND_REVIEW"
            ):
                fail("coverage independence evidence changed")
        elif (
            evidence.get("review_path") != review["review_path"]
            or evidence.get("review_sha256") != review["review_sha256"]
        ):
            fail(f"layer independence evidence changed: {layer}")
    g0 = manifest.get("current_g0_core_status", {})
    if (
        manifest.get("required_predecessor_locks") != []
        or manifest.get("cross_family_state_or_fact_exposure") != "NONE"
        or g0.get("closing_commit") != closure.G0_CORE_COMMIT
        or g0.get("decision_gate_heading") != closure.G0_CORE_GATE_HEADING
        or g0.get("artifact_manifest_sha256")
        != closure.G0_CORE_MANIFEST_SHA256
        or g0.get("relationship_to_family_lock")
        != "FROZEN_RESEARCH_INPUT_NOT_A_COMPLETED_FAMILY_PREDECESSOR"
    ):
        fail("predecessor or cross-family exposure identity changed")
    if (
        manifest.get("caller_visible_family_claim")
        != closure.CALLER_VISIBLE_FAMILY_CLAIM
        or manifest.get("explicitly_excluded_claims")
        != closure.EXPLICITLY_EXCLUDED_CLAIMS
    ):
        fail("family claim or explicit exclusion set changed")
    approval_scope = manifest.get("authorization_granted_by_approving_this_lock", "")
    for fragment in (
        "SIX_OWNER_RESEARCH_PROTOCOL_DECISIONS_ONLY",
        "no pilot",
        "candidate construction",
        "language or specification decision",
        "compiler implementation",
        "production adoption",
        "later separate authorization",
    ):
        if fragment not in approval_scope:
            fail("approval scope is incomplete")
    classes = manifest.get("artifact_class_records", [])
    expected_class_ids = {
        "INSTANTIATED_FAMILY_LOCK_A",
        "FAIL_CLOSED_LOCK_VALIDATOR",
        "CONTRACT_TABLES",
        "SOUNDNESS_FIXTURES_AND_GENERATORS",
        "PERFORMANCE_PROTOCOL_AND_TOOLS",
        "META5_LEDGER",
        "HELD_OUT_CUSTODY_RECORD",
    }
    by_class = {row.get("artifact_class"): row for row in classes}
    if len(classes) != 7 or set(by_class) != expected_class_ids:
        fail("artifact-class manifest is incomplete or duplicated")
    for class_id, row in by_class.items():
        paths = row.get("paths", [])
        digests = row.get("sha256_by_path", {})
        if paths != sorted(set(paths)) or set(paths) != set(digests):
            fail(f"artifact-class paths are not exact: {class_id}")
        current = {
            name: sha256((HERE / name).read_bytes()) for name in paths
        }
        encoded = (
            json.dumps(current, sort_keys=True, separators=(",", ":")) + "\n"
        ).encode("ascii")
        if (
            current != digests
            or row.get("path_sha256_map_sha256") != sha256(encoded)
            or not row.get("producer")
            or not row.get("reviewer")
            or not row.get("status")
        ):
            fail(f"artifact-class producer/reviewer/hash closure changed: {class_id}")
    custody = by_class["HELD_OUT_CUSTODY_RECORD"]
    if (
        custody.get("status") != "BLOCKED_PENDING_EXTERNAL_H_FLATSET_CUSTODY"
        or custody.get("blocker_id") != "PENDING_EXTERNAL_H_FLATSET_CUSTODY"
        or custody.get("hidden_source_or_hash_claimed") is not False
    ):
        fail("held-out custody was falsely claimed frozen")

    owner_record = manifest.get("owner_authorization_record", {})
    if (
        owner_record.get("activities_authorized_after_owner_approval")
        != "SIX_OWNER_RESEARCH_PROTOCOL_DECISIONS_ONLY"
        or "Separate explicit owner authorization"
        not in owner_record.get("candidate_freeze_b_approval_requirement", "")
        or "all applicable cumulative CANDIDATE_FREEZE_B blockers"
        not in owner_record.get("candidate_freeze_b_approval_requirement", "")
        or "Separate explicit owner authorization"
        not in owner_record.get("scored_and_held_out_approval_requirement", "")
        or "all 303 exact contracts"
        not in owner_record.get("family_closure_criterion", "")
        or "NO-SELECTION" not in owner_record.get("family_closure_criterion", "")
    ):
        fail("owner authorization or family-closure boundary is incomplete")
    required_prohibitions = {
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
    }
    if set(owner_record.get("activities_still_prohibited", [])) != required_prohibitions:
        fail("owner authorization prohibition set changed")

    completion = manifest.get("completion_record", [])
    expected_completion = {
        "ALL_FIELD_MARKERS_RESOLVED",
        "CONTRACT_CAPABILITY_WITNESS_CLOSURE",
        "SOUNDNESS_FIXTURE_FREEZE",
        "CONSTRUCTION_CORRECTION_FREEZE",
        "PERFORMANCE_AND_SELECTION_FREEZE",
        "B_FIX_B_P2_NO_TAX_ORACLE_FREEZE",
        "HELD_OUT_CUSTODY_FREEZE",
        "META5_CLOSURE",
        "E0_1_DISPOSITION",
        "INDEPENDENT_HOSTILE_REVIEWS",
        "REPOSITORY_VERIFICATION",
        "OWNER_AUTHORIZATION",
        "DURABILITY",
    }
    completion_by_id = {row.get("completion_item"): row for row in completion}
    if len(completion) != 13 or set(completion_by_id) != expected_completion:
        fail("lock-completion record is incomplete or duplicated")
    expected_statuses = {
        "ALL_FIELD_MARKERS_RESOLVED": "PASS_RESEARCH_INSTANTIATION",
        "CONSTRUCTION_CORRECTION_FREEZE": "BLOCKED_BEFORE_FIRST_CANDIDATE_PROMPT",
        "HELD_OUT_CUSTODY_FREEZE": "BLOCKED_NO_HIDDEN_ARTIFACT_CLAIMED",
        "OWNER_AUTHORIZATION": "BLOCKED_SIX_OWNER_DECISIONS_UNRESOLVED",
        "DURABILITY": "EXTERNAL_FINAL_COMMIT_AND_DECISION_GATE_REQUIRED",
    }
    if any(
        completion_by_id[item].get("status") != status
        for item, status in expected_statuses.items()
    ):
        fail("lock-completion status overstates the research boundary")
    if completion_by_id["HELD_OUT_CUSTODY_FREEZE"].get("blocker_ids") != [
        "PENDING_EXTERNAL_H_FLATSET_CUSTODY"
    ]:
        fail("held-out completion blocker changed")
    return manifest


def verify_report_boundary(manifest: dict[str, object]) -> None:
    report = (HERE / "DENSE-UNIQUE-OWNER-FAMILY-LOCK-A.md").read_text(
        encoding="utf-8"
    )
    claims = manifest["claim_summary"]
    coverage = claims["coverage"]
    soundness = claims["contract_soundness"]
    performance = claims["performance"]
    required = [
        "OWNER_REVIEW_READY_RESEARCH_ONLY",
        "revision `F-DENSE-LOCK-A-R5`",
        closure.LOCK_FROZEN_AT,
        "Integrating author task identity: `/root`",
        "Final independent exact-byte reviewer task identity: `/root/whole_lock_hostile`",
        "`/root/repair_soundness_protocol`, with durable identity",
        "`PERFORMANCE-REVIEW-V5@e42823c8ecf94b2ac5c898c3215c511e9881fd082b7b77a112e98ff3b3b7bfe1`",
        "required predecessor lock set is empty",
        "Cross-family state or fact exposure: `NONE`",
        closure.G0_CORE_COMMIT,
        closure.G0_CORE_GATE_HEADING,
        closure.G0_CORE_MANIFEST_SHA256,
        "C-ATOMIC-TRANSITIONS",
        "C-LINEAR-REBUILD",
        "C-DERIVED-REPAIR",
        "C-PROOF-CARRYING-STATE",
        "C-RUNTIME-TOPOLOGY",
        "NO-SELECTION",
        "B-FIX",
        "B-P2",
        "H-FLATSET",
        "W-SMALL",
        "W-GAP",
        "Candidate Freeze B",
        "DENSE-E01-DISPOSITION-AUTHORITY.tsv",
        "DENSE-E01-MEMBER-CLASSIFICATION.tsv",
        "2973109ddfee2b6caf8f0b4eedbbbfc55ec933e12c842735dd76c490f106e613",
        f"- {coverage['audit_cluster_count']} audit clusters;",
        f"- {coverage['selector_child_count']} selector children;",
        f"- {coverage['target_terminal_count']:,} exact evidence-to-target terminals;",
        f"- {coverage['evidence_member_binding_count']} evidence/member bindings;",
        f"- {coverage['overlay_binding_count']} overlay bindings;",
        f"- {coverage['role_binding_count']} role bindings;",
        f"- {coverage['capability_binding_count']:,} capability bindings;",
        f"- all {coverage['direct_evidence_identity_count']} direct evidence identities anchored exactly once.",
        f"The contract registry contains {soundness['contract_count']} exact member/outcome units.",
        f"All {soundness['candidate_count']} candidates bind to all {soundness['contract_count']} units",
        f"producing exactly {soundness['candidate_binding_count']:,} candidate/contract",
        f"generated {soundness['trace_count']:,} deterministic traces",
        f"Across those traces, {soundness['hostile_case_count']} are hostile cases.",
        f"It rejected {soundness['registered_mutation_count']} registered mutations",
        f"- {performance['operation_gate_count']} standalone same-shape Rust operation gates;",
        f"- {performance['matrix_cell_count']} exact matrix cells, including {performance['timed_primary_cell_count']} primary timed cells;",
        f"- {performance['explicit_blocker_count']} explicit operational blockers",
        (
            f"- {performance['disposition_counts']['TIMED_PRIMARY']} `TIMED_PRIMARY`, "
            f"{performance['disposition_counts']['FUNCTIONAL_ONLY']} `FUNCTIONAL_ONLY`, "
            f"{performance['disposition_counts']['STRUCTURAL_ONLY']} `STRUCTURAL_ONLY`, and "
            f"{performance['disposition_counts']['EXCLUDED']} `EXCLUDED` dispositions;"
        ),
        f"all {claims['e0_1_disposition_count']} rows required by the historical E0.1",
        f"all {claims['e0_1_member_classification_count']} unique exact members",
        "84 new mandatory exact contracts",
        "6 raw or initialization-authority rejection-evidence members",
        "3 lazy lifecycle evidence members",
        "five pairwise-distinct candidate-mechanism contracts",
        "Approving this lock grants only the six owner research-protocol decisions.",
        "BLOCKED_BEFORE_FIRST_CANDIDATE_PROMPT",
        "BLOCKED_NO_HIDDEN_ARTIFACT_CLAIMED",
        "PENDING_EXTERNAL_H_FLATSET_CUSTODY",
    ] + closure.UNRESOLVED_DECISIONS
    normalized_report = re.sub(r"\s+", " ", report)
    missing = [
        fragment for fragment in required
        if re.sub(r"\s+", " ", fragment) not in normalized_report
    ]
    if missing:
        fail("owner dossier is missing boundary or decision text: " + ", ".join(missing))
    forbidden = (
        "candidate construction is authorized",
        "candidate execution is authorized",
        "production implementation is authorized",
    )
    lowered = normalized_report.lower()
    if any(fragment in lowered for fragment in forbidden):
        fail("owner dossier contains an overbroad authorization statement")


def verify_reference_pilot_stage_order() -> None:
    blockers_path = HERE / "DENSE-PERFORMANCE-BLOCKERS.tsv"
    with blockers_path.open(encoding="utf-8", newline="") as handle:
        blockers = list(csv.DictReader(handle, delimiter="\t"))
    if len(blockers) != 27:
        fail("operational blocker domain is not the exact 27 rows")
    by_id = {row["blocker_id"]: row for row in blockers}
    if len(by_id) != 27:
        fail("operational blocker IDs are duplicated")
    pilot_rows = [row for row in blockers if row["blocker_id"] == "PENDING_EXTERNAL_REFERENCE_PILOT"]
    if len(pilot_rows) != 1:
        fail("reference-pilot blocker is absent or duplicated")
    pilot = pilot_rows[0]
    if (
        pilot["earliest_blocked_stage"] != "CANDIDATE_CONSTRUCTION"
        or pilot["status"] != "BLOCKING"
        or "every applicable per-branch REFERENCE_PILOT prerequisite"
        not in pilot["required_resolution"]
    ):
        fail("reference-pilot execution boundary changed")

    statistics = json.loads(
        (HERE / "DENSE-PERFORMANCE-STATISTICS.json").read_text(encoding="utf-8")
    )
    protocol = statistics.get("stage_prerequisite_protocol", {})
    stage_order = [
        "REFERENCE_PILOT",
        "CANDIDATE_CONSTRUCTION",
        "CANDIDATE_FREEZE_B",
    ]
    if (
        protocol.get("schema") != "xlang-dense-stage-prerequisites-v1"
        or protocol.get("pipeline_stage_order") != stage_order
        or protocol.get("side_stages") != ["DESCRIPTIVE_COUNTER_REPORT"]
        or "transitively blocks every later" not in protocol.get(
            "earliest_stage_rule", ""
        )
    ):
        fail("performance stage prerequisite protocol changed")
    per_branch = protocol.get("per_owner_branch", {})
    if len(per_branch) != 8:
        fail("performance stage protocol does not contain eight owner branches")
    pilot_counts: set[int] = set()
    construction_counts: set[int] = set()
    for branch_id, branch_protocol in per_branch.items():
        applicable = {
            row["blocker_id"] for row in blockers
            if branch_id in row["applicable_owner_branch_ids"].split(",")
        }
        cumulative: set[str] = set()
        for stage in stage_order:
            direct = {
                blocker_id for blocker_id in applicable
                if by_id[blocker_id]["earliest_blocked_stage"] == stage
            }
            cumulative |= direct
            recorded = branch_protocol["pipeline"][stage]
            if (
                recorded["direct_blocker_ids"] != sorted(direct)
                or recorded["cumulative_blocker_ids"] != sorted(cumulative)
            ):
                fail(f"stage prerequisite closure changed: {branch_id}/{stage}")
        pilot_counts.add(len(branch_protocol["pipeline"]["REFERENCE_PILOT"]["direct_blocker_ids"]))
        construction_counts.add(len(branch_protocol["pipeline"]["CANDIDATE_CONSTRUCTION"]["cumulative_blocker_ids"]))
        side = {
            blocker_id for blocker_id in applicable
            if by_id[blocker_id]["earliest_blocked_stage"]
            == "DESCRIPTIVE_COUNTER_REPORT"
        }
        if branch_protocol["side_stages"]["DESCRIPTIVE_COUNTER_REPORT"] != sorted(side):
            fail(f"counter side-stage closure changed: {branch_id}")
    if pilot_counts != {8, 9} or construction_counts != {21, 22}:
        fail("per-branch pilot or construction prerequisite counts changed")
    construction = statistics.get("construction_protocol", {})
    expected_by_branch = {
        branch_id: branch["pipeline"]["CANDIDATE_CONSTRUCTION"]["cumulative_blocker_ids"]
        for branch_id, branch in per_branch.items()
    }
    expected_union = sorted({
        blocker_id
        for blocker_ids in expected_by_branch.values()
        for blocker_id in blocker_ids
    })
    if (
        construction.get("required_blocker_ids_by_owner_branch")
        != expected_by_branch
        or construction.get("required_blocker_ids") != expected_union
        or "before the first candidate prompt"
        not in construction.get("first_candidate_prompt_gate", "")
    ):
        fail("candidate construction closure is incomplete")
    expected_freeze_b_union = sorted({
        blocker_id
        for branch in per_branch.values()
        for blocker_id in branch["pipeline"]["CANDIDATE_FREEZE_B"][
            "cumulative_blocker_ids"
        ]
    })
    manifest = json.loads(closure.MANIFEST.read_text(encoding="utf-8"))
    completion = {
        row["completion_item"]: row for row in manifest["completion_record"]
    }
    if completion["CONSTRUCTION_CORRECTION_FREEZE"]["blocker_ids"] != expected_union:
        fail("construction completion record is not the exact cumulative closure")
    for completion_id in (
        "PERFORMANCE_AND_SELECTION_FREEZE",
        "B_FIX_B_P2_NO_TAX_ORACLE_FREEZE",
    ):
        if completion[completion_id]["blocker_ids"] != expected_freeze_b_union:
            fail(f"Freeze-B completion record is incomplete: {completion_id}")
    if completion["CONTRACT_CAPABILITY_WITNESS_CLOSURE"]["blocker_ids"] != [
        "PENDING_EXTERNAL_H_FLATSET_CUSTODY",
        "PENDING_EXTERNAL_W_GAP_FIXTURE",
        "PENDING_EXTERNAL_W_SMALL_FIXTURE",
    ]:
        fail("W/H fixture completion blockers are incomplete")

    dossier = re.sub(
        r"\s+",
        " ",
        (HERE / "DENSE-UNIQUE-OWNER-FAMILY-LOCK-A.md").read_text(
            encoding="utf-8"
        ),
    )
    plan = re.sub(
        r"\s+",
        " ",
        (closure.REPO / "THE-PLAN.md").read_text(encoding="utf-8"),
    )
    dossier_requirements = (
        "hard execution prerequisite to candidate construction",
        "it cannot permit the first candidate prompt or source construction before pilot closure",
        "Only a feasible result permits candidate construction to start",
    )
    plan_requirements = (
        "reference-only pilot and candidate construction contingent on a feasible pilot closure, in that order",
        "pilot artifacts must remain frozen through Candidate Freeze B",
    )
    if any(fragment not in dossier for fragment in dossier_requirements):
        fail("owner dossier contradicts the reference-pilot construction gate")
    if any(fragment not in plan for fragment in plan_requirements):
        fail("THE-PLAN contradicts the reference-pilot construction gate")


def verify_whole_lock_review(manifest_data: bytes) -> None:
    if not closure.WHOLE_LOCK_REVIEW.is_file():
        fail("external whole-lock hostile review is absent")
    review = closure.WHOLE_LOCK_REVIEW.read_text(encoding="utf-8")
    lines = review.splitlines()
    status_lines = [line for line in lines if line.startswith("Status:")]
    result_lines = [line for line in lines if line.startswith("Result:")]
    if status_lines != ["Status: PASS_EXACT_BYTES_WHOLE_LOCK_RESEARCH_ONLY"]:
        fail("whole-lock review status line is absent, duplicated, or not PASS")
    if result_lines != ["Result: PASS"]:
        fail("whole-lock review result line is absent, duplicated, or not PASS")
    required_boundary_lines = (
        "Reviewer task identity: /root/whole_lock_hostile",
        "Reviewer role: independent final exact-byte whole-lock reviewer",
        "Candidate construction authorized: NO",
        "Reference pilot execution authorized: NO",
        "Candidate Freeze B authorized: NO",
        "Candidate-primary execution authorized: NO",
        "Scored or held-out execution authorized: NO",
        "Held-out access authorized: NO",
        "Candidate selection or scoring authorized: NO",
        "Language or specification change authorized: NO",
        "Language or specification decision authorized: NO",
        "Compiler implementation authorized: NO",
        "Production implementation authorized: NO",
        "Production adoption authorized: NO",
        "Compiler or production implementation authorized: NO",
        "E0.1 restart authorized: NO",
        "xlc migration authorized: NO",
        "Default teaching authorized: NO",
    )
    if any(lines.count(line) != 1 for line in required_boundary_lines):
        fail("whole-lock review lacks exact research-only negative authorities")
    reviewed = closure.review_hash_table(review)
    expected_hashes = {
        closure.MANIFEST.name: sha256(manifest_data),
        "DENSE-UNIQUE-OWNER-FAMILY-LOCK-A.md": sha256(
            (HERE / "DENSE-UNIQUE-OWNER-FAMILY-LOCK-A.md").read_bytes()
        ),
        "build_dense_lock.py": sha256((HERE / "build_dense_lock.py").read_bytes()),
        "verify_dense_lock.py": sha256((HERE / "verify_dense_lock.py").read_bytes()),
    }
    if any(reviewed.get(name) != digest for name, digest in expected_hashes.items()):
        fail("whole-lock review does not pin every required exact byte set")


def main() -> None:
    run_local("dense_coverage_authority.py", "--check")
    run_local("test_dense_coverage_authority.py")
    run_local("dense_contract_registry.py", "--check")
    run_local("dense_soundness_oracle.py", "--check")
    run_local("verify_dense_performance.py")
    run_local("dense_e01_disposition.py", "--check")
    manifest = verify_manifest()
    verify_report_boundary(manifest)
    verify_reference_pilot_stage_order()
    verify_whole_lock_review(closure.MANIFEST.read_bytes())
    print(
        "Dense Family Lock A whole-dossier verification: PASS - "
        f"{manifest['artifact_count']} exact artifacts, six unresolved owner "
        "decisions, research-only authorization"
    )


if __name__ == "__main__":
    main()
