#!/usr/bin/env python3
"""Regenerate immutable research-only inputs from the frozen v3 matrix."""

from __future__ import annotations

import csv
import json
from pathlib import Path
from typing import Any

import dense_performance_registry as registry


HERE = Path(__file__).resolve().parent


def read_matrix() -> list[dict[str, str]]:
    path = HERE / registry.OUTPUTS["matrix"]
    with path.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if not rows or list(rows[0]) != registry.SCHEMAS["matrix"]:
        raise registry.ProtocolError("performance matrix is absent or stale")
    return rows


def read_descriptors() -> list[dict[str, Any]]:
    path = HERE / registry.OUTPUTS["descriptors"]
    with path.open(encoding="ascii") as handle:
        rows = [json.loads(line) for line in handle if line.strip()]
    if not rows:
        raise registry.ProtocolError("performance descriptors are absent")
    return rows


def validate(
    matrix: list[dict[str, str]],
    descriptors: list[dict[str, Any]],
) -> None:
    if len(matrix) != len(descriptors):
        raise registry.ProtocolError(
            "matrix and descriptor counts do not match"
        )
    matrix_ids = [row["cell_id"] for row in matrix]
    descriptor_ids = [row["cell_id"] for row in descriptors]
    if matrix_ids != descriptor_ids:
        raise registry.ProtocolError(
            "matrix and descriptors have different ordered cell IDs"
        )
    for cell, descriptor in zip(matrix, descriptors):
        if descriptor.get("candidate_execution_authorized") is not False:
            raise registry.ProtocolError(
                f"descriptor authorizes candidate execution: {cell['cell_id']}"
            )
        trace_sha = registry.digest_value(descriptor["trace_plan"])
        oracle_sha = registry.digest_value(descriptor["oracle_plan"])
        if (
            trace_sha != descriptor["trace_sha256"]
            or trace_sha != cell["trace_sha256"]
            or oracle_sha != descriptor["oracle_sha256"]
            or oracle_sha != cell["oracle_sha256"]
        ):
            raise registry.ProtocolError(
                f"descriptor digest mismatch: {cell['cell_id']}"
            )
        if (
            descriptor["trace_plan"]["candidate_execution_authorized"] is not False
            or descriptor["oracle_plan"]["candidate_execution_authorized"] is not False
        ):
            raise registry.ProtocolError(
                f"nested plan authorizes execution: {cell['cell_id']}"
            )


def main() -> None:
    registry.assert_frozen_contract_inputs()
    matrix = read_matrix()
    descriptors = read_descriptors()
    validate(matrix, descriptors)
    output_rows = registry.generated_input_rows(descriptors)
    registry.write_jsonl(HERE / registry.OUTPUTS["inputs"], output_rows)
    print(
        "Dense performance inputs v3: "
        f"wrote {len(output_rows)} immutable, non-executable inputs"
    )


if __name__ == "__main__":
    main()
