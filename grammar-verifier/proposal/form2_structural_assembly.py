from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
from typing import Any

from form2_inputs import ProtectedInputs, canonical_json, sha256
from form2_patch import unified_patch
from form2_render import BLOCK_PRODUCTIONS, SEMICOLON_PRODUCTIONS
from form2_tree import CandidateParser


STRUCTURAL_ARTIFACT_NAMES = (
    "form2-protected-syntax-repairs.patch",
    "form2-structural-layout-evidence.json",
    "form2-structural-migration.json",
    "form2-structural-migration.patch",
)


class StructuralReportError(RuntimeError):
    pass


@dataclass(frozen=True)
class StructuralRun:
    inputs: ProtectedInputs
    parser: CandidateParser
    tool_sources: dict[str, str]
    files: list[dict[str, Any]]
    blockers: list[dict[str, Any]]
    fixture_defects: list[dict[str, Any]]
    documentation_followups: list[dict[str, Any]]
    zero_derivation_audit: list[dict[str, Any]]
    protected_repairs: list[dict[str, Any]]
    patch_inputs: list[tuple[str, bytes, bytes]]
    repair_patch_inputs: list[tuple[str, bytes, bytes]]
    formatting_patch_inputs: list[tuple[str, bytes, bytes]]
    parse_counts: Counter[str]
    block_counts: Counter[str]
    line_counts: Counter[str]
    production_counts: Counter[str]
    earlier_stage_counts: Counter[str]
    total_empty_blocks: int
    total_lines: int
    total_items: int
    total_requires_transitions: int
    changed_count: int
    canonical_migrated_count: int
    rendered_count: int


def _patch_paths(raw: bytes) -> list[str]:
    paths: list[str] = []
    for line in raw.splitlines():
        if not line.startswith(b"+++ b/"):
            continue
        try:
            path = line.removeprefix(b"+++ b/").decode("utf-8")
        except UnicodeDecodeError as error:
            raise StructuralReportError(
                "migration patch path is not UTF-8"
            ) from error
        if not path or path in paths:
            raise StructuralReportError(
                "migration patch has an empty or duplicate path"
            )
        paths.append(path)
    return paths


def _expectation_projection(inputs: ProtectedInputs) -> bytes:
    return canonical_json(
        [
            {
                "expect": source.manifest["expect"],
                "id": source.identifier,
                "status": source.manifest["status"],
            }
            for source in inputs.sources
            if source.manifest is not None
        ]
    )


def _common_projection(
    run: StructuralRun,
    candidate_productions: set[str],
    expectation_projection: bytes,
) -> dict[str, Any]:
    inventory = [
        {"id": row["id"], "path": row["path"], "sha256": row["before_sha256"]}
        for row in run.files
    ]
    return {
        "authority": (
            "non-authoritative, non-applying successor FORM-2 evidence "
            "requiring owner approval"
        ),
        "candidate": {
            "limits_sha256": run.parser.limits_sha256,
            "path": "grammar-verifier/proposal/kernel-spec-successor-candidate.md",
            "production_count": len(candidate_productions),
            "production_names_sha256": sha256(
                canonical_json(sorted(candidate_productions))
            ),
            "sha256": run.parser.source_sha256,
        },
        "manifest_projection": {
            "compiler_verdict_run": "not available and not required for proposal review",
            "expect_and_status_sha256_after": sha256(expectation_projection),
            "expect_and_status_sha256_before": sha256(expectation_projection),
            "manifest_is_outside_patch": True,
            "manifest_sha256": run.inputs.manifest_sha256,
            "semantic_verdict_preservation_claimed": False,
        },
        "protected_inventory": {
            "inventory": inventory,
            "inventory_sha256": sha256(canonical_json(inventory)),
            "source_count": len(run.files),
        },
        "tool_sources": run.tool_sources,
    }


