from __future__ import annotations

from collections import Counter
import json
from pathlib import Path
from typing import Any

from form2_independent_inputs import MODULE_ROOT, digest


PRIMARY_REPORT = MODULE_ROOT.parent / "evidence/form2-structural-layout-evidence.json"
PRIMARY_PATCH = MODULE_ROOT.parent / "evidence/form2-structural-migration.patch"
PRIMARY_REPAIR_PATCH = (
    MODULE_ROOT.parent / "evidence/form2-protected-syntax-repairs.patch"
)

COMPLETE_STAGE = "complete-derivation"
REPAIR_COMPLETE_STAGE = "syntax-repair-then-complete-derivation"
TAB_RECOVERY_STAGE = "isolated-form2-tab-recovery"
RENDERED_STAGES = frozenset(
    {COMPLETE_STAGE, REPAIR_COMPLETE_STAGE, TAB_RECOVERY_STAGE}
)


class IndependentComparisonError(RuntimeError):
    pass


def _primary_projection() -> tuple[dict[str, Any], dict[str, dict[str, Any]]]:
    if not PRIMARY_REPORT.is_file() or PRIMARY_REPORT.is_symlink():
        raise IndependentComparisonError("primary structural report is absent")
    try:
        value = json.loads(PRIMARY_REPORT.read_bytes())
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise IndependentComparisonError("primary structural report is malformed") from error
    if not isinstance(value, dict) or not isinstance(value.get("files"), list):
        raise IndependentComparisonError("primary structural report has no file rows")
    rows: dict[str, dict[str, Any]] = {}
    for row in value["files"]:
        if not isinstance(row, dict) or not isinstance(row.get("path"), str):
            raise IndependentComparisonError(
                "primary structural report has a malformed file row"
            )
        if row["path"] in rows:
            raise IndependentComparisonError("primary structural report repeats a path")
        rows[row["path"]] = row
    return value, rows


def _compare_field(
    path: str,
    independent: dict[str, Any],
    primary: dict[str, Any],
    independent_field: str,
    primary_field: str,
) -> None:
    if independent.get(independent_field) != primary.get(primary_field):
        raise IndependentComparisonError(
            f"primary/independent {independent_field} mismatch: {path}"
        )


def _compare_file_rows(
    files: list[dict[str, Any]], primary: dict[str, dict[str, Any]]
) -> tuple[dict[str, int], set[str]]:
    independent = {row["path"]: row for row in files}
    if set(primary) != set(independent):
        raise IndependentComparisonError(
            "primary and independent source inventories differ"
        )
    counts: Counter[str] = Counter()
    rendered_fields = (
        ("source_sha256", "before_sha256"),
        ("syntax_repaired_sha256", "syntax_repaired_sha256"),
        ("terminal_projection_sha256_before", "terminal_projection_sha256_before"),
        (
            "terminal_projection_sha256_render_input",
            "terminal_projection_sha256_render_input",
        ),
        ("terminal_projection_sha256_after", "terminal_projection_sha256_after"),
        ("source_forest_projection_sha256", "source_forest_projection_sha256"),
        ("bundle_topology_projection_sha256", "bundle_topology_projection_sha256"),
        ("separator_owner_projection_sha256", "separator_owner_projection_sha256"),
        ("canonical_sha256", "canonical_sha256"),
        ("migration_sha256", "after_sha256"),
    )
    for path in sorted(independent, key=str.encode):
        left = independent[path]
        right = primary[path]
        _compare_field(path, left, right, "source_sha256", "before_sha256")
        exact_parse = right.get("exact_parse")
        if not isinstance(exact_parse, dict):
            raise IndependentComparisonError(
                f"primary exact-parse row is malformed: {path}"
            )
        if left["stage"] in RENDERED_STAGES:
            for left_field, right_field in rendered_fields:
                _compare_field(path, left, right, left_field, right_field)
            if left["stage"] == COMPLETE_STAGE:
                if exact_parse.get("classification") != "one":
                    raise IndependentComparisonError(
                        f"primary lacks an exact derivation: {path}"
                    )
                counts["exact_complete_matches"] += 1
            elif left["stage"] == REPAIR_COMPLETE_STAGE:
                render_parse = right.get("render_parse")
                if (
                    exact_parse.get("classification") != "zero"
                    or not isinstance(render_parse, dict)
                    or render_parse.get("classification") != "one"
                    or right.get("protected_syntax_repair") is None
                ):
                    raise IndependentComparisonError(
                        f"primary lacks the exact syntax-repair transition: {path}"
                    )
                counts["syntax_repair_matches"] += 1
            else:
                if (
                    exact_parse.get("classification") != "zero"
                    or right.get("recovery") is None
                ):
                    raise IndependentComparisonError(
                        f"primary lacks the tab recovery: {path}"
                    )
                counts["isolated_tab_recovery_matches"] += 1
        else:
            if exact_parse.get("classification") != "zero":
                raise IndependentComparisonError(
                    f"primary fabricated a rejected derivation: {path}"
                )
            if right.get("source_forest_projection_sha256") is not None:
                raise IndependentComparisonError(
                    f"primary attached a source forest to a rejection: {path}"
                )
            if right.get("bundle_topology_projection_sha256") is not None:
                raise IndependentComparisonError(
                    f"primary attached bundle topology to a rejection: {path}"
                )
            if right.get("separator_owner_projection_sha256") is not None:
                raise IndependentComparisonError(
                    f"primary attached separator owners to a rejection: {path}"
                )
            if right.get("canonical_sha256") is not None:
                raise IndependentComparisonError(
                    f"primary attached canonical bytes to a rejection: {path}"
                )
            if right.get("after_sha256") != right.get("before_sha256"):
                raise IndependentComparisonError(
                    f"primary migrated a rejection without a tree: {path}"
                )
            counts["no_derivation_matches"] += 1
    blockers = {
        row["path"] for row in files if row["stage"] not in RENDERED_STAGES
    }
    return dict(sorted(counts.items())), blockers


