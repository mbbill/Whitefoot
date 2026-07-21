"""Deterministic packaging for validated grammar-verifier evidence."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import tempfile
from typing import Mapping, Sequence

from runner_inputs import Inputs, SourceRevision, fail, read_regular
from runner_report import RawReport


COMPONENT_NAMES = (
    "oracle.raw",
    "proposal-delta.md",
    "protected-surface-census.json",
    "report.json",
    "report.sha256",
    "static.raw",
)
PACKAGE_FILE_LIMIT = 16_777_216


def _atomic_write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.parent.is_symlink():
        fail("evidence_path", "the evidence directory is a symlink")
    descriptor, temporary = tempfile.mkstemp(prefix=".grammar-evidence-", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def _canonical_json(value: object) -> bytes:
    return (
        json.dumps(
            value,
            allow_nan=False,
            ensure_ascii=True,
            indent=2,
            separators=(",", ": "),
            sort_keys=True,
        )
        + "\n"
    ).encode("ascii")


def _binding(raw: bytes) -> dict[str, object]:
    return {"byte_length": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def validate_published_package(output: Path) -> None:
    """Accept only a complete package selected by its final sidecar marker."""

    marker = read_regular(output / "package.sha256", 256, "package completion marker")
    package = read_regular(output / "package.json", PACKAGE_FILE_LIMIT, "package manifest")
    expected_marker = f"{hashlib.sha256(package).hexdigest()}  package.json\n".encode("ascii")
    if marker != expected_marker:
        fail("package_marker", "the package completion marker does not bind package.json")
    try:
        parsed = json.loads(package.decode("ascii"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        fail("package_manifest", "package.json is not canonical ASCII JSON")
    if _canonical_json(parsed) != package or not isinstance(parsed, dict):
        fail("package_manifest", "package.json is not in canonical form")
    if set(parsed) != {"components", "schema"} or parsed.get("schema") != "whitefoot.grammar-evidence-package.v1":
        fail("package_manifest", "package.json has an unknown schema or field")
    components = parsed.get("components")
    if not isinstance(components, dict) or tuple(sorted(components)) != COMPONENT_NAMES:
        fail("package_manifest", "package.json has a missing or unknown component")
    for name in COMPONENT_NAMES:
        binding = components[name]
        if not isinstance(binding, dict) or set(binding) != {"byte_length", "sha256"}:
            fail("package_manifest", "a package component binding is malformed")
        byte_length = binding.get("byte_length")
        digest = binding.get("sha256")
        if (
            not isinstance(byte_length, int)
            or isinstance(byte_length, bool)
            or byte_length < 0
            or byte_length > PACKAGE_FILE_LIMIT
            or not isinstance(digest, str)
            or len(digest) != 64
            or any(character not in "0123456789abcdef" for character in digest)
        ):
            fail("package_manifest", "a package component binding is noncanonical")
        data = read_regular(output / name, PACKAGE_FILE_LIMIT, f"package component {name}")
        if _binding(data) != binding:
            fail("package_component", f"package component {name} does not match package.json")
    report = read_regular(output / "report.json", PACKAGE_FILE_LIMIT, "canonical evidence report")
    report_sidecar = read_regular(output / "report.sha256", 256, "evidence report digest")
    expected_sidecar = f"{hashlib.sha256(report).hexdigest()}  report.json\n".encode("ascii")
    if report_sidecar != expected_sidecar:
        fail("report_marker", "report.sha256 does not bind report.json")


def write_evidence(
    output: Path,
    root: Path,
    inputs: Inputs,
    revisions: Mapping[str, SourceRevision],
    reports: Sequence[RawReport],
) -> bytes:
    if any(report.failure is not None for report in reports):
        fail("engine_outcome", "a required engine returned a classified failure")
    if len(reports) != 2 or tuple(report.engine for report in reports) != ("static", "oracle"):
        fail("engine_outcome", "the required engine report pair is missing or reordered")
    if reports[0].common != reports[1].common:
        fail("engine_disagreement", "the independent source-coverage ledgers disagree")
    if reports[0].domains != reports[1].domains:
        fail("engine_disagreement", "the independent generated-domain inventories disagree")
    proposal_artifacts = {
        "delta": read_regular(root / "proposal" / "DELTA.md", 1_048_576, "proposal DELTA.md"),
        "protected_surface_census": read_regular(
            root / "proposal" / "protected-surface-census.json",
            1_048_576,
            "proposal protected-surface-census.json",
        ),
    }
    domains = [
        {
            "argument_hex": claim[1].decode("ascii"),
            "document": document.decode("ascii"),
            "id": bytes.fromhex(identifier_hex.decode("ascii")).decode("ascii"),
            "start_hex": claim[0].decode("ascii"),
            "stream_count": int(claim[2]),
            "stream_sha256": claim[3].decode("ascii"),
        }
        for (identifier_hex, document), claim in sorted(reports[0].domains.items())
    ]
    static_observations = reports[0].observations
    oracle_observations = reports[1].observations
    summary = {
        "comparison": {
            "common_block_byte_length": len(reports[0].common or b""),
            "common_block_sha256": hashlib.sha256(reports[0].common or b"").hexdigest(),
            "generated_domain_count": len(reports[0].domains),
            "status": "agree",
        },
        "engines": {
            report.engine: {
                "raw": _binding(report.raw),
                "specific": _binding(report.specific or b""),
            }
            for report in reports
        },
        "environment": {
            "engine_process_model": "posix-rlimit-session-v1",
            "oracle_invocation": "isolated-python3-standard-library",
            "static_build": "cargo-locked-offline-release",
        },
        "inputs": {
            **{section.name: section.binding for section in inputs.sections},
            "expectations": inputs.expectations.binding,
        },
        "observations": {
            "case_delta_status_counts": oracle_observations["case_delta_status_counts"],
            "cases": oracle_observations["cases"],
            "domains": domains,
            "static_delta_status_counts": static_observations["delta_status_counts"],
            "static_transitions": static_observations["transitions"],
        },
        "proposal_artifacts": {name: _binding(data) for name, data in sorted(proposal_artifacts.items())},
        "schema": (
            "whitefoot.grammar-evidence.v2"
            if inputs.installation is not None
            else "whitefoot.grammar-evidence.v1"
        ),
        "source_revisions": {name: revision.value() for name, revision in sorted(revisions.items())},
    }
    if inputs.installation is not None:
        summary["installation"] = dict(inputs.installation)
    report = _canonical_json(summary)
    if len(report) > inputs.limits["max_final_report_bytes"]:
        fail("final_report_size", "the canonical evidence report exceeds its byte limit")
    report_sidecar = f"{hashlib.sha256(report).hexdigest()}  report.json\n".encode("ascii")
    components = {
        "oracle.raw": reports[1].raw,
        "proposal-delta.md": proposal_artifacts["delta"],
        "protected-surface-census.json": proposal_artifacts["protected_surface_census"],
        "report.json": report,
        "report.sha256": report_sidecar,
        "static.raw": reports[0].raw,
    }
    for name, data in components.items():
        _atomic_write(output / name, data)
    package = _canonical_json(
        {
            "components": {name: _binding(data) for name, data in sorted(components.items())},
            "schema": "whitefoot.grammar-evidence-package.v1",
        }
    )
    _atomic_write(output / "package.json", package)
    _atomic_write(output / "package.sha256", f"{hashlib.sha256(package).hexdigest()}  package.json\n".encode("ascii"))
    validate_published_package(output)
    return report