def assemble_structural_artifacts(run: StructuralRun) -> dict[str, bytes]:
    candidate_productions = set(run.parser.grammar.production_by_name)
    missing_blocks = sorted(BLOCK_PRODUCTIONS - set(run.block_counts))
    missing_lines = sorted(SEMICOLON_PRODUCTIONS - set(run.line_counts))
    missing_productions = sorted(
        candidate_productions - {"program"} - set(run.production_counts)
    )
    patch = unified_patch(run.patch_inputs)
    repair_patch = unified_patch(run.repair_patch_inputs)
    formatting_patch = unified_patch(run.formatting_patch_inputs)
    patch_paths = _patch_paths(patch)
    repair_patch_paths = _patch_paths(repair_patch)
    formatting_patch_paths = _patch_paths(formatting_patch)
    expected_patch_paths = [
        row["path"]
        for row in run.files
        if row["after_sha256"] != row["before_sha256"]
    ]
    if patch_paths != expected_patch_paths:
        raise StructuralReportError(
            "migration patch paths do not equal every and only changed source"
        )
    expected_repair_paths = [row["path"] for row in run.protected_repairs]
    if repair_patch_paths != expected_repair_paths:
        raise StructuralReportError(
            "syntax-repair patch paths do not equal the exact proposal inventory"
        )
    expected_formatting_paths = [
        row["path"]
        for row in run.files
        if row["after_sha256"] != row["syntax_repaired_sha256"]
    ]
    if formatting_patch_paths != expected_formatting_paths:
        raise StructuralReportError(
            "FORM-2 layer patch paths do not equal every and only formatting change"
        )

    expectation_projection = _expectation_projection(run.inputs)
    accept_contradictions = sum(
        row["classification"] == "candidate-grammar-contradicts-expected-accept"
        for row in run.blockers
    )
    readiness_blockers = ["independent renderer comparison is pending"]
    if accept_contradictions:
        readiness_blockers.insert(
            0,
            f"{accept_contradictions} expected-accept source(s) have no "
            "complete candidate derivation",
        )
    common = _common_projection(run, candidate_productions, expectation_projection)
    repair_layer = {
        "artifact": "form2-protected-syntax-repairs.patch",
        "authority": (
            "source-only proposal; protected source bytes remain unchanged until "
            "the owner approves these exact edits"
        ),
        "expected_verdict_projection_changed": False,
        "form2_rendering_is_a_later_separate_layer": True,
        "owner_approval_required": True,
        "patch_changed_path_count": len(repair_patch_paths),
        "patch_changed_paths": repair_patch_paths,
        "patch_sha256": sha256(repair_patch),
        "proposals": run.protected_repairs,
        "source_count": len(run.protected_repairs),
    }
    zero_audit = {
        "all_controls_have_one_complete_derivation": True,
        "exact_zero_derivation_source_count": len(run.zero_derivation_audit),
        "records": run.zero_derivation_audit,
        "source_inventory_complete": True,
    }
    summary = {
        "candidate_expected_accept_contradiction_count": accept_contradictions,
        "canonical_migrated_source_count": run.canonical_migrated_count,
        "changed_source_count": run.changed_count,
        "complete_derivation_source_count": run.parse_counts["one"],
        "earlier_stage_counts": dict(sorted(run.earlier_stage_counts.items())),
        "exact_parse_classification_counts": dict(sorted(run.parse_counts.items())),
        "intended_form2_fixture_count": len(run.fixture_defects),
        "compiler_validated": False,
        "evidence_ready_for_owner_review": False,
        "evidence_ready_for_owner_review_blockers": readiness_blockers,
        "migration_authorized": False,
        "no_complete_derivation_source_count": len(run.blockers),
        "patch_changed_path_count": len(patch_paths),
        "patch_changed_paths_sha256": sha256(canonical_json(patch_paths)),
        "primary_evidence_complete": not missing_blocks
        and not missing_lines
        and not missing_productions
        and not accept_contradictions
        and run.rendered_count + len(run.blockers) == len(run.files),
        "rendered_source_count_including_tab_recovery": run.rendered_count,
        "source_count": len(run.files),
        "syntax_repair_source_count": len(run.protected_repairs),
        "unchanged_source_count": len(run.files) - run.changed_count,
        "zero_derivation_audit_source_count": len(run.zero_derivation_audit),
    }
    evidence = {
        **common,
        "blockers": run.blockers,
        "files": run.files,
        "layout_policy": {
            "block_productions": sorted(BLOCK_PRODUCTIONS),
            "indentation": "two ASCII spaces per enclosing brace block; close at reduced depth",
            "inline_gap": (
                "zero if left is ( [ < & . or right is ) ] > , ; . : ( <; "
                "otherwise one ASCII space"
            ),
            "line_bearing_semicolon_productions": sorted(SEMICOLON_PRODUCTIONS),
            "requires_body_transition": "one same-line `} {` boundary",
            "top_level_separator": "one empty line between consecutive item nodes",
        },
        "protected_documentation_followups": run.documentation_followups,
        "protected_syntax_repair_layer": repair_layer,
        "schema": "whitefoot.form2-structural-layout-evidence.v2",
        "topology_policy": {
            "bundle_root": "exactly one global program node, including one-source bundles",
            "empty_source": (
                "no tree node; source identity and byte extent remain in BundleRootExtent"
            ),
            "separator_owner": (
                "derived from adjacent terminal leaves; leading, final, inter-item, "
                "and zero-item boundaries are SourceBytes"
            ),
            "source_projection": "ordered item forest with no program node",
        },
        "structural_counts": {
            "block_production_counts": dict(sorted(run.block_counts.items())),
            "empty_block_count": run.total_empty_blocks,
            "line_bearing_production_counts": dict(sorted(run.line_counts.items())),
            "missing_block_family_coverage": missing_blocks,
            "missing_line_family_coverage": missing_lines,
            "missing_production_coverage": missing_productions,
            "physical_line_count": run.total_lines,
            "production_counts": dict(sorted(run.production_counts.items())),
            "requires_body_transition_count": run.total_requires_transitions,
            "top_level_item_count": run.total_items,
        },
        "summary": summary,
        "zero_derivation_audit": zero_audit,
    }
    migration_keys = (
        "after_byte_count",
        "after_sha256",
        "before_byte_count",
        "before_sha256",
        "canonical_sha256",
        "expect",
        "id",
        "isolated_defect",
        "migration_disposition",
        "oracle_source_trace_forest_projection_sha256",
        "path",
        "protected_syntax_repair",
        "status",
        "syntax_repaired_byte_count",
        "syntax_repaired_sha256",
        "terminal_projection_sha256_after",
        "terminal_projection_sha256_before",
        "terminal_projection_sha256_render_input",
        "source_forest_projection_sha256",
        "bundle_topology_projection_sha256",
        "separator_owner_projection_sha256",
    )
    migration = {
        **common,
        "files": [
            {key: row[key] for key in migration_keys} for row in run.files
        ],
        "isolated_form2_fixture_defects": run.fixture_defects,
        "form2_layer_patch": {
            "changed_path_count": len(formatting_patch_paths),
            "changed_paths_sha256": sha256(canonical_json(formatting_patch_paths)),
            "patch_sha256": sha256(formatting_patch),
        },
        "patch_sha256": sha256(patch),
        "protected_syntax_repair_layer": repair_layer,
        "schema": "whitefoot.form2-structural-migration.v2",
        "summary": summary,
    }
    return {
        "form2-protected-syntax-repairs.patch": repair_patch,
        "form2-structural-layout-evidence.json": canonical_json(evidence),
        "form2-structural-migration.json": canonical_json(migration),
        "form2-structural-migration.patch": patch,
    }
