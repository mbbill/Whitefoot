#!/usr/bin/env python3
"""Build or check the two-model frontend-boundary proposal evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile


MODULE_ROOT = Path(__file__).resolve().parent
VERIFIER_ROOT = MODULE_ROOT.parent
if str(MODULE_ROOT) not in sys.path:
    sys.path.insert(0, str(MODULE_ROOT))

from frontend_boundary_independent import (
    IndependentBoundaryError,
    build_independent,
)
from frontend_boundary_primary import BoundaryModelError, build_primary


EVIDENCE_PATH = VERIFIER_ROOT / "evidence" / "frontend-boundary-evidence.json"
CHECKSUM_PATH = VERIFIER_ROOT / "evidence" / "frontend-boundary-evidence.sha256"
SOURCE_MANIFEST = MODULE_ROOT / "FRONTEND_BOUNDARY_SOURCES"
SOURCE_PATHS = (
    "proposal/frontend-boundary-cases.json",
    "proposal/frontend_boundary_diag_independent.py",
    "proposal/frontend_boundary_diag_primary.py",
    "proposal/frontend_boundary_evidence.py",
    "proposal/frontend_boundary_independent.py",
    "proposal/frontend_boundary_independent_core.py",
    "proposal/frontend_boundary_independent_ingress.py",
    "proposal/frontend_boundary_independent_scope.py",
    "proposal/frontend_boundary_independent_tree.py",
    "proposal/frontend_boundary_primary.py",
    "proposal/frontend_boundary_primary_core.py",
    "proposal/frontend_boundary_primary_ingress.py",
    "proposal/frontend_boundary_primary_scope.py",
    "proposal/frontend_boundary_primary_tree.py",
    "proposal/frontend_boundary_raw_independent.py",
    "proposal/frontend_boundary_raw_primary.py",
    "tests/test_frontend_boundary.py",
    "tests/test_frontend_boundary_raw.py",
)
MAX_SOURCE_BYTES = 1 << 20


class BoundaryEvidenceError(RuntimeError):
    pass


def _sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def canonical_json(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
        + "\n"
    ).encode("ascii")


def _bounded_source(relative: str) -> bytes:
    path = VERIFIER_ROOT / relative
    if path.is_symlink() or not path.is_file():
        raise BoundaryEvidenceError(f"evidence source is not regular: {relative}")
    try:
        size = path.stat().st_size
    except OSError as error:
        raise BoundaryEvidenceError(f"cannot stat evidence source {relative}: {error}") from error
    if size > MAX_SOURCE_BYTES:
        raise BoundaryEvidenceError(f"evidence source exceeds its byte limit: {relative}")
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise BoundaryEvidenceError(f"cannot read evidence source {relative}: {error}") from error
    if len(raw) != size:
        raise BoundaryEvidenceError(f"evidence source changed while reading: {relative}")
    return raw


def _source_revisions() -> tuple[dict[str, dict[str, object]], dict[str, object]]:
    expected = "".join(path + "\n" for path in SOURCE_PATHS).encode("ascii")
    try:
        manifest = SOURCE_MANIFEST.read_bytes()
    except OSError as error:
        raise BoundaryEvidenceError(f"cannot read frontend boundary source manifest: {error}") from error
    if manifest != expected:
        raise BoundaryEvidenceError("frontend boundary source manifest is stale")
    revisions = {
        "grammar-verifier/" + relative: {
            "byte_length": len(raw),
            "sha256": _sha256(raw),
        }
        for relative in SOURCE_PATHS
        for raw in (_bounded_source(relative),)
    }
    return revisions, {
        "byte_length": len(manifest),
        "path": "grammar-verifier/proposal/FRONTEND_BOUNDARY_SOURCES",
        "sha256": _sha256(manifest),
    }


def build_evidence() -> dict[str, object]:
    primary = build_primary()
    independent = build_independent()
    if primary["candidate"] != independent["candidate"]:
        raise BoundaryEvidenceError("models extracted different candidate contracts")
    if primary["descriptor"] != independent["descriptor"]:
        raise BoundaryEvidenceError("models bound different case descriptor bytes")
    primary_evaluation = primary["evaluation"]
    independent_evaluation = independent["evaluation"]
    if primary_evaluation["cases"] != independent_evaluation["cases"]:
        raise BoundaryEvidenceError("models derived different case outcomes")
    if primary_evaluation["requirements"] != independent_evaluation["requirements"]:
        raise BoundaryEvidenceError("models derived different requirement coverage")
    cases = primary_evaluation["cases"]
    if len(cases) != 134:
        raise BoundaryEvidenceError("frontend boundary case inventory changed")
    if primary_evaluation["raw_case_count"] != 34:
        raise BoundaryEvidenceError("primary raw case inventory changed")
    if independent_evaluation["raw_case_count"] != 34:
        raise BoundaryEvidenceError("independent raw case inventory changed")
    revisions, manifest = _source_revisions()
    return {
        "agreement": {
            "case_count": len(cases),
            "raw_case_count": primary_evaluation["raw_case_count"],
            "structured_case_count": len(cases) - primary_evaluation["raw_case_count"],
            "case_projection_sha256": _sha256(canonical_json(cases)),
            "candidate_contract_equal": True,
            "descriptor_binding_equal": True,
            "outcomes_equal": True,
            "requirements_equal": True,
        },
        "authority": "proposal-only executable evidence; not language or compiler authority",
        "candidate": primary["candidate"],
        "cases": cases,
        "descriptor": primary["descriptor"],
        "models": {
            "independent": {
                "implementation": independent["model"],
                "schema": independent_evaluation["schema"],
            },
            "primary": {
                "implementation": primary["model"],
                "schema": primary_evaluation["schema"],
            },
        },
        "requirements": primary_evaluation["requirements"],
        "schema": "whitefoot.frontend-boundary-evidence.v1",
        "source_manifest": manifest,
        "source_revisions": revisions,
    }


def rendered_evidence() -> bytes:
    return canonical_json(build_evidence())


def rendered_checksum(rendered: bytes) -> bytes:
    return f"{_sha256(rendered)}  {EVIDENCE_PATH.name}\n".encode("ascii")


def _read_regular(path: Path) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise BoundaryEvidenceError(
            f"committed frontend boundary artifact is not regular: {path}"
        )
    return path.read_bytes()


def _atomic_write(path: Path, raw: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{path.name}.", dir=path.parent
    )
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            os.fchmod(stream.fileno(), 0o644)
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Build or check independent proposal-only frontend boundary evidence"
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--print", dest="print_report", action="store_true")
    mode.add_argument("--write", action="store_true")
    arguments = parser.parse_args()
    try:
        rendered = rendered_evidence()
        checksum = rendered_checksum(rendered)
        if arguments.print_report:
            sys.stdout.buffer.write(rendered)
        elif arguments.write:
            _atomic_write(EVIDENCE_PATH, rendered)
            _atomic_write(CHECKSUM_PATH, checksum)
        else:
            if _read_regular(EVIDENCE_PATH) != rendered:
                raise BoundaryEvidenceError(
                    "committed frontend boundary evidence is stale"
                )
            if _read_regular(CHECKSUM_PATH) != checksum:
                raise BoundaryEvidenceError(
                    "committed frontend boundary evidence checksum is stale"
                )
    except (
        AssertionError,
        BoundaryEvidenceError,
        BoundaryModelError,
        IndependentBoundaryError,
        OSError,
        ValueError,
    ) as error:
        print(f"frontend boundary evidence: {error}", file=sys.stderr)
        return 1
    if arguments.check:
        print("frontend boundary evidence: two independent models agree")
    elif arguments.write:
        print("frontend boundary evidence: report and checksum written")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
