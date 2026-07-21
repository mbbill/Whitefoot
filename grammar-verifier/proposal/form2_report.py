from __future__ import annotations

import hashlib

from form2_inputs import MAX_SOURCE_BYTES, ProtectedInputs, _read_bounded, sha256
from form2_patch import PatchError
from form2_render import RenderError
from form2_structural_report import (
    STRUCTURAL_ARTIFACT_NAMES,
    StructuralReportError,
    build_structural_artifacts,
)
from form2_topology import TopologyError
from form2_tree import TreeError


ARTIFACT_NAMES = STRUCTURAL_ARTIFACT_NAMES


class ReportError(RuntimeError):
    pass


def _tool_sources() -> dict[str, str]:
    raw = _read_bounded(
        "grammar-verifier/proposal/FORM2_AUDIT_SOURCES", 65_536
    )
    rows = raw.splitlines()
    if not rows or any(not row or row != row.strip() for row in rows):
        raise ReportError("FORM2_AUDIT_SOURCES is not canonical")
    if rows != sorted(rows) or len(rows) != len(set(rows)):
        raise ReportError("FORM2_AUDIT_SOURCES must be sorted and unique")
    sources: dict[str, str] = {}
    for encoded in rows:
        try:
            relative = encoded.decode("ascii")
        except UnicodeDecodeError as error:
            raise ReportError("FORM2_AUDIT_SOURCES path is not ASCII") from error
        source_raw = _read_bounded(
            f"grammar-verifier/{relative}", MAX_SOURCE_BYTES
        )
        sources[f"grammar-verifier/{relative}"] = sha256(source_raw)
    sources["grammar-verifier/proposal/FORM2_AUDIT_SOURCES"] = sha256(raw)
    return dict(sorted(sources.items()))


def build_artifacts(inputs: ProtectedInputs) -> dict[str, bytes]:
    try:
        return build_structural_artifacts(inputs, _tool_sources())
    except (
        PatchError,
        RenderError,
        StructuralReportError,
        TopologyError,
        TreeError,
    ) as error:
        raise ReportError(f"structural FORM-2 evidence failed: {error}") from error


def checksum_index(artifacts: dict[str, bytes]) -> bytes:
    rows = [
        f"{hashlib.sha256(artifacts[name]).hexdigest()}  {name}\n"
        for name in sorted(artifacts)
    ]
    return "".join(rows).encode("ascii")
