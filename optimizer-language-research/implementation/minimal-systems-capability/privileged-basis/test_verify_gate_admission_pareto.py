#!/usr/bin/env python3
"""Hostile mutation tests for the privileged-gate Pareto verifier."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import tempfile


ROOT = Path(__file__).resolve().parent
VERIFIER = ROOT / "verify_gate_admission_pareto.py"


def load_verifier():
    specification = importlib.util.spec_from_file_location(
        "verify_gate_admission_pareto", VERIFIER
    )
    if specification is None or specification.loader is None:
        raise RuntimeError("cannot load gate admission verifier")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def expect_rejection(module, dimensions: bytes, matrix: bytes, label: str) -> None:
    with tempfile.TemporaryDirectory(prefix="xlang-gate-pareto-") as directory:
        root = Path(directory)
        dimension_path = root / module.DIMENSIONS.name
        matrix_path = root / module.MATRIX.name
        dimension_path.write_bytes(dimensions)
        matrix_path.write_bytes(matrix)

        original_dimensions = module.DIMENSIONS
        original_matrix = module.MATRIX
        module.DIMENSIONS = dimension_path
        module.MATRIX = matrix_path
        try:
            try:
                module.verify()
            except ValueError:
                return
            raise AssertionError(f"hostile mutation was accepted: {label}")
        finally:
            module.DIMENSIONS = original_dimensions
            module.MATRIX = original_matrix


def replace_once(data: bytes, old: bytes, new: bytes, label: str) -> bytes:
    if data.count(old) != 1:
        raise AssertionError(
            f"mutation anchor for {label} occurs {data.count(old)} times"
        )
    return data.replace(old, new, 1)


def main() -> int:
    module = load_verifier()
    module.verify()

    dimensions = module.DIMENSIONS.read_bytes()
    matrix = module.MATRIX.read_bytes()

    expect_rejection(
        module,
        dimensions,
        replace_once(
            matrix,
            b"\tORDINARY_SOURCE_NONFORGEABILITY\tREQUIRED\t",
            b"\tORDINARY_SOURCE_NONFORGEABILITY\tBANANA\t",
            "required status",
        ),
        "arbitrary required-cell mutation",
    )
    expect_rejection(
        module,
        dimensions,
        replace_once(
            matrix,
            b"\tRECOMMENDED\tELIMINATED_BY_UNREQUIRED_STATEFUL_COST\t",
            b"\tRECOMMENDED\tRECOMMENDED\t",
            "second recommendation",
        ),
        "multiple recommended candidates",
    )
    expect_rejection(
        module,
        replace_once(
            dimensions,
            b"\tGLOBAL_FRESHNESS\t",
            b"\tGLOBAL_FRESHNESS_RENAMED\t",
            "dimension identity",
        ),
        replace_once(
            matrix,
            b"\tGLOBAL_FRESHNESS\t",
            b"\tGLOBAL_FRESHNESS_RENAMED\t",
            "matrix dimension identity",
        ),
        "coordinated dimension replacement",
    )
    expect_rejection(
        module,
        dimensions,
        matrix.replace(b"\t", b"", 1),
        "shifted matrix columns",
    )

    print("gate admission Pareto hostile mutations: PASS (4/4 rejected)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
