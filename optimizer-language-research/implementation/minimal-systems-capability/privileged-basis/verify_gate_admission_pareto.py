#!/usr/bin/env python3
"""Fail-closed consistency checks for the privileged-gate Pareto matrix."""

from __future__ import annotations

import csv
import hashlib
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parent
DIMENSIONS = ROOT / "GATE-ADMISSION-PARETO-DIMENSIONS.tsv"
MATRIX = ROOT / "GATE-ADMISSION-PARETO-MATRIX.tsv"
SCHEMA = "gate-admission-pareto-v2"
EXPECTED_DIMENSIONS_SHA256 = (
    "492444e5d2421159019dcb2a370e6902bc9cc175b7b490b9a26f7387507834b7"
)
EXPECTED_MATRIX_SHA256 = (
    "7e43cc84106f4b35197adf57b35941b5f6fb8891aa052c22d449c60b48e2922b"
)

DIMENSION_HEADER = [
    "schema_version",
    "dimension_id",
    "category",
    "requirement_status",
    "requirement_basis",
]
MATRIX_HEADER = [
    "schema_version",
    "dimension_id",
    "requirement_status",
    "requirement_basis",
    "fixed_release_entries",
    "stateless_signed_frame_grant",
    "stateful_successor_snapshot",
    "decision_effect",
]
ALLOWED_STATUS = {
    "REQUIRED",
    "OWNER_CHOICE",
    "DESIRABLE",
    "NOT_REQUIRED",
    "LIMIT",
    "COST",
    "DECISION",
}


def require_digest(path: Path, expected: str) -> None:
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual != expected:
        raise ValueError(
            f"{path.name}: byte digest mismatch: expected {expected}, found {actual}"
        )


def read_tsv(path: Path, header: list[str]) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if reader.fieldnames != header:
            raise ValueError(
                f"{path.name}: header mismatch: {reader.fieldnames!r} != {header!r}"
            )
        rows = list(reader)
    if not rows:
        raise ValueError(f"{path.name}: empty table")
    for line_number, row in enumerate(rows, start=2):
        extras = row.get(None)
        if extras:
            raise ValueError(f"{path.name}:{line_number}: extra columns: {extras!r}")
        for key in header:
            if row.get(key) is None or row[key] == "":
                raise ValueError(f"{path.name}:{line_number}: empty {key}")
        if row["schema_version"] != SCHEMA:
            raise ValueError(
                f"{path.name}:{line_number}: unexpected schema {row['schema_version']!r}"
            )
        if row["requirement_status"] not in ALLOWED_STATUS:
            raise ValueError(
                f"{path.name}:{line_number}: unknown status "
                f"{row['requirement_status']!r}"
            )
    return rows


def unique_index(rows: list[dict[str, str]], source: str) -> dict[str, dict[str, str]]:
    result: dict[str, dict[str, str]] = {}
    for row in rows:
        identity = row["dimension_id"]
        if identity in result:
            raise ValueError(f"{source}: duplicate dimension_id {identity}")
        result[identity] = row
    return result


def require_cell(
    rows: dict[str, dict[str, str]], dimension: str, column: str, expected: str
) -> None:
    actual = rows.get(dimension, {}).get(column)
    if actual != expected:
        raise ValueError(
            f"{dimension}.{column}: expected {expected!r}, found {actual!r}"
        )


def verify() -> None:
    require_digest(DIMENSIONS, EXPECTED_DIMENSIONS_SHA256)
    require_digest(MATRIX, EXPECTED_MATRIX_SHA256)

    dimensions = unique_index(
        read_tsv(DIMENSIONS, DIMENSION_HEADER), DIMENSIONS.name
    )
    matrix = unique_index(read_tsv(MATRIX, MATRIX_HEADER), MATRIX.name)

    if dimensions.keys() != matrix.keys():
        missing = sorted(dimensions.keys() - matrix.keys())
        extra = sorted(matrix.keys() - dimensions.keys())
        raise ValueError(f"dimension coverage mismatch: missing={missing}, extra={extra}")

    for identity, dimension in dimensions.items():
        scored = matrix[identity]
        for field in ("schema_version", "requirement_status", "requirement_basis"):
            if dimension[field] != scored[field]:
                raise ValueError(
                    f"{identity}: {field} mismatch: "
                    f"{dimension[field]!r} != {scored[field]!r}"
                )

    if len(matrix) != 42:
        raise ValueError(f"expected 42 frozen dimensions, found {len(matrix)}")

    require_cell(
        matrix,
        "PARETO_IF_EXTENSION_REQUIRED",
        "stateless_signed_frame_grant",
        "RECOMMENDED",
    )
    require_cell(
        matrix,
        "PARETO_IF_EXTENSION_REQUIRED",
        "decision_effect",
        "C_IF_OWNER_SAYS_YES",
    )
    require_cell(
        matrix,
        "PARETO_IF_RELEASE_UPDATE_ACCEPTED",
        "fixed_release_entries",
        "RECOMMENDED",
    )
    require_cell(
        matrix,
        "PARETO_IF_RELEASE_UPDATE_ACCEPTED",
        "decision_effect",
        "F_IF_OWNER_SAYS_NO",
    )

    candidate_columns = (
        "fixed_release_entries",
        "stateless_signed_frame_grant",
        "stateful_successor_snapshot",
    )
    for dimension, selected_column in (
        ("PARETO_IF_EXTENSION_REQUIRED", "stateless_signed_frame_grant"),
        ("PARETO_IF_RELEASE_UPDATE_ACCEPTED", "fixed_release_entries"),
    ):
        selected = [
            column
            for column in candidate_columns
            if matrix[dimension][column] == "RECOMMENDED"
        ]
        if selected != [selected_column]:
            raise ValueError(
                f"{dimension}: expected only {selected_column} to be RECOMMENDED, "
                f"found {selected}"
            )
    require_cell(matrix, "MUTABLE_PROTECTED_AUTH_STATE", "fixed_release_entries", "NONE")
    require_cell(
        matrix,
        "MUTABLE_PROTECTED_AUTH_STATE",
        "stateless_signed_frame_grant",
        "NONE",
    )
    require_cell(
        matrix,
        "MUTABLE_PROTECTED_AUTH_STATE",
        "stateful_successor_snapshot",
        "REQUIRED",
    )
    require_cell(
        matrix,
        "NO_HIGH_LEVEL_PRIVILEGE_LAUNDERING",
        "decision_effect",
        "MANDATORY_COMMON_RULE",
    )
    require_cell(
        matrix,
        "FRAME_TEMPLATE_AUTHORITY_CEILING",
        "fixed_release_entries",
        "PASS_IF_BOUNDED_NO_FORMULA_PARAMETER_GRAMMAR",
    )
    require_cell(
        matrix,
        "EXACT_MINIMAL_PRIVILEGE_CUT_ACCOUNTING",
        "decision_effect",
        "MANDATORY_COMMON_RULE",
    )
    require_cell(
        matrix,
        "FRAME_INSTANCE_WITHOUT_AUTH_RELEASE",
        "requirement_status",
        "OWNER_CHOICE",
    )

    print(
        "gate admission Pareto matrix: PASS "
        f"({len(matrix)} dimensions; conditional C/F result preserved)"
    )


def main() -> int:
    try:
        verify()
    except (OSError, ValueError) as error:
        print(f"gate admission Pareto matrix: FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