def _compare_controls(
    report: dict[str, Any], control_records: list[dict[str, Any]]
) -> None:
    zero_audit = report.get("zero_derivation_audit")
    if not isinstance(zero_audit, dict) or not isinstance(
        zero_audit.get("records"), list
    ):
        raise IndependentComparisonError("primary zero-derivation audit is malformed")
    primary = {
        row.get("path"): row
        for row in zero_audit["records"]
        if isinstance(row, dict) and isinstance(row.get("path"), str)
    }
    independent = {row["path"]: row for row in control_records}
    if set(primary) != set(independent):
        raise IndependentComparisonError(
            "primary and independent failure controls differ"
        )
    for path, left in independent.items():
        right = primary[path]
        for field in ("disposition", "expected_rule", "id"):
            if left[field] != right.get(field):
                raise IndependentComparisonError(
                    f"primary/independent control {field} mismatch: {path}"
                )
        control = right.get("control")
        if (
            not isinstance(control, dict)
            or control.get("sha256") != left["control_sha256"]
            or control.get("unique_complete_derivation") is not True
        ):
            raise IndependentComparisonError(
                f"primary control derivation mismatch: {path}"
            )
        repair = right.get("repair_proposal")
        if left["proposed_repair"]:
            if (
                not isinstance(repair, dict)
                or repair.get("after_sha256") != left["control_sha256"]
            ):
                raise IndependentComparisonError(
                    f"primary repair proposal mismatch: {path}"
                )
        elif repair is not None:
            raise IndependentComparisonError(
                f"primary proposes an unexpected repair: {path}"
            )


def _read_patch(path: Path, description: str) -> bytes:
    if not path.is_file() or path.is_symlink():
        raise IndependentComparisonError(f"primary {description} is absent")
    try:
        return path.read_bytes()
    except OSError as error:
        raise IndependentComparisonError(
            f"cannot read primary {description}: {error}"
        ) from error


def compare_primary(
    files: list[dict[str, Any]],
    candidate_sha256: str,
    independent_patch: bytes,
    independent_repair_patch: bytes,
    control_records: list[dict[str, Any]],
    expectation_projection: bytes,
) -> dict[str, Any]:
    report, primary = _primary_projection()
    candidate = report.get("candidate")
    if not isinstance(candidate, dict) or candidate.get("sha256") != candidate_sha256:
        raise IndependentComparisonError(
            "primary structural report targets another candidate"
        )
    counts, independent_blockers = _compare_file_rows(files, primary)
    _compare_controls(report, control_records)

    blockers = report.get("blockers")
    if not isinstance(blockers, list):
        raise IndependentComparisonError("primary blocker inventory is malformed")
    primary_blockers = {
        row.get("path")
        for row in blockers
        if isinstance(row, dict) and isinstance(row.get("path"), str)
    }
    if primary_blockers != independent_blockers:
        raise IndependentComparisonError(
            "primary and independent remaining negatives differ"
        )

    projection_sha256 = digest(expectation_projection)
    projection = report.get("manifest_projection")
    if (
        not isinstance(projection, dict)
        or projection.get("expect_and_status_sha256_before") != projection_sha256
        or projection.get("expect_and_status_sha256_after") != projection_sha256
        or projection.get("manifest_is_outside_patch") is not True
    ):
        raise IndependentComparisonError("primary expected-verdict projection changed")

    primary_patch = _read_patch(PRIMARY_PATCH, "structural migration patch")
    if independent_patch != primary_patch:
        raise IndependentComparisonError(
            "primary and independent migration patch bytes differ"
        )
    primary_repair = _read_patch(PRIMARY_REPAIR_PATCH, "syntax-repair patch")
    if independent_repair_patch != primary_repair:
        raise IndependentComparisonError(
            "primary and independent syntax-repair patch bytes differ"
        )
    return {
        **counts,
        "control_count": len(control_records),
        "independent_migration_patch_sha256": digest(independent_patch),
        "independent_syntax_repair_patch_sha256": digest(independent_repair_patch),
        "migration_patch_bytes_equal": True,
        "primary_migration_patch_sha256": digest(primary_patch),
        "primary_report_sha256": digest(PRIMARY_REPORT.read_bytes()),
        "primary_syntax_repair_patch_sha256": digest(primary_repair),
        "remaining_negative_count": len(independent_blockers),
        "syntax_repair_patch_bytes_equal": True,
    }
